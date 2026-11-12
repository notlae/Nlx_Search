export function Switch({ on, onChange, disabled }: { on: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
  return (
    <button
      className={"switch" + (on ? " on" : "")}
      aria-pressed={on}
      disabled={disabled}
      onClick={() => onChange(!on)}
    />
  );
}
