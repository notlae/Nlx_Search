import { useEffect, useState } from "react";
import { api, Settings } from "../api";
import pig from "../assets/pig.png";
import { Switch } from "./Switch";

export function Onboarding({ onDone }: { onDone: (s: Settings) => void }) {
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState({ hookEnabled: false, autostart: false, indexEnabled: true, adminMode: false });
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");

  // the window must not hide itself while we are talking to the user
  useEffect(() => {
    api.setSticky(true);
    return () => { api.setSticky(false); };
  }, []);


  const enable = async () => {
    setBusy(true);
    setErr("");
    try {
      const cur = await api.getSettings();
      await api.saveSettings({ ...cur, ...draft, firstRunDone: true });
    } catch (e) {
      setErr(String(e)); // e.g. the user declined the administrator prompt
    }
    const s = await api.getSettings();
    setBusy(false);
    if (s.firstRunDone) setStep(2);
  };

  const skip = async () => {
    setBusy(true);
    try {
      const cur = await api.getSettings();
      await api.saveSettings({ ...cur, hookEnabled: false, autostart: false, indexEnabled: true, adminMode: false, firstRunDone: true });
    } catch (e) {
      setErr(String(e));
    }
    const s = await api.getSettings();
    setBusy(false);
    if (s.firstRunDone) setStep(2);
  };

  const finish = async () => {
    const s = await api.getSettings();
    await api.setSticky(false);
    onDone(s);
    await api.hide();
  };

  return (
    <div className="window" data-theme="aurora">
      <div className="onboard" data-tauri-drag-region>
        <div className="logo">
          <img src={pig} alt="" />
          <h1>NLX_<b>SEARCH</b></h1>
        </div>
        <div className="steps"><i className="on" /><i className={step >= 1 ? "on" : ""} /><i className={step >= 2 ? "on" : ""} /></div>

        {step === 0 && (
          <>
            <h2>Search your whole PC, instantly.</h2>
            <p>
              NLX_SEARCH is a fast, local search launcher. It builds a private index of your apps, files and folders on this computer. Keyboard takeover is optional and starts disabled.
            </p>
            <div className="note">Everything stays on your PC. NLX_SEARCH makes no network connections and has no telemetry.</div>
            <div className="btn-row"><button className="btn primary" onClick={() => setStep(1)}>Continue</button></div>
          </>
        )}

        {step === 1 && (
          <>
            <h2>Choose your setup</h2>
            <p>Safe defaults are enabled. Keyboard takeover is optional and requires restarting NLX_SEARCH after you turn it on.</p>

            <div className="toggle-row">
              <div className="t"><b>Open with the Windows key, Win+S and Win+Q</b>
                <small>Installs a global keyboard hook for Win, Win+S and Win+Q. Off by default. Restart the app after enabling it.</small></div>
              <Switch on={draft.hookEnabled} onChange={(v) => setDraft({ ...draft, hookEnabled: v })} />
            </div>
            <div className="toggle-row">
              <div className="t"><b>Start with Windows</b><small>Runs quietly in the tray at sign-in.</small></div>
              <Switch on={draft.autostart} onChange={(v) => setDraft({ ...draft, autostart: v, adminMode: v ? draft.adminMode : false })} />
            </div>
            <div className="toggle-row">
              <div className="t"><b>Index my files</b><small>Builds a local search index. Needed to find files and folders.</small></div>
              <Switch on={draft.indexEnabled} onChange={(v) => setDraft({ ...draft, indexEnabled: v })} />
            </div>
            <div className="note">
              NLX_SEARCH does not record keystrokes or send your search index anywhere. You can restore normal Windows search by turning keyboard takeover off in Settings and restarting.
            </div>
            {err && <div className="err">{err}</div>}
            <div className="btn-row">
              <button className="btn primary" disabled={busy} onClick={enable}>{busy ? "Working…" : "Enable"}</button>
              <button className="btn" disabled={busy} onClick={skip}>Not now (keep Windows search unchanged)</button>
            </div>
          </>
        )}

        {step === 2 && (
          <>
            <h2>Setup complete ✨</h2>
            <p>Open NLX_SEARCH from the Start menu or system tray. If you enabled keyboard takeover, restart the app to activate it.</p>
            {err && <div className="err">{err}</div>}
            <div className="btn-row"><button className="btn primary" onClick={finish}>Finish</button></div>
          </>
        )}
      </div>
    </div>
  );
}
