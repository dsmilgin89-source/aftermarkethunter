import { useMemo, useState } from "react";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState,
} from "@tanstack/react-table";
import {
  ArrowUpDown,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  Copy,
  Star,
  StarOff,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { ResultRow } from "@/lib/types";
import { ScoreBadge } from "./ScoreBadge";
import { ScoreBreakdown } from "./ScoreBreakdown";
import { Countdown } from "./Countdown";
import { cn } from "@/lib/ipc";

const col = createColumnHelper<ResultRow>();

export function ResultsTable({
  rows,
  watchedIds,
  onToggleWatch,
  compact = false,
}: {
  rows: ResultRow[];
  watchedIds: Set<string>;
  onToggleWatch: (row: ResultRow) => void;
  compact?: boolean;
}) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: "score", desc: true },
  ]);
  const [globalFilter, setGlobalFilter] = useState("");

  const columns = useMemo<ColumnDef<ResultRow, any>[]>(
    () => [
      col.accessor((r) => r.listing.domain, {
        id: "domain",
        header: "Domena",
        cell: (ctx) => (
          <div className="flex items-center gap-2">
            <span className="font-mono text-sm text-text">
              {ctx.row.original.listing.domain}
            </span>
            {ctx.row.original.enrichment.trademark_warning && (
              <span
                title={ctx.row.original.enrichment.trademark_warning}
                className="rounded-sm bg-danger/15 px-1.5 py-0.5 text-[10px] font-medium text-danger"
              >
                TM
              </span>
            )}
          </div>
        ),
      }),
      col.accessor((r) => r.score.total, {
        id: "score",
        header: "Score",
        cell: (ctx) => {
          const s = ctx.row.original.score;
          return (
            <div className="group relative inline-block">
              <ScoreBadge tier={s.tier} total={s.total} />
              <div className="pointer-events-none absolute left-0 top-full z-50 mt-2 opacity-0 transition-opacity group-hover:pointer-events-auto group-hover:opacity-100">
                <ScoreBreakdown score={s} />
              </div>
            </div>
          );
        },
        sortingFn: (a, b) => a.original.score.total - b.original.score.total,
      }),
      col.accessor((r) => r.listing.current_price ?? 0, {
        id: "price",
        header: "Cena",
        cell: (ctx) => {
          const p = ctx.row.original.listing.current_price;
          if (p == null) return <span className="text-subtle">—</span>;
          return (
            <span className="font-mono tabular-nums text-sm text-text">
              {p.toLocaleString("pl-PL", {
                maximumFractionDigits: 0,
              })}{" "}
              {ctx.row.original.listing.currency}
            </span>
          );
        },
      }),
      col.accessor((r) => r.enrichment.age_years ?? 0, {
        id: "age",
        header: "Wiek",
        cell: (ctx) => {
          const a = ctx.row.original.enrichment.age_years;
          if (a == null) return <span className="text-subtle">—</span>;
          return (
            <span className="font-mono tabular-nums text-sm">
              {a.toFixed(1)}y
            </span>
          );
        },
      }),
      col.accessor((r) => r.enrichment.wayback_snapshots ?? 0, {
        id: "wayback",
        header: "Wayback",
        cell: (ctx) => {
          const s = ctx.row.original.enrichment.wayback_snapshots;
          if (s == null) return <span className="text-subtle">—</span>;
          return (
            <span className="font-mono tabular-nums text-sm">
              {s.toLocaleString("pl-PL")}
            </span>
          );
        },
      }),
      col.accessor((r) => r.enrichment.linguistic.brandability, {
        id: "brand",
        header: "Brand",
        cell: (ctx) => (
          <span className="font-mono tabular-nums text-sm text-muted">
            {Math.round(ctx.row.original.enrichment.linguistic.brandability)}
          </span>
        ),
      }),
      col.accessor((r) => r.enrichment.blacklist_hits, {
        id: "risk",
        header: "Risk",
        cell: (ctx) => {
          const hits = ctx.row.original.enrichment.blacklist_hits;
          if (hits === 0) return <span className="text-subtle">—</span>;
          return (
            <span className="rounded-sm bg-danger/15 px-1.5 py-0.5 text-xs font-medium text-danger">
              BL {hits}
            </span>
          );
        },
      }),
      col.accessor((r) => r.listing.ends_at ?? "", {
        id: "ends",
        header: "Koniec",
        cell: (ctx) => <Countdown endsAt={ctx.row.original.listing.ends_at} />,
      }),
      col.accessor((r) => r.listing.marketplace, {
        id: "marketplace",
        header: "Źródło",
        cell: (ctx) => (
          <span className="text-xs text-muted">
            {ctx.row.original.listing.marketplace.replace("_", ".")}
          </span>
        ),
      }),
      col.display({
        id: "actions",
        header: "",
        cell: (ctx) => {
          const r = ctx.row.original;
          const watched = watchedIds.has(r.listing.id);
          return (
            <div className="flex items-center justify-end gap-1">
              <IconButton
                label={watched ? "Usuń z watchlisty" : "Dodaj do watchlisty"}
                onClick={() => onToggleWatch(r)}
              >
                {watched ? (
                  <Star className="h-4 w-4 fill-accent text-accent" />
                ) : (
                  <StarOff className="h-4 w-4" />
                )}
              </IconButton>
              <IconButton
                label="Otwórz aukcję"
                onClick={() => openUrl(r.listing.url)}
              >
                <ExternalLink className="h-4 w-4" />
              </IconButton>
              <IconButton
                label="Kopiuj nazwę"
                onClick={() =>
                  navigator.clipboard.writeText(r.listing.domain)
                }
              >
                <Copy className="h-4 w-4" />
              </IconButton>
            </div>
          );
        },
      }),
    ],
    [watchedIds, onToggleWatch],
  );

  const table = useReactTable({
    data: rows,
    columns,
    state: { sorting, globalFilter },
    onSortingChange: setSorting,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  return (
    <div className="space-y-3">
      {!compact && (
        <div className="flex items-center justify-between">
          <input
            type="text"
            placeholder="Filtruj w wynikach..."
            value={globalFilter}
            onChange={(e) => setGlobalFilter(e.target.value)}
            className="h-9 w-72 rounded-sm border border-border bg-surface px-3 text-sm text-text placeholder:text-subtle focus:border-white/20 focus:outline-none"
          />
          <span className="text-xs text-subtle">
            {table.getRowModel().rows.length} / {rows.length}
          </span>
        </div>
      )}
      <div className="overflow-auto rounded-md border border-border bg-surface">
        <table className="w-full text-sm">
          <thead className="sticky top-0 bg-surface">
            {table.getHeaderGroups().map((hg) => (
              <tr key={hg.id} className="border-b border-border">
                {hg.headers.map((h) => {
                  const canSort = h.column.getCanSort();
                  const sorted = h.column.getIsSorted();
                  return (
                    <th
                      key={h.id}
                      onClick={canSort ? h.column.getToggleSortingHandler() : undefined}
                      className={cn(
                        "px-3 py-2.5 text-left text-[11px] font-medium uppercase tracking-wider text-subtle",
                        canSort && "cursor-pointer select-none hover:text-text",
                      )}
                    >
                      <span className="inline-flex items-center gap-1">
                        {flexRender(h.column.columnDef.header, h.getContext())}
                        {canSort && (
                          sorted === "asc" ? (
                            <ChevronUp className="h-3 w-3" />
                          ) : sorted === "desc" ? (
                            <ChevronDown className="h-3 w-3" />
                          ) : (
                            <ArrowUpDown className="h-3 w-3 opacity-40" />
                          )
                        )}
                      </span>
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className="border-b border-border/60 last:border-0 hover:bg-white/[0.02]"
              >
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} className="px-3 py-2 align-middle">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
            {table.getRowModel().rows.length === 0 && (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-3 py-16 text-center text-sm text-subtle"
                >
                  Brak wyników.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function IconButton({
  children,
  label,
  onClick,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className="flex h-7 w-7 items-center justify-center rounded-sm text-muted transition-colors hover:bg-white/10 hover:text-text"
    >
      {children}
    </button>
  );
}
