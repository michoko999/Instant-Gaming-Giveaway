<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import Ring from '../lib/Ring.svelte';
  import { api, type Outcome, type SessionAction } from '../lib/api';
  import { app, clockTime, duration, fail, shortMessage, t } from '../lib/state.svelte';

  const s = $derived(app.session);
  const manual = $derived(s?.mode === 'manual');
  const paused = $derived(s?.state === 'paused');
  const needsLogin = $derived(s?.state === 'login');

  async function control(action: SessionAction) {
    try {
      await api.sessionControl(action);
    } catch (e) {
      fail(e);
    }
  }

  function showPage() {
    api.showIgWindow().catch(fail);
  }

  const icon: Record<Outcome, 'check' | 'minus-circle' | 'alert' | 'skip' | 'external'> = {
    participated: 'check',
    already: 'minus-circle',
    opened: 'external',
    captcha: 'alert',
    login: 'alert',
    ended: 'minus-circle',
    skipped: 'skip',
    error: 'alert',
  };
  const tone: Record<Outcome, string> = {
    participated: 'var(--accent)',
    already: 'var(--lilac)',
    opened: 'var(--accent)',
    captcha: 'var(--coral)',
    login: 'var(--coral)',
    ended: 'var(--faint)',
    skipped: 'var(--faint)',
    error: 'var(--coral)',
  };
</script>

{#if s}
  <div class="screen page">
    <header class="top">
      <div class="title">
        <img src="/panda.png" alt="" class="logo" />
        <span class="serif name">{manual ? t('session.titleManual') : t('session.titleAuto')}</span>
        <span class="state" class:warn={needsLogin} class:still={paused}>
          <span class="dot" class:pulse={!paused && !needsLogin}></span>
          {needsLogin ? t('session.login') : paused ? t('session.paused') : t('session.running')}
        </span>
        {#if !manual && app.overview?.settings.safeMode}
          <span class="safe"><Icon name="shield" size={14} />{t('session.safe')}</span>
        {/if}
      </div>
      <div class="row">
        {#if paused || needsLogin}
          <button type="button" class="btn" onclick={() => control('resume')}><Icon name="play" size={14} />{t('session.resume')}</button>
        {:else}
          <button type="button" class="btn" onclick={() => control('pause')}><Icon name="pause" size={14} />{t('session.pause')}</button>
        {/if}
        <button type="button" class="btn btn-quiet" onclick={() => control('stop')}><Icon name="stop" size={14} />{t('session.stop')}</button>
      </div>
    </header>

    {#if needsLogin}
      <div class="banner glass">
        <Icon name="alert" />
        <span>{t('session.loginBanner')}</span>
        <button type="button" class="btn btn-sm" onclick={showPage}>{t('session.showPage')}</button>
        <button type="button" class="btn btn-sm btn-coral" onclick={() => control('resume')}>{t('session.resume')}</button>
      </div>
    {/if}

    <div class="body">
      <section class="progress" aria-label={t('session.current')}>
        <Ring
          size={380}
          marker
          label={`${s.index} / ${s.total}`}
          segments={[
            { value: s.index, color: 'var(--accent)' },
            { value: Math.max(0, s.total - s.index), color: 'transparent' },
          ]}
        >
          <span class="serif big">{s.index}</span>
          <span class="eyebrow">{t('ring.of', { n: s.total })}</span>
        </Ring>

        <div class="current">
          <span class="eyebrow">{manual ? t('session.currentManual') : t('session.current')}</span>
          <span class="mono code">
            {s.current ?? t('session.starting')}
            {#if s.current && !paused}
              <span class="dots" aria-hidden="true"><span class="pulse"></span><span class="pulse d2"></span><span class="pulse d3"></span></span>
            {/if}
          </span>
          {#if manual && s.countdown !== null}
            <span class="muted">{t('session.countdown', { n: s.countdown })}</span>
          {:else}
            <span class="muted">{t('session.remaining', { duration: duration(s.remainingSecs) })}</span>
          {/if}
          <div class="row">
            {#if manual}
              <button type="button" class="btn btn-primary" onclick={() => control('next')}><Icon name="next" size={16} />{t('session.next')}</button>
            {/if}
            <button type="button" class="btn btn-sm btn-quiet" onclick={() => control('skip')}><Icon name="skip" size={14} />{t('session.skip')}</button>
          </div>
        </div>

        <dl class="stats">
          <div class="glass">
            <dd class="serif" style="color:var(--accent)">{s.participated + (manual ? s.opened : 0)}</dd>
            <dt>{t('session.participated')}</dt>
          </div>
          <div class="glass">
            <dd class="serif" style="color:var(--lilac)">{s.already}</dd>
            <dt>{t('session.already')}</dt>
          </div>
          <div class="glass">
            <dd class="serif" style="color:var(--coral)">{s.pending}</dd>
            <dt>{t('session.pending')}</dt>
          </div>
        </dl>
      </section>

      <section class="journal glass" aria-labelledby="journal-title">
        <div class="between">
          <h2 id="journal-title" class="serif">{t('session.journal')}</h2>
          <button type="button" class="btn btn-sm" onclick={showPage}><Icon name="eye" size={16} />{t('session.showPage')}</button>
        </div>
        <ol>
          {#each app.logs as entry (entry.at + entry.code)}
            <li>
              <span class="mono time">{clockTime(entry.at)}</span>
              <span class="ico" style="color:{tone[entry.outcome]}"><Icon name={icon[entry.outcome]} /></span>
              <span class="what">
                <span class="mono">{entry.code}</span>
                <span class="small" style="color:{entry.outcome === 'captcha' || entry.outcome === 'error' ? 'var(--coral)' : 'var(--muted)'}">
                  {t(`outcome.${entry.outcome}`)}{entry.message ? ` · ${shortMessage(entry.message)}` : ''}
                </span>
              </span>
            </li>
          {:else}
            <li class="faint small">{t('session.starting')}</li>
          {/each}
        </ol>
        <p class="faint small note">{t('session.pendingNote')}</p>
      </section>
    </div>
  </div>
{/if}

<style>
  .page {
    max-width: 1320px;
    margin: 0 auto;
    padding: 20px 40px 36px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 100%;
  }
  .top {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .logo {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #fff;
    object-fit: cover;
  }
  .name {
    font-size: 26px;
    line-height: 1;
  }
  .state {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .state .dot {
    width: 7px;
    height: 7px;
    background: currentColor;
  }
  .state.warn {
    background: color-mix(in srgb, var(--coral) 16%, transparent);
    color: var(--coral);
  }
  .state.still {
    background: rgba(255, 255, 255, 0.08);
    color: var(--muted);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .safe {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
  }
  .banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: 12px 14px 12px 20px;
    border-color: color-mix(in srgb, var(--coral) 35%, transparent);
    background: color-mix(in srgb, var(--coral) 8%, transparent);
    color: var(--coral);
  }
  .banner span {
    flex: 1;
    min-width: 240px;
    color: var(--ink);
  }
  .body {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 36px;
  }
  .progress {
    flex: 1 1 420px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
  }
  .big {
    font-size: 116px;
    line-height: 0.85;
  }
  .current {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    text-align: center;
  }
  .code {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    font-size: 28px;
    font-weight: 500;
    letter-spacing: 0.08em;
  }
  .dots {
    display: inline-flex;
    gap: 4px;
  }
  .dots span {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
  }
  .dots .d2 {
    animation-delay: 0.2s;
  }
  .dots .d3 {
    animation-delay: 0.4s;
  }
  .stats {
    margin: 0;
    width: 100%;
    max-width: 440px;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
  }
  .stats div {
    display: flex;
    flex-direction: column-reverse;
    align-items: center;
    padding: 12px 8px;
    border-radius: 18px;
  }
  .stats dd {
    margin: 0;
    font-size: 34px;
    line-height: 1.1;
  }
  .stats dt {
    font-size: 12px;
    color: var(--muted);
  }
  .journal {
    flex: 1 1 440px;
    align-self: stretch;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 22px 24px;
    border-radius: 28px;
    min-height: 420px;
    max-height: 640px;
  }
  .between {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 30px;
    line-height: 1;
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1;
    overflow-y: auto;
  }
  li {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 56px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  }
  .time {
    width: 64px;
    font-size: 12px;
    color: var(--faint);
  }
  .ico {
    width: 22px;
    display: flex;
    justify-content: center;
  }
  .what {
    flex: 1;
    display: flex;
    flex-direction: column;
    line-height: 1.35;
    min-width: 0;
  }
  .what .mono {
    font-size: 14px;
    letter-spacing: 0.04em;
  }
  .small {
    font-size: 13px;
  }
  .note {
    margin: 0;
  }
</style>
