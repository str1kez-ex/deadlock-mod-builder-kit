// i18n/index.svelte.ts — Lightweight reactive translation manager (Svelte 5 Runes)
import { browser } from '$app/environment';
import ru from './ru.json';
import en from './en.json';

type TranslationMap = Record<string, string>;

const translations: Record<string, TranslationMap> = { ru, en };

// Reactive language state — stored in localStorage + config
let _lang = $state<string>('ru');

export function setLanguage(lang: 'ru' | 'en') {
  _lang = lang;
  if (browser) {
    localStorage.setItem('dmbk_lang', lang);
  }
}

export function getLanguage(): string {
  return _lang;
}

export function initLanguage(configLang?: string | null) {
  const saved = browser ? localStorage.getItem('dmbk_lang') : null;
  const resolved = saved || configLang || 'ru';
  _lang = resolved as 'ru' | 'en';
}

/**
 * Translate a key. Falls back to en, then to the key itself.
 */
export function t(key: string): string {
  const map = translations[_lang] ?? translations['ru'];
  return map[key] ?? translations['en']?.[key] ?? key;
}
