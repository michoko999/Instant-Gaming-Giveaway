// Accès aux commandes Rust et aux événements, avec les types partagés.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Lang = 'fr' | 'en' | 'es' | 'de' | 'pt' | 'it' | 'pl';
export type SessionMode = 'auto' | 'manual';
export type Status = 'unknown' | 'active' | 'ended';
export type Outcome = 'participated' | 'already' | 'opened' | 'captcha' | 'login' | 'ended' | 'skipped' | 'error';

export interface Source {
  id: string;
  name: string;
  url: string;
  kind: 'aliveJson' | 'csv' | 'page';
  enabled: boolean;
  builtin: boolean;
}

/** unknown : jamais vérifiés · all : actifs, inconnus et terminés pas revus depuis 7 jours · full : tous. */
export type CheckScope = 'unknown' | 'all' | 'full';

export interface Settings {
  language: Lang;
  mode: SessionMode;
  secondsPerGiveaway: number;
  skipDoneThisMonth: boolean;
  safeMode: boolean;
  sessionLimit: number;
  showIgWindow: boolean;
  accent: string;
  skyAnimation: boolean;
  syncOnStart: boolean;
  checkUpdates: boolean;
  onboardingDone: boolean;
  sources: Source[];
}

export interface Counts {
  total: number;
  active: number;
  ended: number;
  unknown: number;
  ignored: number;
  queue: number;
  /** Concours actifs ajoutés depuis la dernière visite de l'écran Concours. */
  fresh: number;
}

export interface MonthCount {
  /** « AAAA-MM » */
  month: string;
  count: number;
}

export interface Stats {
  totalParticipations: number;
  thisMonth: number;
  doneThisMonth: number;
  /** 12 derniers mois, du plus ancien au mois en cours. */
  monthly: MonthCount[];
}

export interface SessionSummary {
  startedAt: string | null;
  finishedAt: string | null;
  mode: SessionMode;
  total: number;
  participated: number;
  already: number;
  opened: number;
  ended: number;
  errors: number;
  skipped: number;
  pending: string[];
  stopped: boolean;
  stopReason: 'warning' | 'captcha' | 'errors' | null;
  limited: number;
}

export interface Meta {
  lastSync: string | null;
  lastCheck: string | null;
  lastSession: SessionSummary | null;
  wins: Win[];
  /** Mois (« AAAA-MM ») dont le bandeau « nouveau mois » a été fermé. */
  seenMonth: string | null;
  /** Dernière visite de l'écran Concours. */
  giveawaysSeenAt: string | null;
}

/** Un compte de l'application figure parmi les gagnants d'un concours. */
export interface Win {
  account: string;
  code: string;
  product: string | null;
  productUrl: string | null;
  detectedAt: string;
  seen: boolean;
}

/** Compte Instant Gaming (chacun a sa connexion, son suivi du mois et son historique). */
export interface Account {
  id: string;
  /** Pseudo lu sur Instant Gaming. */
  name: string | null;
  /** Photo de profil en data: URL. */
  avatar: string | null;
  profileUrl: string | null;
  connected: boolean;
  active: boolean;
  doneThisMonth: number;
  /** Concours qu'une session de ce compte traiterait maintenant. */
  queue: number;
}

export interface Overview {
  accounts: Account[];
  settings: Settings;
  counts: Counts;
  stats: Stats;
  meta: Meta;
  version: string;
  dataDir: string;
  portable: boolean;
  /** Fichier seul, installateur Windows ou Scoop. */
  setup: 'portable' | 'installed' | 'scoop';
  sessionRunning: boolean;
  busy: boolean;
}

export interface Giveaway {
  code: string;
  status: Status;
  checkedAt: string | null;
  lastParticipation: string | null;
  lastOutcome: Outcome | null;
  sources: string[];
  addedAt: string;
  ignored: boolean;
}

export interface HistoryEntry {
  code: string;
  at: string;
  outcome: Outcome;
  mode: SessionMode;
  message?: string;
}

export interface Progress {
  state: 'running' | 'paused' | 'login' | 'finished';
  mode: SessionMode;
  index: number;
  total: number;
  current: string | null;
  participated: number;
  already: number;
  opened: number;
  ended: number;
  errors: number;
  skipped: number;
  pending: number;
  remainingSecs: number;
  countdown: number | null;
}

export interface LogEntry {
  at: string;
  code: string;
  outcome: Outcome;
  message: string | null;
}

export interface CheckProgress {
  done: number;
  total: number;
  active: number;
  ended: number;
  unknown: number;
  code: string;
  status: Status;
}

export interface SourceResult {
  id: string;
  name: string;
  ok: boolean;
  count: number;
  error: string | null;
}

export interface SyncReport {
  added: number;
  markedEnded: number;
  total: number;
  sources: SourceResult[];
}

export interface CheckReport {
  total: number;
  active: number;
  ended: number;
  unknown: number;
}

export interface ImportReport {
  found: number;
  added: number;
}

export interface UpdateInfo {
  current: string;
  latest: string | null;
  available: boolean;
  url: string | null;
  /** La mise à jour peut être installée depuis l'application. */
  canInstall: boolean;
  /** Gestionnaire qui fait les mises à jour à la place de l'application. */
  manager: string | null;
}

export interface BackupInfo {
  id: string;
  createdAt: string;
  /** Version qui utilisait ces données (inconnue avant la 2.1.0). */
  fromVersion: string | null;
  toVersion: string;
  reason: 'update' | 'restore';
}

export interface UpdateProgress {
  received: number;
  total: number;
}

export type SessionAction = 'pause' | 'resume' | 'stop' | 'skip' | 'next';

export const api = {
  overview: () => invoke<Overview>('get_overview'),
  saveSettings: (settings: Settings) => invoke<Settings>('save_settings', { settings }),
  addSource: (name: string, url: string) => invoke<Settings>('add_source', { name, url }),
  removeSource: (id: string) => invoke<Settings>('remove_source', { id }),
  sync: () => invoke<SyncReport>('sync_lists'),
  check: (scope: CheckScope) => invoke<CheckReport>('check_giveaways', { scope }),
  giveaways: () => invoke<Giveaway[]>('list_giveaways'),
  setIgnored: (code: string, ignored: boolean) => invoke<void>('set_ignored', { code, ignored }),
  importList: () => invoke<ImportReport | null>('import_list'),
  addLinks: (text: string) => invoke<ImportReport>('add_links', { text }),
  history: (limit?: number) => invoke<HistoryEntry[]>('get_history', { limit }),
  exportHistory: () => invoke<string | null>('export_history'),
  startSession: (mode?: SessionMode, codes?: string[]) => invoke<number>('start_session', { mode, codes }),
  sessionControl: (action: SessionAction) => invoke<void>('session_control', { action }),
  openLogin: () => invoke<void>('open_login'),
  verifyLogin: (silent = false) => invoke<boolean>('verify_login', { silent }),
  logout: () => invoke<void>('logout'),
  addAccount: () => invoke<string>('add_account'),
  switchAccount: (id: string) => invoke<void>('switch_account', { id }),
  removeAccount: (id: string) => invoke<void>('remove_account', { id }),
  ackWins: () => invoke<void>('ack_wins'),
  dismissMonth: () => invoke<void>('dismiss_month'),
  markGiveawaysSeen: () => invoke<void>('mark_giveaways_seen'),
  notify: (title: string, body: string) => invoke<void>('notify', { title, body }),
  openGiveaway: (code: string) => invoke<void>('open_giveaway', { code }),
  showIgWindow: () => invoke<boolean>('show_ig_window'),
  openExternal: (url: string) => invoke<void>('open_external', { url }),
  openDataDir: () => invoke<void>('open_data_dir'),
  listBackups: () => invoke<BackupInfo[]>('list_backups'),
  restoreBackup: (id: string) => invoke<void>('restore_backup', { id }),
  problemReport: () => invoke<string>('problem_report'),
  checkUpdate: () => invoke<UpdateInfo>('check_update'),
  installUpdate: () => invoke<void>('install_update'),
};

export const events = {
  sessionProgress: (cb: (p: Progress) => void): Promise<UnlistenFn> =>
    listen<Progress>('session-progress', (e) => cb(e.payload)),
  sessionLog: (cb: (l: LogEntry) => void): Promise<UnlistenFn> => listen<LogEntry>('session-log', (e) => cb(e.payload)),
  sessionFinished: (cb: (s: SessionSummary) => void): Promise<UnlistenFn> =>
    listen<SessionSummary>('session-finished', (e) => cb(e.payload)),
  checkProgress: (cb: (c: CheckProgress) => void): Promise<UnlistenFn> =>
    listen<CheckProgress>('check-progress', (e) => cb(e.payload)),
  updateProgress: (cb: (p: UpdateProgress) => void): Promise<UnlistenFn> =>
    listen<UpdateProgress>('update-progress', (e) => cb(e.payload)),
  winsFound: (cb: (wins: Win[]) => void): Promise<UnlistenFn> => listen<Win[]>('wins-found', (e) => cb(e.payload)),
};

export const REPO_URL = 'https://github.com/michoko999/Instant-Gaming-Giveaway';
