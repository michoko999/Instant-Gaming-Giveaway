<script lang="ts">
  // Anneau « orbite » : segments colorés (répartition) ou progression simple.
  import type { Snippet } from 'svelte';

  type Segment = { value: number; color: string };
  let {
    segments,
    size = 420,
    label,
    marker = false,
    children,
  }: { segments: Segment[]; size?: number; label: string; marker?: boolean; children?: Snippet } = $props();

  const R = 160;
  const C = 2 * Math.PI * R;
  const GAP = 8;

  const arcs = $derived.by(() => {
    const total = segments.reduce((sum, s) => sum + Math.max(0, s.value), 0);
    if (total <= 0) return [];
    const visible = segments.filter((s) => s.value > 0);
    const gap = visible.length > 1 ? GAP : 0;
    let offset = 0;
    return visible.map((s) => {
      const length = (s.value / total) * C;
      const arc = { color: s.color, dash: Math.max(0.01, length - gap), offset: -offset };
      offset += length;
      return arc;
    });
  });

  // Position de la tête de progression (premier segment) pour l'effet « étoile ».
  const head = $derived.by(() => {
    const total = segments.reduce((sum, s) => sum + Math.max(0, s.value), 0);
    if (!marker || total <= 0 || segments[0].value <= 0 || segments[0].value >= total) return null;
    const angle = (segments[0].value / total) * 2 * Math.PI;
    return { x: 210 + R * Math.sin(angle), y: 210 - R * Math.cos(angle) };
  });
</script>

<div class="ring" style="width:{size}px;max-width:100%">
  <svg viewBox="0 0 420 420" role="img" aria-label={label}>
    <circle cx="210" cy="210" r="196" fill="none" stroke="rgba(255,255,255,0.10)" stroke-width="1" stroke-dasharray="1 7" stroke-linecap="round" />
    <circle cx="210" cy="210" r={R} fill="none" stroke="rgba(255,255,255,0.07)" stroke-width="12" />
    {#each arcs as arc, i (i)}
      {#if i === 0}
        <circle
          cx="210"
          cy="210"
          r={R}
          fill="none"
          stroke={arc.color}
          stroke-opacity="0.14"
          stroke-width="34"
          stroke-dasharray="{arc.dash} {C}"
          stroke-dashoffset={arc.offset}
          transform="rotate(-90 210 210)"
          class="arc"
          class:smooth={!marker}
        />
      {/if}
      <circle
        cx="210"
        cy="210"
        r={R}
        fill="none"
        stroke={arc.color}
        stroke-width="12"
        stroke-linecap={arcs.length === 1 ? 'round' : 'butt'}
        stroke-dasharray="{arc.dash} {C}"
        stroke-dashoffset={arc.offset}
        transform="rotate(-90 210 210)"
        class="arc"
        class:smooth={!marker}
      />
    {/each}
    {#if head}
      <circle cx={head.x} cy={head.y} r="16" fill="var(--accent)" fill-opacity="0.25" />
      <circle cx={head.x} cy={head.y} r="7" fill="#FFFFFF" />
    {/if}
    <circle cx="210" cy="210" r="126" fill="none" stroke="rgba(255,255,255,0.07)" stroke-width="1" />
  </svg>
  <div class="center">
    {@render children?.()}
  </div>
</div>

<style>
  .ring {
    position: relative;
    aspect-ratio: 1 / 1;
  }
  svg {
    width: 100%;
    height: 100%;
    display: block;
  }
  /* Sans repère « étoile », la répartition s'anime en douceur ; avec repère, l'arc suit l'étoile. */
  .arc.smooth {
    transition: stroke-dasharray 0.6s ease, stroke-dashoffset 0.6s ease;
  }
  .center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    text-align: center;
  }
</style>
