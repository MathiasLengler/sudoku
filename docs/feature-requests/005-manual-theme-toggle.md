# 005: Manual Theme Toggle

**Priority:** P2 – Medium  
**Dependencies:** None

## Summary

Add a manual toggle for switching between light and dark themes, independent of the operating system preference. Currently, the app exclusively follows the OS `prefers-color-scheme` media query with no user override.

## Why It Is Useful

- **User preference:** Many users want to use dark mode during the day or light mode at night, independent of their OS setting. A manual toggle gives them control.
- **Cross-device consistency:** Some users have different OS theme settings on different devices but want a consistent experience in the app.
- **Accessibility:** Some users with visual impairments may find one theme easier to read regardless of their OS setting.
- **Common expectation:** Most modern web apps provide a theme toggle. Its absence can feel like a missing feature.

## Current State

- Both light and dark themes are fully implemented in `sudoku-web/src/app/theme/myTheme.tsx`.
- Theme selection uses `useMediaQuery("(prefers-color-scheme: dark)")` with no manual override.
- Custom MUI theme with distinct color palettes for each mode already exists.
- No theme preference is persisted in `localStorage`.

## Proposed Implementation

1. **New Jotai atom** (`themePreferenceState`):
   - Type: `"light" | "dark" | "system"` (default: `"system"`).
   - Persisted in `localStorage` via the existing `getZodLocalStorage` utility.

2. **Theme resolution logic:**
   - If `"system"` → use `useMediaQuery("(prefers-color-scheme: dark)")` (current behavior).
   - If `"light"` or `"dark"` → use the explicit choice, ignoring OS preference.

3. **Toggle button** in `SudokuAppBar`:
   - Icon button cycling through: System → Light → Dark → System.
   - Icons: `SettingsBrightness` (system), `LightMode` (light), `DarkMode` (dark).
   - Tooltip shows current mode.
   - Alternatively, a simple toggle between light and dark (two-state) if three-state is confusing.

4. **Optional:** Add theme selection to a settings dialog if one is created in the future.

## Challenges and Risks

| Challenge | Mitigation |
|-----------|------------|
| **Three-state confusion:** Users may not understand "System" mode | Use clear icons and tooltips. Default to "System" and show a brief explanation on first use. |
| **Flash of wrong theme:** On page load, the theme may briefly flash the wrong mode before `localStorage` is read | Read the theme preference synchronously from `localStorage` before React hydration. Apply a `data-theme` attribute to `<html>` in the initial HTML or use a blocking script. |
| **CSS variable synchronization:** MUI theme and CSS custom properties need to stay in sync | Use MUI's `ThemeProvider` with the resolved theme; CSS custom properties are derived from the MUI theme (current pattern). |
| **PWA meta theme-color:** The `<meta name="theme-color">` tag needs to update dynamically | Update `theme-color` meta tag when the theme changes via a `useEffect`. |

## Testing Approach

- **Unit tests:**
  - Verify `themePreferenceState` defaults to `"system"`.
  - Verify the resolved theme matches OS preference when set to `"system"`.
  - Verify the resolved theme overrides OS preference when set to `"light"` or `"dark"`.
  - Verify the preference persists to `localStorage` and loads correctly on refresh.
- **Component tests:**
  - Render the toggle button and verify clicking cycles through modes.
  - Verify the correct icon is displayed for each mode.
  - Verify tooltip text matches the current mode.
- **Visual tests:**
  - Screenshot comparison of the app in light mode, dark mode, and system mode.
  - Verify no flash of unstyled content on page load with a persisted preference.
- **Integration tests:**
  - Set preference to "dark", reload page, verify dark theme is applied immediately.
  - Set preference to "system", change OS preference, verify theme updates reactively.
