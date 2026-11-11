<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import Ring from '../lib/Ring.svelte';
  import { api } from '../lib/api';
  import {
    ackWins,
    activeAccount,
    app,
    duration,
    fail,
    minDelay,
    multiCandidates,
    refresh,
    saveSettings,
    since,
    startAllAccounts,
    startSession,
    t,
    updateLists,
    winText,
    SAFE_MIN_DELAY,
  } from '../lib/state.svelte';

  const o = $derived(app.overview);
  const counts = $derived(o?.counts);
  const settings = $derived(o?.settings);
  const connected = $derived(activeAccount()?.connected ?? false);
  const queue = $derived(counts?.queue ?? 0);
  const busy = $derived(app.syncing || app.checking);
  const last = $derived(o?.meta.lastSession ?? null);

  const min = $derived(minDelay(settings));
  let seconds = $state(SAFE_MIN_DELAY);
  $effect(() => {
    seconds = settings?.secondsPerGiveaway ?? SAFE_MIN_DELAY;
  });

  // Estimation d'une session automatique (avec la limite par session).
  const estimate = $derived.by(() => {
    const limit = settings?.sessionLimit ?? 0;
    const count = limit > 0 ? Math.min(queue, limit) : queue;
    return count * (Math.max(seconds, min) + 3);
  });
  const monthDone = $derived(o?.stats.doneThisMonth ?? 0);
  const monthTotal = $derived(monthDone + queue);

  const headline = $derived.by(() => {
    if (queue > 0) return { a: t('home.titleA', { n: queue }), b: t('home.titleB') };
    if ((counts?.active ?? 0) > 0) return { a: t('home.doneA'), b: t('home.doneB') };
    return { a: t('home.emptyA'), b: t('home.emptyB') };
  });

  const lead = $derived.by(() => {
    if (queue === 0) return (counts?.active ?? 0) > 0 ? t('home.leadDone') : t('home.leadEmpty');
    return connected ? t('home.leadAuto') : t('home.connectHint');
  });

  // Gains pas encore vus, et bandeau « nouveau mois » (une fois par mois, si rien n'est encore fait).
  const unseenWins = $derived((o?.meta.wins ?? []).filter((w) => !w.seen));
  const firstWin = $derived(unseenWins[0]);
  const thisMonth = (() => {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
  })();
  const showMonth = $derived(
    queue > 0 && (o?.stats.doneThisMonth ?? 0) === 0 && (o?.stats.totalParticipations ?? 0) > 0 && o?.meta.seenMonth !== thisMonth,
  );
  async function dismissMonth() {
    try {
      await api.dismissMonth();
      await refresh();
    } catch (e) {
      fail(e);
    }
  }

  // Tous les comptes d'un coup : à partir de deux comptes connectés qui ont des concours à faire.
  const multi = $derived(multiCandidates());

  const syncLine = $derived(
    o?.meta.lastSync ? t('home.synced', { when: since(o.meta.lastSync), n: counts?.total ?? 0 }) : t('home.neverSynced'),
  );
</script>

<div class="screen page">
  {#if firstWin}
    <section class="banner win glass" role="status">
      <span class="badge"><Icon name="trophy" size={22} /></span>
      <span class="what">
        <strong>{unseenWins.length > 1 ? t('win.titleMany', { n: unseenWins.length }) : t('win.title')}</strong>
        {#each unseenWins.slice(0, 3) as w (w.account + w.code + (w.productUrl ?? ''))}
          <span class="muted small">{winText(w)}</span>
        {/each}
        <span class="faint small">{t('win.hint')}</span>
      </span>
      {#if firstWin.productUrl}
        <button type="button" class="btn btn-sm" onclick={() => firstWin?.productUrl && api.openExternal(firstWin.productUrl).catch(fail)}>
          <Icon name="external" size={14} />{t('win.see')}
        </button>
      {/if}
      <button type="button" class="btn btn-sm btn-primary" onclick={ackWins}>{t('win.ok')}</button>
    </section>
  {/if}

  {#if showMonth}
    <section class="banner glass" role="status">
      <span class="badge moon"><Icon name="moon" size={20} /></span>
      <span class="what">
        <strong>{t('month.title')}</strong>
        <span class="muted small">{t('month.text', { n: queue })}</span>
      </span>
      <button type="button" class="btn btn-sm btn-quiet icon-btn" aria-label={t('common.close')} onclick={dismissMonth}>
        <Icon name="close" size={14} />
      </button>
    </section>
  {/if}

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
        <span class="faint small">{t('home.estimate', { duration: duration(estimate) })}</span>
        {#if multi.length >= 2}
          <button type="button" class="btn btn-quiet all" onclick={startAllAccounts} disabled={busy}>
            <Icon name="users" size={16} />{t('home.startAll', { n: multi.length })}
          </button>
        {/if}
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
        {min}
        max="15"
        step="1"
        bind:value={seconds}
        onchange={() => saveSettings({ secondsPerGiveaway: Number(seconds) })}
      />
    </section>

    <section class="tile glass">
      <h2 class="label">{t('home.month')}</h2>
      <p class="serif value">{t('home.monthDone', { done: monthDone, total: monthTotal })}</p>
      <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax={monthTotal} aria-valuenow={monthDone} aria-label={t('home.month')}>
        <span style="width:{monthTotal > 0 ? (monthDone / monthTotal) * 100 : 0}%"></span>
      </div>
      <p class="muted small">{t('home.monthHint')}</p>
    </section>
  </div>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px 14px 18px;
    margin-bottom: 18px;
    border-radius: 20px;
    animation: igfadein 0.3s ease-out;
  }
  .banner .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .banner strong {
    font-size: 15px;
    font-weight: 600;
  }
  .badge {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 16%, transparent);
  }
  .badge.moon {
    color: var(--lilac);
    background: color-mix(in srgb, var(--lilac) 16%, transparent);
  }
  .win {
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
    box-shadow: 0 0 40px color-mix(in srgb, var(--accent) 14%, transparent);
  }
  .all {
    align-self: flex-start;
    margin-top: 4px;
    padding-left: 0;
  }
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
  .bar {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 999px;
    background: var(--accent);
    transition: width 0.6s ease;
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
