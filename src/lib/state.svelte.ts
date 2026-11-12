// État global de l'interface (runes Svelte 5), traductions et actions partagées.
import {
  api,
  events,
  REPO_URL,
  type Account,
  type CheckProgress,
  type CheckScope,
  type Lang,
  type LogEntry,
  type Overview,
  type Progress,
  type SessionMode,
  type SessionSummary,
  type Settings,
  type UpdateInfo,
  type Win,
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
  /** La fenêtre de connexion est ouverte : on attend « J'ai terminé ». */
  loginPending: false,
  verifying: false,
  /** Session enchaînée sur tous les comptes connectés. */
  multi: null as MultiRun | null,
  /** Résultats par compte de la dernière série « tous les comptes » (pour le récapitulatif). */
  multiResults: [] as { id: string; summary: SessionSummary }[],
});

export interface MultiRun {
  /** Comptes à traiter, dans l'ordre. */
  ids: string[];
  /** Compte en cours (indice dans `ids`). */
  index: number;
  /** Compte actif avant le lancement, remis à la fin. */
  original: string;
  results: { id: string; summary: SessionSummary }[];
  /** Secondes avant le compte suivant (pause entre deux comptes), sinon null. */
  waiting: number | null;
}

/** Pause entre deux comptes, pour ne pas enchaîner les connexions trop vite. */
const BETWEEN_ACCOUNTS = 15;

/** Traduit une clé, avec remplacement des {variables}. */
export function t(key: DictKey, vars: Record<string, string | number> = {}): string {
  const dict = DICTS[app.lang] ?? fr;
  let text = dict[key] ?? fr[key] ?? key;
  // « singulier|pluriel » : la forme est choisie d'après {n}.
  if (text.includes('|') && typeof vars.n === 'number') {
    const forms = text.split('|');
    text = forms[new Intl.PluralRules(app.lang).select(vars.n) === 'one' ? 0 : forms.length - 1];
  }
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

/** Mode prudent : temps minimum entre deux concours (même valeur que côté Rust). */
export const SAFE_MIN_DELAY = 8;

/** Plus petit temps par concours autorisé avec ces réglages. */
export function minDelay(settings: Settings | undefined | null): number {
  return settings?.safeMode ?? true ? SAFE_MIN_DELAY : 2;
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
export async function updateLists(scope: CheckScope = 'unknown', withSync = true) {
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

export async function startSession(mode: SessionMode, codes?: string[]): Promise<boolean> {
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
    return true;
  } catch (e) {
    fail(e);
    return false;
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
    app.session = null;
    await refresh().catch(() => {});
    if (app.multi) {
      await multiSessionFinished(summary);
      return;
    }
    app.summary = summary;
    app.multiResults = [];
    app.screen = 'recap';
    notifySessionEnd(summary);
  });
  await events.winsFound(async (wins) => {
    await refresh().catch(() => {});
    const first = wins[0];
    if (!first) return;
    const text = winText(first);
    toast(text);
    notifyInBackground(t('win.notifyTitle'), text);
  });
  await events.checkProgress((progress) => {
    app.check = progress;
  });
}

/** Compte actif (celui des sessions, du suivi du mois et de l'historique affichés). */
export function activeAccount(): Account | undefined {
  const list = app.overview?.accounts ?? [];
  return list.find((a) => a.active) ?? list[0];
}

/** Pseudo du compte, ou « Compte 2 » tant qu'il n'est pas connu. */
export function accountName(account: Account | undefined): string {
  if (!account) return '';
  if (account.name) return account.name;
  const index = (app.overview?.accounts ?? []).findIndex((a) => a.id === account.id);
  return t('account.unnamed', { n: index + 1 });
}

/** Ouvre la fenêtre de connexion du compte actif. */
export async function openLogin() {
  try {
    await api.openLogin();
    app.loginPending = true;
    toast(t('onb.waiting'));
  } catch (e) {
    fail(e);
  }
}

/** Vérifie la connexion du compte actif (et lit son pseudo et sa photo). */
export async function verifyLogin(): Promise<boolean> {
  if (app.verifying) return false;
  app.verifying = true;
  try {
    const ok = await api.verifyLogin();
    await refresh();
    const account = activeAccount();
    if (ok) {
      app.loginPending = false;
      toast(account?.name ? t('account.connectedAs', { name: account.name }) : t('account.connected'));
    } else {
      toast(t('onb.notConnected'), 'error');
    }
    return ok;
  } catch (e) {
    fail(e);
    return false;
  } finally {
    app.verifying = false;
  }
}

/** Relit en arrière-plan le pseudo et la photo d'un compte connecté qui n'en a pas encore. */
function refreshIdentity() {
  const account = activeAccount();
  if (!account?.connected || account.name || app.overview?.sessionRunning) return;
  api
    .verifyLogin(true)
    .then(() => refresh())
    .catch(() => {});
}

/** Ouvre les paramètres sur la section des comptes. */
export function showAccounts() {
  app.screen = 'settings';
  setTimeout(() => document.getElementById('accounts')?.scrollIntoView({ block: 'center', behavior: 'smooth' }), 60);
}

export async function addAccount() {
  try {
    await api.addAccount();
    app.loginPending = false;
    await refresh();
    showAccounts();
    await openLogin();
  } catch (e) {
    fail(e);
  }
}

export async function switchAccount(id: string) {
  if (activeAccount()?.id === id) return;
  try {
    await api.switchAccount(id);
    app.loginPending = false;
    await refresh();
    toast(t('account.switched', { name: accountName(activeAccount()) }));
    refreshIdentity();
  } catch (e) {
    fail(e);
  }
}

export async function removeAccount(id: string) {
  try {
    await api.removeAccount(id);
    app.loginPending = false;
    await refresh();
  } catch (e) {
    fail(e);
  }
}

export async function logoutAccount() {
  try {
    await api.logout();
    app.loginPending = false;
    await refresh();
  } catch (e) {
    fail(e);
  }
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
  refreshIdentity();
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

// ---------------------------------------------------------------------------
// Notifications du système : seulement quand l'application est en arrière-plan.

function notifyInBackground(title: string, body: string) {
  if (document.hasFocus()) return;
  api.notify(title, body).catch(() => {});
}

function notifySessionEnd(summary: SessionSummary) {
  const pending = summary.pending.length;
  notifyInBackground(
    t('notify.sessionTitle'),
    pending > 0
      ? t('notify.sessionBodyPending', { n: summary.participated, pending })
      : t('notify.sessionBody', { n: summary.participated }),
  );
}

// ---------------------------------------------------------------------------
// Gains

/** « Tu as gagné le concours BOBLENNON avec Michoko… » (le jeu, c'est le gagnant qui le choisit). */
export function winText(win: Win): string {
  const account = (app.overview?.accounts ?? []).find((a) => a.id === win.account);
  return t(win.product ? 'win.textGame' : 'win.text', {
    game: win.product ?? '',
    code: win.code,
    name: accountName(account),
  });
}

export async function ackWins() {
  try {
    await api.ackWins();
    await refresh();
  } catch (e) {
    fail(e);
  }
}

// ---------------------------------------------------------------------------
// Tous les comptes d'un coup : une session automatique par compte connecté, à la suite.

/** Comptes qu'une session « tous les comptes » traiterait. */
export function multiCandidates(): Account[] {
  return (app.overview?.accounts ?? []).filter((a) => a.connected && a.queue > 0);
}

export async function startAllAccounts() {
  const list = multiCandidates();
  if (list.length === 0) {
    toast(t('error.no-giveaways'), 'error');
    return;
  }
  app.multi = {
    ids: list.map((a) => a.id),
    index: 0,
    original: activeAccount()?.id ?? list[0].id,
    results: [],
    waiting: null,
  };
  await runMultiAccount();
}

async function runMultiAccount() {
  const run = app.multi;
  if (!run) return;
  const id = run.ids[run.index];
  try {
    if (activeAccount()?.id !== id) {
      await api.switchAccount(id);
      await refresh();
    }
  } catch (e) {
    fail(e);
    await endMulti();
    return;
  }
  if (!(await startSession('auto'))) await endMulti();
}

let multiTimer: ReturnType<typeof setInterval> | undefined;

async function multiSessionFinished(summary: SessionSummary) {
  const run = app.multi;
  if (!run) return;
  run.results.push({ id: run.ids[run.index], summary });
  // Arrêt demandé, ou arrêt de sécurité du mode prudent : on ne passe pas aux autres comptes.
  const stop = summary.stopped || !!summary.stopReason;
  if (stop || run.index + 1 >= run.ids.length) {
    await endMulti();
    return;
  }
  app.screen = 'session';
  run.waiting = BETWEEN_ACCOUNTS;
  clearInterval(multiTimer);
  multiTimer = setInterval(() => {
    if (!app.multi || app.multi.waiting === null) return clearInterval(multiTimer);
    app.multi.waiting -= 1;
    if (app.multi.waiting <= 0) nextAccountNow();
  }, 1000);
}

/** Passe tout de suite au compte suivant (pendant la pause). */
export function nextAccountNow() {
  const run = app.multi;
  if (!run || run.waiting === null) return;
  clearInterval(multiTimer);
  run.waiting = null;
  run.index += 1;
  runMultiAccount();
}

/** Termine la série : récapitulatif commun et retour au compte de départ. */
export async function endMulti() {
  clearInterval(multiTimer);
  const run = app.multi;
  if (!run) return;
  app.multi = null;
  if (run.results.length > 0) {
    const total = run.results.reduce(
      (acc, { summary: s }) => ({
        ...acc,
        total: acc.total + s.total,
        participated: acc.participated + s.participated,
        already: acc.already + s.already,
        opened: acc.opened + s.opened,
        ended: acc.ended + s.ended,
        errors: acc.errors + s.errors,
        skipped: acc.skipped + s.skipped,
        stopped: acc.stopped || s.stopped,
        stopReason: acc.stopReason ?? s.stopReason,
        finishedAt: s.finishedAt,
      }),
      { ...run.results[0].summary, total: 0, participated: 0, already: 0, opened: 0, ended: 0, errors: 0, skipped: 0, pending: [] as string[], limited: 0 },
    );
    total.startedAt = run.results[0].summary.startedAt;
    total.pending = run.results.flatMap((r) => r.summary.pending);
    app.summary = total;
    app.multiResults = run.results;
    notifySessionEnd(total);
  }
  if (activeAccount()?.id !== run.original) {
    await api.switchAccount(run.original).catch(() => {});
    await refresh().catch(() => {});
  }
  app.screen = run.results.length > 0 ? 'recap' : 'home';
}

// ---------------------------------------------------------------------------
// Signaler un problème

/** Ouvre un ticket GitHub prérempli : l'utilisateur décrit le problème, l'application ajoute
 * la version et les dernières erreurs (rien de personnel). Il relit tout avant d'envoyer. */
export async function reportProblem() {
  try {
    const tech = await api.problemReport();
    const body = [t('report.what'), '', '', t('report.expected'), '', '', t('report.tech'), '', tech, ''].join('\n');
    const url =
      `${REPO_URL}/issues/new?labels=bug` +
      `&title=${encodeURIComponent(t('report.issueTitle'))}` +
      `&body=${encodeURIComponent(body)}`;
    await api.openExternal(url);
  } catch (e) {
    fail(e);
  }
}
