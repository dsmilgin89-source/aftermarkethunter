import type { Score } from "@/lib/types";

/** Hover-card content explaining why a domain scored what it scored. */
export function ScoreBreakdown({ score }: { score: Score }) {
  return (
    <div className="w-80 rounded-md border border-border bg-surface p-4 shadow-xl">
      <div className="mb-3 flex items-baseline justify-between">
        <h4 className="text-sm font-medium text-text">Why this score?</h4>
        <span className="font-mono text-2xl font-semibold text-text">
          {Math.round(score.total)}
        </span>
      </div>
      <dl className="space-y-2 text-xs">
        <Row label="SEO" value={score.seo} />
        <Row label="Relevance" value={score.relevance} />
        <Row label="Value" value={score.value} />
        <Row label="Risk penalty" value={-score.risk_penalty} danger />
      </dl>
      {score.explanation.length > 0 && (
        <ul className="mt-3 space-y-1 border-t border-border pt-3 text-xs text-muted">
          {score.explanation.map((line, i) => (
            <li key={i}>• {line}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

function Row({
  label,
  value,
  danger,
}: {
  label: string;
  value: number;
  danger?: boolean;
}) {
  const pct = Math.max(0, Math.min(100, Math.abs(value)));
  return (
    <div>
      <div className="mb-0.5 flex items-center justify-between">
        <dt className="text-muted">{label}</dt>
        <dd
          className={`font-mono tabular-nums ${danger ? "text-danger" : "text-text"}`}
        >
          {Math.round(value)}
        </dd>
      </div>
      <div className="h-1 overflow-hidden rounded-sm bg-white/5">
        <div
          className={`h-full ${danger ? "bg-danger" : "bg-accent"}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
