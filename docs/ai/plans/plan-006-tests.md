# Plan 006: Tests for Manage Plan and UI BDD Coverage

## Goal
Close gaps between BDD feature files and automated tests for the GUI, focusing on Manage Plan, Results filters, persistence, and protected-path safeguards.

## Scope
- Playwright e2e coverage for Manage Plan (tree/list, detail pane, pending deletes, protected-path blocking, batch actions, list→tree focus).
- Playwright e2e for Results filters/search and save/load cancel.
- Vitest + Testing Library unit tests for Manage Plan and Results filtering.
- Use fixtures/stubs where Tauri runtime is unavailable; mark Tauri-only flows.

## Gaps vs Features
- gui/scan: no scan flow test (pending Tauri/fixtures).
- gui/results_visual: only summary cards; missing filter/search behavior.
- gui/manage_plan & gui/plan_edit: no tests for tree/list interactions, pending deletes, detail pane, protected delete blocking, batch actions, list→tree focus.
- persistence/save_load_plan: no save/load or save-cancel tests.
- safety/protected_paths: no UI-level delete-disabled assertions.

## Implementation Steps
1) Playwright e2e – Manage Plan
   - Fixture plan seeded via localStorage or mock load (non-Tauri).
   - Tests: Tree/List toggle; selecting entry shows detail pane; pending-deletes filter; delete disabled on protected; list “Focus in tree” moves selection; batch delete artifacts skips protected; batch keep protected.
2) Playwright e2e – Results
   - Use same fixture plan; verify action filter and search narrow list; summary still renders.
3) Playwright e2e – Persistence
   - Save cancel shows “Save canceled.” (web mode).
   - (If Tauri available) tagged test to save/load real file and verify reload of entries.
4) Vitest component tests
   - Manage Plan: render with stub plan; detail pane shows selected entry; pending-deletes filter works; delete buttons disabled on protected.
   - Results: filters/search reduce visible entries.

## Definition of Done
- Playwright suite covers Manage Plan tree/list, detail, pending deletes, protected delete blocking, batch actions, list→tree focus.
- Playwright covers Results filters/search.
- Save cancel covered; Tauri save/load covered when runtime available.
- Vitest component tests for Manage Plan and Results filters.
