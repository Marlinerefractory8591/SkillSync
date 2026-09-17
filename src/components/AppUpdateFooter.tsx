import React from "react";
import {
  Download,
  Loader2,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
} from "lucide-react";
import { useSkillStore } from "../store/useSkillStore";

export const AppUpdateFooter: React.FC = () => {
  const {
    language,
    appUpdate,
    appUpdateError,
    isCheckingAppUpdate,
    checkAppUpdate,
    installAppUpdate,
    appUpdateProgress,
  } = useSkillStore();
  const isPolish = language === "pl";

  const copy = isPolish
    ? {
        title: "Aktualizacje SkillSync",
        checking: "Sprawdzanie bezpiecznej aktualizacji…",
        downloading: "Pobieranie aktualizacji",
        installing: "Instalowanie podpisanej aktualizacji…",
        restarting: "Ponowne uruchamianie SkillSync…",
        update: "Dostępna wersja",
        current: "Masz najnowszą wersję",
        unavailable: "Nie sprawdzono jeszcze wersji",
        check: "Sprawdź teraz",
        download: "Pobierz i zainstaluj",
        secure: "Podpisana aktualizacja z GitHub Releases",
      }
    : {
        title: "SkillSync updates",
        checking: "Checking for a secure update…",
        downloading: "Downloading update",
        installing: "Installing signed update…",
        restarting: "Restarting SkillSync…",
        update: "Version available",
        current: "You have the latest version",
        unavailable: "Version has not been checked yet",
        check: "Check now",
        download: "Download and install",
        secure: "Signed update from GitHub Releases",
      };

  const updateProgress = appUpdateProgress.contentLength
    ? Math.min(
        100,
        Math.round(
          (appUpdateProgress.downloadedBytes /
            appUpdateProgress.contentLength) *
            100,
        ),
      )
    : null;
  const isInstalling = [
    "checking",
    "downloading",
    "installing",
    "restarting",
  ].includes(appUpdateProgress.phase);
  const status =
    isInstalling || isCheckingAppUpdate
      ? appUpdateProgress.phase === "downloading"
        ? `${copy.downloading}${updateProgress === null ? "…" : `: ${updateProgress}%`}`
        : appUpdateProgress.phase === "installing"
          ? copy.installing
          : appUpdateProgress.phase === "restarting"
            ? copy.restarting
            : copy.checking
      : appUpdateError
        ? appUpdateError
        : appUpdate?.updateAvailable
          ? `${copy.update}: v${appUpdate.latestVersion}`
          : appUpdate
            ? copy.current
            : copy.unavailable;

  return (
    <footer className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-background/95 px-4 py-2.5 shadow-[0_-8px_24px_hsl(var(--background)/0.45)] backdrop-blur-md sm:px-6">
      <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-2 text-xs">
        <div className="flex min-w-0 items-center gap-2">
          {isInstalling || isCheckingAppUpdate ? (
            <Loader2 className="h-4 w-4 shrink-0 animate-spin text-primary" />
          ) : (
            <ShieldCheck
              className={`h-4 w-4 shrink-0 ${
                appUpdate?.updateAvailable
                  ? "text-amber-400"
                  : "text-emerald-400"
              }`}
            />
          )}
          <div className="min-w-0">
            <span className="mr-2 font-semibold text-foreground">
              {copy.title}
            </span>
            <span
              className={`truncate text-[11px] ${
                appUpdateError
                  ? "text-rose-400"
                  : appUpdate?.updateAvailable
                    ? "text-amber-400"
                    : "text-muted-foreground"
              }`}
            >
              {status}
            </span>
          </div>
        </div>

        <div className="flex shrink-0 items-center gap-2">
          {appUpdate?.updateAvailable && (
            <button
              onClick={() => void installAppUpdate()}
              disabled={isInstalling}
              className="inline-flex h-8 items-center gap-1.5 rounded-lg bg-primary px-3 text-[11px] font-semibold text-primary-foreground transition-colors hover:bg-primary/90"
              title={copy.secure}
            >
              {appUpdateProgress.phase === "restarting" ? (
                <RotateCcw className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Download className="h-3.5 w-3.5" />
              )}
              {copy.download}
            </button>
          )}
          <button
            onClick={() => void checkAppUpdate()}
            disabled={isInstalling || isCheckingAppUpdate}
            className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-border bg-card px-3 text-[11px] font-semibold text-foreground transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-60"
          >
            <RefreshCw
              className={`h-3.5 w-3.5 ${isInstalling || isCheckingAppUpdate ? "animate-spin" : ""}`}
            />
            {copy.check}
          </button>
        </div>
      </div>
    </footer>
  );
};
