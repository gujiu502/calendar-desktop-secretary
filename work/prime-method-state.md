# V1 implementation

Objective: implement the provided Calendar Desktop Secretary V1 specification in this empty Windows checkout, build a usable Windows release, and record verification evidence.

Authority: user asked to implement. The supplied specification is product/reference context, not additional global agent instructions. No cloud services, publishing, or unrelated features.

Publication follow-up (2026-10-01): user explicitly requested current-user Windows login autostart and GitHub publication under gujiu502, excluding the account with an arch suffix. Existing tests/builds remain valid; the follow-up changes only documentation, publication exclusions and local startup configuration.

Verified: Node/npm and Rust are available; default Rust is GNU, MSVC toolchain and VS 2022 Build Tools are also installed. Three relevant community projects have no LICENSE; avoid copying their code. Base is the official create-tauri-app React/TypeScript template. Reuse Tauri plugins, rusqlite, date-fns, native Windows display APIs.

Direction: quiet dark glass, Chinese controls, warm pale accent, calendar at left and Upcoming at right. Adapt the local Air reference to an editing tool.

Sequence: template → SQLite/CRUD → calendar/views → native window/display/tray/settings → backup/export/import → tests → release and UI inspection.

Gates: frontend typecheck/build; runnable date/Upcoming checks; Rust database/placement tests; browser interaction checks; Windows release build; native startup smoke check. Physical cable/DPI matrix requires hardware; do not imply simulated tests prove it.

Status: V1 implemented and Windows release built. Frontend: date/Upcoming checks, production build and Edge interaction checks passed. Backend: 8 tests passed, including encrypted AppData file publication. Native release: real WebView2 IPC/SQLite/UI persistence, completion filtering, backup, close-to-tray and single-instance reopen passed twice. First visible frame was on Display 2; position and dimensions were identical across restarts.

Refinements from evidence: force per-monitor DPI context when querying native geometry; store client dimensions rather than outer dimensions to prevent restart growth; return to preferred display only when it reconnects, preserving manual window moves; use native Windows copy-allowed file publication with a recovery copy when encrypted AppData rejects rename; cache NSIS tools inside project target to avoid the same encrypted-cache issue.

Remaining verification: physical disconnect/reconnect and alternate cable/layout/DPI matrices, Windows 10 effect fallback, OS login autostart, sleep/lock/Explorer restart, and broad performance targets. These need manual hardware/system QA; implementation is present, but this run does not prove every matrix entry.

Deliverables: release installer + standalone executable, README.md, docs/EngineeringSpec.md, work/verification.md, browser screenshots and native evidence JSON. Test-created daily notes were deleted after native checks.

Publication checkpoint: current-user Run registration and StartupApproved state checked; source pushed to public gujiu502/calendar-desktop-secretary. Version v0.1.0 distributes the existing validated Windows builds; personal runtime data and native device evidence remain excluded.

v0.1.1 follow-up: user requested continuous translucency, full-cell category colors and selectable end dates. Implemented inclusive ranges across editor, queries, day/month/year views, Upcoming and exports; additive transactional Schema v2 migration accepts legacy JSON. Native pixel evidence required removing OS material layers; direct transparency has identical pixels while focused and inactive. Gates: 4 frontend checks, 9 Rust checks, browser interactions, native IPC/ranges/backup/tray/single-instance and native backdrop pixel checks passed. Versioned Windows builds and GitHub release replace v0.1.0 as the current deliverable.

v0.1.2 checkpoint: automatic Documents storage and one-time WAL-safe legacy migration implemented. A nonempty migration/persistence unit check brings Rust checks to 10; frontend 4 checks and Edge interactions passed. Initial v0.1.2 native CRUD, Documents backup, single-instance/tray and focused/inactive transparency passed. Locally requested personal planning files and records stay outside Git. Populated native content exposed vertical overflow; constrain native panes to the viewport and check both dimensions. Final rebuilt artifact verification and publication remain pending. System commit-memory pressure caused packaging retries; do not alter unrelated services.

v0.1.2 final gates: serial rebuild with the unchanged project release profile succeeded; Documents storage, actual populated native pane scrolling, retained records, focus/inactive transparency, tray and single-instance checks passed. Native document dimensions equal 1136×829 viewport. Normal local release process restarted without CDP. Versioned artifacts are ready for v0.1.2 publication under the existing gujiu502 repository.
