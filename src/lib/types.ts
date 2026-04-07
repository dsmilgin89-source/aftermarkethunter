// Mirrors src-tauri/src/model.rs. Hand-maintained — keep in sync.

export type ScoringProfile = "seo_hunter" | "brand_builder" | "bargain";

export type AuctionType = "auction" | "buy_now" | "both" | "unknown";

export type ScoreTier = "excellent" | "good" | "fair" | "poor";

export interface Query {
  phrase: string;
  tlds: string[];
  max_price?: number | null;
  min_age_years?: number | null;
  min_wayback_snapshots?: number | null;
  sources: string[];
  profile: ScoringProfile;
}

export interface Listing {
  id: string;
  marketplace: string;
  domain: string;
  tld: string;
  current_price?: number | null;
  buy_now_price?: number | null;
  currency: string;
  auction_type: AuctionType;
  ends_at?: string | null;
  url: string;
  fetched_at: string;
}

export interface LinguisticReport {
  length: number;
  has_hyphen: boolean;
  has_digit: boolean;
  vowel_ratio: number;
  syllable_estimate: number;
  brandability: number;
  pronounceability: number;
}

export interface Enrichment {
  domain: string;
  whois_created?: string | null;
  age_years?: number | null;
  wayback_first?: string | null;
  wayback_last?: string | null;
  wayback_snapshots?: number | null;
  blacklist_hits: number;
  google_indexed_estimate?: number | null;
  linguistic: LinguisticReport;
  trademark_warning?: string | null;
  // Free SEO signals
  openpagerank_score?: number | null;
  openpagerank_rank?: number | null;
  similarweb_rank?: number | null;
  similarweb_monthly_visits?: number | null;
  has_mx: boolean;
  has_spf: boolean;
  has_dmarc: boolean;
  // Paid providers (optional)
  ahrefs_dr?: number | null;
  majestic_tf?: number | null;
  majestic_cf?: number | null;
  moz_da?: number | null;
  fetched_at?: string | null;
}

export interface Score {
  total: number;
  seo: number;
  relevance: number;
  value: number;
  risk_penalty: number;
  tier: ScoreTier;
  explanation: string[];
}

export interface ResultRow {
  listing: Listing;
  enrichment: Enrichment;
  score: Score;
}

export interface WatchlistEntry {
  listing_id: string;
  max_bid?: number | null;
  notes?: string | null;
  added_at: string;
}

export interface SavedSearch {
  id: number;
  name: string;
  query: Query;
  notify: boolean;
  created_at: string;
}

export interface SearchProgress {
  phase: "scraping" | "enriching" | "scoring" | "done";
  detail: string;
  current: number;
  total: number | null;
  marketplace: string | null;
}

export interface CebulaThresholds {
  minScore: number;
  maxPrice: number;
  minAge: number;
  minWayback: number;
  noBlacklist: boolean;
  noTrademark: boolean;
}

export const DEFAULT_CEBULA_THRESHOLDS: CebulaThresholds = {
  minScore: 55,
  maxPrice: 300,
  minAge: 3,
  minWayback: 10,
  noBlacklist: true,
  noTrademark: true,
};

export const DEFAULT_QUERY: Query = {
  phrase: "",
  tlds: [],
  sources: [],
  profile: "seo_hunter",
};
