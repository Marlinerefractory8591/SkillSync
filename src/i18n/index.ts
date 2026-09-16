import { pl } from './locales/pl';
import { en } from './locales/en';
import { de } from './locales/de';
import { es } from './locales/es';
import { fr } from './locales/fr';
import { ja } from './locales/ja';
import { zh } from './locales/zh';
import { Language, Translations } from './types';

export const translations: Record<Language, Translations> = {
  pl,
  en,
  de,
  es,
  fr,
  ja,
  zh,
};

export const supportedLanguages: { code: Language; label: string; flag: string }[] = [
  { code: 'pl', label: 'Polski', flag: '🇵🇱' },
  { code: 'en', label: 'English', flag: '🇺🇸' },
  { code: 'de', label: 'Deutsch', flag: '🇩🇪' },
  { code: 'es', label: 'Español', flag: '🇪🇸' },
  { code: 'fr', label: 'Français', flag: '🇫🇷' },
  { code: 'ja', label: '日本語', flag: '🇯🇵' },
  { code: 'zh', label: '简体中文', flag: '🇨🇳' },
];

export * from './types';
export * from './useTranslation';
