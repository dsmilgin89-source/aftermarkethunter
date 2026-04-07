import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Trash2, ExternalLink } from "lucide-react";
import { ipc } from "@/lib/ipc";
import { Countdown } from "@/components/Countdown";

export function WatchlistView() {
  const qc = useQueryClient();
  const watchQ = useQuery({
    queryKey: ["watchlist"],
    queryFn: () => ipc.listWatchlist(),
  });

  const removeM = useMutation({
    mutationFn: (id: string) => ipc.removeFromWatchlist(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["watchlist"] }),
  });

  const entries = watchQ.data ?? [];

  return (
    <div className="mx-auto max-w-[1200px] space-y-6 p-10">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Watchlist
        </h1>
        <p className="mt-1 text-sm text-muted">
          Aukcje, które śledzisz. Odliczanie jest na żywo; ostatnia godzina
          podświetlana na czerwono.
        </p>
      </header>

      {entries.length === 0 ? (
        <div className="rounded-md border border-dashed border-border bg-surface/50 p-16 text-center text-sm text-muted">
          Watchlist pusta. Dodaj aukcje z widoku Hunt klikając ikonę gwiazdki.
        </div>
      ) : (
        <div className="overflow-hidden rounded-md border border-border bg-surface">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border">
                <Th>Listing ID</Th>
                <Th>Max bid</Th>
                <Th>Notatki</Th>
                <Th>Dodane</Th>
                <Th className="text-right">Akcje</Th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => (
                <tr
                  key={e.listing_id}
                  className="border-b border-border/60 last:border-0"
                >
                  <td className="px-4 py-3 font-mono text-xs text-text">
                    {e.listing_id}
                  </td>
                  <td className="px-4 py-3 font-mono tabular-nums text-text">
                    {e.max_bid ? `${e.max_bid} PLN` : "—"}
                  </td>
                  <td className="px-4 py-3 text-muted">{e.notes ?? "—"}</td>
                  <td className="px-4 py-3 text-subtle">
                    {new Date(e.added_at).toLocaleString("pl-PL")}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <button
                      onClick={() => removeM.mutate(e.listing_id)}
                      className="inline-flex h-7 w-7 items-center justify-center rounded-sm text-muted hover:bg-white/10 hover:text-danger"
                      aria-label="Usuń z watchlisty"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function Th({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return (
    <th
      className={`px-4 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-subtle ${className}`}
    >
      {children}
    </th>
  );
}
