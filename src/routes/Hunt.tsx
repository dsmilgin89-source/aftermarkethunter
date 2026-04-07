import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Download, Save, Sparkles, AlertTriangle } from "lucide-react";
import { load } from "@tauri-apps/plugin-store";
import {
  DEFAULT_QUERY,
  DEFAULT_CEBULA_THRESHOLDS,
  type Query,
  type ResultRow,
  type CebulaThresholds,
} from "@/lib/types";
import { ipc } from "@/lib/ipc";
import { SearchBar } from "@/components/SearchBar";
import { ResultsTable } from "@/components/ResultsTable";
import { ProgressBar } from "@/components/ProgressBar";
import { CebulaDeals } from "@/components/CebulaDeals";

const RECOMMENDATION_LIMIT = 10;

export function HuntView() {
  const qc = useQueryClient();
  const [query, setQuery] = useState<Query>({ ...DEFAULT_QUERY });
  const [rows, setRows] = useState<ResultRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  const [cebulaThresholds, setCebulaThresholds] = useState<CebulaThresholds>({
    ...DEFAULT_CEBULA_THRESHOLDS,
  });
  const [openpagerankKey, setOpenpagerankKey] = useState("");

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json");
        const stored = await store.get<CebulaThresholds>("cebula");
        if (stored) setCebulaThresholds(stored);
        const oprKey = (await store.get<string>("apiKeys.openpagerank")) ?? "";
        setOpenpagerankKey(oprKey);
      } catch {
        // plugin not ready
      }
    })();
  }, []);

  const watchlistQ = useQuery({
    queryKey: ["watchlist"],
    queryFn: () => ipc.listWatchlist(),
  });
  const watchedIds = useMemo(
    () => new Set(watchlistQ.data?.map((w) => w.listing_id) ?? []),
    [watchlistQ.data],
  );

  const searchM = useMutation({
    mutationFn: (q: Query) => ipc.search(q, openpagerankKey),
    onSuccess: (data) => {
      setRows(data);
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const toggleWatchM = useMutation({
    mutationFn: async (row: ResultRow) => {
      if (watchedIds.has(row.listing.id)) {
        await ipc.removeFromWatchlist(row.listing.id);
      } else {
        await ipc.addToWatchlist(row.listing.id, null, null);
      }
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["watchlist"] }),
  });

  const saveSearchM = useMutation({
    mutationFn: () =>
      ipc.saveSearch(
        query.phrase || "Bez nazwy",
        query,
        true,
      ),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["savedSearches"] }),
  });

  async function exportCsv() {
    const csv = await ipc.exportResultsCsv(rows);
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `aftermarket-hunter-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  const recommendations = rows.slice(0, RECOMMENDATION_LIMIT);

  return (
    <div className="mx-auto max-w-[1400px] space-y-10 p-10">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Polowanie
        </h1>
        <p className="text-sm text-muted">
          Wpisz frazę lub nazwę. Skanujemy aftermarket.pl, wzbogacamy każdą
          domenę danymi SEO (Wayback, WHOIS, PageRank, DNS, Similarweb)
          i scorujemy pod wybrany profil.
        </p>
      </header>

      <SearchBar
        value={query}
        onChange={setQuery}
        onSubmit={() => searchM.mutate(query)}
        loading={searchM.isPending}
      />

      <ProgressBar visible={searchM.isPending} />

      {error && (
        <div className="flex items-start gap-2 rounded-md border border-danger/30 bg-danger/10 p-3 text-sm text-danger">
          <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0" />
          <div>{error}</div>
        </div>
      )}

      {rows.length === 0 && !searchM.isPending && !error && <EmptyState />}

      {rows.length > 0 && (
        <>
          <CebulaDeals
            rows={rows}
            thresholds={cebulaThresholds}
            watchedIds={watchedIds}
            onToggleWatch={(r) => toggleWatchM.mutate(r)}
          />

          <section className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-accent" />
                <h2 className="text-base font-medium text-text">
                  Top rekomendacje
                </h2>
                <span className="text-xs text-subtle">
                  ({recommendations.length})
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => saveSearchM.mutate()}
                  className="flex items-center gap-1.5 rounded-sm border border-border bg-surface px-2.5 py-1.5 text-xs text-muted hover:text-text"
                >
                  <Save className="h-3.5 w-3.5" />
                  Zapisz query
                </button>
                <button
                  onClick={exportCsv}
                  className="flex items-center gap-1.5 rounded-sm border border-border bg-surface px-2.5 py-1.5 text-xs text-muted hover:text-text"
                >
                  <Download className="h-3.5 w-3.5" />
                  Eksport CSV
                </button>
              </div>
            </div>
            <ResultsTable
              rows={recommendations}
              watchedIds={watchedIds}
              onToggleWatch={(r) => toggleWatchM.mutate(r)}
              compact
            />
          </section>

          <section className="space-y-4">
            <div className="flex items-center gap-2">
              <h2 className="text-base font-medium text-text">Wszystkie wyniki</h2>
              <span className="text-xs text-subtle">({rows.length})</span>
            </div>
            <ResultsTable
              rows={rows}
              watchedIds={watchedIds}
              onToggleWatch={(r) => toggleWatchM.mutate(r)}
            />
          </section>
        </>
      )}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="rounded-md border border-dashed border-border bg-surface/50 p-16 text-center">
      <h3 className="text-base font-medium text-text">Zacznij polowanie</h3>
      <p className="mt-2 text-sm text-muted">
        Spróbuj fraz takich jak{" "}
        <span className="font-mono text-text">seo</span>,{" "}
        <span className="font-mono text-text">kawa</span>, czy{" "}
        <span className="font-mono text-text">sklep</span>. Użyj filtrów by
        ograniczyć TLD, cenę lub minimalny wiek domeny.
      </p>
    </div>
  );
}
