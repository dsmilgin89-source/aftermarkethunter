import { useState } from "react";
import {
  Bookmark,
  Crosshair,
  Settings,
  Star,
  Sparkles,
} from "lucide-react";
import { cn } from "@/lib/ipc";
import { HuntView } from "@/routes/Hunt";
import { WatchlistView } from "@/routes/Watchlist";
import { SavedSearchesView } from "@/routes/SavedSearches";
import { SettingsView } from "@/routes/Settings";

type Route = "hunt" | "watchlist" | "saved" | "settings";

const NAV: { id: Route; label: string; icon: React.ComponentType<any> }[] = [
  { id: "hunt", label: "Hunt", icon: Crosshair },
  { id: "watchlist", label: "Watchlist", icon: Star },
  { id: "saved", label: "Saved", icon: Bookmark },
  { id: "settings", label: "Settings", icon: Settings },
];

export default function App() {
  const [route, setRoute] = useState<Route>("hunt");

  return (
    <div className="flex h-screen w-screen bg-bg text-text">
      <Sidebar current={route} onNavigate={setRoute} />
      <main className="flex-1 overflow-auto">
        {route === "hunt" && <HuntView />}
        {route === "watchlist" && <WatchlistView />}
        {route === "saved" && <SavedSearchesView onRun={() => setRoute("hunt")} />}
        {route === "settings" && <SettingsView />}
      </main>
    </div>
  );
}

function Sidebar({
  current,
  onNavigate,
}: {
  current: Route;
  onNavigate: (r: Route) => void;
}) {
  return (
    <aside className="flex w-52 flex-col border-r border-border bg-surface">
      <div className="flex h-14 items-center gap-2 border-b border-border px-5">
        <Sparkles className="h-4 w-4 text-accent" aria-hidden />
        <div>
          <div className="text-sm font-semibold leading-none text-text">
            Aftermarket
          </div>
          <div className="text-[11px] uppercase tracking-wider text-subtle">
            Hunter
          </div>
        </div>
      </div>
      <nav className="flex-1 space-y-0.5 p-2">
        {NAV.map((item) => {
          const Icon = item.icon;
          const active = current === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                "flex w-full items-center gap-2.5 rounded-sm px-3 py-2 text-sm transition-colors",
                active
                  ? "bg-white/[0.06] text-text"
                  : "text-muted hover:bg-white/[0.03] hover:text-text",
              )}
            >
              <Icon className="h-4 w-4" aria-hidden />
              {item.label}
            </button>
          );
        })}
      </nav>
      <footer className="border-t border-border px-5 py-3 text-[11px] text-subtle">
        v0.1.0 · personal
      </footer>
    </aside>
  );
}
