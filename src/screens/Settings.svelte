<script lang="ts">
  import Avatar from '../lib/Avatar.svelte';
  import Icon from '../lib/Icon.svelte';
  import { api, REPO_URL, type Lang, type Source } from '../lib/api';
  import {
    ACCENTS,
    LANGUAGES,
    accountName,
    addAccount,
    app,
    fail,
    logoutAccount,
    minDelay,
    openLogin,
    refresh,
    removeAccount,
    saveSettings,
    switchAccount,
    t,
    toast,
    verifyLogin,
    SAFE_MIN_DELAY,
  } from '../lib/state.svelte';

  const o = $derived(app.overview);
  const s = $derived(o?.settings);

  const min = $derived(minDelay(s));
  let seconds = $state(SAFE_MIN_DELAY);
  $effect(() => {
    seconds = s?.secondsPerGiveaway ?? SAFE_MIN_DELAY;
  });

  const MAX_ACCOUNTS = 8;
  const accounts = $derived(o?.accounts ?? []);
  // Retirer un compte se confirme d'un second clic.
  let confirmRemove = $state<string | null>(null);
  let confirmTimer: ReturnType<typeof setTimeout> | undefined;
  function askRemove(id: string) {
    if (confirmRemove === id) {
      confirmRemove = null;
      removeAccount(id);
      return;
    }
    confirmRemove = id;
    clearTimeout(confirmTimer);
    confirmTimer = setTimeout(() => (confirmRemove = null), 4000);
  }
  let sourceName = $state('');
  let sourceUrl = $state('');

  const LIMITS = [20, 40, 80, 0];

  async function toggleSource(source: Source) {
    if (!s) return;
    await saveSettings({ sources: s.sources.map((x) => (x.id === source.id ? { ...x, enabled: !x.enabled } : x)) });
  }

  async function addSource(event: SubmitEvent) {
    event.preventDefault();
    try {
      await api.addSource(sourceName, sourceUrl);
      sourceName = '';
      sourceUrl = '';
      await refresh();
    } catch (e) {
      fail(e);
    }
  }

  async function removeSource(id: string) {
    try {
      await api.removeSource(id);
      await refresh();
    } catch (e) {
      fail(e);
    }
  }
</script>

{#if s && o}
  <div class="screen page">
    <header class="head">
      <h1 class="serif">{t('set.title')}</h1>
      <span class="faint small">{t('set.saved')}</span>
    </header>

    <div class="grid">
      <div class="col">
        <section class="card glass">
          <h2>{t('set.general')}</h2>
          <div class="line">
            <label for="set-lang">{t('set.language')}</label>
            <select
              id="set-lang"
              class="field"
              value={s.language}
              onchange={(e) => saveSettings({ language: (e.currentTarget as HTMLSelectElement).value as Lang })}
            >
              {#each LANGUAGES as l (l.code)}
                <option value={l.code}>{l.label}</option>
              {/each}
            </select>
          </div>
          <div class="line">
            <span id="accent-label">{t('set.accent')}</span>
            <div class="swatches" role="group" aria-labelledby="accent-label">
              {#each ACCENTS as color (color)}
                <button
                  type="button"
                  class="swatch"
                  style="background:{color}"
                  aria-label={color}
                  aria-pressed={s.accent.toUpperCase() === color}
                  onclick={() => saveSettings({ accent: color })}
                ></button>
              {/each}
            </div>
          </div>
          <label class="line">
            <span class="stack"><span>{t('set.sky')}</span><span class="faint small">{t('set.skyHint')}</span></span>
            <input type="checkbox" class="switch" checked={s.skyAnimation} onchange={() => saveSettings({ skyAnimation: !s.skyAnimation })} />
          </label>
        </section>

        <section class="card glass">
          <h2>{t('set.session')}</h2>
          <div class="line">
            <span>{t('set.mode')}</span>
            <div class="seg">
              <button type="button" aria-pressed={s.mode === 'auto'} onclick={() => saveSettings({ mode: 'auto' })}>{t('set.modeAuto')}</button>
              <button type="button" aria-pressed={s.mode === 'manual'} onclick={() => saveSettings({ mode: 'manual' })}>{t('set.modeManual')}</button>
            </div>
          </div>
          <div class="line">
            <span class="stack">
              <label for="set-delay">{t('set.delay')}</label>
              <span class="faint small">{s.safeMode ? `${t('set.delayHint')} ${t('set.delaySafe')}` : t('set.delayHint')}</span>
            </span>
            <div class="range">
              <input
                id="set-delay"
                type="range"
                {min}
                max="15"
                step="1"
                bind:value={seconds}
                onchange={() => saveSettings({ secondsPerGiveaway: Number(seconds) })}
              />
              <span class="serif val">{seconds} s</span>
            </div>
          </div>
          <label class="line">
            <span>{t('set.skipMonth')}</span>
            <input
              type="checkbox"
              class="switch"
              checked={s.skipDoneThisMonth}
              onchange={() => saveSettings({ skipDoneThisMonth: !s.skipDoneThisMonth })}
            />
          </label>
          <label class="line">
            <span class="stack">
              <span class="row-inline"><Icon name="shield" size={16} />{t('set.safe')}</span>
              <span class="faint small">{t('set.safeHint')}</span>
            </span>
            <input
              type="checkbox"
              class="switch"
              checked={s.safeMode}
              onchange={() =>
                saveSettings({
                  safeMode: !s.safeMode,
                  // Le mode prudent impose un minimum : on remonte le réglage tout de suite.
                  secondsPerGiveaway: s.safeMode ? s.secondsPerGiveaway : Math.max(s.secondsPerGiveaway, SAFE_MIN_DELAY),
                })}
            />
          </label>
          <label class="line">
            <span class="stack">
              <span class="row-inline"><Icon name="eye" size={16} />{t('set.showIg')}</span>
              <span class="faint small">{t('set.showIgHint')}</span>
            </span>
            <input type="checkbox" class="switch" checked={s.showIgWindow} onchange={() => saveSettings({ showIgWindow: !s.showIgWindow })} />
          </label>
          <div class="line">
            <span class="stack">
              <span id="limit-label">{t('set.limit')}</span>
              <span class="faint small">{t('set.limitHint')}</span>
            </span>
            <div class="seg" role="group" aria-labelledby="limit-label">
              {#each LIMITS as limit (limit)}
                <button type="button" aria-pressed={s.sessionLimit === limit} onclick={() => saveSettings({ sessionLimit: limit })}>
                  {limit === 0 ? t('set.unlimited') : limit}
                </button>
              {/each}
            </div>
          </div>
        </section>

      </div>

      <div class="col">
        <section class="card glass" id="accounts">
          <h2>{t('set.accounts')}</h2>
          {#each accounts as account (account.id)}
            <div class="line account">
              <span class="who">
                <Avatar {account} size={38} />
                <span class="stack min0">
                  <span class="row-inline">
                    <span class="ellipsis">{accountName(account)}</span>
                    {#if account.active && accounts.length > 1}<span class="chip">{t('set.activeAccount')}</span>{/if}
                  </span>
                  <span class="faint small status">
                    <span class="dot" style="background:{account.connected ? 'var(--accent)' : 'var(--faint)'}"></span>
                    {account.connected ? t('account.doneMonth', { n: account.doneThisMonth }) : t('set.accountDisconnected')}
                  </span>
                </span>
              </span>
              <div class="row">
                {#if account.active}
                  {#if account.connected}
                    <button type="button" class="btn btn-sm" onclick={verifyLogin} disabled={app.verifying}>{t('set.verify')}</button>
                    <button type="button" class="btn btn-sm btn-quiet" onclick={logoutAccount}><Icon name="logout" size={14} />{t('set.logout')}</button>
                  {:else if app.loginPending}
                    <button type="button" class="btn btn-sm btn-primary" onclick={verifyLogin} disabled={app.verifying}>
                      {app.verifying ? t('onb.verifying') : t('onb.done')}
                    </button>
                  {:else}
                    <button type="button" class="btn btn-sm btn-primary" onclick={openLogin}><Icon name="login" size={14} />{t('set.login')}</button>
                  {/if}
                {:else}
                  <button type="button" class="btn btn-sm" onclick={() => switchAccount(account.id)}>{t('set.useAccount')}</button>
                {/if}
                {#if accounts.length > 1}
                  <button
                    type="button"
                    class="btn btn-sm btn-quiet"
                    class:danger={confirmRemove === account.id}
                    title={t('set.removeHint')}
                    aria-label={t('set.removeAccount')}
                    onclick={() => askRemove(account.id)}
                  >
                    <Icon name="trash" size={14} />{confirmRemove === account.id ? t('set.removeConfirm') : ''}
                  </button>
                {/if}
              </div>
            </div>
          {/each}
          <div class="line">
            <span class="faint small hint">{t('set.accountsHint')}</span>
            <button type="button" class="btn btn-sm" onclick={addAccount} disabled={accounts.length >= MAX_ACCOUNTS}>
              <Icon name="plus" size={14} />{t('account.add')}
            </button>
          </div>
        </section>

        <section class="card glass">
          <h2>{t('set.sources')}</h2>
          {#each s.sources as source (source.id)}
            <div class="line">
              <span class="stack min0">
                <span class="mono src">{source.name}</span>
                <span class="faint small ellipsis">{source.url}</span>
              </span>
              <div class="row">
                {#if !source.builtin}
                  <button type="button" class="btn btn-sm btn-quiet icon-btn" aria-label={t('set.remove')} onclick={() => removeSource(source.id)}>
                    <Icon name="trash" size={16} />
                  </button>
                {/if}
                <input type="checkbox" class="switch" checked={source.enabled} aria-label={source.name} onchange={() => toggleSource(source)} />
              </div>
            </div>
          {/each}
          <form class="add" onsubmit={addSource}>
            <span class="small muted">{t('set.addSource')}</span>
            <div class="row">
              <input class="field grow" placeholder={t('set.sourceName')} bind:value={sourceName} aria-label={t('set.sourceName')} />
              <input class="field grow2" placeholder={t('set.sourceUrl')} bind:value={sourceUrl} required aria-label={t('set.sourceUrl')} />
              <button type="submit" class="btn btn-sm"><Icon name="plus" size={14} />{t('set.add')}</button>
            </div>
          </form>
          <label class="line">
            <span>{t('set.syncOnStart')}</span>
            <input type="checkbox" class="switch" checked={s.syncOnStart} onchange={() => saveSettings({ syncOnStart: !s.syncOnStart })} />
          </label>
        </section>

        <section class="card glass">
          <h2>{t('set.data')}</h2>
          <div class="line">
            <span class="stack min0">
              <span>{o.portable ? t('set.portable') : t('set.notPortable')}</span>
              <span class="mono faint small ellipsis">{o.dataDir}</span>
            </span>
            <button type="button" class="btn btn-sm" onclick={() => api.openDataDir().catch(fail)}><Icon name="folder" size={14} />{t('set.openFolder')}</button>
          </div>
          <label class="line">
            <span>{t('set.updates')}</span>
            <input type="checkbox" class="switch" checked={s.checkUpdates} onchange={() => saveSettings({ checkUpdates: !s.checkUpdates })} />
          </label>
        </section>

        <section class="card glass">
          <h2>{t('set.about')}</h2>
          <div class="line">
            <span class="status"><Icon name="shield" size={18} /> {t('set.version', { v: o.version })} · {t('set.license')}</span>
            <button type="button" class="btn btn-sm" onclick={() => api.openExternal(REPO_URL).catch(fail)}>
              <Icon name="external" size={14} />{t('set.github')}
            </button>
          </div>
          <p class="faint small foot">{t('footer.notAffiliated')}</p>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .page {
    max-width: 1180px;
    margin: 0 auto;
    padding: 8px 40px 40px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }
  h1 {
    margin: 0;
    font-size: 56px;
    line-height: 1;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(460px, 100%), 1fr));
    gap: 18px;
    align-items: start;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .card {
    padding: 6px 22px 10px;
  }
  h2 {
    margin: 0;
    padding: 14px 0 8px;
    font-family: var(--serif);
    font-weight: 400;
    font-size: 26px;
  }
  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px 16px;
    min-height: 58px;
    padding: 8px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.07);
  }
  label.line {
    cursor: pointer;
  }
  .stack {
    display: flex;
    flex-direction: column;
    line-height: 1.35;
    flex: 1 1 240px;
    min-width: 0;
  }
  .min0 {
    min-width: 0;
    flex: 1;
  }
  .ellipsis {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .src {
    font-size: 13px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .swatches {
    display: flex;
    gap: 10px;
  }
  .swatch {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.15);
  }
  .swatch[aria-pressed='true'] {
    border-color: var(--bg);
    box-shadow: 0 0 0 2px var(--ink);
  }
  .range {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 220px;
  }
  .val {
    font-size: 22px;
    width: 44px;
    text-align: right;
  }
  .row-inline {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .row-inline :global(svg) {
    color: var(--accent);
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .who {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    flex: 1 1 200px;
  }
  .row-inline .chip {
    flex: none;
  }
  .hint {
    flex: 1 1 220px;
    line-height: 1.45;
  }
  .btn.danger {
    color: var(--coral);
    background: color-mix(in srgb, var(--coral) 12%, transparent);
  }
  .add {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.07);
  }
  /* Le nom doit rester lisible dans toutes les langues (« Nom (facultatif) »…). */
  .grow {
    flex: 1 1 160px;
  }
  .grow2 {
    flex: 2 1 180px;
  }
  .small {
    font-size: 12px;
  }
  .foot {
    margin: 0 0 8px;
  }
</style>
