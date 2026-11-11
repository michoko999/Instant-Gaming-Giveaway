<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import Ring from '../lib/Ring.svelte';
  import { app, duration, saveSettings, since, startSession, t, updateLists } from '../lib/state.svelte';

  const o = $derived(app.overview);
  const counts = $derived(o?.counts);
  const settings = $derived(o?.settings);
  const connected = $derived(settings?.accountConnected ?? false);
  const queue = $derived(counts?.queue ?? 0);
  const busy = $derived(app.syncing || app.checking);
  const last = $derived(o?.meta.lastSession ?? null);

  let seconds = $state(4);
  $effect(() => {
    seconds = settings?.secondsPerGiveaway ?? 4;
  });

  const headline = $derived.by(() => {
    if (queue > 0) return { a: t('home.titleA', { n: queue }), b: t('home.titleB') };
    if ((counts?.active ?? 0) > 0) return { a: t('home.doneA'), b: t('home.doneB') };
    return { a: t('home.emptyA'), b: t('home.emptyB') };
  });

  const lead = $derived.by(() => {
    if (queue === 0) return (counts?.active ?? 0) > 0 ? t('home.leadDone') : t('home.leadEmpty');
    return connected ? t('home.leadAuto') : t('home.connectHint');
  });

  const syncLine = $derived(
    o?.meta.lastSync ? t('home.synced', { when: since(o.meta.lastSync), n: counts?.total ?? 0 }) : t('home.neverSynced'),
  );
</script>

<div class="screen page">
  <section class="hero">
    <div class="intro">
      <span class="eyebrow">{syncLine}</span>
      <h1 class="serif">{headline.a} <em>{headline.b}</em></h1>
      <p class="muted lead">{lead}</p>

      {#if queue > 0}
        <div class="row">
          {#if connected}
            <button type="button" class="btn btn-primary btn-lg" onclick={() => startSession('auto')} disabled={busy}>
              <Icon name="sparkle" />
              {t('home.startAuto')}
            </button>
          {:else}
            <button type="button" class="btn btn-primary btn-lg" onclick={() => (app.screen = 'settings')}>
              <Icon name="login" />
              {t('home.connect')}
            </button>
          {/if}
          <button type="button" class="btn btn-lg" onclick={() => startSession('manual')} disabled={busy}>
            {t('home.startManual')}
          </button>
        </div>
        <span class="faint small">{t('home.estimate', { duration: duration(queue * (seconds + 3)) })}</span>
      {/if}

      <button type="button" class="btn btn-quiet refresh" onclick={() => updateLists('all')} disabled={busy}>
        <span class:spin={busy} class="spin-wrap"><Icon name="refresh" size={16} /></span>
        {#if app.checking && app.check}
          {t('home.checking', { done: app.check.done, total: app.check.total })}
        {:else if busy}
          {t('home.refreshing')}
        {:else}
          {t('home.refresh')}
        {/if}
      </button>
    </div>

    <div class="orbit">
      <Ring
        size={400}
        label={`${t('legend.active', { n: counts?.active ?? 0 })}, ${t('legend.ended', { n: counts?.ended ?? 0 })}, ${t('legend.unknown', { n: counts?.unknown ?? 0 })}`}
        segments={[
          { value: counts?.active ?? 0, color: 'var(--accent)' },
          { value: counts?.ended ?? 0, color: '#3A3C78' },
          { value: counts?.unknown ?? 0, color: 'var(--lilac)' },
        ]}
      >
        <span class="serif big">{counts?.active ?? 0}</span>
        <span class="eyebrow">{t('ring.active')}</span>
      </Ring>
      <ul class="legend">
        <li><span class="dot" style="background:var(--accent)"></span>{t('legend.active', { n: counts?.active ?? 0 })}</li>
        <li><span class="dot" style="background:var(--ended)"></span>{t('legend.ended', { n: counts?.ended ?? 0 })}</li>
        <li><span class="dot" style="background:var(--lilac)"></span>{t('legend.unknown', { n: counts?.unknown ?? 0 })}</li>
      </ul>
    </div>
  </section>

  <div class="tiles">
    <section class="tile glass">
      <h2 class="label">{t('home.lastSession')}</h2>
      {#if last?.finishedAt}
        <p class="serif value">{since(last.finishedAt)}</p>
        <p class="muted small">{t('home.lastSessionDetail', { n: last.participated + last.opened, e: last.errors })}</p>
      {:else}
        <p class="serif value">{t('home.noSession')}</p>
      {/if}
    </section>

    <section class="tile glass">
      <div class="between">
        <label class="label" for="home-rhythm">{t('home.rhythm')}</label>
        <span class="serif value">{seconds} s <span class="muted small sans">{t('home.perGiveaway')}</span></span>
      </div>
      <input
        id="home-rhythm"
        type="range"
        min="2"
        max="15"
        step="1"
        bind:value={seconds}
        onchange={() => saveSettings({ secondsPerGiveaway: Number(seconds) })}
      />
    </section>

    <section class="tile glass">
      <h2 class="label"><span class="flame"><Icon name="flame" size={14} /></span>{t('home.streak')}</h2>
      <p class="serif value">{t('home.streakDays', { n: o?.stats.streakDays ?? 0 })}</p>
      <p class="muted small">
        {t('home.streakDetail', { total: o?.stats.totalParticipations ?? 0, best: o?.stats.bestStreak ?? 0 })}
      </p>
    </section>
  </div>
</div>

<style>
  .page {
    max-width: 1280px;
    margin: 0 auto;
    padding: 8px 56px 40px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }
  .hero {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 32px;
  }
  .intro {
    flex: 1 1 460px;
    max-width: 620px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 20px;
  }
  h1 {
    margin: 0;
    font-size: clamp(56px, 7vw, 92px);
    line-height: 0.95;
    letter-spacing: -0.02em;
  }
  .lead {
    margin: 0;
    max-width: 480px;
    font-size: 17px;
    text-wrap: pretty;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }
  .refresh {
    padding-left: 0;
  }
  .spin-wrap {
    display: inline-flex;
  }
  .orbit {
    flex: 0 1 400px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }
  .big {
    font-size: 120px;
    line-height: 0.85;
  }
  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 18px;
    font-size: 13px;
    color: var(--muted);
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 16px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 20px 24px;
  }
  .tile p {
    margin: 0;
  }
  .label {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .flame {
    color: var(--accent);
    display: inline-flex;
  }
  .value {
    font-size: 34px;
    line-height: 1.1;
  }
  .between {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .small {
    font-size: 13px;
  }
  .sans {
    font-family: var(--sans);
  }
</style>
