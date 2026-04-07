import { useEffect, useState } from "react";
import { cn } from "@/lib/ipc";

/** Renders remaining time to an auction end. Cyan on comfort, red when urgent. */
export function Countdown({ endsAt }: { endsAt?: string | null }) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, []);

  if (!endsAt) return <span className="text-subtle">—</span>;
  const end = new Date(endsAt).getTime();
  const diff = end - now;
  if (diff <= 0) return <span className="text-subtle">ended</span>;

  const s = Math.floor(diff / 1000);
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;

  const urgent = diff < 3600_000;
  const text = d > 0 ? `${d}d ${h}h` : h > 0 ? `${h}h ${m}m` : `${m}m ${sec}s`;

  return (
    <span
      className={cn(
        "font-mono tabular-nums text-sm",
        urgent ? "text-danger" : "text-text",
      )}
    >
      {text}
    </span>
  );
}
