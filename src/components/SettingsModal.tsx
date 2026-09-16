import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSkillStore } from "../store/useSkillStore";
import { useTranslation, supportedLanguages } from "../i18n";
import { api } from "../lib/ipc";
import { AppConfig, AppUpdateInfo, MonitoredPath } from "../types/skillsync";
import {
  X,
  Sliders,
  FolderOpen,
  ArrowUpCircle,
  Palette,
  ShieldAlert,
  Plus,
  Trash2,
  Check,
  Globe,
  RefreshCw,
} from "lucide-react";

export const SettingsModal: React.FC = () => {
  const { t, language, setLanguage } = useTranslation();
  const { isSettingsOpen, closeSettings, config, saveConfig, theme, setTheme } =
    useSkillStore();

  const [activeTab, setActiveTab] = useState<
    "general" | "paths" | "updates" | "appearance" | "advanced"
  >("paths");
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(config);
  const [newPathInput, setNewPathInput] = useState("");
  const [newScopeInput, setNewScopeInput] = useState("global");
  const [savedToast, setSavedToast] = useState(false);
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [isCheckingAppUpdate, setIsCheckingAppUpdate] = useState(false);
  const [appUpdateError, setAppUpdateError] = useState<string | null>(null);
  const appUpdateCopy =
    language === "pl"
      ? {
          title: "Wersja SkillSync",
          check: "Sprawdź aktualizacje",
          checking: "Sprawdzanie…",
          current: "Zainstalowana wersja",
          noRelease: "Nie znaleziono opublikowanego wydania.",
          upToDate: "Masz najnowszą opublikowaną wersję.",
          available: "Dostępna jest nowsza wersja",
        }
      : {
          title: "SkillSync version",
          check: "Check for updates",
          checking: "Checking…",
          current: "Installed version",
          noRelease: "No published release was found.",
          upToDate: "You have the latest published version.",
          available: "A newer version is available",
        };

  useEffect(() => {
    if (config) {
      // This is an editable draft, intentionally reset when its persisted source changes.
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setLocalConfig(JSON.parse(JSON.stringify(config)));
    }
  }, [config, isSettingsOpen]);

  useEffect(() => {
    if (!isSettingsOpen) return;

    void api
      .getAppVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion(null));
  }, [isSettingsOpen]);

  if (!isSettingsOpen || !localConfig) return null;

  const handleSave = async () => {
    await saveConfig(localConfig);
    setSavedToast(true);
    setTimeout(() => {
      setSavedToast(false);
      closeSettings();
    }, 800);
  };

  const handleAddPath = () => {
    if (!newPathInput.trim()) return;
    const newPath: MonitoredPath = {
      id: `p-${Date.now()}`,
      path: newPathInput.trim(),
      scope: newScopeInput,
      enabled: true,
    };
    setLocalConfig({
      ...localConfig,
      paths: {
        ...localConfig.paths,
        monitored: [...localConfig.paths.monitored, newPath],
      },
    });
    setNewPathInput("");
  };

  const handleCheckAppUpdate = async () => {
    setIsCheckingAppUpdate(true);
    setAppUpdateError(null);
    try {
      setAppUpdate(await api.checkAppUpdate());
    } catch (error) {
      setAppUpdateError(
        error instanceof Error
          ? error.message
          : "Nie udało się sprawdzić wersji aplikacji.",
      );
    } finally {
      setIsCheckingAppUpdate(false);
    }
  };

  const handleRemovePath = (id: string) => {
    setLocalConfig({
      ...localConfig,
      paths: {
        ...localConfig.paths,
        monitored: localConfig.paths.monitored.filter((p) => p.id !== id),
      },
    });
  };

  const handleTogglePath = (id: string) => {
    setLocalConfig({
      ...localConfig,
      paths: {
        ...localConfig.paths,
        monitored: localConfig.paths.monitored.map((p) =>
          p.id === id ? { ...p, enabled: !p.enabled } : p,
        ),
      },
    });
  };

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-10">
        {/* Backdrop */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={closeSettings}
          className="fixed inset-0 bg-background/80 backdrop-blur-sm"
        />

        {/* Modal Window */}
        <motion.div
          initial={{ opacity: 0, scale: 0.96, y: 8 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.96, y: 8 }}
          transition={{ type: "spring", stiffness: 350, damping: 25 }}
          className="relative w-full max-w-2xl max-h-[85vh] overflow-hidden rounded-2xl border border-border bg-card shadow-2xl flex flex-col"
        >
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-border">
            <div className="flex items-center gap-2">
              <Sliders className="w-5 h-5 text-primary" />
              <h2 className="text-base font-bold text-foreground">
                {t.preferences}
              </h2>
            </div>
            <button
              onClick={closeSettings}
              className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-muted transition-all"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Navigation tabs */}
          <div className="border-b border-border px-4 py-3 sm:px-6">
            <div className="grid grid-cols-2 gap-1 rounded-xl border border-border bg-muted/30 p-1 text-xs font-medium sm:grid-cols-3 lg:grid-cols-5">
              <button
                onClick={() => setActiveTab("paths")}
                className={`flex min-w-0 min-h-10 items-center justify-center gap-1.5 rounded-lg border px-2 transition-all ${
                  activeTab === "paths"
                    ? "border-primary/40 bg-primary/10 text-primary font-semibold shadow-sm"
                    : "border-transparent text-muted-foreground hover:bg-card hover:text-foreground"
                }`}
              >
                <FolderOpen className="w-3.5 h-3.5" />
                <span className="truncate">{t.monitoredPaths}</span>
              </button>

              <button
                onClick={() => setActiveTab("updates")}
                className={`flex min-w-0 min-h-10 items-center justify-center gap-1.5 rounded-lg border px-2 transition-all ${
                  activeTab === "updates"
                    ? "border-primary/40 bg-primary/10 text-primary font-semibold shadow-sm"
                    : "border-transparent text-muted-foreground hover:bg-card hover:text-foreground"
                }`}
              >
                <ArrowUpCircle className="w-3.5 h-3.5" />
                <span className="truncate">{t.updatesAndQueue}</span>
              </button>

              <button
                onClick={() => setActiveTab("appearance")}
                className={`flex min-w-0 min-h-10 items-center justify-center gap-1.5 rounded-lg border px-2 transition-all ${
                  activeTab === "appearance"
                    ? "border-primary/40 bg-primary/10 text-primary font-semibold shadow-sm"
                    : "border-transparent text-muted-foreground hover:bg-card hover:text-foreground"
                }`}
              >
                <Palette className="w-3.5 h-3.5" />
                <span className="truncate">{t.appearance}</span>
              </button>

              <button
                onClick={() => setActiveTab("general")}
                className={`flex min-w-0 min-h-10 items-center justify-center gap-1.5 rounded-lg border px-2 transition-all ${
                  activeTab === "general"
                    ? "border-primary/40 bg-primary/10 text-primary font-semibold shadow-sm"
                    : "border-transparent text-muted-foreground hover:bg-card hover:text-foreground"
                }`}
              >
                <Sliders className="w-3.5 h-3.5" />
                <span className="truncate">{t.general}</span>
              </button>

              <button
                onClick={() => setActiveTab("advanced")}
                className={`flex min-w-0 min-h-10 items-center justify-center gap-1.5 rounded-lg border px-2 transition-all ${
                  activeTab === "advanced"
                    ? "border-primary/40 bg-primary/10 text-primary font-semibold shadow-sm"
                    : "border-transparent text-muted-foreground hover:bg-card hover:text-foreground"
                }`}
              >
                <ShieldAlert className="w-3.5 h-3.5" />
                <span className="truncate">{t.advanced}</span>
              </button>
            </div>
          </div>

          {/* Content Area */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6 text-xs">
            {/* Paths Tab */}
            {activeTab === "paths" && (
              <div className="space-y-4">
                <div>
                  <h3 className="text-sm font-semibold text-foreground">
                    {t.configuredDirectories}
                  </h3>
                  <p className="text-muted-foreground text-[11px] mt-0.5">
                    {t.configuredDirectoriesDesc}
                  </p>
                </div>

                {/* Path list */}
                <div className="space-y-2">
                  {localConfig.paths.monitored.map((p) => (
                    <div
                      key={p.id}
                      className="p-3 rounded-xl border border-border bg-card/60 flex items-center justify-between gap-3"
                    >
                      <div className="flex items-center gap-2.5 overflow-hidden">
                        <input
                          type="checkbox"
                          checked={p.enabled}
                          onChange={() => handleTogglePath(p.id)}
                          className="w-4 h-4 rounded border-border text-primary focus:ring-primary"
                        />
                        <div className="truncate">
                          <p className="font-mono text-xs text-foreground truncate">
                            {p.path}
                          </p>
                          <span className="text-[10px] uppercase font-semibold text-primary/80">
                            Scope: {p.scope}
                          </span>
                        </div>
                      </div>

                      <button
                        onClick={() => handleRemovePath(p.id)}
                        className="w-7 h-7 rounded-lg flex items-center justify-center text-muted-foreground hover:text-rose-400 hover:bg-rose-500/10 transition-all shrink-0"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  ))}
                </div>

                {/* Add new path input */}
                <div className="pt-2 border-t border-border/60">
                  <h4 className="text-xs font-semibold text-foreground mb-2">
                    {t.addNewPath}
                  </h4>
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={newPathInput}
                      onChange={(e) => setNewPathInput(e.target.value)}
                      placeholder="/Users/username/ai-skills or ~/projects/my-skills"
                      className="flex-1 h-9 px-3 text-xs rounded-lg border border-border bg-card text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary"
                    />
                    <select
                      value={newScopeInput}
                      onChange={(e) => setNewScopeInput(e.target.value)}
                      className="h-9 px-2 text-xs rounded-lg border border-border bg-card text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
                    >
                      <option value="global">Global</option>
                      <option value="codex">Codex</option>
                      <option value="claude">Claude</option>
                      <option value="cursor">Cursor</option>
                      <option value="antigravity">Antigravity</option>
                      <option value="custom">Custom</option>
                    </select>
                    <button
                      onClick={handleAddPath}
                      disabled={!newPathInput.trim()}
                      className="h-9 px-3 rounded-lg bg-primary hover:bg-primary/90 text-primary-foreground font-semibold flex items-center gap-1 transition-all disabled:opacity-40"
                    >
                      <Plus className="w-4 h-4" />
                      {t.add}
                    </button>
                  </div>
                </div>
              </div>
            )}

            {/* Updates Tab */}
            {activeTab === "updates" && (
              <div className="space-y-4">
                <div>
                  <h3 className="text-sm font-semibold text-foreground">
                    {t.updatesAndQueue}
                  </h3>
                  <p className="text-muted-foreground text-[11px] mt-0.5">
                    {t.autoCheckFrequencyDesc}
                  </p>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                    <div>
                      <p className="font-semibold text-foreground">
                        {t.autoCheckFrequency}
                      </p>
                      <p className="text-[11px] text-muted-foreground">
                        {t.autoCheckFrequencyDesc}
                      </p>
                    </div>
                    <select
                      value={localConfig.updates.autoCheckFrequency}
                      onChange={(e) =>
                        setLocalConfig({
                          ...localConfig,
                          updates: {
                            ...localConfig.updates,
                            autoCheckFrequency: e.target
                              .value as AppConfig["updates"]["autoCheckFrequency"],
                          },
                        })
                      }
                      className="h-8 px-2 text-xs rounded-lg border border-border bg-card text-foreground"
                    >
                      <option value="hourly">Every Hour</option>
                      <option value="every_6_hours">Every 6 Hours</option>
                      <option value="daily">Daily</option>
                      <option value="manual">Manual Only</option>
                    </select>
                  </div>

                  <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                    <div>
                      <p className="font-semibold text-foreground">
                        {t.parallelWorkers}
                      </p>
                      <p className="text-[11px] text-muted-foreground">
                        {t.parallelWorkersDesc}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      <input
                        type="range"
                        min="1"
                        max="10"
                        value={localConfig.updates.concurrencyLimit}
                        onChange={(e) =>
                          setLocalConfig({
                            ...localConfig,
                            updates: {
                              ...localConfig.updates,
                              concurrencyLimit: Number(e.target.value),
                            },
                          })
                        }
                        className="w-24 accent-primary"
                      />
                      <span className="font-mono text-xs w-5 text-right font-bold text-foreground">
                        {localConfig.updates.concurrencyLimit}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                    <div>
                      <p className="font-semibold text-foreground">
                        {t.backupRetention}
                      </p>
                      <p className="text-[11px] text-muted-foreground">
                        {t.backupRetentionDesc}
                      </p>
                    </div>
                    <div className="flex items-center gap-1.5 font-mono">
                      <input
                        type="number"
                        min="1"
                        max="90"
                        value={localConfig.updates.backupRetentionDays}
                        onChange={(e) =>
                          setLocalConfig({
                            ...localConfig,
                            updates: {
                              ...localConfig.updates,
                              backupRetentionDays: Number(e.target.value),
                            },
                          })
                        }
                        className="w-16 h-8 px-2 text-xs rounded-lg border border-border bg-card text-foreground"
                      />
                      <span className="text-muted-foreground text-[11px]">
                        {t.days}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Appearance Tab */}
            {activeTab === "appearance" && (
              <div className="space-y-4">
                <div>
                  <h3 className="text-sm font-semibold text-foreground">
                    {t.themeTitle}
                  </h3>
                  <p className="text-muted-foreground text-[11px] mt-0.5">
                    {t.themeDesc}
                  </p>
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <button
                    onClick={() => {
                      setTheme("dark");
                      setLocalConfig({
                        ...localConfig,
                        appearance: {
                          ...localConfig.appearance,
                          theme: "dark",
                        },
                      });
                    }}
                    className={`p-4 rounded-xl border text-left flex flex-col justify-between h-24 transition-all ${
                      theme === "dark"
                        ? "border-primary bg-primary/10 shadow-sm shadow-primary/20"
                        : "border-border bg-card/60 hover:bg-card"
                    }`}
                  >
                    <span className="font-semibold text-foreground">
                      {t.darkTheme}
                    </span>
                    <span className="text-[11px] text-muted-foreground">
                      {t.darkThemeDesc}
                    </span>
                  </button>

                  <button
                    onClick={() => {
                      setTheme("light");
                      setLocalConfig({
                        ...localConfig,
                        appearance: {
                          ...localConfig.appearance,
                          theme: "light",
                        },
                      });
                    }}
                    className={`p-4 rounded-xl border text-left flex flex-col justify-between h-24 transition-all ${
                      theme === "light"
                        ? "border-primary bg-primary/10 shadow-sm shadow-primary/20"
                        : "border-border bg-card/60 hover:bg-card"
                    }`}
                  >
                    <span className="font-semibold text-foreground">
                      {t.lightTheme}
                    </span>
                    <span className="text-[11px] text-muted-foreground">
                      {t.lightThemeDesc}
                    </span>
                  </button>
                </div>
              </div>
            )}

            {/* General Tab (Language & Startup) */}
            {activeTab === "general" && (
              <div className="space-y-4">
                <div className="rounded-xl border border-border bg-card/60 p-4">
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <div>
                      <p className="font-semibold text-foreground">
                        {appUpdateCopy.title}
                      </p>
                      <p className="mt-0.5 text-[11px] text-muted-foreground">
                        {appUpdateCopy.current}:{" "}
                        {appVersion ? `v${appVersion}` : "—"}
                      </p>
                    </div>
                    <button
                      onClick={handleCheckAppUpdate}
                      disabled={isCheckingAppUpdate}
                      className="flex h-9 items-center gap-1.5 rounded-lg border border-border bg-card px-3 text-xs font-semibold text-foreground transition-all hover:bg-muted disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      <RefreshCw
                        className={`h-3.5 w-3.5 ${isCheckingAppUpdate ? "animate-spin text-primary" : ""}`}
                      />
                      {isCheckingAppUpdate
                        ? appUpdateCopy.checking
                        : appUpdateCopy.check}
                    </button>
                  </div>
                  {appUpdate && (
                    <p
                      className={`mt-3 text-[11px] ${appUpdate.updateAvailable ? "text-amber-400" : "text-emerald-400"}`}
                    >
                      {appUpdate.latestVersion
                        ? appUpdate.updateAvailable
                          ? `${appUpdateCopy.available}: v${appUpdate.latestVersion}.`
                          : appUpdateCopy.upToDate
                        : appUpdateCopy.noRelease}
                    </p>
                  )}
                  {appUpdateError && (
                    <p className="mt-3 text-[11px] text-rose-400">
                      {appUpdateError}
                    </p>
                  )}
                </div>

                {/* Language Selection Grid */}
                <div className="p-4 rounded-xl border border-border bg-card/60 space-y-2">
                  <div className="flex items-center gap-2 text-foreground font-semibold">
                    <Globe className="w-4 h-4 text-primary" />
                    <span>{t.languageSelect}</span>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Select your preferred language interface / Wybierz język
                    interfejsu.
                  </p>

                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 pt-2">
                    {supportedLanguages.map((lang) => (
                      <button
                        key={lang.code}
                        onClick={() => {
                          setLanguage(lang.code);
                          setLocalConfig({
                            ...localConfig,
                            general: {
                              ...localConfig.general,
                              language: lang.code,
                            },
                          });
                        }}
                        className={`p-2.5 rounded-lg border text-left text-xs flex items-center justify-between transition-all ${
                          language === lang.code
                            ? "border-primary bg-primary/15 text-primary font-semibold"
                            : "border-border bg-background hover:bg-muted text-foreground"
                        }`}
                      >
                        <span className="flex items-center gap-2">
                          <span className="text-base">{lang.flag}</span>
                          <span>{lang.label}</span>
                        </span>
                        {language === lang.code && (
                          <Check className="w-3.5 h-3.5 text-primary" />
                        )}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                  <div>
                    <p className="font-semibold text-foreground">
                      {t.launchAtLogin}
                    </p>
                    <p className="text-[11px] text-muted-foreground">
                      {t.launchAtLoginDesc}
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localConfig.general.launchAtLogin}
                    onChange={(e) =>
                      setLocalConfig({
                        ...localConfig,
                        general: {
                          ...localConfig.general,
                          launchAtLogin: e.target.checked,
                        },
                      })
                    }
                    className="w-4 h-4 rounded border-border text-primary focus:ring-primary"
                  />
                </div>

                <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                  <div>
                    <p className="font-semibold text-foreground">
                      {t.minimizeToTray}
                    </p>
                    <p className="text-[11px] text-muted-foreground">
                      {t.minimizeToTrayDesc}
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    checked={localConfig.general.minimizeToTray}
                    onChange={(e) =>
                      setLocalConfig({
                        ...localConfig,
                        general: {
                          ...localConfig.general,
                          minimizeToTray: e.target.checked,
                        },
                      })
                    }
                    className="w-4 h-4 rounded border-border text-primary focus:ring-primary"
                  />
                </div>
              </div>
            )}

            {/* Advanced Tab */}
            {activeTab === "advanced" && (
              <div className="space-y-3">
                <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                  <div>
                    <p className="font-semibold text-foreground">
                      {t.gitTimeout}
                    </p>
                    <p className="text-[11px] text-muted-foreground">
                      {t.gitTimeoutDesc}
                    </p>
                  </div>
                  <input
                    type="number"
                    min="10"
                    max="120"
                    value={localConfig.advanced.gitTimeoutSeconds}
                    onChange={(e) =>
                      setLocalConfig({
                        ...localConfig,
                        advanced: {
                          ...localConfig.advanced,
                          gitTimeoutSeconds: Number(e.target.value),
                        },
                      })
                    }
                    className="w-16 h-8 px-2 text-xs rounded-lg border border-border bg-card text-foreground font-mono"
                  />
                </div>

                <div className="flex items-center justify-between p-3 rounded-xl border border-border bg-card/60">
                  <div>
                    <p className="font-semibold text-foreground">
                      {t.logLevel}
                    </p>
                    <p className="text-[11px] text-muted-foreground">
                      {t.logLevelDesc}
                    </p>
                  </div>
                  <select
                    value={localConfig.advanced.logLevel}
                    onChange={(e) =>
                      setLocalConfig({
                        ...localConfig,
                        advanced: {
                          ...localConfig.advanced,
                          logLevel: e.target
                            .value as AppConfig["advanced"]["logLevel"],
                        },
                      })
                    }
                    className="h-8 px-2 text-xs rounded-lg border border-border bg-card text-foreground font-mono"
                  >
                    <option value="info">INFO</option>
                    <option value="debug">DEBUG</option>
                    <option value="warn">WARN</option>
                    <option value="error">ERROR</option>
                  </select>
                </div>
              </div>
            )}
          </div>

          {/* Footer Actions */}
          <div className="px-6 py-4 border-t border-border flex items-center justify-between">
            <span className="text-[11px] text-muted-foreground font-mono">
              Config: {language.toUpperCase()}
            </span>

            <div className="flex items-center gap-2">
              <button
                onClick={closeSettings}
                className="px-3.5 py-1.5 text-xs font-medium rounded-lg border border-border text-foreground hover:bg-muted transition-all"
              >
                {t.cancel}
              </button>
              <button
                onClick={handleSave}
                className="px-4 py-1.5 text-xs font-semibold rounded-lg bg-primary hover:bg-primary/90 text-primary-foreground shadow-sm shadow-primary/25 transition-all flex items-center gap-1.5"
              >
                {savedToast ? <Check className="w-3.5 h-3.5" /> : null}
                {savedToast ? t.saved : t.savePreferences}
              </button>
            </div>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
};
