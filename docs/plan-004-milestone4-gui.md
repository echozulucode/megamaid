# Milestone 4: Tauri GUI Integration - Implementation Plan

**Status:** PLANNING
**Target:** Create a modern desktop GUI for Megamaid using Tauri

## Overview

Milestone 4 will add a visual interface to Megamaid, making it accessible to non-technical users while maintaining all CLI functionality. The GUI will provide interactive plan review, real-time progress visualization, and intuitive disk usage visualization.

## Goals

1. **Modern Desktop Application**
   - Cross-platform GUI using Tauri (Windows 11 primary, Linux future)
   - Native performance with web technologies
   - Small bundle size (<15MB)

2. **Visual Components**
   - Interactive file tree/list view
   - Disk usage visualization (tree map or sunburst chart)
   - Real-time scan progress with statistics
   - Plan review and editing interface

3. **User Experience**
   - Intuitive workflow: Scan → Review → Execute
   - Drag-and-drop directory selection
   - Interactive filtering and sorting
   - Visual feedback for all operations

4. **Backend Integration**
   - Tauri commands for Rust backend
   - Event streaming for progress updates
   - State management for scan results
   - Error handling and user notifications

## Architecture Decision: Frontend Framework

### Options Considered

1. **React + TypeScript**
   - Pros: Large ecosystem, excellent tooling, team familiarity
   - Cons: Larger bundle size, more boilerplate
   - Best for: Complex state management, large teams

2. **Vue 3 + TypeScript**
   - Pros: Smaller bundle, cleaner syntax, good TypeScript support
   - Cons: Smaller ecosystem than React
   - Best for: Balanced approach, rapid development

3. **Svelte + TypeScript**
   - Pros: Smallest bundle, fastest runtime, minimal boilerplate
   - Cons: Smaller ecosystem, newer framework
   - Best for: Performance-critical apps, small teams

4. **Vanilla TypeScript + Lit**
   - Pros: No framework overhead, web components
   - Cons: More manual work, less structure
   - Best for: Minimal dependencies, maximum control

### Recommendation: **Svelte + TypeScript**

**Rationale:**
- Smallest bundle size (important for desktop app)
- Excellent TypeScript support
- Reactive by default (perfect for real-time updates)
- Minimal boilerplate (faster development)
- Growing ecosystem with Tauri community support
- Modern features (stores, reactive statements)

**Alternative:** Vue 3 if more mature ecosystem is preferred

## Phase Breakdown

### Phase 4.1: Tauri Setup & Foundation (Week 1)

**Goal:** Set up Tauri infrastructure and basic app shell

**Tasks:**
1. Initialize Tauri project structure
   - Install Tauri CLI
   - Create `src-tauri` directory
   - Configure `tauri.conf.json`
   - Set up build scripts

2. Set up Svelte + TypeScript frontend
   - Initialize Vite + Svelte project
   - Configure TypeScript
   - Set up Tailwind CSS for styling
   - Create basic app layout

3. Create foundational Tauri commands
   - `scan_directory(path)` - Trigger scan
   - `get_scan_results()` - Retrieve results
   - `load_plan(path)` - Load existing plan
   - `save_plan(path, plan)` - Save plan

4. Implement basic IPC (Inter-Process Communication)
   - Event system for progress updates
   - Error handling framework
   - State synchronization

**Deliverables:**
- Working Tauri app shell
- Basic frontend with routing
- Foundational Tauri commands
- 5+ integration tests

---

### Phase 4.2: Scan Interface & Progress (Week 2)

**Goal:** Implement directory scanning UI with real-time progress

**Components:**
1. **Directory Selection**
   - Folder picker dialog (native OS dialog)
   - Recent directories list
   - Drag-and-drop support
   - Path validation

2. **Scan Configuration**
   - Max depth slider
   - Skip hidden files toggle
   - Large file threshold input
   - Detection rules checkboxes

3. **Progress Display**
   - Real-time file count
   - Scan speed (files/sec)
   - Current directory display
   - Estimated time remaining
   - Cancelable scan

4. **Backend Integration**
   - Stream scan progress events
   - Update UI reactively
   - Handle scan errors gracefully
   - Cache scan results

**Deliverables:**
- Functional scan interface
- Real-time progress updates
- Scan configuration UI
- 10+ component tests

---

### Phase 4.3: Results Visualization (Week 3)

**Goal:** Display scan results with interactive visualizations

**Components:**
1. **File Tree View**
   - Hierarchical file/directory display
   - Sortable columns (name, size, modified)
   - Expandable/collapsible nodes
   - Search/filter functionality
   - Multi-select support

2. **Disk Usage Visualization**
   - Tree map chart (DaisyDisk-style)
   - Color-coded by detection rule
   - Interactive: click to drill down
   - Tooltips with file details
   - Size-based rectangles

3. **Statistics Panel**
   - Total files scanned
   - Total size
   - Breakdown by detection rule
   - Largest files/directories
   - Recommended actions

4. **Detail View**
   - File metadata display
   - Detection reason
   - Recommended action
   - Path breadcrumb

**Deliverables:**
- Interactive file tree
- Disk usage chart (using d3.js or recharts)
- Statistics dashboard
- Detail panel
- 15+ component tests

---

### Phase 4.4: Plan Review & Editing (Week 4)

**Goal:** Enable interactive plan review and modification

**Components:**
1. **Plan Editor**
   - List of all flagged entries
   - Action selector (Delete/Review/Keep)
   - Batch action changes
   - Filter by action type
   - Undo/redo support

2. **Action Assignment**
   - Visual indicators (icons, colors)
   - Quick action buttons
   - Context menu
   - Keyboard shortcuts
   - Confirmation dialogs

3. **Plan Management**
   - Save plan to YAML
   - Load existing plan
   - Export to different formats
   - Plan comparison (before/after)

4. **Verification Display**
   - Drift detection warnings
   - File status indicators
   - Safety recommendations
   - Risk assessment

**Deliverables:**
- Plan editing interface
- Action management UI
- Plan save/load functionality
- Verification UI
- 12+ component tests

---

### Phase 4.5: Execution & Monitoring (Week 5)

**Goal:** Execute cleanup plans with real-time monitoring

**Components:**
1. **Execution Configuration**
   - Execution mode selector (DryRun/Batch)
   - Backup directory picker
   - Recycle bin toggle
   - Fail-fast option
   - Safety warnings

2. **Execution Progress**
   - Real-time operation log
   - Progress bar
   - Files deleted count
   - Space freed counter
   - Speed metrics

3. **Results Display**
   - Success/failure summary
   - Transaction log viewer
   - Error details
   - Rollback options (if backup enabled)

4. **Notifications**
   - Desktop notifications
   - Sound effects (optional)
   - Completion alerts
   - Error notifications

**Deliverables:**
- Execution interface
- Real-time progress monitoring
- Results display
- Notification system
- 10+ integration tests

---

### Phase 4.6: Polish & Distribution (Week 6)

**Goal:** Finalize UI/UX and prepare for distribution

**Tasks:**
1. **UI Polish**
   - Consistent theming
   - Dark/light mode toggle
   - Accessibility improvements (WCAG 2.1)
   - Responsive layouts
   - Loading states
   - Empty states

2. **Performance Optimization**
   - Lazy loading for large datasets
   - Virtual scrolling for file lists
   - Debounced updates
   - Memory management

3. **Error Handling**
   - User-friendly error messages
   - Recovery suggestions
   - Error logging
   - Crash reporting (optional)

4. **Distribution**
   - Windows installer (.msi)
   - Code signing (if available)
   - Auto-updater setup
   - Release notes

5. **Documentation**
   - User guide
   - Screenshots
   - Video walkthrough
   - FAQ

**Deliverables:**
- Polished UI/UX
- Windows installer
- User documentation
- Performance benchmarks
- E2E tests

---

## Technical Stack

### Frontend
- **Framework:** Svelte 4 + TypeScript
- **Build Tool:** Vite 5
- **Styling:** Tailwind CSS 3
- **Charts:** D3.js or Recharts
- **Icons:** Lucide Icons or Heroicons
- **State:** Svelte stores
- **Testing:** Vitest + Testing Library

### Backend (Tauri)
- **Tauri Version:** 2.0 (latest stable)
- **Rust Backend:** Existing megamaid crate
- **IPC:** Tauri commands + events
- **File Dialogs:** Tauri Dialog API
- **Notifications:** Tauri Notification API

### Development Tools
- **IDE:** VS Code with Svelte + Rust extensions
- **Debugging:** Chrome DevTools + Rust debugging
- **Testing:** Vitest (frontend) + Rust tests (backend)
- **CI/CD:** GitHub Actions (build + test)

---

## Tauri Commands Design

### Core Commands

```rust
// Scan operations
#[tauri::command]
async fn scan_directory(path: String, config: ScanConfig) -> Result<ScanSummary, String>

#[tauri::command]
async fn get_scan_results() -> Result<Vec<FileEntry>, String>

#[tauri::command]
fn cancel_scan() -> Result<(), String>

// Plan operations
#[tauri::command]
async fn generate_plan() -> Result<CleanupPlan, String>

#[tauri::command]
async fn load_plan(path: String) -> Result<CleanupPlan, String>

#[tauri::command]
async fn save_plan(path: String, plan: CleanupPlan) -> Result<(), String>

#[tauri::command]
async fn update_entry_action(index: usize, action: CleanupAction) -> Result<(), String>

// Verification
#[tauri::command]
async fn verify_plan(plan: CleanupPlan) -> Result<VerificationResult, String>

// Execution
#[tauri::command]
async fn execute_plan(plan: CleanupPlan, config: ExecutionConfig) -> Result<ExecutionSummary, String>

// Utilities
#[tauri::command]
async fn open_file_dialog() -> Result<Option<String>, String>

#[tauri::command]
async fn open_directory_dialog() -> Result<Option<String>, String>
```

### Events

```typescript
// Progress events
listen('scan:progress', (event: { filesScanned: number, currentPath: string }) => {})
listen('scan:complete', (event: { totalFiles: number, duration: number }) => {})
listen('scan:error', (event: { error: string }) => {})

listen('execute:progress', (event: { completed: number, total: number }) => {})
listen('execute:operation', (event: { path: string, status: string }) => {})
listen('execute:complete', (event: { summary: ExecutionSummary }) => {})
```

---

## UI Wireframes (Text-based)

### Main Window Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Megamaid                                           [─] [□] [×]   │
├─────────────────────────────────────────────────────────────────┤
│ [Scan] [Review] [Execute]                         Settings Help │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Scan New Directory                                              │
│  ┌─────────────────────────────────────────────┐  [Browse]      │
│  │ C:\Projects                                  │                │
│  └─────────────────────────────────────────────┘                │
│                                                                   │
│  Configuration:                                                  │
│  ☑ Skip hidden files    Max depth: [○──────○] 10                │
│  ☑ Find build artifacts Large file: [100] MB                     │
│                                                                   │
│  [Start Scan]                                                    │
│                                                                   │
│  Recent Scans:                                                   │
│  • C:\Projects - 1,234 files, 2.5 GB (2 hours ago)              │
│  • D:\Downloads - 5,678 files, 15 GB (1 day ago)                │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### Scan Progress

```
┌─────────────────────────────────────────────────────────────────┐
│ Scanning C:\Projects...                                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  [████████████████████──────] 75%                                │
│                                                                   │
│  Files scanned:   12,345                                         │
│  Speed:          1,234 files/sec                                 │
│  Current:        src\components\FileTree.svelte                  │
│  Elapsed:        10.2s                                           │
│  Remaining:      ~3s                                             │
│                                                                   │
│                                        [Cancel]                  │
└─────────────────────────────────────────────────────────────────┘
```

### Results View

```
┌─────────────────────────────────────────────────────────────────┐
│ Scan Results: C:\Projects                                        │
├──────────────────────────┬──────────────────────────────────────┤
│ Tree Map                 │ File List                            │
│ ┌──────────────────────┐ │ Name          Size      Action      │
│ │                      │ │ ├─ target/    1.2 GB    Delete      │
│ │   target/            │ │ ├─ node_m..   850 MB    Delete      │
│ │   (1.2 GB)           │ │ ├─ large.db   500 MB    Review      │
│ │                      │ │ └─ ...                              │
│ │  node_modules/       │ │                                     │
│ │  (850 MB)            │ │ [⌕ Search]  [Filter ▼]  [Sort ▼]   │
│ │                      │ │                                     │
│ │    large.db          │ │                                     │
│ │    (500 MB)          │ │                                     │
│ └──────────────────────┘ │                                     │
│                          │                                     │
│ Statistics:              │ Selected: target/                   │
│ • Total: 2.5 GB         │ Size: 1.2 GB                        │
│ • Delete: 2.0 GB        │ Rule: Build Artifacts               │
│ • Review: 500 MB        │ Action: Delete                      │
│ • Keep: 0 MB            │                                     │
│                          │ [Keep] [Review] [Delete]            │
├──────────────────────────┴──────────────────────────────────────┤
│                                        [Save Plan] [Execute]    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Testing Strategy

### Unit Tests (Frontend)
- Component rendering
- Event handling
- State management
- Utility functions
- Target: >80% coverage

### Integration Tests (Tauri)
- Command invocation
- Event emission
- State synchronization
- Error handling
- Target: >70% coverage

### E2E Tests
- Complete scan workflow
- Plan editing workflow
- Execution workflow
- Error scenarios
- Target: Critical paths covered

---

## Success Criteria

1. **Functionality**
   - ✅ All CLI features available in GUI
   - ✅ Scan 100K files without UI lag
   - ✅ Real-time progress updates
   - ✅ Interactive plan editing
   - ✅ Safe execution with feedback

2. **Performance**
   - ✅ App startup <2 seconds
   - ✅ Scan UI updates <100ms latency
   - ✅ Tree map rendering <1 second for 10K files
   - ✅ Memory usage <200MB for typical scans

3. **UX**
   - ✅ Intuitive workflow (user testing)
   - ✅ Clear visual feedback
   - ✅ Helpful error messages
   - ✅ Responsive UI (no freezing)

4. **Distribution**
   - ✅ Windows installer <15MB
   - ✅ Clean install/uninstall
   - ✅ No admin rights required
   - ✅ Works on Windows 10+

---

## Timeline Estimate

- **Phase 4.1:** 5-7 days (Setup & Foundation)
- **Phase 4.2:** 5-7 days (Scan Interface)
- **Phase 4.3:** 7-10 days (Visualization)
- **Phase 4.4:** 5-7 days (Plan Editing)
- **Phase 4.5:** 5-7 days (Execution)
- **Phase 4.6:** 5-7 days (Polish & Distribution)

**Total:** 4-6 weeks for complete GUI

---

## Dependencies & Prerequisites

### System Requirements (Development)
- Node.js 18+ (for Vite + Svelte)
- Rust 1.70+ (for Tauri)
- Windows 10/11 SDK
- WebView2 Runtime (Windows)

### Installed via npm
- Tauri CLI
- Vite
- Svelte
- TypeScript
- Tailwind CSS
- D3.js (for charts)
- Vitest (testing)

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Large file lists lag UI | High | Implement virtual scrolling, pagination |
| Memory usage too high | Medium | Stream results, limit cache size |
| WebView2 not installed | Medium | Bundle WebView2 installer, detect & prompt |
| Bundle size too large | Low | Tree-shake dependencies, lazy load modules |
| Cross-platform issues | Low | Start with Windows, test Linux in VM |

---

## Open Questions

1. **Frontend Framework:** Svelte vs Vue 3?
   - Recommendation: Svelte for size/performance
   - Fallback: Vue if ecosystem concerns

2. **Chart Library:** D3.js vs Recharts vs custom canvas?
   - D3.js: Most powerful, steeper learning curve
   - Recharts: Easier, less flexible
   - Custom canvas: Maximum performance, most work

3. **Theme:** Light/dark/both?
   - Recommendation: Both with system preference detection

4. **Distribution:** Self-contained vs. installer?
   - Recommendation: MSI installer for Windows

5. **Auto-update:** Implement in Milestone 4 or defer to 5?
   - Recommendation: Defer to Milestone 5

---

## Next Steps

1. **Decision:** Frontend framework (Svelte recommended)
2. **Decision:** Chart library (D3.js recommended)
3. **Setup:** Initialize Tauri project
4. **Setup:** Initialize frontend project
5. **Begin:** Phase 4.1 implementation

---

**Ready to proceed with Phase 4.1?**
