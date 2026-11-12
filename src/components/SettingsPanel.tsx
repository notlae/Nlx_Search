import { useState } from "react";
import { api, Settings } from "../api";
import { Switch } from "./Switch";

export function SettingsPanel({ settings, onChange, flash }: {
  settings: Settings;
  onChange: (s: Settings) => void;
  flash: (m: string) => void;
}) {
  const [confirmUninstall, setConfirmUninstall] = useState(false);

  const update = async (patch: Partial<Settings>) => {
    try {
      onChange(await api.saveSettings({ ...settings, ...patch }));
    } catch (e) {
      flash(String(e));
      try { onChange(await api.getSettings()); }
      catch (reloadError) { flash(`Could not reload settings: ${String(reloadError)}`); }
    }
  };

  return (
    <div className="panel">
      <h2>Settings</h2>
      <p>Make NLX_SEARCH feel like yours. Visual changes apply immediately.</p>

      <div className="theme-card">
        <div className="theme-copy"><b>Appearance</b><small>Choose the color system for your launcher.</small></div>
        <div className="theme-options" role="group" aria-label="Choose theme">
          {(["aurora", "midnight", "neon", "light"] as const).map((theme) => (
            <button key={theme} className={`theme-choice ${settings.theme === theme ? "active" : ""}`} onClick={() => update({ theme })} aria-pressed={settings.theme === theme}>
              <span className={`theme-swatch ${theme}`} />
              <span>{({ aurora: "Aurora", midnight: "Midnight", neon: "Neon", light: "Light" } as const)[theme]}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="toggle-row">
        <div className="t"><b>Windows-key takeover</b><small>Optional global keyboard hook for Win, Win+S and Win+Q. Off by default. Restart NLX_SEARCH after changing this setting.</small></div>
        <Switch on={settings.hookEnabled} onChange={(v) => update({ hookEnabled: v })} />
      </div>
      <div className="toggle-row">
        <div className="t"><b>Start with Windows</b></div>
        <Switch on={settings.autostart} onChange={(v) => update({ autostart: v })} />
      </div>
      <div className="toggle-row">
        <div className="t"><b>Administrator startup</b><small>Disabled in this build. NLX_SEARCH does not need elevated permissions for normal search.</small></div>
        <Switch on={false} disabled onChange={() => {}} />
      </div>
      <div className="toggle-row">
        <div className="t"><b>Index my files</b></div>
        <Switch on={settings.indexEnabled} onChange={(v) => update({ indexEnabled: v })} />
      </div>

      <div className="btn-row">
        <button className="btn" onClick={async () => {
          try { await api.reindex(); flash("Reindex requested"); }
          catch (e) { flash(`Could not start indexing: ${String(e)}`); }
        }}>Reindex now</button>
        <button className="btn danger" onClick={() => setConfirmUninstall(true)}>Remove startup entries &amp; quit</button>
      </div>
      {confirmUninstall && (
        <div className="note">
          This turns the Windows-key takeover off, removes the current-user startup entry and quits.
          Windows search works normally afterwards. To fully uninstall, delete the app folder (or use Windows “Apps &amp; features”).
          <div className="btn-row">
            <button className="btn primary" onClick={async () => {
              try { await api.uninstall(); }
              catch (e) { flash(`Could not remove startup entries: ${String(e)}`); }
            }}>Yes, remove and quit</button>
            <button className="btn" onClick={() => setConfirmUninstall(false)}>Cancel</button>
          </div>
        </div>
      )}
    </div>
  );
}
