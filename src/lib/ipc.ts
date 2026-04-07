// Thin wrappers around Tauri `invoke`. Centralised so type errors surface once.

import { invoke } from "@tauri-apps/api/core";
import type {
  Listing,
  Query,
  ResultRow,
  SavedSearch,
  WatchlistEntry,
} from "./types";

export const ipc = {
  search: (query: Query, openpagerankKey?: string) =>
    invoke<ResultRow[]>("search", { query, openpagerankKey: openpagerankKey ?? null }),

  listRecentResults: (limit?: number) =>
    invoke<Listing[]>("list_recent_results", { limit }),

  getListing: (id: string) => invoke<Listing | null>("get_listing", { id }),

  addToWatchlist: (
    listingId: string,
    maxBid?: number | null,
    notes?: string | null,
  ) =>
    invoke<void>("add_to_watchlist", {
      listingId,
      maxBid: maxBid ?? null,
      notes: notes ?? null,
    }),

  removeFromWatchlist: (listingId: string) =>
    invoke<void>("remove_from_watchlist", { listingId }),

  listWatchlist: () => invoke<WatchlistEntry[]>("list_watchlist"),

  saveSearch: (name: string, query: Query, notify: boolean) =>
    invoke<number>("save_search", { name, query, notify }),

  listSavedSearches: () => invoke<SavedSearch[]>("list_saved_searches"),

  deleteSavedSearch: (id: number) => invoke<void>("delete_saved_search", { id }),

  getApiKeyStatus: () =>
    invoke<Record<string, boolean>>("get_api_key_status"),

  exportResultsCsv: (rows: ResultRow[]) =>
    invoke<string>("export_results_csv", { rows }),
};

export function cn(...classes: (string | false | null | undefined)[]): string {
  return classes.filter(Boolean).join(" ");
}
