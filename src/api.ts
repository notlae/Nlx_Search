import { invoke } from "@tauri-apps/api/core";

export type Kind = "file" | "folder" | "app" | "image" | "video" | "audio";
export type Filter = "all" | "apps" | "files" | "folders" | "images" | "videos" | "audio";

export interface Item {
  id: number;
  name: string;
  path: string;
  parent: string;
  kind: Kind;
  size: number | null;
  mtime: number | null;
  fav: boolean;
  score: number;
}

export interface SearchResponse {
  queryId: number;
  best: Item | null;
  apps: Item[];
  folders: Item[];
  files: Item[];
  images: Item[];
  recent: Item[];
}

export interface Settings {
  firstRunDone: boolean;
  hookEnabled: boolean;
  autostart: boolean;
  indexEnabled: boolean;
  adminMode: boolean;
  excludedPaths: string[];
  theme: "aurora" | "midnight" | "neon" | "light";
}

export interface DriveInfo { letter: string; free: number; total: number }
export interface IndexStatus { state: "idle" | "scanning"; count: number }

export const api = {
  search: (query: string, filter: Filter, queryId: number) =>
    invoke<SearchResponse>("search", { query, filter, queryId }),
  favorites: () => invoke<Item[]>("list_favorites"),
  open: (path: string) => invoke<void>("open_item", { path }),
  openLocation: (path: string) => invoke<void>("open_location", { path }),
  runAsAdmin: (path: string) => invoke<void>("run_as_admin", { path }),
  properties: (path: string) => invoke<void>("show_properties", { path }),
  remove: (path: string) => invoke<void>("delete_item", { path }),
  rename: (path: string, newName: string) => invoke<void>("rename_item", { path, newName }),
  toggleFavorite: (path: string) => invoke<boolean>("toggle_favorite", { path }),
  thumb: (path: string) => invoke<string | null>("get_thumb", { path }),
  drives: () => invoke<DriveInfo[]>("drive_info"),
  indexStatus: () => invoke<IndexStatus>("index_status"),
  reindex: () => invoke<void>("reindex"),
  hide: () => invoke<void>("hide_window"),
  setSticky: (sticky: boolean) => invoke<void>("set_sticky", { sticky }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { new: settings }),
  uninstall: () => invoke<void>("uninstall_cleanup"),
};

export function fmtSize(n: number | null): string {
  if (n == null) return "";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v >= 10 || i === 0 ? Math.round(v) : v.toFixed(1)} ${u[i]}`;
}

export function fmtDate(t: number | null): string {
  if (t == null) return "";
  return new Date(t * 1000).toLocaleDateString();
}

export function kindLabel(i: Item): string {
  switch (i.kind) {
    case "app": return "Application";
    case "folder": return "Folder";
    case "image": return "Image";
    case "video": return "Video";
    case "audio": return "Audio";
    default: {
      const dot = i.name.lastIndexOf(".");
      return dot > 0 ? `${i.name.slice(dot + 1).toUpperCase()} file` : "File";
    }
  }
}
