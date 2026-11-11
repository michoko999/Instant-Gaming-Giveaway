<script lang="ts">
  // Icônes au trait (SVG en ligne, aucune ressource externe).
  type Name =
    | 'sparkle'
    | 'play'
    | 'pause'
    | 'stop'
    | 'skip'
    | 'next'
    | 'check'
    | 'alert'
    | 'minus-circle'
    | 'refresh'
    | 'upload'
    | 'download'
    | 'eye'
    | 'user'
    | 'login'
    | 'logout'
    | 'lock'
    | 'monitor'
    | 'globe'
    | 'folder'
    | 'external'
    | 'close'
    | 'plus'
    | 'search'
    | 'trash'
    | 'shield'
    | 'flame'
    | 'chevron-down';

  let { name, size = 18 }: { name: Name; size?: number } = $props();

  const paths: Record<Name, string> = {
    sparkle: '',
    play: '',
    pause: '',
    stop: '',
    skip: 'M5 5v14l10-7z M19 5v14',
    next: 'M5 12h14 M13 6l6 6-6 6',
    check: 'M5 12.5l4.5 4.5L19 7.5',
    alert: 'M12 9v4 M12 17h.01 M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
    'minus-circle': 'M12 4a8 8 0 1 0 0 16a8 8 0 1 0 0-16z M8.5 12h7',
    refresh: 'M20 11a8 8 0 1 0-2.3 5.7 M20 4v7h-7',
    upload: 'M12 15V3 M7 8l5-5 5 5 M4 15v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4',
    download: 'M12 3v12 M7 10l5 5 5-5 M4 15v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4',
    eye: 'M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6z',
    user: 'M12 4a4 4 0 1 0 0 8a4 4 0 1 0 0-8z M4 21c1.5-4 4.5-6 8-6s6.5 2 8 6',
    login: 'M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4 M10 17l5-5-5-5 M15 12H3',
    logout: 'M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4 M16 17l5-5-5-5 M21 12H9',
    lock: 'M7 11V8a5 5 0 0 1 10 0v3 M5 11h14v10H5z',
    monitor: 'M3 4h18v12H3z M8 20h8 M12 16v4',
    globe: 'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18z M3 12h18 M12 3c2.5 2.7 3.8 5.7 3.8 9s-1.3 6.3-3.8 9c-2.5-2.7-3.8-5.7-3.8-9S9.5 5.7 12 3z',
    folder: 'M3 6a1 1 0 0 1 1-1h5l2 2h9a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z',
    external: 'M14 4h6v6 M20 4l-9 9 M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5',
    close: 'M6 6l12 12 M18 6 6 18',
    plus: 'M12 5v14 M5 12h14',
    'chevron-down': 'M6 9l6 6 6-6',
    search: 'M11 4a7 7 0 1 0 0 14a7 7 0 1 0 0-14z M20 20l-3.5-3.5',
    trash: 'M4 7h16 M9 7V4h6v3 M6 7l1 13h10l1-13',
    shield: 'M12 3 4 6v6c0 4.5 3.4 8.3 8 9 4.6-.7 8-4.5 8-9V6z M8.5 12l2.5 2.5 4.5-5',
    flame: 'M12 3c1 3 4 4.5 4 9a4 4 0 1 1-8 0c0-2 1-3.2 2-4 0 2 1 3 2 3 0-3-1-5 0-8z',
  };
</script>

<svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true" class="icon">
  {#if name === 'sparkle'}
    <path fill="currentColor" d="M12 2C12.6 8 16 11.4 22 12C16 12.6 12.6 16 12 22C11.4 16 8 12.6 2 12C8 11.4 11.4 8 12 2Z" />
  {:else if name === 'play'}
    <path fill="currentColor" d="M7 4.5v15a1 1 0 0 0 1.5.86l12-7.5a1 1 0 0 0 0-1.72l-12-7.5A1 1 0 0 0 7 4.5z" />
  {:else if name === 'pause'}
    <rect fill="currentColor" x="6" y="4.5" width="4" height="15" rx="1" />
    <rect fill="currentColor" x="14" y="4.5" width="4" height="15" rx="1" />
  {:else if name === 'stop'}
    <rect fill="currentColor" x="5.5" y="5.5" width="13" height="13" rx="2" />
  {:else}
    <path d={paths[name]} fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
  {/if}
</svg>

<style>
  .icon {
    flex: none;
  }
</style>
