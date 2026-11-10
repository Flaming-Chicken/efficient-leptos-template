//! Browser test: pages have real URLs. Deep links, link clicks and the back button all
//! show the right page and title.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

async fn next_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

fn pathname() -> String {
    web_sys::window().unwrap().location().pathname().unwrap()
}

fn click_link(href: &str) {
    let selector = format!("header a[href='{href}']");
    document()
        .query_selector(&selector)
        .unwrap()
        .unwrap_or_else(|| panic!("no header link to {href}"))
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
}

#[wasm_bindgen_test]
async fn urls_drive_navigation() {
    let window = web_sys::window().unwrap();
    // Deep link: open /about directly.
    window
        .history()
        .unwrap()
        .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/about"))
        .unwrap();
    leptos::mount::mount_to_body(app::App);
    next_frame().await;
    assert!(
        document().title().starts_with("About |"),
        "title: {}",
        document().title()
    );
    let current = document()
        .query_selector("header a[aria-current='page']")
        .unwrap()
        .expect("the current page's link is marked");
    assert_eq!(current.get_attribute("href").as_deref(), Some("/about"));

    // Clicking a link changes the URL and the page.
    click_link("/");
    next_frame().await;
    assert_eq!(pathname(), "/");
    assert!(document().title().starts_with("Dashboard |"));

    // The back button returns to the previous page.
    window.history().unwrap().back().unwrap();
    next_frame().await;
    next_frame().await;
    assert_eq!(pathname(), "/about");
    assert!(document().title().starts_with("About |"));

    // An unknown address shows the not-found page instead of a blank screen.
    window
        .history()
        .unwrap()
        .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/no-such-page"))
        .unwrap();
    window
        .dispatch_event(&web_sys::Event::new("popstate").unwrap())
        .unwrap();
    next_frame().await;
    assert!(document().query_selector(".not-found").unwrap().is_some());
}
