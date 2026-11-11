<script lang="ts">
  import Icon from '../lib/Icon.svelte';
  import Ring from '../lib/Ring.svelte';
  import { api, type SessionMode } from '../lib/api';
  import { app, duration, fail, saveSettings, t, updateLists } from '../lib/state.svelte';

  type Step = 'connect' | 'waiting' | 'check' | 'ready';
  let step = $state<Step>('connect');
  let verifying = $state(false);
  let notConnected = $state(false);
  let mode = $state<SessionMode>('auto');
  let seconds = $state(app.overview?.settings.secondsPerGiveaway ?? 4);

  const stepIndex = $derived(step === 'connect' || step === 'waiting' ? 0 : step === 'check' ? 1 : 2);
  const connected = $derived(app.overview?.settings.accountConnected ?? false);
  const counts = $derived(app.overview?.counts);

  async function login() {
    try {
      await api.openLogin();
      step = 'waiting';
      notConnected = false;
    } catch (e) {
      fail(e);
    }
  }

  async function verify() {
    verifying = true;
    notConnected = false;
    try {
      const ok = await api.verifyLogin();
      if (ok) {
        await goCheck();
      } else {
        notConnected = true;
      }
    } catch (e) {
      fail(e);
    } finally {
      verifying = false;
    }
  }

  async function skipAccount() {
    mode = 'manual';
    await saveSettings({ mode: 'manual' });
    await goCheck();
  }

  async function goCheck() {
    step = 'check';
    await updateLists('all');
    mode = app.overview?.settings.accountConnected ? 'auto' : 'manual';
    step = 'ready';
  }

  async function finish() {
    await saveSettings({ onboardingDone: true, mode, secondsPerGiveaway: seconds });
    app.screen = 'home';
  }

  const steps = ['onb.step1', 'onb.step2', 'onb.step3'] as const;
</script>

<div class="screen wrap">
  <ol class="steps glass" aria-label="Étapes">
    {#each steps as key, i (key)}
      <li aria-current={i === stepIndex ? 'step' : undefined} class:done={i < stepIndex}>
        <span class="num">
          {#if i < stepIndex}<Icon name="check" size={13} />{:else}{i + 1}{/if}
        </span>
        {t(key)}
      </li>
    {/each}
  </ol>

  {#if step === 'connect' || step === 'waiting'}
    <div class="portal">
      <img src="/panda.png" alt="" />
    </div>
    <div class="head">
      <h1 class="serif">{t('onb.titleA')} <em>{t('onb.titleB')}</em></h1>
      <p class="muted lead">{t('onb.lead')}</p>
    </div>

    {#if step === 'connect'}
      <div class="actions">
        <button type="button" class="btn btn-primary btn-lg" onclick={login}>
          <Icon name="login" />
          {t('onb.login')}
        </button>
        <button type="button" class="btn btn-quiet" onclick={skipAccount}>{t('onb.skip')}</button>
      </div>
    {:else}
      <div class="waiting glass">
        <p>{t('onb.waiting')}</p>
        {#if notConnected}<p class="warn">{t('onb.notConnected')}</p>{/if}
        <div class="row">
          <button type="button" class="btn btn-primary" onclick={verify} disabled={verifying}>
            {#if verifying}<span class="spin"><Icon name="refresh" size={16} /></span>{t('onb.verifying')}{:else}<Icon
                name="check"
                size={16}
              />{t('onb.done')}{/if}
          </button>
          <button type="button" class="btn" onclick={login} disabled={verifying}>
            <Icon name="external" size={16} />
            {t('onb.login')}
          </button>
          <button type="button" class="btn btn-quiet" onclick={skipAccount} disabled={verifying}>{t('onb.skip')}</button>
        </div>
      </div>
    {/if}

    <ul class="safe glass">
      <li><span class="ico"><Icon name="lock" size={16} /></span>{t('onb.safe1')}</li>
      <li><span class="ico"><Icon name="monitor" size={16} /></span>{t('onb.safe2')}</li>
      <li><span class="ico"><Icon name="logout" size={16} /></span>{t('onb.safe3')}</li>
    </ul>
  {:else if step === 'check'}
    <div class="head">
      <h1 class="serif">{t('onb.checkTitle')}</h1>
      <p class="muted lead">{t('onb.checkLead')}</p>
    </div>
    {@const c = app.check}
    <Ring
      size={320}
      marker
      label={c ? t('home.checking', { done: c.done, total: c.total }) : t('onb.fetching')}
      segments={c ? [{ value: c.done, color: 'var(--accent)' }, { value: Math.max(0, c.total - c.done), color: 'transparent' }] : []}
    >
      {#if c}
        <span class="serif big">{c.done}</span>
        <span class="eyebrow">{t('ring.of', { n: c.total })}</span>
      {:else}
        <span class="spin accent"><Icon name="refresh" size={28} /></span>
        <span class="eyebrow">{t('onb.fetching')}</span>
      {/if}
    </Ring>
    {#if c}
      <ul class="legend">
        <li><span class="dot" style="background:var(--accent)"></span>{t('legend.active', { n: c.active })}</li>
        <li><span class="dot" style="background:var(--ended)"></span>{t('legend.ended', { n: c.ended })}</li>
        <li><span class="dot" style="background:var(--lilac)"></span>{t('legend.unknown', { n: c.unknown })}</li>
      </ul>
    {/if}
  {:else}
    <div class="head">
      {#if (counts?.queue ?? 0) > 0}
        <h1 class="serif">{t('home.titleA', { n: counts?.queue ?? 0 })} <em>{t('home.titleB')}</em></h1>
        <p class="muted lead">{t('onb.readyLead')}</p>
      {:else}
        <h1 class="serif">{t('home.emptyA')} <em>{t('home.emptyB')}</em></h1>
        <p class="muted lead">{t('home.leadEmpty')} {t('onb.readyLead')}</p>
      {/if}
    </div>
    <div class="choices glass">
      <div class="choice">
        <span class="label">{t('set.mode')}</span>
        <div class="seg">
          <button type="button" aria-pressed={mode === 'auto'} disabled={!connected} onclick={() => (mode = 'auto')}>
            {t('set.modeAuto')}
          </button>
          <button type="button" aria-pressed={mode === 'manual'} onclick={() => (mode = 'manual')}>{t('set.modeManual')}</button>
        </div>
        {#if !connected}<span class="faint small">{t('home.connectHint')}</span>{/if}
      </div>
      <div class="choice">
        <label class="label" for="onb-delay">{t('set.delay')} · <span class="serif num-inline">{seconds} s</span></label>
        <input id="onb-delay" type="range" min="2" max="15" step="1" bind:value={seconds} />
        {#if (counts?.queue ?? 0) > 0}
          <span class="faint small">{duration((counts?.queue ?? 0) * (seconds + (mode === 'auto' ? 3 : 0)))}</span>
        {/if}
      </div>
    </div>
    <button type="button" class="btn btn-primary btn-lg" onclick={finish}>
      <Icon name="sparkle" />
      {t('onb.finish')}
    </button>
  {/if}

  <p class="faint small foot">{t('footer.notAffiliated')}</p>
</div>

<style>
  .wrap {
    max-width: 660px;
    margin: 0 auto;
    min-height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 22px;
    padding: 32px 24px;
    text-align: center;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 6px;
    display: flex;
    gap: 4px;
    border-radius: 999px;
    font-size: 13px;
  }
  .steps li {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 14px 0 6px;
    border-radius: 999px;
    color: var(--muted);
  }
  .steps li[aria-current='step'] {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: var(--ink);
    font-weight: 600;
  }
  .num {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.22);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
  }
  .steps li[aria-current='step'] .num,
  .steps li.done .num {
    border: 0;
    background: var(--accent);
    color: var(--on-accent);
  }
  .portal {
    width: 116px;
    height: 116px;
    border-radius: 50%;
    padding: 10px;
    border: 1px solid color-mix(in srgb, var(--accent) 55%, transparent);
    box-shadow: 0 0 0 10px color-mix(in srgb, var(--accent) 6%, transparent), 0 0 90px color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .portal img {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: #fff;
    object-fit: cover;
  }
  .head {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
  }
  h1 {
    margin: 0;
    font-size: 56px;
    line-height: 1.02;
    letter-spacing: -0.01em;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    max-width: 500px;
    font-size: 16px;
    text-wrap: pretty;
  }
  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .waiting {
    width: 100%;
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .waiting p {
    margin: 0;
  }
  .warn {
    color: var(--coral);
    font-size: 14px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
  }
  .safe {
    list-style: none;
    margin: 0;
    padding: 8px 20px;
    width: 100%;
    text-align: left;
  }
  .safe li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 11px 0;
    font-size: 14px;
  }
  .safe li + li {
    border-top: 1px solid rgba(255, 255, 255, 0.07);
  }
  .ico {
    flex: none;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.06);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .big {
    font-size: 96px;
    line-height: 0.9;
  }
  .accent {
    color: var(--accent);
    display: inline-flex;
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
  .choices {
    width: 100%;
    padding: 20px 24px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 20px;
    text-align: left;
  }
  .choice {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }
  .num-inline {
    font-size: 20px;
    letter-spacing: 0;
    text-transform: none;
    color: var(--ink);
  }
  .small {
    font-size: 12px;
  }
  .foot {
    margin: 8px 0 0;
  }
</style>
