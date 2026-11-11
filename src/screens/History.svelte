<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import MonthChart from '../lib/MonthChart.svelte';
  import { api, type HistoryEntry, type Outcome } from '../lib/api';
  import { app, clockTime, fail, shortMessage, t, toast } from '../lib/state.svelte';

  let entries = $state<HistoryEntry[]>([]);
  let loaded = $state(false);

  onMount(async () => {
    try {
      entries = await api.history(1000);
    } catch (e) {
      fail(e);
    } finally {
      loaded = true;
    }
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
  .time {
    width: 72px;
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
