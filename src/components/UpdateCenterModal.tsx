import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSkillStore } from "../store/useSkillStore";
import { useTranslation } from "../i18n/useTranslation";
import { formatSkillVersion } from "../lib/version-display";
import {
  X,
  Layers,
  ArrowUpCircle,
  Clock,
  Loader2,
  CheckCircle2,
  AlertCircle,
} from "lucide-react";

export const UpdateCenterModal: React.FC = () => {
  const { t, language } = useTranslation();
  const {
    isUpdateCenterOpen,
    closeUpdateCenter,
    skills,
    batchUpdating,
    batchProgress,
    batchUpdateAll,
  } = useSkillStore();

  if (!isUpdateCenterOpen) return null;

  const outdated = skills.filter((s) => s.updateAvailable);
  const upToDate = skills.filter(
    (s) => !s.updateAvailable && s.status !== "error",
  );

  const progressPct =
    batchProgress.total > 0
      ? Math.round((batchProgress.completed / batchProgress.total) * 100)
      : 0;

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-10">
        {/* Backdrop */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={closeUpdateCenter}
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
              <Layers className="w-5 h-5 text-primary" />
              <h2 className="text-base font-bold text-foreground">
                {t("updateCenterTitle")}
              </h2>
            </div>
            <button
              onClick={closeUpdateCenter}
              className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-muted transition-all"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* Body */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6 text-xs">
            {/* Active Batch Progress Card */}
            {batchUpdating && (
              <div className="p-4 rounded-xl border border-primary/30 bg-primary/10 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Loader2 className="w-4 h-4 text-primary animate-spin" />
                    <span className="font-semibold text-foreground">
                      {t("batchInProgress")} ({batchProgress.completed}/
                      {batchProgress.total})
                    </span>
                  </div>
                  <span className="font-mono font-bold text-primary">
                    {progressPct}%
                  </span>
                </div>

                {/* Progress bar */}
                <div className="w-full h-2 rounded-full bg-primary/20 overflow-hidden">
                  <motion.div
                    className="h-full bg-primary rounded-full"
                    initial={{ width: 0 }}
                    animate={{ width: `${progressPct}%` }}
                    transition={{ ease: "easeInOut", duration: 0.25 }}
                  />
                </div>

                {batchProgress.currentItem && (
                  <p className="text-[11px] text-muted-foreground font-mono">
                    {t("currentlyUpdating")}:{" "}
                    <strong className="text-foreground">
                      {batchProgress.currentItem}
                    </strong>
                  </p>
                )}
              </div>
            )}

            {/* Outdated Skills Section */}
            <div>
              <div className="flex items-center justify-between mb-3">
                <h3 className="font-semibold text-sm text-foreground flex items-center gap-1.5">
                  <AlertCircle className="w-4 h-4 text-amber-400" />
                  {t("availableUpdates")} ({outdated.length})
                </h3>

                {outdated.length > 0 && !batchUpdating && (
                  <button
                    onClick={() => batchUpdateAll()}
                    className="px-3 py-1 rounded-md text-xs font-semibold bg-primary hover:bg-primary/90 text-primary-foreground shadow-sm transition-all flex items-center gap-1"
                  >
                    <ArrowUpCircle className="w-3.5 h-3.5" />
                    {t("updateAll")}
                  </button>
                )}
              </div>

              {outdated.length === 0 ? (
                <div className="p-4 rounded-xl border border-border bg-card/40 text-center text-muted-foreground">
                  {t("allUpToDate")}
                </div>
              ) : (
                <div className="space-y-2">
                  {outdated.map((s) => (
                    <div
                      key={s.id}
                      className="p-3 rounded-xl border border-border bg-card/60 flex items-center justify-between"
                    >
                      <div>
                        <p className="font-semibold text-foreground text-xs">
                          {s.name}
                        </p>
                        <p className="text-[11px] font-mono text-muted-foreground">
                          {formatSkillVersion(s.currentVersion, language)} ➔{" "}
                          <span className="text-amber-400 font-semibold">
                            {formatSkillVersion(s.latestVersion, language)}
                          </span>
                        </p>
                      </div>

                      <span className="px-2 py-0.5 rounded text-[10px] bg-amber-500/10 text-amber-400 border border-amber-500/20 font-medium">
                        {t("pending")}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Up to Date Skills Section */}
            <div>
              <h3 className="font-semibold text-sm text-foreground flex items-center gap-1.5 mb-3">
                <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                {t("verified")} ({upToDate.length})
              </h3>

              <div className="space-y-2 max-h-48 overflow-y-auto">
                {upToDate.map((s) => (
                  <div
                    key={s.id}
                    className="p-2.5 rounded-lg border border-border/60 bg-card/30 flex items-center justify-between text-xs"
                  >
                    <span className="font-medium text-foreground">
                      {s.name}
                    </span>
                    <span className="font-mono text-muted-foreground text-[11px]">
                      {formatSkillVersion(s.currentVersion, language)}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Historical Audit Info */}
            <div className="p-3 rounded-xl border border-border bg-muted/20 flex items-center gap-3 text-xs text-muted-foreground">
              <Clock className="w-4 h-4 text-muted-foreground shrink-0" />
              <span>{t("backupInfo")}</span>
            </div>
          </div>

          {/* Footer */}
          <div className="px-6 py-4 border-t border-border flex justify-end">
            <button
              onClick={closeUpdateCenter}
              className="px-4 py-1.5 text-xs font-medium rounded-lg border border-border text-foreground hover:bg-muted transition-all"
            >
              {t("done")}
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
};
