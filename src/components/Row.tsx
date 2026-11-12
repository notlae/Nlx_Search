import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { FolderOpen, Copy, Pencil, Trash2, Info, ShieldCheck, Star, Play, File, Folder, Image, Video, Music } from "lucide-react";
import { api, fmtDate, fmtSize, Item, kindLabel } from "../api";

export type ActionName = "open" | "location" | "copy" | "rename" | "delete" | "props" | "admin" | "fav";

function KindIcon({ k }: { k: Item["kind"] }) {
  const p = { size: 18 };
  switch (k) {
    case "folder": return <Folder {...p} />;
    case "image": return <Image {...p} />;
    case "video": return <Video {...p} />;
    case "audio": return <Music {...p} />;
    default: return <File {...p} />;
  }
}

export function Row({ it, selected, renaming, onSelect, onAct, onRenameCommit, onRenameCancel }: {
  it: Item;
  selected: boolean;
  renaming: boolean;
  onSelect: () => void;
  onAct: (a: ActionName) => void;
  onRenameCommit: (name: string) => void;
  onRenameCancel: () => void;
}) {
  const [thumb, setThumb] = useState<string | null>(null);
  const [draft, setDraft] = useState(it.name);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (it.kind !== "image") return;
    let alive = true;
    api.thumb(it.path).then((t) => { if (alive) setThumb(t); }).catch(() => {});
    return () => { alive = false; };
  }, [it.path, it.kind]);

  useEffect(() => {
    if (!renaming) return;
    setDraft(it.name);
    const timer = window.setTimeout(() => {
      const el = inputRef.current;
      if (!el) return;
      el.focus();
      const dot = it.name.lastIndexOf(".");
      el.setSelectionRange(0, dot > 0 && it.kind !== "folder" ? dot : it.name.length);
    }, 0);
    return () => window.clearTimeout(timer);
  }, [renaming, it.name, it.kind]);

  const isApp = it.kind === "app";
  const isStore = it.path.startsWith("shell:");
  const location = isApp ? (isStore ? "Installed app" : "Start Menu") : it.parent;

  const btn = (a: ActionName, title: string, icon: ReactNode, cls = "") => (
    <button key={a} className={"iconbtn " + cls} title={title} onClick={(e) => { e.stopPropagation(); onAct(a); }}>{icon}</button>
  );

  return (
    <div className={"row" + (selected ? " selected" : "")} onClick={onSelect} onDoubleClick={() => onAct("open")}>
      <div className={"icon" + (isApp ? " app" : "")}>
        {isApp ? it.name.charAt(0).toUpperCase() : thumb ? <img src={thumb} alt="" /> : <KindIcon k={it.kind} />}
      </div>

      <div className="text">
        {renaming ? (
          <input
            ref={inputRef}
            className="rename"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              e.stopPropagation();
              if (e.key === "Enter") onRenameCommit(draft);
              if (e.key === "Escape") onRenameCancel();
            }}
            onBlur={(e) => {
              // Let Enter commit first; blur only cancels when focus genuinely leaves the row.
              if (!e.currentTarget.parentElement?.parentElement?.contains(e.relatedTarget as Node | null)) onRenameCancel();
            }}
          />
        ) : (
          <div className="name">{it.name}</div>
        )}
        <div className="sub">{location}</div>
      </div>

      <div className="meta">
        <span className="w">{kindLabel(it)}</span>
        {!isApp && <span className="w">{fmtSize(it.size)}</span>}
        {!isApp && <span className="w">{fmtDate(it.mtime)}</span>}
      </div>

      <div className="actions">
        {btn("open", "Open (Enter)", <Play size={15} />)}
        {isApp && !isStore && btn("admin", "Run as administrator", <ShieldCheck size={15} />)}
        {!isApp && btn("location", "Open location (Ctrl+Enter)", <FolderOpen size={15} />)}
        {!isApp && btn("copy", "Copy path (Ctrl+Shift+C)", <Copy size={15} />)}
        {!isApp && btn("rename", "Rename (F2)", <Pencil size={15} />)}
        {!isApp && btn("delete", "Delete to Recycle Bin (Ctrl+Delete)", <Trash2 size={15} />)}
        {!isApp && btn("props", "Properties (Alt+Enter)", <Info size={15} />)}
        {btn("fav", it.fav ? "Remove favorite" : "Add favorite", <Star size={15} fill={it.fav ? "currentColor" : "none"} />, it.fav ? "fav" : "")}
      </div>
    </div>
  );
}
