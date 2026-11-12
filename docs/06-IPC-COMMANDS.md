# 06 · IPC Commands and Events

All commands return `Result<T, String>`. Validate every path in Rust.

## Types
```ts
type Kind = "file" | "folder" | "app" | "image" | "video" | "audio";
type Filter = "all" | "apps" | "files" | "folders" | "images" | "videos" | "audio";

interface SearchResult {
  id: number;
  name: string;
  path: string;          // folder location shown under the name
  kind: Kind;
  size?: number;         // bytes
  mtime?: number;        // unix seconds
  icon?: string;         // data URL or cached icon path
  score: number;
}

interface SearchResponse {
  queryId: number;
  best?: SearchResult;
  apps: SearchResult[];
  folders: SearchResult[];
  files: SearchResult[];
  images: SearchResult[];
}
```

## Commands
| Command | Args | Behavior |
|---|---|---|
| `search` | `query, filter, queryId` | Ranked, grouped results |
| `open_item` | `path` | `ShellExecuteW "open"`; records usage |
| `open_location` | `path` | `explorer.exe /select,"<path>"` |
| `copy_path` | `path` | Writes to clipboard |
| `rename_item` | `path, newName` | Validates name, no path separators |
| `delete_item` | `path` | **Recycle Bin** only (`trash` crate), UI confirms first |
| `show_properties` | `path` | `ShellExecuteExW` with the `properties` verb |
| `run_as_admin` | `path` | Verb `runas` (apps only) |
| `toggle_favorite` | `path` | Updates `favorites` |
| `recent_searches` | none | Last 10 queries |
| `drive_info` | none | Free/total for fixed drives |
| `index_status` | none | `{ state, count }` |
| `reindex` | none | Starts a full rescan |
| `set_setting` / `get_settings` | | Autostart, hook enabled, excluded folders |
| `hide_window` | none | Hides the launcher |

## Events (Rust → UI)
| Event | Payload |
|---|---|
| `nlx://shown` | none (UI focuses and selects the input) |
| `nlx://index-status` | `{ state: "idle" \| "scanning", count: number }` |
| `nlx://drive-info` | `[{ letter: "C:", free: number, total: number }]` |

## Stale result handling
The UI increments `queryId` per keystroke and ignores responses whose `queryId` is older than the latest.
