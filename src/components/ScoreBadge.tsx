import type { ScoreTier } from "@/lib/types";
import { cn } from "@/lib/ipc";

const STYLES: Record<ScoreTier, string> = {
  excellent: "bg-accent-soft text-accent border-accent/40",
  good: "bg-white/10 text-text border-white/20",
  fair: "bg-white/[0.04] text-muted border-white/10",
  poor: "bg-transparent text-subtle border-white/10",
};

const LABELS: Record<ScoreTier, string> = {
  excellent: "EXC",
  good: "GOOD",
  fair: "FAIR",
  poor: "POOR",
};

export function ScoreBadge({
  tier,
  total,
  showValue = true,
}: {
  tier: ScoreTier;
  total: number;
  showValue?: boolean;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-sm border px-2 py-0.5 text-xs font-medium",
        STYLES[tier],
      )}
    >
      {showValue && (
        <span className="font-mono tabular-nums">{Math.round(total)}</span>
      )}
      <span className="text-[10px] uppercase tracking-wider">
        {LABELS[tier]}
      </span>
    </span>
  );
}
