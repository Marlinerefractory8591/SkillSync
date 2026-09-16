import React from 'react';
import { AlertTriangle, ShieldCheck } from 'lucide-react';
import { useSkillStore } from '../store/useSkillStore';

export const DirtyWorktreeConfirmation: React.FC = () => {
  const {
    pendingDirtyUpdate,
    confirmDirtyUpdate,
    dismissDirtyUpdate,
    language,
  } = useSkillStore();

  if (!pendingDirtyUpdate) return null;

  const isPolish = language === 'pl';
  const copy = isPolish
    ? {
        eyebrow: 'Lokalne zmiany wymagają decyzji',
        title: `Zaktualizować ${pendingDirtyUpdate.skillName} mimo zmian?`,
        body: 'Git wykrył zmodyfikowane śledzone pliki. Wymuszenie aktualizacji może zastąpić ich bieżącą zawartość wersją z upstreamu.',
        snapshot: 'Przed wymuszeniem zostanie utworzona nowa migawka bezpieczeństwa, aby można było wykonać rollback.',
        cancel: 'Nie aktualizuj',
        confirm: 'Aktualizuj mimo zmian',
      }
    : {
        eyebrow: 'Local changes need a decision',
        title: `Update ${pendingDirtyUpdate.skillName} despite local changes?`,
        body: 'Git found modified tracked files. A forced update can replace their current contents with the upstream version.',
        snapshot: 'A new safety snapshot will be created before the forced update so rollback remains available.',
        cancel: 'Do not update',
        confirm: 'Update despite changes',
      };

  return (
    <div className="fixed inset-0 z-[80] flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby="dirty-update-title">
      <button
        aria-label={copy.cancel}
        onClick={dismissDirtyUpdate}
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
      />
      <section className="relative w-full max-w-lg rounded-2xl border border-amber-500/35 bg-card p-6 shadow-2xl">
        <div className="flex gap-3">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/15 text-amber-400">
            <AlertTriangle className="h-5 w-5" />
          </div>
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-wider text-amber-400">{copy.eyebrow}</p>
            <h2 id="dirty-update-title" className="mt-1 text-base font-semibold text-foreground">{copy.title}</h2>
          </div>
        </div>
        <p className="mt-4 text-sm leading-relaxed text-muted-foreground">{copy.body}</p>
        <p className="mt-3 flex gap-2 rounded-lg border border-border bg-muted/40 p-3 text-xs leading-relaxed text-foreground/90">
          <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
          {copy.snapshot}
        </p>
        <p className="mt-3 break-words text-[11px] text-muted-foreground">{pendingDirtyUpdate.reason}</p>
        <div className="mt-5 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <button onClick={dismissDirtyUpdate} className="rounded-lg border border-border px-4 py-2 text-xs font-semibold text-foreground transition-colors hover:bg-muted">
            {copy.cancel}
          </button>
          <button onClick={confirmDirtyUpdate} className="rounded-lg bg-amber-500 px-4 py-2 text-xs font-semibold text-amber-950 transition-colors hover:bg-amber-400">
            {copy.confirm}
          </button>
        </div>
      </section>
    </div>
  );
};
