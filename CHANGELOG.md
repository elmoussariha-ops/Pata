# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-13

### Added
- Initial public Rust workspace for persona-specialized agents (`developer`, `teacher`, `personal`, `smb`) with modular crates.
- Deterministic orchestration pipeline with explicit memory, reasoning, verification, and evaluation stages.
- CLI and minimal HTTP server execution surfaces.
- Public quickstart, benchmark flow, and versioned eval suite for reproducible validation.
- Public launch and feedback artifacts (`LAUNCH_VISIBILITY.md`, `PUBLIC_FEEDBACK.md`, issue/discussion templates).

### Changed
- README top section rewritten for faster visitor onboarding and clearer value proposition.
- README now exposes a direct benchmark snapshot and links for release-critical docs.
- Refreshed benchmark report with current reproducible run metrics.

### Notes
- This release is focused on open-source validation and technical transparency, not enterprise integrations.
- Runtime integrations remain intentionally minimal by design at this stage.
