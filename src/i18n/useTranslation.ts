import { useSkillStore } from "../store/useSkillStore";
import { Translations } from "./types";
import { translations } from "./index";

export type TFunction = Translations & ((key: keyof Translations) => string);

export const useTranslation = () => {
  const language = useSkillStore((state) => state.language);
  const setLanguage = useSkillStore((state) => state.setLanguage);

  const currentStrings: Translations =
    translations[language] || translations.en;

  const t = ((key: keyof Translations): string => {
    return currentStrings[key] ?? (key as string);
  }) as TFunction;

  // Copy all properties onto the callable function
  Object.assign(t, currentStrings);

  return { t, language, setLanguage };
};
