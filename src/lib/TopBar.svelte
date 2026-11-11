<script lang="ts">
  import Avatar from './Avatar.svelte';
  import Icon from './Icon.svelte';
  import { accountName, activeAccount, addAccount, app, showAccounts, switchAccount, t, type Screen } from './state.svelte';

  const links: { screen: Screen; key: 'nav.home' | 'nav.giveaways' | 'nav.history' | 'nav.settings' }[] = [
    { screen: 'home', key: 'nav.home' },
    { screen: 'giveaways', key: 'nav.giveaways' },
    { screen: 'history', key: 'nav.history' },
    { screen: 'settings', key: 'nav.settings' },
  ];

  const MAX_ACCOUNTS = 8;
  const accounts = $derived(app.overview?.accounts ?? []);
  const active = $derived(activeAccount());
  const connected = $derived(active?.connected ?? false);
  const label = $derived(
    connected ? (active?.name ?? t('account.connected')) : accounts.length > 1 ? accountName(active) : t('account.disconnected'),
  );

  // Menu des comptes : se ferme au clic à l'extérieur ou avec Échap.
  let open = $state(false);
  let box = $state<HTMLDivElement>();
  function outside(event: MouseEvent) {
    if (open && box && !box.contains(event.target as Node)) open = false;
  }
</script>

<svelte:window onclick={outside} onkeydown={(e) => e.key === 'Escape' && (open = false)} />

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
  <div class="accounts" bind:this={box}>
    <button
      type="button"
      class="btn account"
      aria-haspopup="menu"
      aria-expanded={open}
      title={t('account.menu')}
      onclick={() => (open = !open)}
    >
      <Avatar account={active} size={32} />
      <span class="acct-name">{label}</span>
      {#if connected}<span class="dot glow"></span>{/if}
      <span class="chev" class:up={open}><Icon name="chevron-down" size={14} /></span>
    </button>
    {#if open}
      <div class="menu glass" role="menu" aria-label={t('account.menu')}>
        {#each accounts as account (account.id)}
          <button
            type="button"
            class="item"
            role="menuitemradio"
            aria-checked={account.active}
            onclick={() => {
              open = false;
              switchAccount(account.id);
            }}
          >
            <Avatar {account} size={30} />
            <span class="who">
              <span class="nm">{accountName(account)}</span>
              <span class="sub">
                {account.connected ? t('account.doneMonth', { n: account.doneThisMonth }) : t('account.disconnected')}
              </span>
            </span>
            {#if account.active}<span class="tick"><Icon name="check" size={16} /></span>{/if}
          </button>
        {/each}
        <div class="sep"></div>
        <button
          type="button"
          class="item action"
          role="menuitem"
          disabled={accounts.length >= MAX_ACCOUNTS}
          onclick={() => {
            open = false;
            addAccount();
          }}
        >
          <span class="ico"><Icon name="plus" size={16} /></span>{t('account.add')}
        </button>
        <button
          type="button"
          class="item action"
          role="menuitem"
          onclick={() => {
            open = false;
            showAccounts();
          }}
        >
          <span class="ico"><Icon name="user" size={16} /></span>{t('account.manage')}
        </button>
      </div>
    {/if}
  </div>
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
  .accounts {
    position: relative;
  }
  .account {
    padding-left: 6px;
    padding-right: 12px;
    font-size: 13px;
    max-width: 280px;
  }
  .acct-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chev {
    display: inline-flex;
    color: var(--faint);
    transition: transform 0.15s ease;
  }
  .chev.up {
    transform: rotate(180deg);
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    z-index: 30;
    width: 280px;
    padding: 6px;
    border-radius: 18px;
    background: rgba(14, 17, 42, 0.97);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.5);
    animation: igfadein 0.15s ease-out;
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: 0;
    border-radius: 12px;
    background: transparent;
    color: var(--ink);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }
  .item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.07);
  }
  .item:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .item[aria-checked='true'] {
    background: rgba(255, 255, 255, 0.06);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nm {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 11px;
    color: var(--faint);
  }
  .tick {
    display: inline-flex;
    color: var(--accent);
  }
  .sep {
    height: 1px;
    margin: 6px 4px;
    background: rgba(255, 255, 255, 0.08);
  }
  .action {
    color: var(--muted);
  }
  .ico {
    width: 30px;
    display: inline-flex;
    justify-content: center;
  }
  .glow {
    width: 6px;
    height: 6px;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
</style>
