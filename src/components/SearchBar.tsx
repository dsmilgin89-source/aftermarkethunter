import { useState } from "react";
import { Search, Sliders } from "lucide-react";
import type { Query, ScoringProfile } from "@/lib/types";
import { DEFAULT_QUERY } from "@/lib/types";

const TLDS = ["pl", "com.pl", "com", "net", "net.pl", "org", "io", "eu"];

const PROFILES: { value: ScoringProfile; label: string; hint: string }[] = [
  { value: "seo_hunter", label: "SEO Hunter", hint: "Age + Wayback + metrics" },
  { value: "brand_builder", label: "Brand Builder", hint: "Short + brandable" },
  { value: "bargain", label: "Bargain", hint: "Cheapest vs estimate" },
];

export function SearchBar({
  value,
  onChange,
  onSubmit,
  loading,
}: {
  value: Query;
  onChange: (q: Query) => void;
  onSubmit: () => void;
  loading: boolean;
}) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
      className="space-y-4"
    >
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search
            className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-subtle"
            aria-hidden
          />
          <input
            autoFocus
            type="text"
            placeholder="fraza lub nazwa domeny..."
            value={value.phrase}
            onChange={(e) => onChange({ ...value, phrase: e.target.value })}
            className="h-11 w-full rounded-md border border-border bg-surface pl-9 pr-3 text-sm text-text placeholder:text-subtle focus:border-white/20 focus:outline-none"
          />
        </div>
        <button
          type="button"
          onClick={() => setShowAdvanced((v) => !v)}
          className="flex h-11 items-center gap-2 rounded-md border border-border bg-surface px-3 text-sm text-muted hover:text-text"
        >
          <Sliders className="h-4 w-4" aria-hidden />
          Filtry
        </button>
        <button
          type="submit"
          disabled={loading || !value.phrase.trim()}
          className="h-11 rounded-md bg-white px-6 text-sm font-medium text-black transition-colors hover:bg-white/90 disabled:cursor-not-allowed disabled:bg-white/5 disabled:text-white/70"
        >
          {loading ? "Polowanie..." : "Poluj"}
        </button>
      </div>

      {showAdvanced && (
        <div className="grid grid-cols-2 gap-4 rounded-md border border-border bg-surface p-4 lg:grid-cols-4">
          <Field label="Max cena (PLN)">
            <input
              type="number"
              value={value.max_price ?? ""}
              onChange={(e) =>
                onChange({
                  ...value,
                  max_price: e.target.value ? Number(e.target.value) : null,
                })
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </Field>
          <Field label="Min wiek (lata)">
            <input
              type="number"
              value={value.min_age_years ?? ""}
              onChange={(e) =>
                onChange({
                  ...value,
                  min_age_years: e.target.value ? Number(e.target.value) : null,
                })
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </Field>
          <Field label="Min Wayback snapshots">
            <input
              type="number"
              value={value.min_wayback_snapshots ?? ""}
              onChange={(e) =>
                onChange({
                  ...value,
                  min_wayback_snapshots: e.target.value
                    ? Number(e.target.value)
                    : null,
                })
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </Field>
          <Field label="Profil scoringu">
            <select
              value={value.profile}
              onChange={(e) =>
                onChange({
                  ...value,
                  profile: e.target.value as ScoringProfile,
                })
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            >
              {PROFILES.map((p) => (
                <option key={p.value} value={p.value}>
                  {p.label} — {p.hint}
                </option>
              ))}
            </select>
          </Field>
          <Field label="TLD" className="col-span-2 lg:col-span-4">
            <div className="flex flex-wrap gap-1.5">
              {TLDS.map((tld) => {
                const active = value.tlds.includes(tld);
                return (
                  <button
                    type="button"
                    key={tld}
                    onClick={() =>
                      onChange({
                        ...value,
                        tlds: active
                          ? value.tlds.filter((t) => t !== tld)
                          : [...value.tlds, tld],
                      })
                    }
                    className={`rounded-sm border px-2 py-1 text-xs ${
                      active
                        ? "border-accent/40 bg-accent-soft text-accent"
                        : "border-border bg-surface-2 text-muted hover:text-text"
                    }`}
                  >
                    .{tld}
                  </button>
                );
              })}
              <button
                type="button"
                onClick={() => onChange({ ...value, tlds: [] })}
                className="rounded-sm px-2 py-1 text-xs text-subtle hover:text-muted"
              >
                wyczyść
              </button>
            </div>
          </Field>
        </div>
      )}
    </form>
  );
}

function Field({
  label,
  children,
  className,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <label className={`block space-y-1.5 ${className ?? ""}`}>
      <span className="text-[11px] uppercase tracking-wider text-subtle">
        {label}
      </span>
      {children}
    </label>
  );
}

export { DEFAULT_QUERY };
