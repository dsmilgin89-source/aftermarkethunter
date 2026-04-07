import { ExternalLink, Star, StarOff } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { CebulaThresholds, ResultRow } from "@/lib/types";
import { ScoreBadge } from "@/components/ScoreBadge";
import { cn } from "@/lib/ipc";

const MAX_DEALS = 5;

export function isCebulaDeal(
  row: ResultRow,
  t: CebulaThresholds,
): boolean {
  if (row.score.total < t.minScore) return false;
  const price = row.listing.current_price ?? row.listing.buy_now_price;
  if (price == null || price > t.maxPrice) return false;
  if (t.minAge > 0 && (row.enrichment.age_years ?? 0) < t.minAge) return false;
  if (t.minWayback > 0 && (row.enrichment.wayback_snapshots ?? 0) < t.minWayback)
    return false;
  if (t.noBlacklist && row.enrichment.blacklist_hits > 0) return false;
  if (t.noTrademark && row.enrichment.trademark_warning) return false;
  return true;
}

export function CebulaDeals({
  rows,
  thresholds,
  watchedIds,
  onToggleWatch,
}: {
  rows: ResultRow[];
  thresholds: CebulaThresholds;
  watchedIds: Set<string>;
  onToggleWatch: (row: ResultRow) => void;
}) {
  const deals = rows.filter((r) => isCebulaDeal(r, thresholds)).slice(0, MAX_DEALS);

  if (deals.length === 0) return null;

  return (
    <section className="space-y-3">
      <div className="flex items-center gap-2">
        <span className="text-lg" role="img" aria-label="onion">
          🧅
        </span>
        <h2 className="text-base font-medium text-text">Cebula Deals</h2>
        <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-xs font-medium text-amber-400">
          {deals.length}
        </span>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
        {deals.map((row) => {
          const watched = watchedIds.has(row.listing.id);
          const price = row.listing.current_price ?? row.listing.buy_now_price;
          return (
            <div
              key={row.listing.id}
              className="space-y-2 rounded-lg border border-amber-500/25 bg-gradient-to-b from-amber-500/5 to-transparent p-4"
            >
              <div className="flex items-start justify-between">
                <span className="flex-1 truncate text-sm font-semibold text-text">
                  {row.listing.domain}
                </span>
                <ScoreBadge tier={row.score.tier} total={row.score.total} />
              </div>
              <div className="flex items-center gap-3 text-xs text-muted">
                {price != null && (
                  <span className="font-medium text-amber-400">
                    {price.toLocaleString("pl-PL")} {row.listing.currency}
                  </span>
                )}
                {row.enrichment.age_years != null && (
                  <span>{row.enrichment.age_years.toFixed(1)}y</span>
                )}
                {row.enrichment.wayback_snapshots != null && (
                  <span>WB: {row.enrichment.wayback_snapshots}</span>
                )}
              </div>
              <div className="flex items-center gap-1 pt-1">
                <button
                  onClick={() => onToggleWatch(row)}
                  className={cn(
                    "rounded p-1 text-xs hover:bg-surface-2",
                    watched ? "text-accent" : "text-subtle",
                  )}
                  title={watched ? "Usuń z watchlisty" : "Dodaj do watchlisty"}
                >
                  {watched ? (
                    <Star className="h-3.5 w-3.5 fill-current" />
                  ) : (
                    <StarOff className="h-3.5 w-3.5" />
                  )}
                </button>
                <button
                  onClick={() => openUrl(row.listing.url)}
                  className="rounded p-1 text-xs text-subtle hover:bg-surface-2 hover:text-text"
                  title="Otwórz w przeglądarce"
                >
                  <ExternalLink className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
