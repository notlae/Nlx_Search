# 02 · UI Specification

## Non-negotiables
1. **No right-side preview panel.** Results span the full central width.
2. Left sidebar, top search bar, pig logo, bottom status bar stay.
3. Dark translucent glass. **No gradients.**
4. Must look like real Windows 10 software, with Spotlight-level polish.
5. Usability over decoration.

## Wireframe (about 1000 x 640 px window)
```
┌────────────────────────────────────────────────────────────────────┐
│ 🐷 NLX_SEARCH   [🔍 Search anything on your PC...      Ctrl+K]  ⚙ ─ ✕│
├────────────┬───────────────────────────────────────────────────────┤
│ ▣ Everything│ BEST MATCH                                            │
│   Apps      │ ┌───────────────────────────────────────────────────┐ │
│   Files     │ │ [icon] CapCut   Application · Recently used   Open ↵│ │
│   Folders   │ └───────────────────────────────────────────────────┘ │
│   Images    │ APPLICATIONS                                          │
│   Videos    │ [icon] CapCut          C:\...\CapCut.exe    App       │
│   Audio     │ FOLDERS                                               │
│ ─────────── │ [icon] CapCut Drafts   D:\CapCut Drafts     Folder    │
│   Recent    │ FILES                                                 │
│   Favorites │ [icon] capcut_project.cproj  D:\Projects\..  2.4 MB  12/4/2024│
│   Settings  │ IMAGES                                                │
│             │ [thumb] capcut_logo.png      D:\Pictures     1.2 MB  11/20/2024│
├────────────┴───────────────────────────────────────────────────────┤
│ C: 124 GB free · D: 931 GB free   ● Indexing 24,871 files   ↑↓ Enter Esc│
└────────────────────────────────────────────────────────────────────┘
```
There is **no column to the right of the results**.

## Design tokens (`styles/tokens.css`)
```css
:root {
  --bg:        #17191D;  /* main background */
  --surface:   #202329;  /* secondary surfaces */
  --elevated:  #292D34;  /* elevated surfaces */
  --accent:    #16A34A;  /* selection, focus */
  --text:      #F5F5F5;
  --text-dim:  #A1A1AA;

  /* glass layers: flat rgba, never gradients */
  --glass-bg:     rgba(23, 25, 29, 0.62);
  --glass-border: rgba(255, 255, 255, 0.08);
  --accent-bg:    rgba(22, 163, 74, 0.14);
  --accent-line:  rgba(22, 163, 74, 0.55);

  --radius-window: 8px;
  --radius-row: 6px;
  --font: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
}
```

## Components
| Component | Details |
|---|---|
| **TitleBar** | Pig logo (`pig.png`, 28 px) + `NLX_SEARCH` wordmark; settings, minimize, close at right. Custom controls since `decorations: false`. |
| **SearchBar** | 48 px tall, `--surface` fill, 1 px `--glass-border`; on focus the border becomes `--accent-line`. Search icon left, `Ctrl+K` chip right. Placeholder: "Search anything on your PC..." |
| **Sidebar** | 200 px wide. Items: Everything, Apps, Files, Folders, Images, Videos, Audio, divider, Recent Searches, Favorites, Settings. Lucide icons (`layout-grid`, `app-window`, `file`, `folder`, `image`, `video`, `music`, `clock`, `star`, `settings`). Selected = `--accent-bg` + 2 px left accent bar. Optional count badges. |
| **Section header** | 11 px uppercase, `--text-dim`, letter-spacing 0.06em |
| **ResultRow** | 52 px tall, horizontal. Left: 32 px icon or thumbnail. Middle: filename (`--text`) over path (`--text-dim`). Right: type, size, modified date. Selected = `--accent-bg` + 1 px `--accent-line`. |
| **Inline actions** | On hover/selection, small icon buttons (28 px) appear at the row's right edge, replacing the size/date text: Open, Open Location, Copy Path, Rename, Delete, Properties. `Alt+Enter` = Properties. |
| **Context menu** | Right-click or `Shift+F10`. Compact list, 6 px radius, `--elevated` fill. |
| **StatusBar** | 28 px. C:/D: free space, indexing state + file count, keyboard hints. |

## Keyboard
| Key | Action |
|---|---|
| `Win` / `Win+S` / `Win+Q` | Show/hide launcher (global) |
| `Ctrl+K` | Focus search bar (in window) |
| `↑ ↓` | Move selection |
| `Enter` | Open selected |
| `Ctrl+Enter` | Open location |
| `Ctrl+Shift+C` | Copy path |
| `F2` | Rename |
| `Ctrl+Delete` | Delete to Recycle Bin (confirm) |
| `Alt+Enter` | Properties |
| `Tab` / `Shift+Tab` | Cycle category filter |
| `Esc` | Clear query, then hide |

## States to design
Empty (recent items), loading (indexing), no results, error (index unavailable), delete confirmation, rename inline edit.

## Do / Don't
- ✅ Flat translucent layers, 1 px hairline borders, subtle shadow on the window only
- ✅ Single accent color, used sparingly
- ❌ Gradients, neon glows, colored borders, oversized buttons, heavily rounded cards
- ❌ Any permanent right-side panel
