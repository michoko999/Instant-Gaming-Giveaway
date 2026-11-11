// État global de l'interface (runes Svelte 5), traductions et actions partagées.
import {
  api,
  events,
  type CheckProgress,
  type Lang,
  type LogEntry,
  type Overview,
  type Progress,
  type SessionMode,
  type SessionSummary,
  type Settings,
  type UpdateInfo,
} from './api';
import { fr, type DictKey, type Dict } from './i18n/fr';
import { en } from './i18n/en';
import { es } from './i18n/es';
import { de } from './i18n/de';
import { pt } from './i18n/pt';
import { it } from './i18n/it';
import { pl } from './i18n/pl';

export type Screen = 'loading' | 'onboarding' | 'home' | 'session' | 'recap' | 'giveaways' | 'history' | 'settings';

const DICTS: Record<Lang, Dict> = { fr, en, es, de, pt, it, pl };

export const LANGUAGES: { code: Lang; label: string }[] = [
  { code: 'fr', label: 'Français' },
  { code: 'en', label: 'English' },
  { code: 'es', label: 'Español' },
  { code: 'de', label: 'Deutsch' },
  { code: 'pt', label: 'Português' },
  { code: 'it', label: 'Italiano' },
  { code: 'pl', label: 'Polski' },
];

export const ACCENTS = ['#FFD27A', '#B9A8FF', '#7FE3D4', '#FF9FB8'];

export const app = $state({
  lang: 'fr' as Lang,
  screen: 'loading' as Screen,
  overview: null as Overview | null,
  session: null as Progress | null,
  logs: [] as LogEntry[],
  summary: null as SessionSummary | null,
  check: null as CheckProgress | null,
  syncing: false,
  checking: false,
  update: null as UpdateInfo | null,
  toast: null as { text: string; tone: 'info' | 'error' } | null,
});

/** Traduit une clé, avec remplacement des {variables}. */
export function t(key: DictKey, vars: Record<string, string | number> = {}): string {
  const dict = DICTS[app.lang] ?? fr;
  let text = dict[key] ?? fr[key] ?? key;
  for (const [name, value] of Object.entries(vars)) {
    text = text.replaceAll(`{${name}}`, String(value));
  }
  return text;
}

/** Traduit un code d'erreur renvoyé par Rust, sinon affiche le message brut. */
export function errorText(error: unknown): string {
  const raw = typeof error === 'string' ? error : error instanceof Error ? error.message : String(error);
  const key = `error.${raw}` as DictKey;
  if (key in fr) return t(key);
  return t('error.generic', { message: raw });
}

/** Message court pour un code connu (journal, historique), sinon le texte tel quel. */
export function shortMessage(message: string | null | undefined): string {
  if (!message) return '';
  const key = `error.${message}` as DictKey;
  return key in fr ? t(key) : message;
}

let toastTimer: ReturnType<typeof setTimeout> | undefined;
export function toast(text: string, tone: 'info' | 'error' = 'info') {
  app.toast = { text, tone };
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (app.toast = null), tone === 'error' ? 7000 : 4500);
}

export function fail(error: unknown) {
  toast(errorText(error), 'error');
}

export function since(iso: string | null | undefined): string {
  if (!iso) return t('time.never');
  const seconds = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (seconds < 60) return t('time.justNow');
  if (seconds < 3600) return t('time.minutes', { n: Math.floor(seconds / 60) });
  if (seconds < 86400) return t('time.hours', { n: Math.floor(seconds / 3600) });
  return t('time.days', { n: Math.floor(seconds / 86400) });
}

export function duration(totalSeconds: number): string {
  const s = Math.max(0, Math.round(totalSeconds));
  if (s < 60) return t('duration.sec', { s });
  const m = Math.floor(s / 60);
  const rest = s % 60;
  if (m >= 10 || rest === 0) return t('duration.min', { m: rest >= 30 ? m + 1 : m });
  return t('duration.minSec', { m, s: rest });
}

export function clockTime(iso: string): string {
  return new Date(iso).toLocaleTimeString(app.lang, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

export function dateTime(iso: string): string {
  return new Date(iso).toLocaleString(app.lang, { dateStyle: 'medium', timeStyle: 'short' });
}

export function applyAccent(accent: string) {
  document.documentElement.style.setProperty('--accent', accent);
}

export async function refresh() {
  const overview = await api.overview();
  app.overview = overview;
  app.lang = overview.settings.language;
  document.documentElement.lang = app.lang;
  applyAccent(overview.settings.accent);
  return overview;
}

export async function saveSettings(patch: Partial<Settings>) {
  if (!app.overview) return;
  try {
    const next = { ...app.overview.settings, ...patch };
    const saved = await api.saveSettings(next);
    app.overview.settings = saved;
    app.lang = saved.language;
    document.documentElement.lang = app.lang;
    applyAccent(saved.accent);
    await refresh();
  } catch (e) {
    fail(e);
  }
}

/** Récupère les listes puis vérifie les concours (tous, ou seulement les nouveaux). */
export async function updateLists(scope: 'all' | 'unknown' = 'unknown', withSync = true) {
  if (app.syncing || app.checking) return;
  try {
    if (withSync) {
      app.syncing = true;
      const report = await api.sync();
      app.syncing = false;
      if (report.sources.every((s) => !s.ok)) {
        fail(report.sources.map((s) => `${s.name} : ${s.error ?? '?'}`).join(' · '));
      }
    }
    app.checking = true;
    app.check = null;
    await api.check(scope);
  } catch (e) {
    fail(e);
  } finally {
    app.syncing = false;
    app.checking = false;
    app.check = null;
    await refresh().catch(() => {});
  }
}

export async function startSession(mode: SessionMode, codes?: string[]) {
  try {
    app.logs = [];
    app.summary = null;
    app.session = null;
    const total = await api.startSession(mode, codes);
    // Le premier événement de progression peut arriver avant la réponse : on ne l'écrase pas.
    app.session ??= {
      state: 'running',
      mode,
      index: 0,
      total,
      current: null,
      participated: 0,
      already: 0,
      opened: 0,
      ended: 0,
      errors: 0,
      skipped: 0,
      pending: 0,
      remainingSecs: 0,
      countdown: null,
    };
    app.screen = 'session';
  } catch (e) {
    fail(e);
  }
}

let listenersReady = false;
export async function setupListeners() {
  if (listenersReady) return;
  listenersReady = true;
  await events.sessionProgress((progress) => {
    if (progress.state === 'finished') return;
    app.session = progress;
    if (app.screen !== 'session') app.screen = 'session';
  });
  await events.sessionLog((entry) => {
    app.logs = [entry, ...app.logs].slice(0, 200);
  });
  await events.sessionFinished(async (summary) => {
    app.summary = summary;
    app.session = null;
    app.screen = 'recap';
    await refresh().catch(() => {});
  });
  await events.checkProgress((progress) => {
    app.check = progress;
  });
}

export async function boot() {
  await setupListeners();
  const overview = await refresh();
  if (!overview.settings.onboardingDone) {
    const preferred = navigator.language.slice(0, 2) as Lang;
    if (LANGUAGES.some((l) => l.code === preferred) && preferred !== overview.settings.language) {
      await saveSettings({ language: preferred });
    }
    app.screen = 'onboarding';
    return;
  }
  app.screen = 'home';
  if (overview.settings.checkUpdates) {
    api
      .checkUpdate()
      .then((info) => (app.update = info.available ? info : null))
      .catch(() => {});
  }
  if (overview.settings.syncOnStart && !overview.busy && !overview.sessionRunning) {
    const lastCheck = overview.meta.lastCheck ? new Date(overview.meta.lastCheck).getTime() : 0;
    const stale = Date.now() - lastCheck > 6 * 3600 * 1000;
    updateLists(stale ? 'all' : 'unknown');
  }
}
