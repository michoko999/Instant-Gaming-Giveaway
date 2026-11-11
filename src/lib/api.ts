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
  accountConnected: boolean;
  sources: Source[];
}

export interface Counts {
  total: number;
  active: number;
  ended: number;
  unknown: number;
  ignored: number;
  queue: number;
}

export interface Stats {
  totalParticipations: number;
  thisMonth: number;
  doneThisMonth: number;
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
}

export interface Overview {
  settings: Settings;
  counts: Counts;
  stats: Stats;
  meta: Meta;
  version: string;
  dataDir: string;
  portable: boolean;
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
}

export type SessionAction = 'pause' | 'resume' | 'stop' | 'skip' | 'next';

export const api = {
  overview: () => invoke<Overview>('get_overview'),
  saveSettings: (settings: Settings) => invoke<Settings>('save_settings', { settings }),
  addSource: (name: string, url: string) => invoke<Settings>('add_source', { name, url }),
  removeSource: (id: string) => invoke<Settings>('remove_source', { id }),
  sync: () => invoke<SyncReport>('sync_lists'),
  check: (scope: 'all' | 'unknown') => invoke<CheckReport>('check_giveaways', { scope }),
  giveaways: () => invoke<Giveaway[]>('list_giveaways'),
  setIgnored: (code: string, ignored: boolean) => invoke<void>('set_ignored', { code, ignored }),
  importList: () => invoke<ImportReport | null>('import_list'),
  history: (limit?: number) => invoke<HistoryEntry[]>('get_history', { limit }),
  exportHistory: () => invoke<string | null>('export_history'),
  startSession: (mode?: SessionMode, codes?: string[]) => invoke<number>('start_session', { mode, codes }),
  sessionControl: (action: SessionAction) => invoke<void>('session_control', { action }),
  openLogin: () => invoke<void>('open_login'),
  verifyLogin: () => invoke<boolean>('verify_login'),
  logout: () => invoke<void>('logout'),
  openGiveaway: (code: string) => invoke<void>('open_giveaway', { code }),
  showIgWindow: () => invoke<boolean>('show_ig_window'),
  openExternal: (url: string) => invoke<void>('open_external', { url }),
  openDataDir: () => invoke<void>('open_data_dir'),
  checkUpdate: () => invoke<UpdateInfo>('check_update'),
};

export const events = {
  sessionProgress: (cb: (p: Progress) => void): Promise<UnlistenFn> =>
    listen<Progress>('session-progress', (e) => cb(e.payload)),
  sessionLog: (cb: (l: LogEntry) => void): Promise<UnlistenFn> => listen<LogEntry>('session-log', (e) => cb(e.payload)),
  sessionFinished: (cb: (s: SessionSummary) => void): Promise<UnlistenFn> =>
    listen<SessionSummary>('session-finished', (e) => cb(e.payload)),
  checkProgress: (cb: (c: CheckProgress) => void): Promise<UnlistenFn> =>
    listen<CheckProgress>('check-progress', (e) => cb(e.payload)),
};

export const REPO_URL = 'https://github.com/michoko999/Instant-Gaming-Giveaway';
