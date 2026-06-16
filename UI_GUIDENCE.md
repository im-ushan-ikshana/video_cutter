# Simple Video Cutter — Full App Building Prompt (Tauri + Svelte + Sidecar FFmpeg)

You are an expert senior full‑stack developer and UI designer. Build a **fully functional desktop application** for a “Simple Video Cutter” tool using **Tauri v2** (Rust backend) and **Svelte 4** (frontend) with a **sidecar FFmpeg binary** for video processing. The application must follow the exact design language, layout, motion, and component specifications below. Every interactive element must be implemented with production‑quality code and polished micro‑interactions. All states (loading, empty, error, edge cases) must be handled. Output the complete project structure with all source files, ready to run.

---

## 1. Technical Stack & Environment
- **Desktop shell:** Tauri v2 (Rust backend)
- **Frontend framework:** Svelte 4 + TypeScript, built with Vite
- **Styling:** Tailwind CSS with a custom theme (CSS variables for all design tokens below)
- **State management:** Svelte stores (writable/derived)
- **Video processing:** Native FFmpeg binary bundled as a **Tauri sidecar**. The sidecar is invoked from Rust commands for all video operations.
- **File dialogs:** Tauri plugin `dialog` (native open/save dialogs)
- **Other:** `@tauri-apps/api` for frontend‑backend communication, event system for progress updates.

---

## 2. Design Philosophy & Visual Language
**Philosophy:** Minimal but premium, clean hierarchy, fast and lightweight feeling, functional over decorative, subtle motion, pixel‑perfect alignment, no clutter.

**Aesthetic:** Modern, sleek, slight glass‑morphism only if subtle, flat UI with soft depth, rounded corners (not overly round), clean shadows, strong spacing rhythm.  
**Reference:** Adobe Express simplicity, Microsoft Fluent feel, Arc Browser cleanliness, DaVinci Resolve dark minimalism (much simpler).  
**Avoid:** Overly colorful interfaces, gamer aesthetics, heavy gradients, skeuomorphic buttons, sharp‑edged legacy dialogs.

---

## 3. Color Theme System (Tailwind CSS Tokens)

### Dark Theme (Primary)
- Main background: `#111315`
- Secondary surface: `#171A1D`
- Elevated card: `#1D2125`
- Borders: always `1px` solid, default `rgba(255,255,255,0.08)`
- Text primary: `#FFFFFF`
- Text secondary: `#A5ADB8`
- Text muted: `#6E7781`
- Accent (Option A – soft blue): `#4C8DFF`
  - Hover: `#6BA2FF`
  - Pressed: `#3A78E0`
- Success: `#2BB673`
- Warning: `#FFB020`
- Danger: `#E5484D`

### Light Theme
- Background: `#F7F8FA`
- Cards: `#FFFFFF`
- Border: `rgba(0,0,0,0.08)`
- Text: dark neutral greys (e.g., primary `#1A1C1E`, secondary `#5A5E64`)
- Accent and semantic colors remain the same.

Theme toggle must switch instantly via CSS class on `<html>`, using the defined tokens.

---

## 4. Border Rules (Strict)
Every interactive container, input, card, dialog, and group must use a **1px border system**.
- Normal state: neutral border (dark: `rgba(255,255,255,0.08)`, light: `rgba(0,0,0,0.08)`)
- Hover state: slight accent tint (dark: `rgba(76,141,255,0.2)`)
- Focus state: `1px accent border` + subtle outer glow `box-shadow: 0 0 0 3px rgba(76,141,255,0.12)`
- Never use thick borders.

Apply to: input fields, dialogs, timeline container, preview panel, cards, settings sections, drop zones, toolbar groups, context menus.

---

## 5. Typography
- Font family: `'Inter', 'Segoe UI', system-ui, sans-serif`
- Sizing: Title `24px`, Section `18px`, Body `14‑15px`, Small `12px`
- Line‑height: balanced, readable (body ~1.5)
- Avoid oversized typography.

---

## 6. Motion Design
- Philosophy: Fast, subtle, professional.
- Timing: Hover `120ms`, dialog open `180ms`, page transitions `220ms`, timeline drag real‑time smooth.
- Easing: `ease-out` (or `cubic-bezier(0.16, 1, 0.3, 1)`). No bouncy animations.
- Button press: scale `1 → 0.98` over `120ms`.
- Drag‑over animation: border glow, scale `1 → 1.02` over `200ms`, dashed animated border motion, background tint accent 5‑8% opacity.

---

## 7. Window Layout Structure
- **Top Bar (52–58px height):**
  - Left: app icon + title “Simple Video Cutter”
  - Center: current file name (truncated if needed)
  - Right: theme toggle (sun/moon icon), settings gear, help (?), Tauri window controls (decorations set to `false`, custom titlebar buttons)
  - Subtle 1px bottom border.
- **Main Layout (two‑panel responsive, minimum window width 960px):**
  - Left (60–65%): Video Preview Area
  - Right (35–40%): Controls Panel (card‑based sections)
  - Bottom: Timeline Section
- Outer padding: `24px`
- Internal spacing: `16px` grid rhythm
- If window width < 900px, right panel collapses into a bottom sheet or tabs.

---

## 8. Video Preview Area
- Container: rounded corners `12‑14px`, dark elevated surface, `1px` border, soft shadow.
- Empty state: centered illustration/icon (film reel or video camera), large text “Drop video here”, subtext “Drag & drop or browse a file”.
- Buttons: primary “Choose Video”, secondary “Paste file path” (opens a small dialog with text input).
- Idle animation: subtle floating pulse every ~5 seconds on the icon or container shadow.
- Drag & drop active: entire area reacts – border glows, scale up to `1.02`, dashed border animation, background tint accent at 5‑8% opacity, text changes to “Release to import video”, icon lifts up.
- Video playback uses the native `<video>` element; controls hidden, state managed via Svelte.

---

## 9. Timeline UI (Professional Trimming)
- Structure: video strip thumbnail timeline (generated from the loaded video).
- Elements:
  - Start trim handle (left) & end trim handle (right) – accent colored, rounded, width `8px`, cursor `ew-resize`.
  - Current playhead – thin accent line with a small head that can be dragged.
  - Time ruler above with tick marks.
  - Zoom slider (bottom‑right) controlling pixel‑per‑second.
- Selected cut region: accent translucent overlay (accent color 10% opacity).
- Unselected areas: slightly dimmed (dark overlay 30%).
- Handles interaction: hover → soft glow; drag → smooth movement; holding Shift while dragging snaps to the playhead or keyframe (if auto‑snap is on).
- Thumbnail generation: use the frontend `<video>` element + `<canvas>` to capture frames every 5 seconds; cache the resulting data URLs. Show gray placeholders while thumbnails are being generated.
- Audio waveform: optional thin translucent layer below thumbnails (toggle in settings). If implemented, generate waveform data via sidecar (e.g., `ffprobe` or `ffmpeg` with `showwavespic`).
- Playhead scrubbing: instant, update the preview frame while dragging.
- Frame accuracy: all cuts snap to the nearest frame.

---

## 10. Playback Controls
- Center‑aligned below the preview (or as a floating overlay at the bottom).
- Icons: Play/Pause, Skip Backward 5s, Skip Forward 5s, Frame Step Back, Frame Step Forward, Mute toggle, Volume slider.
- Seekbar: thin `4‑5px` height, thumb small and elegant, expands slightly on hover.
- All buttons show a tooltip with the action and its keyboard shortcut.

---

## 11. Right‑Side Control Panel (Card‑based)
- **Cut Settings card:**
  - Start Time (editable timecode input, format HH:MM:SS.ff)
  - End Time (same)
  - Duration Preview (read‑only)
  - Format Output dropdown: MP4, WebM, MOV
  - Smart auto‑snap toggle (snap handles to nearest keyframe when enabled)
- **Export card:**
  - Output path selector (text field + “Browse” button that opens a native save dialog)
  - Export quality slider (1–100, mapped to CRF 28–18)
  - Large primary CTA: “Export Clip” (accent background)
- **Save/Open project** (small text buttons): “Save Project” (saves .json with file reference and trim points), “Load Project” (restores a saved session).

---

## 12. Button System
- Height: `40‑44px`, border‑radius `10‑12px`.
- Primary: accent background, white text, hover lifts (shadow increase, `translateY(-1px)`), pressed compresses to scale `0.98`.
- Secondary: neutral surface, 1px border, same text color as body.
- Danger: red accent background.
- All buttons: transitions `120ms`, focus ring as per border rules.

---

## 13. Dialog Boxes
All dialogs: centered card, rounded `16px`, `1px` border, soft shadow, subtle backdrop blur (`backdrop‑filter: blur(4px)`), transition `opacity + scale` 180ms.

**Examples:**
- **Open File Dialog:** Use Tauri’s native file dialog to pick a video file; after selection, show a loading state and probe the video.
- **Paste File Path Dialog:** Small prompt with a text input; on confirm, validate the path via backend and import.
- **Export Confirmation Dialog:** Shows file name, duration, resolution, estimated file size (calculated from bitrate), output location. Actions: “Export” (primary), “Cancel”, checkbox “Open folder after export”.
- **Unsaved Progress Dialog:** When closing app or loading a new video while current edit is unsaved. Text: “Discard current edit?” Actions: Cancel, Save Project, Discard (danger).
- **Error dialog:** For unsupported formats, corrupt files, export failures, or missing sidecar.

---

## 14. Settings UI
Tabbed layout (no router, just Svelte reactive state). Tabs: General, Performance, Theme, Export, Shortcuts.
- Each tab: card with 1px border, consistent spacing.
- General: startup behavior (open last project), language (if needed).
- Performance: hardware acceleration toggle (enable FFmpeg multithreading), proxy resolution for timeline (full, half, quarter).
- Theme: Dark / Light / System.
- Export: default format, default quality, default output folder.
- Shortcuts: table showing action and current keybinding (editable).
- Toggle switches: modern pill style, smooth animation.

---

## 15. Core Functionality & Data Flow (Tauri Integration)
- **Rust backend (Tauri):**
  - Command `probe_video(path: String) -> VideoMetadata`: runs sidecar `ffprobe` to get duration, codec, resolution, frame rate. Returns JSON metadata.
  - Command `export_clip(options: ExportOptions) -> Result<(), String>`: spawns sidecar `ffmpeg` with trimming arguments (`-ss`, `-to`, codec settings based on format/quality). Streams progress (percentage) via Tauri events.
  - Command `validate_path(path: String) -> bool`: checks if file exists.
- **Frontend (Svelte):**
  - Store: `videoFileUrl` (asset:// path from Tauri), `videoDuration`, `trimStart`, `trimEnd`, `currentTime`, `isPlaying`, `zoomLevel`, `volume`, `muted`.
  - Derived: selected duration = `trimEnd - trimStart`.
  - On file selection: call `probe_video` and set metadata.
  - Trimming: restrict `trimStart < trimEnd`, within 0–duration.
  - Export: call `export_clip` with current settings; listen to progress events to update a progress bar; on completion, show success dialog and optionally reveal file in system explorer.
- **Project management:** Save/Load as a JSON file containing the video file path (original or copy reference), trim points, and current settings. Handle missing files gracefully.

---

## 16. Thumbnail Generation & Sidecar Usage
- Thumbnails for the timeline are generated **entirely in the frontend** using the `<video>` element and `<canvas>` to avoid unnecessary sidecar calls.
- Sidecar is used only for:
  - Video probing (duration, resolution, codec).
  - Export/trimming (ffmpeg with lossless cut if possible, or re‑encode).
  - Optional: audio waveform extraction (if enabled in settings).
- The sidecar binary is bundled using Tauri’s `externalBin` configuration. Ensure the app checks for its existence at startup.

---

## 17. Loading, Empty, Error & Edge States
- **Loading video:** after file selection, show a skeleton placeholder for timeline thumbnails and a spinner on the preview until probing finishes.
- **Export progress:** modal with a real progress bar, estimated time remaining, and a “Cancel” button (sends kill signal to the sidecar process).
- **Empty states:**
  - No video loaded: illustration + helpful text + CTA.
  - No recent files: placeholder message in the recent list.
  - No export history: small muted text “No exports yet”.
- **Error toasts:** Non‑blocking notifications for minor issues (unsupported format, file not found). Critical errors (export failure, sidecar missing) shown as modal dialogs.
- **Edge cases:**
  - Drop‑down items that don’t apply (e.g., MOV not supported) → show a warning but do not block.
  - Extremely long videos → throttle thumbnail generation and show a progress indicator.
  - User tries to trim zero‑length segment → disable export button and show inline error.

---

## 18. Accessibility & Keyboard Navigation
- Full keyboard navigation: logical tab order, visible focus states (accent glow ring).
- **Default shortcuts (customizable in settings):**
  - `Space` – Play/Pause
  - `Left arrow` – Frame step back
  - `Right arrow` – Frame step forward
  - `Shift + Left` – Skip back 5s
  - `Shift + Right` – Skip forward 5s
  - `I` – Set in point (trimStart) to current time
  - `O` – Set out point (trimEnd) to current time
  - `Delete` – Clear selection / reset trim
  - `Ctrl+Z` / `Cmd+Z` – Undo (trim changes)
  - `Ctrl+Shift+Z` – Redo
  - `Ctrl+S` – Save project
  - `Ctrl+E` – Export
- **Screen reader:** All interactive elements have appropriate aria‑labels; timeline slider uses `role="slider"` with proper attributes; video preview has alt text.
- **Focus trap** inside modals.
- Tooltips on every icon button, describing the action and its shortcut.

---

## 19. Additional Specifications
- **Paste file path** button opens a small dialog with a text field; on submit, it calls `validate_path` and, if valid, imports that video.
- **Volume control** slider next to the mute button, always visible.
- **Timeline drag handles** width `8px`, cursor `ew-resize`.
- **Zoom slider** range: from showing the entire video in the timeline width down to ~2 seconds of video fitting the width.
- **Export quality mapping:** quality 1 → CRF 28, quality 100 → CRF 18, linear interpolation.
- **Format codecs:** MP4 (H.264 + AAC), WebM (VP9 + Opus), MOV (ProRes if sidecar supports, otherwise H.264).
- **Estimated file size:** rough calculation `(video_bitrate + audio_bitrate) * duration * 1.1`.
- **Custom titlebar:** Tauri window with `decorations: false`; custom controls (minimize, maximize, close) styled to match the theme.

---

## 20. Performance & Code Quality
- Use Svelte’s reactivity efficiently; debounce timeline zoom and playhead updates.
- Virtualise long timeline thumbnails if needed (only render visible ones).
- Sidecar communication must be asynchronous; no blocking of the UI.
- Rust code must handle sidecar process management robustly (kill on cancel, cleanup).
- Tailwind config must extend with the exact color tokens and spacing scale from this spec.

---

## 21. Final Output
Produce the complete project – including Tauri configuration (`tauri.conf.json`), Rust source files (commands, sidecar management), Svelte components (layout, preview, timeline, controls, dialogs, settings), and Tailwind configuration – that fully implements the described UI and functionality. All design tokens, border rules, motion values, and interaction states must be faithfully recreated. The app must feel polished, professional, and performant, just like a modern creative tool.
