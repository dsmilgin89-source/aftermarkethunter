import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SearchProgress } from "@/lib/types";
import { cn } from "@/lib/ipc";

const PHASE_LABELS: Record<string, string> = {
  scraping: "Scraping",
  enriching: "Enriching",
  scoring: "Scoring",
  done: "Gotowe",
};

const PHASE_COLORS: Record<string, string> = {
  scraping: "bg-accent",
  enriching: "bg-blue-400",
  scoring: "bg-green-400",
  done: "bg-green-400",
};

export function ProgressBar({ visible }: { visible: boolean }) {
  const [progress, setProgress] = useState<SearchProgress | null>(null);
  const [fading, setFading] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    listen<SearchProgress>("search-progress", (event) => {
      const p = event.payload;
      if (p.phase === "done") {
        setProgress(p);
        setFading(true);
        setTimeout(() => {
          setProgress(null);
          setFading(false);
        }, 800);
      } else {
        setFading(false);
        setProgress(p);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (visible) {
      setFading(false);
    }
  }, [visible]);

  if (!visible && !progress) return null;
  if (!progress && visible) {
    return (
      <div className="rounded-md border border-border bg-surface p-4">
        <div className="flex items-center gap-3">
          <div className="h-2 w-2 animate-pulse rounded-full bg-accent" />
          <span className="text-sm text-muted">Inicjowanie wyszukiwania...</span>
        </div>
      </div>
    );
  }
  if (!progress) return null;

  const percentage =
    progress.total && progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : null;

  const phaseLabel = PHASE_LABELS[progress.phase] ?? progress.phase;
  const barColor = PHASE_COLORS[progress.phase] ?? "bg-accent";

  return (
    <div
      className={cn(
        "rounded-md border border-border bg-surface p-4 transition-opacity duration-500",
        fading && "opacity-0",
      )}
    >
      <div className="mb-2 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div
            className={cn(
              "h-2 w-2 rounded-full",
              progress.phase === "done" ? "bg-green-400" : "animate-pulse bg-accent",
            )}
          />
          <span className="text-sm font-medium text-text">{phaseLabel}</span>
        </div>
        {percentage !== null && (
          <span className="text-xs tabular-nums text-muted">{percentage}%</span>
        )}
      </div>
      <p className="mb-2 text-xs text-muted">{progress.detail}</p>
      {percentage !== null && (
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-surface-2">
          <div
            className={cn("h-full rounded-full transition-all duration-300", barColor)}
            style={{ width: `${percentage}%` }}
          />
        </div>
      )}
    </div>
  );
}
