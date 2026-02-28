# 003: Accessibility (a11y) Improvements

**Priority:** P1 – High  
**Dependencies:** None

## Summary

Improve the accessibility of the sudoku web application to support screen readers, keyboard-only navigation, and users with visual or motor impairments. The current implementation has minimal ARIA attributes and relies primarily on visual cues.

## Why It Is Useful

- **Inclusive design:** Accessibility is both an ethical imperative and, in many jurisdictions, a legal requirement (WCAG 2.1 AA). Sudoku is a logic puzzle that does not inherently require vision — a well-built accessible version can be enjoyed by blind and low-vision users.
- **Wider audience:** Estimated 15% of the global population experiences some form of disability. Better accessibility opens the app to a significantly larger user base.
- **SEO and quality:** Semantic HTML and ARIA attributes improve the overall quality of the markup, benefiting automated tools and search engines.
- **Keyboard users:** Power users who prefer keyboard-only interaction benefit from proper focus management and ARIA live regions.

## Current State

- **ARIA attributes:** Only `aria-label` on `MyIconButton` components. No semantic grid roles.
- **Grid semantics:** The sudoku grid uses `<div>` elements without `role="grid"`, `role="row"`, or `role="gridcell"` attributes.
- **Cell announcements:** Screen readers cannot identify cell position, value, candidates, or error state.
- **Focus management:** Keyboard navigation (arrow keys) works, but focus is managed via a single `tabIndex={0}` on the root div rather than individual cells.
- **Color contrast:** Theme colors may not meet WCAG AA contrast ratios (not audited).
- **No skip navigation:** No mechanism to skip to the grid or control panel.

## Proposed Implementation

### Phase 1: Semantic Grid Structure
1. Add `role="grid"` to the main grid container.
2. Add `role="row"` to each row of cells.
3. Add `role="gridcell"` to each cell element.
4. Add `aria-label` to each cell (e.g., "Row 3, Column 5, value 7" or "Row 3, Column 5, empty, candidates 2, 4, 8").
5. Add `aria-selected="true"` to the currently selected cell.
6. Add `aria-invalid="true"` to cells with incorrect values.

### Phase 2: Live Regions and Announcements
1. Add an `aria-live="polite"` region for game status announcements:
   - "Puzzle solved!" on completion.
   - "Hint: Naked Single at Row 3, Column 5" when a hint is shown.
   - "Value 7 placed at Row 3, Column 5" on cell input.
2. Announce undo/redo actions.
3. Announce mode changes (candidate mode, sticky mode).

### Phase 3: Focus Management
1. Make individual cells focusable with `tabIndex`.
2. Implement roving `tabIndex` pattern: only the selected cell has `tabIndex={0}`, others have `tabIndex={-1}`.
3. Arrow key navigation moves both selection and focus.
4. Tab key moves focus to the value selector / toolbar (logical tab order).

### Phase 4: Color Contrast and Visual Accessibility
1. Audit all color combinations against WCAG AA contrast ratios (4.5:1 for text, 3:1 for UI).
2. Ensure cell states (selected, guide, error, fixed) are distinguishable without color alone (add patterns or icons).
3. Add a high-contrast mode option.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Verbose announcements:** Grid cells generate excessive screen reader output | Use concise `aria-label` patterns. Let the user navigate at their own pace rather than announcing all cells. |
| **Performance:** Adding `aria-label` to all 81+ cells may slow rendering | Compute labels lazily or memoize. For Base3, 81 labels are negligible; for Base4 (256 cells), consider virtualization. |
| **Cross-reader compatibility:** Screen readers (NVDA, JAWS, VoiceOver) interpret ARIA differently | Test with at least NVDA (Windows) and VoiceOver (macOS/iOS). Follow WAI-ARIA grid pattern specification. |
| **Roving tabIndex complexity:** Managing focus across 81 cells adds state management overhead | Use a dedicated `useFocusManager` hook. Leverage the existing `selectedPosState` atom for focus tracking. |
| **Breaking existing UX:** Adding tab stops may disrupt existing keyboard shortcuts | Ensure the grid captures keyboard events before they propagate. Maintain existing shortcut behavior. |
| **Candidate mode verbosity:** Announcing 9 candidates per cell is noisy | Announce only non-empty candidates and use grouping (e.g., "candidates 1, 3, 7"). |

## Testing Approach

- **Automated accessibility audits:**
  - Integrate `axe-core` (via `@axe-core/playwright` or `vitest-axe`) into the test suite.
  - Run automated WCAG 2.1 AA checks on the rendered grid.
  - Verify no critical or serious accessibility violations.
- **Unit tests:**
  - Verify `aria-label` values for cells in various states (empty, valued, candidates, error, fixed).
  - Verify `aria-selected` toggles correctly when selection changes.
  - Verify `aria-live` region content updates on game events.
- **Manual testing:**
  - Test with NVDA on Windows and VoiceOver on macOS.
  - Verify a blind user can navigate the grid, understand cell contents, and input values using only a screen reader.
  - Verify tab order is logical: grid → value selector → toolbar → menu.
- **Color contrast tests:**
  - Use browser DevTools or Lighthouse to audit contrast ratios.
  - Verify all text and interactive elements meet WCAG AA thresholds.
