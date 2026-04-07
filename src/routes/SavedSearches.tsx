import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Play, Trash2, Bell, BellOff } from "lucide-react";
import { ipc } from "@/lib/ipc";

export function SavedSearchesView({ onRun }: { onRun: () => void }) {
  const qc = useQueryClient();
  const savedQ = useQuery({
    queryKey: ["savedSearches"],
    queryFn: () => ipc.listSavedSearches(),
  });

  const delM = useMutation({
    mutationFn: (id: number) => ipc.deleteSavedSearch(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["savedSearches"] }),
  });

  const entries = savedQ.data ?? [];

  return (
    <div className="mx-auto max-w-[1200px] space-y-6 p-10">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Saved searches
        </h1>
        <p className="mt-1 text-sm text-muted">
          Zapisane zapytania. Włączone powiadomienia wyślą notyfikację systemową
          kiedy background scheduler znajdzie nowe dopasowania.
        </p>
      </header>

      {entries.length === 0 ? (
        <div className="rounded-md border border-dashed border-border bg-surface/50 p-16 text-center text-sm text-muted">
          Brak zapisanych zapytań. Zapisz query z widoku Hunt używając przycisku
          „Zapisz query".
        </div>
      ) : (
        <div className="space-y-3">
          {entries.map((s) => (
            <div
              key={s.id}
              className="flex items-center justify-between rounded-md border border-border bg-surface p-4"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <h3 className="text-sm font-medium text-text">{s.name}</h3>
                  {s.notify ? (
                    <Bell className="h-3.5 w-3.5 text-accent" />
                  ) : (
                    <BellOff className="h-3.5 w-3.5 text-subtle" />
                  )}
                </div>
                <div className="mt-1 font-mono text-xs text-muted">
                  phrase="{s.query.phrase}" · profile={s.query.profile}
                  {s.query.tlds.length > 0 && ` · tld=${s.query.tlds.join(",")}`}
                  {s.query.max_price && ` · max=${s.query.max_price}`}
                </div>
              </div>
              <div className="flex gap-1">
                <button
                  onClick={onRun}
                  className="inline-flex h-8 items-center gap-1.5 rounded-sm border border-border bg-surface-2 px-3 text-xs text-muted hover:text-text"
                >
                  <Play className="h-3 w-3" />
                  Uruchom
                </button>
                <button
                  onClick={() => delM.mutate(s.id)}
                  className="inline-flex h-8 w-8 items-center justify-center rounded-sm text-muted hover:bg-white/10 hover:text-danger"
                  aria-label="Usuń"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
