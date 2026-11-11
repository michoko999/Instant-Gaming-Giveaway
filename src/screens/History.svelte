<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import MonthChart from '../lib/MonthChart.svelte';
  import { api, type HistoryEntry, type Outcome } from '../lib/api';
  import { accountName, activeAccount, app, clockTime, dateTime, fail, shortMessage, t, toast } from '../lib/state.svelte';

  let entries = $state<HistoryEntry[]>([]);
  // Gains repérés (tous comptes), du plus récent au plus ancien.
  const wins = $derived([...(app.overview?.meta.wins ?? [])].reverse());
  const accountOf = (id: string) => (app.overview?.accounts ?? []).find((a) => a.id === id);
  let loaded = $state(false);

  // Historique du compte actif : rechargé quand on change de compte.
  const accountId = $derived(activeAccount()?.id);
  $effect(() => {
    void accountId;
    api
      .history(1000)
      .then((list) => (entries = list))
      .catch(fail)
      .finally(() => (loaded = true));
  });

  const groups = $derived.by(() => {
    const map = new Map<string, HistoryEntry[]>();
    for (const e of entries) {
      const day = new Date(e.at).toLocaleDateString(app.lang, { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' });
      const list = map.get(day) ?? [];
      list.push(e);
      map.set(day, list);
    }
    return [...map.entries()];
  });

  const stats = $derived(app.overview?.stats);

  async function exportCsv() {
    try {
      const path = await api.exportHistory();
      if (path) toast(t('hist.exported', { path }));
    } catch (e) {
      fail(e);
    }
  }

  const chip: Record<Outcome, string> = {
    participated: 'chip-active',
    opened: 'chip-active',
    already: 'chip-unknown',
    captcha: 'chip-coral',
    login: 'chip-coral',
    error: 'chip-coral',
    ended: 'chip-ended',
    skipped: 'chip-ended',
  };
</script>

<div class="screen page">
  <header class="head">
    <h1 class="serif">{t('hist.title')}</h1>
    <button type="button" class="btn btn-sm" onclick={exportCsv} disabled={entries.length === 0}>
      <Icon name="download" size={16} />{t('hist.export')}
    </button>
  </header>

  <dl class="stats">
    <div class="glass"><dd class="serif accent">{stats?.totalParticipations ?? 0}</dd><dt>{t('hist.total')}</dt></div>
    <div class="glass"><dd class="serif">{stats?.thisMonth ?? 0}</dd><dt>{t('hist.month')}</dt></div>
    <div class="glass"><dd class="serif">{app.overview?.counts.queue ?? 0}</dd><dt>{t('hist.remaining')}</dt></div>
  </dl>

  {#if wins.length > 0}
    <section class="wins glass" aria-labelledby="wins-title">
      <h2 id="wins-title" class="label">{t('win.history')}</h2>
      {#each wins as w (w.account + w.code + (w.productUrl ?? '') + w.detectedAt)}
        <div class="win">
          <span class="trophy"><Icon name="trophy" size={18} /></span>
          <span class="what">
            <span class="game">{w.product ?? t('win.someGame')}</span>
            <span class="faint small">
              <span class="mono">{w.code}</span> · {accountName(accountOf(w.account))} · {dateTime(w.detectedAt)}
            </span>
          </span>
          {#if w.productUrl}
            <button type="button" class="btn btn-sm btn-quiet" onclick={() => w.productUrl && api.openExternal(w.productUrl).catch(fail)}>
              <Icon name="external" size={14} />{t('win.see')}
            </button>
          {/if}
        </div>
      {/each}
    </section>
  {/if}

  {#if stats && stats.totalParticipations > 0}
    <MonthChart data={stats.monthly} />
  {/if}

  {#if loaded && entries.length === 0}
    <p class="faint empty">{t('hist.empty')}</p>
  {/if}

  {#each groups as [day, list] (day)}
    <section class="day glass">
      <h2 class="label">{day}</h2>
      {#each list as e (e.at + e.code)}
        <div class="item">
          <span class="mono time">{clockTime(e.at)}</span>
          <span class="mono code">{e.code}</span>
          <span class="chip {chip[e.outcome]}">{t(`outcome.${e.outcome}`)}</span>
          <span class="faint small msg">{shortMessage(e.message)}</span>
          <span class="faint small">{e.mode === 'auto' ? t('hist.auto') : t('hist.manual')}</span>
        </div>
      {/each}
    </section>
  {/each}
</div>

<style>
  .wins {
    padding: 16px 20px;
    border-radius: 24px;
    border-color: color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .wins h2 {
    margin: 0 0 6px;
  }
  .win {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
  }
  .win + .win {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  .trophy {
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 16%, transparent);
  }
  .win .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .game {
    font-weight: 500;
  }
  .page {
    max-width: 1000px;
    margin: 0 auto;
    padding: 8px 40px 40px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  h1 {
    margin: 0;
    font-size: 56px;
    line-height: 1;
  }
  .stats {
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
  }
  .stats div {
    display: flex;
    flex-direction: column-reverse;
    padding: 16px 20px;
  }
  .stats dd {
    margin: 0;
    font-size: 40px;
    line-height: 1.1;
  }
  .stats dt {
    font-size: 13px;
    color: var(--muted);
  }
  .accent {
    color: var(--accent);
  }
  .day {
    padding: 16px 20px 8px;
  }
  .day h2 {
    margin-bottom: 6px;
  }
  .item {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    min-height: 46px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  /* Police à chasse fixe : toutes les heures ont la même largeur, AM/PM compris. */
  .time {
    min-width: 72px;
    white-space: nowrap;
    font-size: 12px;
    color: var(--faint);
  }
  .code {
    width: 200px;
    font-size: 14px;
    letter-spacing: 0.04em;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .msg {
    flex: 1;
    min-width: 80px;
  }
  .small {
    font-size: 12px;
  }
  .empty {
    text-align: center;
    padding: 40px 0;
    margin: 0;
  }
</style>
