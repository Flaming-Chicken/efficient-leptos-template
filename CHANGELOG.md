# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-11-14

### Added
- `AppState::schema_version` (`CURRENT_SCHEMA_VERSION = 1`) backed by `spodeian_persist::StateMigrator` for forward-compatible JSON snapshots and legacy unversioned payload upgrades.
- Recoverable `<ErrorBoundary>` around application routes with `ErrorFallbackView` (`role="alert"`, details list, and reactive reset action).
- Interactive `cargo-generate.toml` configuration supporting optional Tauri v2 desktop, offline PWA, and Cloudflare Pages scaffolding.
- `.github/dependabot.yml` automation for weekly `cargo` and `github-actions` updates.
- Property-based tests (`proptest`) covering versioned JSON/BSON serialization, legacy schema migration, RFC 4180 CSV round-trips, and theme cycle normalization, plus SSR component tests for `ErrorFallbackView` and `AboutPage`.

### Changed
- Deduplicated static site generation (`crates/ssg`), DOM theme/dyslexia bridges, modal focus trapping, telemetry KPI cards, and PWA diagnostics onto `spodeian-core` (`spodeian-web-utils::pwa`, `spodeian-persist`, and `spodeian-leptos-ui`).
- Replaced the hardcoded `"Version 0.1.0"` badge in `AboutPage` with dynamic `env!("CARGO_PKG_VERSION")`.

## [0.1.0] - 2026-08-18

### Added
- Initial multi-crate Leptos 0.8 + Tauri v2 + Cloudflare Pages template with static SSR + hydration, Trunk desktop CSR, `spodeian-cache` persistence, WCAG AA contrast verification, and offline PWA support.
