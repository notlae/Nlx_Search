# 10 · Corrected Mockup Prompt

## Why the last three mockups failed
The image generator **ignored** the "no right-side panel" rule: all three outputs still had a CapCut preview column (icon, Open, Open Location, Details). Causes:
- The rule was stated as a negative. Generators handle negatives poorly.
- It was buried among many other requirements.
- Reference-style UIs (Spotlight/Windows 11) usually have a detail panel, so the model fell back to it.

## Fix
State the layout **positively and first**, and describe the right edge explicitly. Generate several variants and pick the correct one. Alternatively, skip image generation and build the real UI directly (docs 02 and 08).

## Prompt (copy-paste)
```
Windows 10 desktop screenshot, 16:9. A single floating launcher window named NLX_SEARCH
with TWO COLUMNS ONLY: a narrow left navigation sidebar, and one wide results area that
extends all the way to the right edge of the window. The results rows run the full width
of the window; the right edge of every row ends at the window border. Nothing else sits
to the right of the results list.

Top bar: pig logo (use the attached pig.png) + "NLX_SEARCH" wordmark at left, a large
search bar with placeholder "Search anything on your PC..." and a "Ctrl + K" chip,
settings/minimize/close at far right.

Left sidebar items with simple line icons: Everything (selected, subtle green #16A34A
tint), Apps, Files, Folders, Images, Videos, Audio, Recent Searches, Favorites, Settings.

Results for the query "CapCut", full-width horizontal rows grouped under small headers:
Best Match (CapCut app, selected with thin green border; small inline buttons Open,
Open Location, Copy Path appear at the right end of this row), Applications (CapCut),
Folders (CapCut Drafts, D:\CapCut Drafts), Files (capcut_project.cproj, D:\Projects\CapCut,
2.4 MB, 12/4/2024), Images (capcut_logo.png with thumbnail, capcut_screenshot.png).
Each row: icon/thumbnail, filename, path, type, size, modified date.

Bottom status bar: C: and D: free space, "Indexing 24,871 files", keyboard hints
"↑↓ navigate  Enter open  Esc close".

Style: dark translucent frosted glass, wallpaper faintly visible through blur, thin
translucent border, soft shadow. Flat solid colors only: background #17191D, surfaces
#202329, elevated #292D34, accent #16A34A, text #F5F5F5, secondary text #A1A1AA.
Segoe UI typography, Windows 10 window controls, crisp realistic UI, premium and minimal.
Flat fills only, no gradients, no neon.
```

## Tips
- If a panel still appears, add: *"The window is exactly two sections wide: sidebar and results."*
- Ask for a **crop of the right half** as a second image to confirm there is no panel.
- Iterate with image-edit: "remove the right column and stretch the results to the window edge."
- Never accept an AI mockup as a spec. The Markdown docs in this folder are the spec.
