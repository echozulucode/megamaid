# Milestone 4 Remediation Plan (GUI safety & intent alignment)

## Problems Observed
- Detector/Planner can mark entire project roots for deletion (e.g., repo root, “.”) instead of just junk (node_modules/target/dist/etc.).
- Source/config directories and repos with VCS markers aren’t fully protected; size-based rules can flag code.
- UI action buttons only change in-memory state; Delete is not guarded for protected paths; no batch actions.
- No IPC integration tests to verify scan/detect/plan safety on real project shapes.

## Objectives
1) Prevent deletes of project roots/source trees; focus deletes on junk artifacts and large binaries.
2) Make Plan/Results UI actionable: edit actions, batch changes, save/load with safeguards.
3) Add IPC + E2E coverage to stop regressions.

## Actions
1) Detector/Planner Safety
   - Add repo-aware guard: if path contains VCS markers (.git/.hg), manifests (package.json, Cargo.toml), or high source-file ratio, default to Keep/Review; never Delete automatically.
   - Restrict build-artifact deletes to known junk patterns; block Delete on “.” or repo root.
   - Add optional “archive” action for large source dirs (Review/Archive instead of Delete).
2) Planner Defaults
   - Downgrade Delete → Review when protected path detected; refuse plans that Delete “.” or repo root.
   - Surface rule/reason in UI prominently.
3) UI Safeguards
   - Disable Delete buttons for protected paths (root, VCS-containing, manifest-containing).
   - Add batch actions: “Delete all build artifacts”, “Keep/Review all source/manifests”.
   - Persist edits and give toasts for Save/Load and inline edits; keep “Archive” hook for future execution.
4) Tests
   - IPC integration tests (5+): fixture repo with src + node_modules; assert only junk flagged Delete.
   - E2E: action edits persist after Save/Reload; Delete disabled on protected paths.

## Definition of Done
- UI supports inline edits, batch safety, save/load; Delete disabled on protected paths.
- Detector/Planner cannot produce Delete on repo root/source dirs; junk patterns still auto-delete.
- IPC + E2E tests cover safe defaults and plan edit/save flows.
