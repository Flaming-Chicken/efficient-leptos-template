//! Safe, zero-unwrap runtime detection and IPC bridge helpers for Tauri v2 desktop webviews
//! versus serverless web browsers.

/// Returns `true` when running inside a Tauri v2 desktop webview (`window.__TAURI_INTERNALS__` or
/// `window.__TAURI__`), and `false` on standard browsers or SSR hosts.
pub fn is_tauri_desktop() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return false;
        };
        let has_internals = js_sys::Reflect::get(
            &window,
            &wasm_bindgen::JsValue::from_str("__TAURI_INTERNALS__"),
        )
        .map(|val| !val.is_undefined() && !val.is_null())
        .unwrap_or(false);
        if has_internals {
            return true;
        }
        js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__TAURI__"))
            .map(|val| !val.is_undefined() && !val.is_null())
            .unwrap_or(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// Returns a human-readable runtime target label identifying whether the UI is executing inside
/// a Tauri v2 webview, a browser hydration session, or static SSR rendering.
pub fn runtime_environment_label() -> &'static str {
    if is_tauri_desktop() {
        "Tauri v2 Native Desktop Webview (IPC Enabled)"
    } else if cfg!(target_arch = "wasm32") {
        "WebAssembly Browser Hydration / SPA"
    } else {
        "Static SSR Pre-Renderer / Native Host"
    }
}

/// Invokes a backend Tauri v2 command (`command`) with JSON arguments (`args_json`), returning
/// the JSON-serialized response or an error when running outside a Tauri webview.
pub async fn invoke_tauri_command(command: &str, args_json: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().ok_or_else(|| "Browser window unavailable".to_string())?;
        let internals = js_sys::Reflect::get(
            &window,
            &wasm_bindgen::JsValue::from_str("__TAURI_INTERNALS__"),
        )
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())
        .ok_or_else(|| "Not running inside a Tauri v2 webview".to_string())?;

        let invoke_fn =
            js_sys::Reflect::get(&internals, &wasm_bindgen::JsValue::from_str("invoke"))
                .map_err(|_| "Failed to read __TAURI_INTERNALS__.invoke".to_string())?
                .dyn_into::<js_sys::Function>()
                .map_err(|_| "__TAURI_INTERNALS__.invoke is not callable".to_string())?;

        let payload_src = if args_json.trim().is_empty() {
            "{}"
        } else {
            args_json
        };
        let parsed_args = js_sys::JSON::parse(payload_src)
            .map_err(|_| "Invalid JSON arguments for Tauri IPC".to_string())?;
        let promise_val = invoke_fn
            .call2(
                &internals,
                &wasm_bindgen::JsValue::from_str(command),
                &parsed_args,
            )
            .map_err(|_| format!("Tauri command '{command}' invocation failed"))?;

        let promise: js_sys::Promise = promise_val
            .dyn_into()
            .map_err(|_| "Tauri IPC call did not return a Promise".to_string())?;
        let resolved = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| format!("Tauri command '{command}' rejected"))?;

        if resolved.is_undefined() || resolved.is_null() {
            return Ok("null".to_string());
        }
        Ok(js_sys::JSON::stringify(&resolved)
            .ok()
            .and_then(|s| s.as_string())
            .unwrap_or_else(|| "null".to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (command, args_json);
        Err("Tauri webview IPC is only callable from a wasm32 desktop webview".to_string())
    }
}
