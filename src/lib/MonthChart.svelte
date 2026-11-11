<script lang="ts">
  import type { MonthCount } from './api';
  import { app, t } from './state.svelte';

  let { data }: { data: MonthCount[] } = $props();

  // Survol : détail d'un mois. Au clavier et pour les lecteurs d'écran, chaque colonne a son
  // libellé et le bouton « Tableau » donne toutes les valeurs sans survol.

  let showTable = $state(false);
  let active = $state<number | null>(null);

  const last = $derived(data.length - 1);
  const max = $derived(Math.max(0, ...data.map((d) => d.count)));

  // Graduations « rondes » : au plus 4 intervalles.
  const step = $derived([1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000].find((s) => Math.ceil(max / s) <= 4) ?? Math.ceil(max / 4));
  const top = $derived(Math.max(step, Math.ceil(max / step) * step));
  const ticks = $derived(Array.from({ length: Math.round(top / step) + 1 }, (_, i) => i * step));

  const toDate = (month: string) => new Date(Number(month.slice(0, 4)), Number(month.slice(5, 7)) - 1, 1);
  const shortMonth = (month: string) => toDate(month).toLocaleDateString(app.lang, { month: 'short' }).replace('.', '');
  const longMonth = (month: string) => toDate(month).toLocaleDateString(app.lang, { month: 'long', year: 'numeric' });
  const num = (n: number) => n.toLocaleString(app.lang);
  const pct = (n: number) => `${(n / top) * 100}%`;
</script>

<section class="chart glass">
  <header class="top">
    <h2 class="label" id="month-chart-title">{t('hist.chartTitle')}</h2>
    <button type="button" class="btn btn-sm btn-quiet" aria-pressed={showTable} onclick={() => (showTable = !showTable)}>
      {showTable ? t('hist.chartGraph') : t('hist.chartTable')}
    </button>
  </header>

  {#if showTable}
    <table>
      <thead>
        <tr><th scope="col">{t('hist.chartMonth')}</th><th scope="col" class="num">{t('hist.chartCount')}</th></tr>
      </thead>
      <tbody>
        {#each [...data].reverse() as d (d.month)}
          <tr><td>{longMonth(d.month)}</td><td class="num">{num(d.count)}</td></tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <div class="plot" role="group" aria-labelledby="month-chart-title">
      <div class="grid" aria-hidden="true">
        {#each ticks as tick (tick)}
          <div class="tick" class:base={tick === 0} style:bottom={pct(tick)}><span>{num(tick)}</span></div>
        {/each}
      </div>
      <div class="cols">
        {#each data as d, i (d.month)}
          <div
            class="col"
            class:current={i === last}
            class:active={active === i}
            role="img"
            aria-label="{longMonth(d.month)} : {num(d.count)}"
            onpointerenter={() => (active = i)}
            onpointerleave={() => (active = null)}
          >
            <div class="area">
              {#if d.count > 0}<div class="bar" style:height={pct(d.count)}></div>{/if}
              {#if i === last && active !== i}
                <span class="cap" style:bottom={pct(d.count)}>{num(d.count)}</span>
              {/if}
              {#if active === i}
                <div class="tip" class:left={i > last - 2} style:bottom={pct(d.count)}>
                  <strong>{num(d.count)}</strong>
                  <span>{longMonth(d.month)}</span>
                </div>
              {/if}
            </div>
            <span class="month" aria-hidden="true">{shortMonth(d.month)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .chart {
    padding: 18px 20px 14px;
    border-radius: 24px;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 14px;
  }
  .top h2 {
    margin: 0;
  }
  /* Les colonnes et la grille partagent la même hauteur : 26 px réservés en bas aux mois. */
  .plot {
    position: relative;
    height: 220px;
    padding-left: 34px;
  }
  .grid {
    position: absolute;
    inset: 0 0 26px 34px;
  }
  .tick {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: rgba(255, 255, 255, 0.06);
  }
  .tick.base {
    background: var(--line-strong);
  }
  .tick span {
    position: absolute;
    right: calc(100% + 10px);
    top: -7px;
    font-size: 11px;
    line-height: 14px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }
  .cols {
    position: relative;
    height: 100%;
    display: flex;
  }
  .col {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    cursor: default;
  }
  .area {
    position: relative;
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: flex-end;
  }
  .bar {
    width: min(24px, 56%);
    border-radius: 4px 4px 0 0;
    background: var(--accent);
    opacity: 0.6;
    transition: opacity 0.15s ease;
  }
  .col.current .bar,
  .col.active .bar {
    opacity: 1;
  }
  .cap {
    position: absolute;
    left: 50%;
    transform: translate(-50%, -6px);
    font-size: 12px;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
  }
  .tip {
    position: absolute;
    left: 50%;
    z-index: 2;
    transform: translate(-50%, -10px);
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 12px;
    border-radius: 12px;
    background: rgba(12, 15, 38, 0.96);
    border: 1px solid var(--line-strong);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
    white-space: nowrap;
    pointer-events: none;
  }
  .tip.left {
    left: auto;
    right: 0;
    transform: translate(0, -10px);
  }
  .tip strong {
    font-size: 18px;
    font-weight: 600;
    color: var(--ink);
  }
  .tip span {
    font-size: 12px;
    color: var(--muted);
  }
  .month {
    height: 26px;
    padding-top: 8px;
    text-align: center;
    font-size: 11px;
    color: var(--faint);
    white-space: nowrap;
  }
  .col.current .month {
    color: var(--muted);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
  }
  th {
    text-align: left;
    font-weight: 500;
    font-size: 12px;
    color: var(--faint);
    padding: 0 0 8px;
  }
  td {
    padding: 8px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    text-transform: capitalize;
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
</style>
