import { useEffect, useState } from "react";
import { Key, Check } from "lucide-react";
import { load } from "@tauri-apps/plugin-store";

type Provider = "ahrefs" | "majestic" | "moz" | "dataforseo" | "serpapi";

const PROVIDERS: {
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
  });
  const [saved, setSaved] = useState<Record<Provider, boolean>>({
    ahrefs: false,
    majestic: false,
    moz: false,
    dataforseo: false,
    serpapi: false,
  });

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json");
        const next: Record<Provider, string> = { ...keys };
        for (const p of PROVIDERS) {
          const v = (await store.get<string>(`apiKeys.${p.id}`)) ?? "";
          next[p.id] = v;
        }
        setKeys(next);
      } catch {
        // plugin not ready — that's fine in browser-only dev preview
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
          Płatne dostawcy SEO
        </h2>
        {PROVIDERS.map((p) => (
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
            Aktywne: <span className="text-text">aftermarket.pl</span>
          </p>
          <p className="mt-1">
            W przygotowaniu:{" "}
            <span className="text-subtle">premium.pl, dropped.pl</span> —
            scrapery zostaną uaktywnione gdy ich selektory zostaną dopięte do
            aktualnego layoutu obu serwisów.
          </p>
        </div>
      </section>
    </div>
  );
}
