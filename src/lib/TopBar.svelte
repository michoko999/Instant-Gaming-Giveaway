<script lang="ts">
  import Icon from './Icon.svelte';
  import { app, t, type Screen } from './state.svelte';

  const links: { screen: Screen; key: 'nav.home' | 'nav.giveaways' | 'nav.history' | 'nav.settings' }[] = [
    { screen: 'home', key: 'nav.home' },
    { screen: 'giveaways', key: 'nav.giveaways' },
    { screen: 'history', key: 'nav.history' },
    { screen: 'settings', key: 'nav.settings' },
  ];

  const connected = $derived(app.overview?.settings.accountConnected ?? false);
</script>

<header class="bar">
  <div class="brand">
    <img src="/panda.png" alt="" class="logo" />
    <span class="serif name">Giveaway IG</span>
  </div>
  <nav aria-label="Navigation" class="nav glass">
    {#each links as link (link.screen)}
      <button
        type="button"
        class="link"
        aria-current={app.screen === link.screen ? 'page' : undefined}
        onclick={() => (app.screen = link.screen)}
      >
        {t(link.key)}
      </button>
    {/each}
  </nav>
  <button type="button" class="btn account" onclick={() => (app.screen = 'settings')}>
    <span class="avatar" class:on={connected}><Icon name="user" size={16} /></span>
    <span>{connected ? t('account.connected') : t('account.disconnected')}</span>
    {#if connected}<span class="dot glow"></span>{/if}
  </button>
</header>

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 20px 40px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #fff;
    object-fit: cover;
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.15);
  }
  .name {
    font-size: 26px;
    line-height: 1;
  }
  .nav {
    display: flex;
    gap: 2px;
    padding: 5px;
    border-radius: 999px;
  }
  .link {
    height: 36px;
    padding: 0 16px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font-size: 14px;
    cursor: pointer;
  }
  .link:hover {
    color: var(--ink);
  }
  .link[aria-current='page'] {
    background: rgba(255, 255, 255, 0.11);
    color: var(--ink);
    font-weight: 500;
  }
  .account {
    padding-left: 6px;
    font-size: 13px;
  }
  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.08);
    color: var(--muted);
  }
  .avatar.on {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    color: var(--accent);
  }
  .glow {
    width: 6px;
    height: 6px;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
</style>
