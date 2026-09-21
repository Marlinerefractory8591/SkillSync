import React, { useEffect, useState } from "react";
import { useSkillStore } from "../store/useSkillStore";
import { useTranslation, supportedLanguages } from "../i18n";
import {
  Search,
  RefreshCw,
  Zap,
  Settings as SettingsIcon,
  Sun,
  Moon,
  Layers,
  ArrowUpCircle,
  Globe,
  Check,
} from "lucide-react";
import { hasTrackingBranch } from "../lib/branch-tracking";

export const Navbar: React.FC = () => {
  const { t, language, setLanguage } = useTranslation();
  const {
    skills,
    searchQuery,
    setSearchQuery,
    selectedScope,
    setSelectedScope,
    statusFilter,
    setStatusFilter,
    itemTypeFilter,
    setItemTypeFilter,
    branchFilter,
    setBranchFilter,
    queuedChecks,
    isScanning,
    fetchSkills,
    batchUpdateAll,
    batchUpdating,
    openSettings,
    openUpdateCenter,
    theme,
    toggleTheme,
  } = useSkillStore();

  const [isLangOpen, setIsLangOpen] = useState(false);

  const totalSkills = skills.length;
  const outdatedCount = skills.filter((s) => s.updateAvailable).length;
  const missingBranchCount = skills.filter(
    (skill) => Boolean(skill.remoteUrl) && !hasTrackingBranch(skill),
  ).length;

  // Keyboard shortcut listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const cmdKey = isMac ? e.metaKey : e.ctrlKey;

      if (cmdKey && e.key.toLowerCase() === "r") {
        e.preventDefault();
        fetchSkills(true);
      } else if (cmdKey && e.key.toLowerCase() === "u") {
        e.preventDefault();
        batchUpdateAll();
      } else if (cmdKey && e.key === ",") {
        e.preventDefault();
        openSettings();
      } else if (e.key === "/" && document.activeElement?.tagName !== "INPUT") {
        e.preventDefault();
        document.getElementById("search-input")?.focus();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [fetchSkills, batchUpdateAll, openSettings]);

  const scopes = [
    { id: "all", label: t.allScopes },
    { id: "global", label: "Global" },
    { id: "codex", label: "Codex" },
    { id: "claude", label: "Claude Code" },
    { id: "cursor", label: "Cursor" },
    { id: "antigravity", label: "Antigravity" },
  ];

  const currentLangObj =
    supportedLanguages.find((l) => l.code === language) ||
    supportedLanguages[0];

  return (
    <header className="sticky top-0 z-30 border-b border-border bg-background/80 backdrop-blur-md transition-colors">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Top brand & actions row */}
        <div className="flex items-center justify-between h-16 gap-4">
          {/* Logo & Brand */}
          <div className="flex items-center gap-3 select-none">
            <div className="w-9 h-9 rounded-lg bg-primary/15 border border-primary/30 flex items-center justify-center text-primary shadow-sm shadow-primary/20">
              <Zap className="w-5 h-5 fill-primary" />
            </div>
            <div>
              <span className="font-bold text-lg tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text">
                {t.appName}
              </span>
              <p className="text-[11px] text-muted-foreground hidden sm:block">
                {t.appSubtitle}
              </p>
            </div>
          </div>

          {/* Search bar */}
          <div className="flex-1 max-w-md relative">
            <Search className="w-4 h-4 text-muted-foreground absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            <input
              id="search-input"
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t.searchPlaceholder}
              className="w-full h-9 pl-9 pr-8 text-xs rounded-lg bg-card/60 border border-border text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary transition-all"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-xs text-muted-foreground hover:text-foreground"
              >
                ✕
              </button>
            )}
          </div>

          {/* Action buttons */}
          <div className="flex items-center gap-2">
            {/* Rescan button */}
            <button
              onClick={() => fetchSkills(true)}
              disabled={isScanning}
              title={`${t.rescan} (Cmd/Ctrl + R)`}
              className="flex items-center gap-1.5 h-9 px-3 text-xs font-medium rounded-lg border border-border bg-card/50 hover:bg-card text-foreground transition-all disabled:opacity-50"
            >
              <RefreshCw
                className={`w-3.5 h-3.5 ${isScanning ? "animate-spin text-primary" : ""}`}
              />
              <span className="hidden sm:inline">{t.rescan}</span>
            </button>

            {/* Update All button */}
            <button
              onClick={() => batchUpdateAll()}
              disabled={outdatedCount === 0 || batchUpdating}
              title={`${t.updateAll} (Cmd/Ctrl + U)`}
              className="flex items-center gap-1.5 h-9 px-3 text-xs font-semibold rounded-lg bg-primary hover:bg-primary/90 text-primary-foreground shadow-sm shadow-primary/25 transition-all disabled:opacity-40 disabled:pointer-events-none"
            >
              <ArrowUpCircle className="w-4 h-4" />
              <span>{t.updateAll}</span>
              {outdatedCount > 0 && (
                <span className="ml-0.5 px-1.5 py-0.2 rounded-full text-[10px] bg-primary-foreground/20 text-primary-foreground font-mono">
                  {outdatedCount}
                </span>
              )}
            </button>

            <div className="h-4 w-[1px] bg-border mx-1" />

            {/* Language Switcher Dropdown */}
            <div className="relative">
              <button
                onClick={() => setIsLangOpen(!isLangOpen)}
                title={t.languageSelect}
                className="h-9 px-2.5 flex items-center gap-1.5 rounded-lg border border-border bg-card/50 hover:bg-card text-foreground transition-all text-xs font-medium"
              >
                <Globe className="w-3.5 h-3.5 text-muted-foreground" />
                <span>{currentLangObj.flag}</span>
                <span className="uppercase font-mono text-[10px]">
                  {currentLangObj.code}
                </span>
              </button>

              {isLangOpen && (
                <div
                  className="absolute right-0 mt-1 w-36 py-1.5 rounded-xl border border-border bg-card shadow-xl z-50 animate-fade-in"
                  onClick={() => setIsLangOpen(false)}
                >
                  {supportedLanguages.map((lang) => (
                    <button
                      key={lang.code}
                      onClick={() => setLanguage(lang.code)}
                      className={`w-full px-3 py-1.5 text-left text-xs flex items-center justify-between hover:bg-muted transition-colors ${
                        language === lang.code
                          ? "text-primary font-semibold bg-primary/10"
                          : "text-foreground"
                      }`}
                    >
                      <span className="flex items-center gap-2">
                        <span>{lang.flag}</span>
                        <span>{lang.label}</span>
                      </span>
                      {language === lang.code && (
                        <Check className="w-3.5 h-3.5 text-primary" />
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Update Center Button */}
            <button
              onClick={openUpdateCenter}
              title={t.updateCenterTitle}
              className="w-9 h-9 flex items-center justify-center rounded-lg border border-border bg-card/50 hover:bg-card text-foreground transition-all"
            >
              <Layers className="w-4 h-4" />
            </button>

            {/* Theme Toggle */}
            <button
              onClick={toggleTheme}
              title={theme === "dark" ? t.lightTheme : t.darkTheme}
              className="w-9 h-9 flex items-center justify-center rounded-lg border border-border bg-card/50 hover:bg-card text-foreground transition-all"
            >
              {theme === "dark" ? (
                <Sun className="w-4 h-4 text-amber-400" />
              ) : (
                <Moon className="w-4 h-4 text-slate-700" />
              )}
            </button>

            {/* Settings */}
            <button
              onClick={openSettings}
              title={`${t.preferences} (Cmd/Ctrl + ,)`}
              className="w-9 h-9 flex items-center justify-center rounded-lg border border-border bg-card/50 hover:bg-card text-foreground transition-all"
            >
              <SettingsIcon className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Bottom category filter tabs & metrics row */}
        <div className="flex flex-wrap items-center justify-between pb-3 pt-2 text-xs gap-3 border-t border-border/40">
          {/* Status Filter Tabs (Wszystkie / Wymagane aktualizacje / Zaktualizowane) */}
          <div className="flex items-center gap-1.5 p-0.5 rounded-lg bg-card/60 border border-border">
            <button
              onClick={() => setStatusFilter("all")}
              className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${
                statusFilter === "all"
                  ? "bg-primary text-primary-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              {t.statusAll} ({totalSkills})
            </button>

            <button
              onClick={() => setStatusFilter("updates")}
              className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all ${
                statusFilter === "updates"
                  ? "bg-amber-500 text-black shadow-sm font-semibold"
                  : "text-amber-400 hover:bg-amber-500/10"
              }`}
            >
              <Zap className="w-3.5 h-3.5 fill-current" />
              <span>{t.requiredUpdates}</span>
              {outdatedCount > 0 && (
                <span
                  className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold ${
                    statusFilter === "updates"
                      ? "bg-black/20 text-black"
                      : "bg-amber-500/20 text-amber-400"
                  }`}
                >
                  {outdatedCount}
                </span>
              )}
            </button>

            <button
              onClick={() => setStatusFilter("up_to_date")}
              className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-all ${
                statusFilter === "up_to_date"
                  ? "bg-emerald-600 text-white shadow-sm font-semibold"
                  : "text-muted-foreground hover:text-emerald-400"
              }`}
            >
              <Check className="w-3.5 h-3.5" />
              <span>{t.statusUpToDate}</span>
            </button>
          </div>

          <div className="flex items-center gap-1 rounded-lg border border-border bg-card/60 p-0.5">
            {(
              [
                ["all", "All"],
                ["skill", "Skills"],
                ["mcp", "MCP"],
                ["plugin", "Plugins"],
              ] as const
            ).map(([type, label]) => (
              <button
                key={type}
                onClick={() => setItemTypeFilter(type)}
                className={`px-2.5 py-1 rounded-md text-xs font-medium transition-all ${
                  itemTypeFilter === type
                    ? "bg-primary text-primary-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          <button
            onClick={() =>
              setBranchFilter(branchFilter === "missing" ? "all" : "missing")
            }
            className={`px-2.5 py-1 rounded-md border text-xs font-medium transition-all ${
              branchFilter === "missing"
                ? "border-amber-500/40 bg-amber-500/10 text-amber-400"
                : "border-border bg-card/60 text-muted-foreground hover:text-foreground"
            }`}
            title="Show packages that need a tracking branch assigned"
          >
            Bez gałęzi {missingBranchCount > 0 ? `(${missingBranchCount})` : ""}
          </button>

          {/* Scopes Filter */}
          <div className="flex items-center gap-1.5 overflow-x-auto py-1">
            {scopes.map((s) => (
              <button
                key={s.id}
                onClick={() => setSelectedScope(s.id)}
                className={`px-2.5 py-1 rounded-md text-xs font-medium transition-all ${
                  selectedScope === s.id
                    ? "bg-primary/15 text-primary border border-primary/30"
                    : "text-muted-foreground hover:text-foreground hover:bg-card border border-transparent"
                }`}
              >
                {s.label}
              </button>
            ))}
          </div>

          {/* Quick Metrics */}
          <div className="hidden lg:flex items-center gap-3 text-[11px] text-muted-foreground font-mono">
            <span>
              {t.total}:{" "}
              <strong className="text-foreground">{totalSkills}</strong>
            </span>
            <span>•</span>
            <span>
              {t.outdated}:{" "}
              <strong
                className={
                  outdatedCount > 0 ? "text-amber-400" : "text-emerald-400"
                }
              >
                {outdatedCount}
              </strong>
            </span>
            {queuedChecks > 0 && (
              <>
                <span>•</span>
                <span className="text-primary">Kolejka: {queuedChecks}</span>
              </>
            )}
          </div>
        </div>
      </div>
    </header>
  );
};
