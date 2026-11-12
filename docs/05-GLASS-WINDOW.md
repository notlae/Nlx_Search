# 05 · Glass Window (acrylic on Windows 10)

## `tauri.conf.json` window (Tauri v2)
```json
{
  "app": {
    "windows": [{
      "label": "main",
      "title": "NLX_SEARCH",
      "width": 1000,
      "height": 640,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "visible": false,
      "resizable": false,
      "center": true,
      "shadow": true
    }],
    "security": { "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'" }
  }
}
```
Check the exact key names in the Tauri v2 config reference.

## Apply acrylic (Rust)
```rust
use window_vibrancy::apply_acrylic;
use tauri::Manager;

let w = app.get_webview_window("main").unwrap();
#[cfg(target_os = "windows")]
apply_acrylic(&w, Some((23, 25, 29, 140)))  // RGBA tint matching #17191D
    .expect("acrylic unsupported");
```
- `apply_blur` is an alternative with a lighter, flatter look.
- Fall back to a solid `--bg` (opaque) if the call fails.

## Web side
```css
html, body, #root { background: transparent; margin: 0; }
.window {
  background: var(--glass-bg);          /* flat rgba, NO gradient */
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-window);
  box-shadow: 0 12px 40px rgba(0,0,0,.45);
}
```

## ⚠️ Known limits (Windows 10)
1. **Rounded corners:** acrylic fills the whole rectangular window, so CSS `border-radius` will **not** clip the blur. You'll see square blur behind round corners. Options:
   - Use small or zero radius. This is authentic Windows 10 and the easiest fix.
   - Clip with `SetWindowRgn` (rounded region), but its edges are not anti-aliased.
   - Skip native blur and render a pre-blurred wallpaper snapshot as the background (more work, full control).
2. **Drag/resize lag:** `window-vibrancy` documents performance issues for acrylic on Win10 1903+ during move/resize. The launcher is fixed-size and not dragged, so this is low risk.
3. **Win10 version:** acrylic needs a fairly recent build (see the crate README for the minimum).
4. **Transparency + WebView2:** make sure no element paints an opaque white background (the default `<body>`).
5. The mockups' rounded, glassy look is closer to Windows 11. On Windows 10, expect square-ish glass.

## Show/hide behavior
- Hide on blur (`WindowEvent::Focused(false)`), except while a context menu is open.
- Show: center on the monitor containing the cursor, then `show()` and `set_focus()`.
- Keep the WebView alive (hide, don't destroy) for instant open.
