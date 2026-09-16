import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  } catch {
    return dateString;
  }
}

export function getScopeBadgeColor(scope: string): string {
  switch (scope.toLowerCase()) {
    case 'codex':
      return 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30';
    case 'claude':
      return 'bg-amber-500/15 text-amber-400 border-amber-500/30';
    case 'cursor':
      return 'bg-cyan-500/15 text-cyan-400 border-cyan-500/30';
    case 'antigravity':
      return 'bg-purple-500/15 text-purple-400 border-purple-500/30';
    case 'global':
      return 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30';
    default:
      return 'bg-slate-500/15 text-slate-400 border-slate-500/30';
  }
}
