<script lang="ts">
  import type { Account } from './api';
  import Icon from './Icon.svelte';

  // Photo de profil Instant Gaming, sinon l'initiale du pseudo, sinon une silhouette.
  let { account, size = 32 }: { account: Account | undefined; size?: number } = $props();

  const initial = $derived((account?.name ?? '').trim().charAt(0).toUpperCase());
</script>

<span
  class="avatar"
  class:on={account?.connected}
  style:width="{size}px"
  style:height="{size}px"
  style:font-size="{Math.round(size * 0.44)}px"
  aria-hidden="true"
>
  {#if account?.avatar}
    <img src={account.avatar} alt="" />
  {:else if initial}
    <span class="initial">{initial}</span>
  {:else}
    <Icon name="user" size={Math.round(size / 2)} />
  {/if}
</span>

<style>
  .avatar {
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.08);
    color: var(--muted);
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.12);
  }
  .avatar.on {
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    color: var(--accent);
    box-shadow: 0 0 0 1.5px color-mix(in srgb, var(--accent) 70%, transparent);
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .initial {
    font-family: var(--serif, 'Instrument Serif', serif);
    line-height: 1;
  }
</style>
