# Plan: Manage Plan View (Tree-First Plan Editing)

## Goal
Replace the current Results tree preview with a dedicated, tree-first “Manage Plan” view that makes it unmistakable which folders/files will be deleted, kept, or reviewed, with protected-path safeguards.

## Scope
- New navigation entry: “Manage Plan” (or promote existing Plan to this layout).
- Tree-first experience with detail pane; list view remains for fast search/filter.
- Protected-path enforcement: delete disabled for repo/manifests/base path.
- Batch actions and clear “Pending deletes” spotlight.

## Desired UX
- Default to Tree tab (Tree | List). List deep-links to Tree selection.
- Left: hierarchical tree with per-node chips (size, delete/review/keep counts). Colors: red=delete, yellow=review, green=keep. Protected badge where applicable. Delete controls hidden/disabled on protected.
- Right detail pane for selected node: action selector (Keep/Review/Delete), rule/reason, size/mtime, “This will be deleted/kept” banner, and batch apply to subtree.
- Top filters: action filter (all/delete/review/keep), search by path, “Pending deletes” toggle to show only delete candidates.
- Inline batch buttons: “Delete all build artifacts”, “Keep all protected paths” (existing) plus “Mark subtree keep/review/delete” (respect protection).

## Key Changes
1) Routing/UI
   - Add Manage Plan page (or refactor current Plan) with Tree/List tabs.
   - Move tree view from Results into this page; Results stays summary/read-only.
2) Data shaping
   - Reuse plan entries, still hiding “.” and base path.
   - Ensure protected-path helper is shared (UI + backend).
3) Interactions
   - Selecting a node updates detail pane; list clicks focus the tree node.
   - Delete disabled for protected entries; batch actions skip protected.
   - “Pending deletes” view to make deletions explicit.
4) Messaging
   - Banner in detail pane: “This item/subtree will be deleted/kept/reviewed.”
   - Toasts for save/load and action changes.

## Implementation Steps
1) Structure
   - Add Manage Plan route/page with tabs (Tree default, List secondary).
   - Move tree component from Results to Manage Plan; keep summary cards.
2) Tree view
   - Build tree from plan entries; aggregate size/counts per node.
   - Color-coded chips and protected badges; disable delete on protected.
   - Node selection drives detail pane; list click selects node.
3) Detail pane
   - Action selector (keep/review/delete) with protection guard.
   - Show rule, reason, size, mtime; “will be deleted/kept” banner.
   - Batch apply to subtree (respect protected nodes).
4) Filters
   - Action filter, search, “Pending deletes” toggle.
5) Persistence
   - Reuse save/load flows; ensure plan stats update on edits.
6) Tests
   - Svelte unit/e2e: delete disabled on protected; pending-deletes filter works; list-to-tree focus; batch skip protected.
   - IPC/plan tests: root/base path excluded; protected paths never default to delete.

## Definition of Done
- Manage Plan page shipped with Tree (default) and List tabs.
- Tree clearly shows what will be deleted/kept/reviewed; protected paths cannot be deleted.
- “Pending deletes” filter makes deletions explicit.
- Detail pane edits update plan and stats; save/load still work.
- Tests cover protected-path delete blocking, pending-deletes view, and list-to-tree focus.*** End Patch"}}`')}}">
