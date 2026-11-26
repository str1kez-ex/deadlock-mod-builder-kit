// stores/settings.svelte.ts — UI scale, language, theme (Svelte 5 Runes)
import { browser } from '$app/environment';
import { invoke } from '@tauri-apps/api/core';
import { setLanguage, initLanguage } from '$lib/i18n/index.svelte';

// ── UI Scale ──────────────────────────────────────────────────────────────────
export const SCALE_OPTIONS = [80, 90, 100, 110, 125, 140] as const;
export type ScaleValue = (typeof SCALE_OPTIONS)[number];

let _uiScale = $state<ScaleValue>(100);

export function getUiScale(): ScaleValue {
  return _uiScale;
}

export function setUiScale(scale: ScaleValue) {
  _uiScale = scale;
  applyScale(scale);
  if (browser) {
    localStorage.setItem('dmbk_ui_scale', String(scale));
    invoke('set_ui_scale', { scale }).catch(() => {});
  }
}

function applyScale(scale: number) {
  if (browser && typeof document !== 'undefined') {
    const el = document.getElementById('svelte') as HTMLElement | null;
    if (el) {
      el.style.zoom = `${scale}%`;
    }
  }
}

export function initUiScale(configScale?: number | null) {
  const saved = browser ? localStorage.getItem('dmbk_ui_scale') : null;
  const resolved = Number(saved || configScale || 100) as ScaleValue;
  _uiScale = resolved;
  applyScale(resolved);
}

// ── Language ──────────────────────────────────────────────────────────────────
export type Lang = 'ru' | 'en';

let _lang = $state<Lang>('ru');

export function getLang(): Lang {
  return _lang;
}

export function setLang(lang: Lang) {
  _lang = lang;
  setLanguage(lang);
  if (browser) {
    localStorage.setItem('dmbk_lang', lang);
    if (typeof document !== 'undefined') {
      document.documentElement.lang = lang;
    }
    invoke('set_ui_language', { lang }).catch(() => {});
  }
}

export function initLang(configLang?: string | null) {
  initLanguage(configLang);
  const saved = browser ? localStorage.getItem('dmbk_lang') : null;
  const resolved = ((saved || configLang || 'ru') === 'en' ? 'en' : 'ru') as Lang;
  _lang = resolved;
  if (browser && typeof document !== 'undefined') {
    document.documentElement.lang = resolved;
  }
}
