<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import { api } from '../lib/api';
  import { app, duration, fail, refresh, t } from '../lib/state.svelte';

  const s = $derived(app.summary ?? app.overview?.meta.lastSession ?? null);
  const seconds = $derived(
    s?.startedAt && s?.finishedAt ? (new Date(s.finishedAt).getTime() - new Date(s.startedAt).getTime()) / 1000 : 0,
  );
  const manualOnly = $derived(s ? s.mode === 'manual' && s.participated === 0 : false);

  function open(code: string) {
    api.openGiveaway(code).catch(fail);
  }

  async function goHome() {
    await refresh().catch(() => {});
    app.screen = 'home';
  }
</script>

{#if s}
  <div class="screen page">
    <div class="orb">
      <svg viewBox="0 0 300 300" aria-hidden="true">
        <circle cx="150" cy="150" r="140" fill="none" stroke="rgba(255,255,255,0.10)" stroke-width="1" stroke-dasharray="1 7" stroke-linecap="round" />
        <circle cx="150" cy="150" r="112" fill="none" stroke="var(--accent)" stroke-opacity="0.16" stroke-width="30" />
        <circle cx="150" cy="150" r="112" fill="none" stroke="var(--accent)" stroke-width="10" />
        <path transform="translate(236 18)" fill="var(--accent)" d="M14 0C14.7 7 18.6 11.2 26 12C18.6 12.8 14.7 17 14 24C13.3 17 9.4 12.8 2 12C9.4 11.2 13.3 7 14 0Z" />
        <path transform="translate(22 60)" fill="#D9D0FF" d="M9 0C9.5 4.5 12 7.2 16.8 7.7C12 8.2 9.5 10.9 9 15.4C8.5 10.9 6 8.2 1.2 7.7C6 7.2 8.5 4.5 9 0Z" />
        <path transform="translate(250 220)" fill="#FFFFFF" fill-opacity="0.8" d="M7 0C7.4 3.5 9.3 5.6 13 6C9.3 6.4 7.4 8.5 7 12C6.6 8.5 4.7 6.4 1 6C4.7 5.6 6.6 3.5 7 0Z" />
      </svg>
      <div class="panda"><img src="/panda.png" alt="" /></div>
    </div>

    <div class="head">
      <h1 class="serif">
        {s.stopped ? t('recap.stoppedA') : t('recap.titleA')}
        {#if manualOnly}
          <em>{t('recap.openedB', { n: s.opened })}</em> {t('recap.openedC')}
        {:else}
          <em>{t('recap.titleB', { n: s.participated })}</em> {t('recap.titleC')}
        {/if}
      </h1>
      <p class="muted">
        {t('recap.detail', {
          total: s.participated + s.already + s.opened + s.ended + s.errors + s.skipped + s.pending.length,
          duration: duration(seconds),
          already: s.already,
          pending: s.pending.length,
        })}
      </p>
    </div>

    {#each s.pending as code (code)}
      <section class="pending glass">
        <span class="coral"><Icon name="alert" size={20} /></span>
        <span class="what">
          <span class="mono">{code}</span>
          <span class="muted small">{t('recap.pendingHint')}</span>
        </span>
        <button type="button" class="btn btn-coral" onclick={() => open(code)}>{t('recap.openPage')}</button>
      </section>
    {/each}

    <div class="row">
      <button type="button" class="btn btn-primary btn-lg" onclick={goHome}>{t('recap.home')}</button>
      <button type="button" class="btn btn-lg" onclick={() => (app.screen = 'history')}>{t('recap.history')}</button>
    </div>
  </div>
{/if}

<style>
  .page {
    max-width: 760px;
    margin: 0 auto;
    min-height: 100%;
    padding: 32px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 22px;
    text-align: center;
  }
  .orb {
    position: relative;
    width: 280px;
    height: 280px;
  }
  .orb svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .panda {
    position: absolute;
    inset: 66px;
    border-radius: 50%;
    padding: 6px;
    background: rgba(255, 255, 255, 0.06);
    box-shadow: 0 0 80px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .panda img {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: #fff;
    object-fit: cover;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: center;
  }
  h1 {
    margin: 0;
    font-size: clamp(44px, 5.5vw, 66px);
    line-height: 1;
    letter-spacing: -0.015em;
    text-wrap: balance;
  }
  .head p {
    margin: 0;
    font-size: 16px;
  }
  .pending {
    width: 100%;
    max-width: 560px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    padding: 12px 12px 12px 20px;
    border-color: color-mix(in srgb, var(--coral) 30%, transparent);
    background: color-mix(in srgb, var(--coral) 7%, transparent);
    text-align: left;
  }
  .coral {
    color: var(--coral);
    display: flex;
  }
  .what {
    flex: 1;
    min-width: 180px;
    display: flex;
    flex-direction: column;
    line-height: 1.35;
  }
  .small {
    font-size: 13px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 12px;
  }
</style>
