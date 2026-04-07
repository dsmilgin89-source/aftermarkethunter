import { useEffect, useState } from "react";
import { Key, Check } from "lucide-react";
import { load } from "@tauri-apps/plugin-store";
import type { CebulaThresholds } from "@/lib/types";
import { DEFAULT_CEBULA_THRESHOLDS } from "@/lib/types";

type Provider = "ahrefs" | "majestic" | "moz" | "dataforseo" | "serpapi" | "openpagerank";

const PAID_PROVIDERS: {
  id: Provider;
  label: string;
  hint: string;
}[] = [
  {
    id: "ahrefs",
    label: "Ahrefs API",
    hint: "Domain Rating, backlinks, referring domains",
  },
  {
    id: "majestic",
    label: "Majestic",
    hint: "Trust Flow, Citation Flow, topical trust",
  },
  { id: "moz", label: "Moz Links", hint: "Domain Authority, Page Authority" },
  {
    id: "dataforseo",
    label: "DataForSEO",
    hint: "Keyword volume (PL), CPC, SERP snapshots",
  },
  {
    id: "serpapi",
    label: "SerpApi",
    hint: "Real-time Google SERP queries for relevance",
  },
];

export function SettingsView() {
  const [keys, setKeys] = useState<Record<Provider, string>>({
    ahrefs: "",
    majestic: "",
    moz: "",
    dataforseo: "",
    serpapi: "",
    openpagerank: "",
  });
  const [saved, setSaved] = useState<Record<Provider, boolean>>({
    ahrefs: false,
    majestic: false,
    moz: false,
    dataforseo: false,
    serpapi: false,
    openpagerank: false,
  });

  const [cebula, setCebula] = useState<CebulaThresholds>({
    ...DEFAULT_CEBULA_THRESHOLDS,
  });
  const [cebulaSaved, setCebulaSaved] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json");
        const next: Record<Provider, string> = { ...keys };
        for (const p of [...PAID_PROVIDERS, { id: "openpagerank" as Provider }]) {
          const v = (await store.get<string>(`apiKeys.${p.id}`)) ?? "";
          next[p.id] = v;
        }
        setKeys(next);

        const storedCebula = await store.get<CebulaThresholds>("cebula");
        if (storedCebula) setCebula(storedCebula);
      } catch {
        // plugin not ready
      }
    })();
  }, []);

  async function save(provider: Provider) {
    try {
      const store = await load("settings.json");
      await store.set(`apiKeys.${provider}`, keys[provider]);
      await store.save();
      setSaved((s) => ({ ...s, [provider]: true }));
      setTimeout(() => setSaved((s) => ({ ...s, [provider]: false })), 1500);
    } catch (e) {
      console.error(e);
    }
  }

  async function saveCebula() {
    try {
      const store = await load("settings.json");
      await store.set("cebula", cebula);
      await store.save();
      setCebulaSaved(true);
      setTimeout(() => setCebulaSaved(false), 1500);
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div className="mx-auto max-w-[900px] space-y-8 p-10">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Settings
        </h1>
        <p className="mt-1 text-sm text-muted">
          Klucze API opcjonalnych dostawców. Bez nich aplikacja używa tylko
          darmowych źródeł (Wayback, WHOIS, blacklisty, heurystyki językowe).
          Klucze są przechowywane lokalnie w pliku <code>settings.json</code>.
        </p>
      </header>

      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Cebula Deals — Progi
        </h2>
        <p className="text-xs text-muted">
          Skonfiguruj kiedy domena kwalifikuje się jako "Cebula Deal" —
          wyjątkowo dobra oferta w wyjątkowo dobrej cenie.
        </p>
        <div className="grid grid-cols-2 gap-4 rounded-md border border-amber-500/25 bg-surface p-4 lg:grid-cols-3">
          <CebulaField label="Min. score (0-100)">
            <input
              type="number"
              min={0}
              max={100}
              value={cebula.minScore}
              onChange={(e) =>
                setCebula((c) => ({ ...c, minScore: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Max cena (PLN)">
            <input
              type="number"
              min={0}
              value={cebula.maxPrice}
              onChange={(e) =>
                setCebula((c) => ({ ...c, maxPrice: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Min. wiek (lata)">
            <input
              type="number"
              min={0}
              value={cebula.minAge}
              onChange={(e) =>
                setCebula((c) => ({ ...c, minAge: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Min. Wayback snapshots">
            <input
              type="number"
              min={0}
              value={cebula.minWayback}
              onChange={(e) =>
                setCebula((c) => ({
                  ...c,
                  minWayback: Number(e.target.value),
                }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Brak blacklist hits">
            <ToggleSwitch
              checked={cebula.noBlacklist}
              onChange={(v) => setCebula((c) => ({ ...c, noBlacklist: v }))}
            />
          </CebulaField>
          <CebulaField label="Brak trademark warnings">
            <ToggleSwitch
              checked={cebula.noTrademark}
              onChange={(v) => setCebula((c) => ({ ...c, noTrademark: v }))}
            />
          </CebulaField>
        </div>
        <button
          onClick={saveCebula}
          className="flex h-9 items-center gap-1.5 rounded-sm bg-white px-4 text-xs font-medium text-black hover:bg-white/90"
        >
          {cebulaSaved ? (
            <>
              <Check className="h-3.5 w-3.5" /> Saved
            </>
          ) : (
            "Zapisz progi"
          )}
        </button>
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Darmowe narzędzia SEO
        </h2>
        <p className="text-xs text-muted">
          Open PageRank — darmowy klucz API (bez karty kredytowej). Wzbogaca
          scoring o dane PageRank bez płatnych narzędzi. Similarweb i DNS
          działają automatycznie, bez klucza.
        </p>
        <div className="flex items-start gap-3 rounded-md border border-border bg-surface p-4">
          <Key className="mt-1 h-4 w-4 text-subtle" />
          <div className="flex-1 space-y-1.5">
            <div className="flex items-baseline justify-between">
              <label htmlFor="openpagerank" className="text-sm font-medium text-text">
                Open PageRank
              </label>
              <a
                href="https://www.domcop.com/openpagerank/"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[11px] text-accent hover:underline"
              >
                Zarejestruj się (darmowy klucz) →
              </a>
            </div>
            <div className="flex gap-2">
              <input
                id="openpagerank"
                type="password"
                value={keys.openpagerank}
                onChange={(e) =>
                  setKeys((k) => ({ ...k, openpagerank: e.target.value }))
                }
                placeholder="API key..."
                className="h-9 flex-1 rounded-sm border border-border bg-surface-2 px-2 font-mono text-xs text-text placeholder:text-subtle focus:border-white/20 focus:outline-none"
              />
              <button
                onClick={() => save("openpagerank")}
                className="flex h-9 items-center gap-1.5 rounded-sm bg-white px-3 text-xs font-medium text-black hover:bg-white/90"
              >
                {saved.openpagerank ? (
                  <>
                    <Check className="h-3.5 w-3.5" /> Saved
                  </>
                ) : (
                  "Save"
                )}
              </button>
            </div>
          </div>
        </div>
        <div className="rounded-md border border-border bg-surface p-4 text-sm text-muted">
          <p>Działają automatycznie (bez klucza):</p>
          <ul className="mt-1 list-disc pl-5 text-xs">
            <li><span className="text-text">Similarweb</span> — globalny ranking i estymata ruchu</li>
            <li><span className="text-text">DNS Quality</span> — MX, SPF, DMARC (sygnał aktywnej domeny)</li>
          </ul>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Płatne dostawcy SEO
        </h2>
        {PAID_PROVIDERS.map((p) => (
          <div
            key={p.id}
            className="flex items-start gap-3 rounded-md border border-border bg-surface p-4"
          >
            <Key className="mt-1 h-4 w-4 text-subtle" />
            <div className="flex-1 space-y-1.5">
              <div className="flex items-baseline justify-between">
                <label htmlFor={p.id} className="text-sm font-medium text-text">
                  {p.label}
                </label>
                <span className="text-[11px] text-subtle">{p.hint}</span>
              </div>
              <div className="flex gap-2">
                <input
                  id={p.id}
                  type="password"
                  value={keys[p.id]}
                  onChange={(e) =>
                    setKeys((k) => ({ ...k, [p.id]: e.target.value }))
                  }
                  placeholder="API key..."
                  className="h-9 flex-1 rounded-sm border border-border bg-surface-2 px-2 font-mono text-xs text-text placeholder:text-subtle focus:border-white/20 focus:outline-none"
                />
                <button
                  onClick={() => save(p.id)}
                  className="flex h-9 items-center gap-1.5 rounded-sm bg-white px-3 text-xs font-medium text-black hover:bg-white/90"
                >
                  {saved[p.id] ? (
                    <>
                      <Check className="h-3.5 w-3.5" /> Saved
                    </>
                  ) : (
                    "Save"
                  )}
                </button>
              </div>
            </div>
          </div>
        ))}
      </section>

      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Scrapery
        </h2>
        <div className="rounded-md border border-border bg-surface p-4 text-sm text-muted">
          <p>
            Aktywne: <span className="text-text">aftermarket.pl</span> (pełna
            paginacja, max 50 stron)
          </p>
        </div>
      </section>
    </div>
  );
}

function CebulaField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block space-y-1.5">
      <span className="text-[11px] uppercase tracking-wider text-subtle">
        {label}
      </span>
      {children}
    </label>
  );
}

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 rounded-full transition-colors ${
        checked ? "bg-accent" : "border border-border bg-surface-2"
      }`}
    >
      <span
        className={`absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
          checked ? "translate-x-5" : ""
        }`}
      />
    </button>
  );
}
