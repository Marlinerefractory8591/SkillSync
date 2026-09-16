const UNKNOWN_VERSION_VALUES = new Set(['', 'unknown']);

export function isKnownSkillVersion(version: string | null | undefined): boolean {
  return !UNKNOWN_VERSION_VALUES.has((version ?? '').trim().toLowerCase());
}

export function formatSkillVersion(version: string | null | undefined, language: string): string {
  if (!isKnownSkillVersion(version)) {
    return language === 'pl' ? 'Nieznana wersja' : 'Unknown version';
  }

  return `v${version}`;
}
