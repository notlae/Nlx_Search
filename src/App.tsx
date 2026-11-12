import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  AppWindow, Clock, File, Folder, Image, LayoutGrid, Minus, Music, Search, Settings as Cog, Star, Video, X,
} from "lucide-react";
import { api, DriveInfo, Filter, IndexStatus, Item, SearchResponse, Settings } from "./api";
import pig from "./assets/pig.png";
import { ActionName, Row } from "./components/Row";
import { SettingsPanel } from "./components/SettingsPanel";
import { Onboarding } from "./components/Onboarding";

type View = "search" | "recent" | "favorites" | "settings";
type Section = { title: string; items: Item[] };

const FILTERS: { id: Filter; label: string; icon: JSX.Element }[] = [
  { id: "all", label: "Everything", icon: <LayoutGrid size={17} /> },
  { id: "apps", label: "Apps", icon: <AppWindow size={17} /> },
  { id: "files", label: "Files", icon: <File size={17} /> },
  { id: "folders", label: "Folders", icon: <Folder size={17} /> },
  { id: "images", label: "Images", icon: <Image size={17} /> },
  { id: "videos", label: "Videos", icon: <Video size={17} /> },
  { id: "audio", label: "Audio", icon: <Music size={17} /> },
];

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  useEffect(() => { api.getSettings().then(setSettings).catch(() => {}); }, []);

  if (!settings) return <div className="window" />;
  if (!settings.firstRunDone) return <Onboarding onDone={setSettings} />;
  return <Launcher settings={settings} onSettings={setSettings} />;
}

function Launcher({ settings, onSettings }: { settings: Settings; onSettings: (s: Settings) => void }) {
  const [view, setView] = useState<View>("search");
  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");
  const [resp, setResp] = useState<SearchResponse | null>(null);
  const [favs, setFavs] = useState<Item[]>([]);
  const [sel, setSel] = useState(0);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<Item | null>(null);
  const [toast, setToast] = useState("");
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [status, setStatus] = useState<IndexStatus>({ state: "idle", count: 0 });
  const inputRef = useRef<HTMLInputElement>(null);
  const qid = useRef(0);
  const toastTimer = useRef<number | undefined>(undefined);

  const flash = useCallback((m: string) => {
    setToast(m);
    window.clearTimeout(toastTimer.current);
    toastTimer.current = window.setTimeout(() => setToast(""), 2800);
  }, []);

  const refresh = useCallback(async () => {
    if (view === "settings") return;
    if (view === "favorites") { setFavs(await api.favorites()); return; }
    const q = view === "search" ? query : "";
    const id = ++qid.current;
    const r = await api.search(q, filter, id);
    if (r.queryId === qid.current) setResp(r);
  }, [view, query, filter]);

  const refreshRef = useRef(refresh);
  refreshRef.current = refresh;

  useEffect(() => {
    const t = window.setTimeout(() => { refresh().catch((e) => flash(String(e))); }, 25);
    return () => window.clearTimeout(t);
  }, [refresh, flash]);

  useEffect(() => { setSel(0); }, [query, filter, view]);

  // events from the Rust side
  useEffect(() => {
    const subs: Promise<UnlistenFn>[] = [
      listen("nlx://shown", () => {
        setView("search");
        setRenaming(null);
        inputRef.current?.focus();
        inputRef.current?.select();
        api.drives().then(setDrives).catch(() => {});
        refreshRef.current().catch(() => {});
      }),
      listen<IndexStatus>("nlx://index-status", (e) => setStatus(e.payload)),
    ];
    api.drives().then(setDrives).catch(() => {});
    api.indexStatus().then(setStatus).catch(() => {});
    return () => { subs.forEach((p) => p.then((f) => f())); };
  }, []);

  // keep the window open while a dialog / rename is active
  useEffect(() => { api.setSticky(confirm !== null || renaming !== null).catch(() => {}); }, [confirm, renaming]);

  const sections: Section[] = useMemo(() => {
    if (view === "favorites") return [{ title: "Favorites", items: favs }];
    if (!resp) return [];
    if (view === "recent" || !query.trim()) return [{ title: "Recent", items: resp.recent }];
    const s: Section[] = [];
    if (resp.best) s.push({ title: "Best match", items: [resp.best] });
    if (resp.apps.length) s.push({ title: "Applications", items: resp.apps });
    if (resp.folders.length) s.push({ title: "Folders", items: resp.folders });
    if (resp.files.length) s.push({ title: "Files", items: resp.files });
    if (resp.images.length) s.push({ title: "Images", items: resp.images });
    return s;
  }, [view, resp, favs, query]);

  const flat = useMemo(() => sections.flatMap((s) => s.items), [sections]);

  useEffect(() => {
    document.querySelector(".row.selected")?.scrollIntoView({ block: "nearest" });
  }, [sel, flat]);

  const act = useCallback(async (a: ActionName, it: Item | undefined) => {
    if (!it) return;
    try {
      switch (a) {
        case "open": await api.open(it.path); await api.hide(); break;
        case "location": await api.openLocation(it.path); await api.hide(); break;
        case "copy": await navigator.clipboard.writeText(it.path); flash("Path copied"); break;
        case "rename": if (it.kind !== "app") setRenaming(it.path); break;
        case "delete": if (it.kind !== "app") setConfirm(it); break;
        case "props": await api.properties(it.path); break;
        case "admin": await api.runAsAdmin(it.path); await api.hide(); break;
        case "fav": await api.toggleFavorite(it.path); await refresh(); break;
      }
    } catch (e) {
      flash(String(e));
    }
  }, [flash, refresh]);

  const doDelete = useCallback(async () => {
    if (!confirm) return;
    const it = confirm;
    setConfirm(null);
    try {
      await api.remove(it.path);
      flash("Moved to Recycle Bin");
      await refresh();
    } catch (e) {
      flash(String(e));
    }
  }, [confirm, flash, refresh]);

  const commitRename = async (it: Item, name: string) => {
    setRenaming(null);
    if (name.trim() === it.name) return;
    try {
      await api.rename(it.path, name);
      await refresh();
    } catch (e) {
      flash(String(e));
    }
    inputRef.current?.focus();
  };

  // global keyboard navigation (inside the window)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (confirm) {
        if (e.key === "Escape") setConfirm(null);
        if (e.key === "Enter") doDelete();
        return;
      }
      if (renaming !== null) return; // the rename input handles its own keys
      const target = e.target as HTMLElement | null;
      const editing = !!target && (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName));
      const appShortcut = ["ArrowDown", "ArrowUp", "Enter", "Escape", "Tab", "F2"].includes(e.key)
        || (e.ctrlKey && (e.key.toLowerCase() === "k" || (e.shiftKey && e.key.toLowerCase() === "c") || e.key === "Delete"));
      if (editing && !appShortcut) return;
      const it = flat[sel];
      const k = e.key;
      if (k === "ArrowDown") { e.preventDefault(); setSel((s) => Math.min(s + 1, Math.max(flat.length - 1, 0))); }
      else if (k === "ArrowUp") { e.preventDefault(); setSel((s) => Math.max(s - 1, 0)); }
      else if (k === "Enter") {
        e.preventDefault();
        if (e.ctrlKey) act("location", it);
        else if (e.altKey) act("props", it);
        else act("open", it);
      }
      else if (k === "Tab" && (view === "search" || view === "recent")) {
        e.preventDefault();
        const i = FILTERS.findIndex((f) => f.id === filter);
        const n = (i + (e.shiftKey ? -1 : 1) + FILTERS.length) % FILTERS.length;
        setFilter(FILTERS[n].id);
      }
      else if (e.ctrlKey && k.toLowerCase() === "k") { e.preventDefault(); inputRef.current?.focus(); inputRef.current?.select(); }
      else if (e.ctrlKey && e.shiftKey && k.toLowerCase() === "c") { e.preventDefault(); act("copy", it); }
      else if (k === "F2") { e.preventDefault(); act("rename", it); }
      else if (e.ctrlKey && k === "Delete") { e.preventDefault(); act("delete", it); }
      else if (k === "Escape") {
        e.preventDefault();
        if (view !== "search") setView("search");
        else if (query) setQuery("");
        else api.hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [confirm, renaming, flat, sel, view, filter, query, act, doDelete]);

  const nav = (id: string, label: string, icon: JSX.Element, active: boolean, onClick: () => void) => (
    <button key={id} className={"nav" + (active ? " active" : "")} onClick={onClick}>{icon}<span>{label}</span></button>
  );

  let idx = -1;
  return (
    <div className="window" data-theme={settings.theme || "aurora"}>
      <div className="topbar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <img src={pig} alt="" />
          <span>NLX_<b>SEARCH</b></span>
        </div>
        <div className="searchbar">
          <Search size={18} color="var(--text-dim)" />
          <input
            ref={inputRef}
            autoFocus
            value={query}
            placeholder="Search anything on your PC..."
            spellCheck={false}
            onChange={(e) => { setQuery(e.target.value); if (view !== "search") setView("search"); }}
          />
          <span className="kbd">Ctrl + K</span>
        </div>
        <div className="winbtns">
          <button className="iconbtn" title="Settings" onClick={() => setView("settings")}><Cog size={17} /></button>
          <button className="iconbtn" title="Hide" onClick={() => api.hide()}><Minus size={17} /></button>
          <button className="iconbtn danger" title="Close (keeps running in the tray)" onClick={() => api.hide()}><X size={17} /></button>
        </div>
      </div>

      <div className="body">
        <div className="sidebar">
          {FILTERS.map((f) => nav(f.id, f.label, f.icon, view === "search" && filter === f.id, () => { setView("search"); setFilter(f.id); }))}
          <div className="divider" />
          {nav("recent", "Recent", <Clock size={17} />, view === "recent", () => setView("recent"))}
          {nav("fav", "Favorites", <Star size={17} />, view === "favorites", () => setView("favorites"))}
          {nav("settings", "Settings", <Cog size={17} />, view === "settings", () => setView("settings"))}
        </div>

        {view === "settings" ? (
          <SettingsPanel settings={settings} onChange={onSettings} flash={flash} />
        ) : (
          <div className="results">
            {flat.length === 0 && (
              <div className="empty">
                {query.trim() && view === "search"
                  ? "No results"
                  : view === "favorites"
                  ? "No favorites yet. Hover a result and click the star."
                  : "Type to search. Things you open will show up here."}
              </div>
            )}
            {sections.map((s) => s.items.length > 0 && (
              <div key={s.title}>
                <div className="section-title">{s.title}</div>
                {s.items.map((it) => {
                  idx += 1;
                  const my = idx;
                  return (
                    <Row
                      key={it.path}
                      it={it}
                      selected={my === sel}
                      renaming={renaming === it.path}
                      onSelect={() => setSel(my)}
                      onAct={(a) => { setSel(my); act(a, it); }}
                      onRenameCommit={(n) => commitRename(it, n)}
                      onRenameCancel={() => { setRenaming(null); inputRef.current?.focus(); }}
                    />
                  );
                })}
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="status">
        <div className="grp">
          {drives.slice(0, 3).map((d) => (<span key={d.letter}>{d.letter} {Math.round(d.free / 1e9)} GB free</span>))}
        </div>
        <div className="grp">
          <span><i className={"dot" + (status.state === "scanning" ? " busy" : "")} />
            {status.state === "scanning" ? "Indexing" : "Indexed"} {status.count.toLocaleString()} items</span>
          <span>↑↓ navigate · Enter open · Ctrl+Enter location · Tab filter · Esc close</span>
        </div>
      </div>

      {confirm && (
        <div className="overlay">
          <div className="dialog">
            <h3>Move to Recycle Bin?</h3>
            <p>{confirm.path}</p>
            <div className="btn-row">
              <button className="btn primary" onClick={doDelete}>Move to Recycle Bin</button>
              <button className="btn" onClick={() => setConfirm(null)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}
