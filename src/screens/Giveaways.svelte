<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '../lib/Icon.svelte';
  import { api, type Giveaway } from '../lib/api';
  import { app, fail, since, t, toast, updateLists } from '../lib/state.svelte';

  type Filter = 'all' | 'active' | 'ended' | 'unknown' | 'ignored';
  let items = $state<Giveaway[]>([]);
  let filter = $state<Filter>('active');
  let query = $state('');

  async function load() {
    try {
      items = await api.giveaways();
    } catch (e) {
      fail(e);
    }
  }

  onMount(load);

  // Recharge la liste après une mise à jour.
  let wasBusy = false;
  $effect(() => {
    const busy = app.syncing || app.checking;
    if (wasBusy && !busy) load();
    wasBusy = busy;
  });

  const busy = $derived(app.syncing || app.checking);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    return items.filter((g) => {
      if (q && !g.code.toLowerCase().includes(q)) return false;
      switch (filter) {
        case 'ignored':
          return g.ignored;
        case 'all':
          return true;
        default:
          return !g.ignored && g.status === filter;
      }
    });
  });

  const count = (f: Filter) =>
    f === 'all' ? items.length : f === 'ignored' ? items.filter((g) => g.ignored).length : items.filter((g) => !g.ignored && g.status === f).length;

  const filters: { id: Filter; key: 'gw.filterAll' | 'gw.filterActive' | 'gw.filterEnded' | 'gw.filterUnknown' | 'gw.filterIgnored' }[] = [
    { id: 'active', key: 'gw.filterActive' },
    { id: 'unknown', key: 'gw.filterUnknown' },
    { id: 'ended', key: 'gw.filterEnded' },
    { id: 'ignored', key: 'gw.filterIgnored' },
    { id: 'all', key: 'gw.filterAll' },
  ];

  async function toggleIgnore(g: Giveaway) {
    try {
      await api.setIgnored(g.code, !g.ignored);
      g.ignored = !g.ignored;
    } catch (e) {
      fail(e);
    }
  }

  async function importFile() {
    try {
      const report = await api.importList();
      if (!report) return;
      toast(t('gw.imported', { added: report.added, found: report.found }));
      await updateLists('unknown', false);
    } catch (e) {
      fail(e);
    }
  }

  const chip = { active: 'chip-active', ended: 'chip-ended', unknown: 'chip-unknown' } as const;
  const statusKey = { active: 'status.active', ended: 'status.ended', unknown: 'status.unknown' } as const;
</script>

<div class="screen page">
  <header class="head">
    <h1 class="serif">{t('gw.title')}</h1>
    <div class="row">
      <button type="button" class="btn btn-sm" onclick={() => updateLists('unknown')} disabled={busy}>
        <span class:spin={app.syncing} class="flex"><Icon name="download" size={16} /></span>{t('gw.sync')}
      </button>
      <button type="button" class="btn btn-sm" onclick={() => updateLists('all', false)} disabled={busy}>
        <span class:spin={app.checking} class="flex"><Icon name="refresh" size={16} /></span>
        {#if app.checking && app.check}{t('home.checking', { done: app.check.done, total: app.check.total })}{:else}{t('gw.check')}{/if}
      </button>
      <button type="button" class="btn btn-sm" onclick={importFile} disabled={busy}><Icon name="upload" size={16} />{t('gw.import')}</button>
    </div>
  </header>

  <div class="tools">
    <label class="search">
      <span class="sr">{t('gw.search')}</span>
      <Icon name="search" size={16} />
      <input class="field" type="search" placeholder={t('gw.search')} bind:value={query} />
    </label>
    <div class="seg">
      {#each filters as f (f.id)}
        <button type="button" aria-pressed={filter === f.id} onclick={() => (filter = f.id)}>{t(f.key)} · {count(f.id)}</button>
      {/each}
    </div>
  </div>

  <section class="list glass">
    {#each filtered as g (g.code)}
      <div class="item" class:dim={g.ignored}>
        <span class="mono code">{g.code}</span>
        <span class="chip {chip[g.status]}">{t(statusKey[g.status])}</span>
        <span class="muted small when">
          {g.lastParticipation ? t('gw.lastParticipation', { when: since(g.lastParticipation) }) : t('gw.neverParticipated')}
        </span>
        <div class="actions">
          <button type="button" class="btn btn-sm" onclick={() => api.openGiveaway(g.code).catch(fail)}>
            <Icon name="external" size={14} />{t('gw.open')}
          </button>
          <button type="button" class="btn btn-sm btn-quiet" onclick={() => toggleIgnore(g)}>
            {g.ignored ? t('gw.unignore') : t('gw.ignore')}
          </button>
        </div>
      </div>
    {:else}
      <p class="faint empty">{t('gw.empty')}</p>
    {/each}
  </section>
  <p class="faint small">{t('gw.count', { n: filtered.length })}</p>
</div>

<style>
  .page {
    max-width: 1100px;
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
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .flex {
    display: inline-flex;
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1 1 260px;
    max-width: 360px;
    color: var(--faint);
  }
  .search :global(svg) {
    position: absolute;
    left: 14px;
  }
  .search input {
    width: 100%;
    padding-left: 40px;
    border-radius: 999px;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }
  .list {
    padding: 6px 20px;
    border-radius: 24px;
  }
  .item {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    min-height: 58px;
    padding: 6px 0;
  }
  .item + .item {
    border-top: 1px solid rgba(255, 255, 255, 0.07);
  }
  .item.dim {
    opacity: 0.55;
  }
  .code {
    width: 200px;
    font-size: 14px;
    letter-spacing: 0.04em;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .when {
    flex: 1;
    min-width: 140px;
  }
  .actions {
    display: flex;
    gap: 4px;
  }
  .small {
    font-size: 13px;
  }
  .empty {
    padding: 28px 0;
    text-align: center;
    margin: 0;
  }
</style>
