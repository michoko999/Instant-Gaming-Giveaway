<script lang="ts">
  import { onMount } from 'svelte';
  import Sky from './lib/Sky.svelte';
  import TopBar from './lib/TopBar.svelte';
  import Icon from './lib/Icon.svelte';
  import { api } from './lib/api';
  import { app, boot, fail, t } from './lib/state.svelte';
  import Onboarding from './screens/Onboarding.svelte';
  import Home from './screens/Home.svelte';
  import Session from './screens/Session.svelte';
  import Recap from './screens/Recap.svelte';
  import Giveaways from './screens/Giveaways.svelte';
  import History from './screens/History.svelte';
  import Settings from './screens/Settings.svelte';

  onMount(() => {
    boot().catch(fail);
  });

  const withNav = $derived(['home', 'giveaways', 'history', 'settings'].includes(app.screen));
</script>

<Sky animated={app.overview?.settings.skyAnimation ?? true} />

<div class="shell">
  {#if withNav}
    <TopBar />
    {#if app.update}
      <div class="update glass">
        <Icon name="sparkle" size={16} />
        <span>{t('update.available', { version: app.update.latest ?? '' })}</span>
        <button type="button" class="btn btn-sm" onclick={() => app.update?.url && api.openExternal(app.update.url).catch(fail)}>
          {t('update.open')}
        </button>
        <button type="button" class="btn btn-sm btn-quiet icon-btn" aria-label={t('common.close')} onclick={() => (app.update = null)}>
          <Icon name="close" size={14} />
        </button>
      </div>
    {/if}
  {/if}

  <main class="content">
    {#if app.screen === 'loading'}
      <div class="loading"><span class="serif">{t('common.loading')}</span></div>
    {:else if app.screen === 'onboarding'}
      <Onboarding />
    {:else if app.screen === 'home'}
      <Home />
    {:else if app.screen === 'session'}
      <Session />
    {:else if app.screen === 'recap'}
      <Recap />
    {:else if app.screen === 'giveaways'}
      <Giveaways />
    {:else if app.screen === 'history'}
      <History />
    {:else if app.screen === 'settings'}
      <Settings />
    {/if}
  </main>
</div>

{#if app.toast}
  <div class="toast glass" class:error={app.toast.tone === 'error'} role="status">
    <span>{app.toast.text}</span>
    <button type="button" class="btn btn-sm btn-quiet icon-btn" aria-label={t('common.close')} onclick={() => (app.toast = null)}>
      <Icon name="close" size={14} />
    </button>
  </div>
{/if}

<style>
  .shell {
    position: relative;
    z-index: 1;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .loading {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    color: var(--muted);
  }
  .update {
    margin: 0 40px 8px;
    padding: 8px 8px 8px 18px;
    display: flex;
    align-items: center;
    gap: 12px;
    border-radius: 999px;
    color: var(--accent);
    font-size: 14px;
  }
  .update span {
    flex: 1;
    color: var(--ink);
  }
  .toast {
    position: fixed;
    z-index: 10;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    max-width: min(640px, calc(100% - 48px));
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 10px 10px 20px;
    border-radius: 18px;
    background: rgba(17, 21, 46, 0.92);
    font-size: 14px;
  }
  .toast.error {
    border-color: color-mix(in srgb, var(--coral) 45%, transparent);
  }
</style>
