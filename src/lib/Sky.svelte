<script lang="ts">
  // Ciel étoilé : étoiles de tailles et couleurs variées, Voie lactée, scintillement,
  // rotation très lente et étoiles filantes occasionnelles.
  let { animated = true }: { animated?: boolean } = $props();

  type Star = { x: number; y: number; d: number; c: string; g: string; o?: number; dur?: number; del?: number };
  type Bright = Star & { sl: number; k: string };

  function buildSky() {
    let seed = 20261111;
    const rnd = () => {
      seed = (seed + 0x6d2b79f5) | 0;
      let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
    const tints: [string, string][] = [
      ['#FFFFFF', 'rgba(255,255,255,'],
      ['#FFFFFF', 'rgba(255,255,255,'],
      ['#FFFFFF', 'rgba(255,255,255,'],
      ['#E3E9FF', 'rgba(205,218,255,'],
      ['#C6D5FF', 'rgba(170,195,255,'],
      ['#FFF1DC', 'rgba(255,232,200,'],
      ['#FFD7AE', 'rgba(255,200,150,'],
    ];
    const tint = () => tints[Math.floor(Math.pow(rnd(), 2) * tints.length)];
    const ang = -Math.PI / 6;
    const ca = Math.cos(ang);
    const sa = Math.sin(ang);
    const stars: Star[] = [];
    const twinkle: Star[] = [];
    for (let i = 0; i < 1000; i++) {
      const band = i < 380;
      let x: number;
      let y: number;
      if (band) {
        const t = (rnd() - 0.5) * 1.1;
        const off = (rnd() + rnd() + rnd() - 1.5) * 0.05;
        x = 50 + 100 * (t * ca - off * sa);
        y = 50 + 100 * (t * sa + off * ca);
      } else {
        x = rnd() * 100;
        y = rnd() * 100;
      }
      if (x < 0 || x > 100 || y < 0 || y > 100) continue;
      const m = Math.pow(rnd(), band ? 5 : 3);
      const d = band ? 0.5 + m * 1.1 : 0.55 + m * 1.9;
      const [c, g] = tint();
      if (!band && d > 1.25 && rnd() < 0.55) {
        twinkle.push({ x, y, d, c, g: `0 0 ${(d * 2.2).toFixed(2)}px ${g}0.45)`, dur: 1.8 + rnd() * 4.5, del: -rnd() * 6 });
      } else {
        const o = band ? 0.22 + rnd() * 0.45 : 0.28 + m * 0.55 + rnd() * 0.15;
        stars.push({ x, y, d, c, o, g: d > 1.6 ? `0 0 ${(d * 1.8).toFixed(2)}px ${g}0.35)` : 'none' });
      }
    }
    const bright: Bright[] = [];
    for (let i = 0; i < 26; i++) {
      const d = 2.1 + rnd() * 1.5;
      const [c, g] = tint();
      const sl = i < 9 ? 14 + rnd() * 26 : 0;
      bright.push({
        x: 4 + rnd() * 92,
        y: 4 + rnd() * 92,
        d,
        c,
        g: `0 0 ${(d * 2.5).toFixed(2)}px ${(d * 0.5).toFixed(2)}px ${g}0.55), 0 0 ${(d * 7).toFixed(2)}px ${(d * 1.5).toFixed(2)}px ${g}0.18)`,
        sl,
        k: `${g}0.6)`,
        dur: 2.5 + rnd() * 4,
        del: -rnd() * 6,
      });
    }
    return { stars, twinkle, bright };
  }

  const sky = buildSky();
  const dot = (s: Star) =>
    `left:${s.x.toFixed(2)}%;top:${s.y.toFixed(2)}%;width:${s.d.toFixed(2)}px;height:${s.d.toFixed(2)}px;margin:${(-s.d / 2).toFixed(2)}px 0 0 ${(-s.d / 2).toFixed(2)}px;background:${s.c};box-shadow:${s.g}`;
</script>

<div class="sky" class:still={!animated} aria-hidden="true">
  <div class="nebula"></div>
  <div class="rotor">
    <div class="milky milky-glow"></div>
    <div class="milky milky-core"></div>
    <div class="milky milky-dust"></div>
    {#each sky.stars as s, i (i)}
      <span class="star" style="{dot(s)};opacity:{s.o?.toFixed(2)}"></span>
    {/each}
    {#each sky.twinkle as s, i (i)}
      <span class="star tw" style="{dot(s)};animation-duration:{s.dur?.toFixed(2)}s;animation-delay:{s.del?.toFixed(2)}s"></span>
    {/each}
    {#each sky.bright as s, i (i)}
      <span
        class="bright tw"
        style="left:{s.x.toFixed(2)}%;top:{s.y.toFixed(2)}%;animation-duration:{s.dur?.toFixed(2)}s;animation-delay:{s.del?.toFixed(2)}s"
      >
        {#if s.sl > 0}
          <span class="spike h" style="left:{(-s.sl / 2).toFixed(2)}px;width:{s.sl.toFixed(2)}px;background:linear-gradient(90deg,transparent,{s.k},transparent)"></span>
          <span class="spike v" style="top:{(-s.sl / 2).toFixed(2)}px;height:{s.sl.toFixed(2)}px;background:linear-gradient(180deg,transparent,{s.k},transparent)"></span>
        {/if}
        <span
          class="core"
          style="left:{(-s.d / 2).toFixed(2)}px;top:{(-s.d / 2).toFixed(2)}px;width:{s.d.toFixed(2)}px;height:{s.d.toFixed(2)}px;background:{s.c};box-shadow:{s.g}"
        ></span>
      </span>
    {/each}
  </div>
  <div class="shoot shoot-a"><span class="tail" style="width:180px"></span><span class="head"></span></div>
  <div class="shoot shoot-b"><span class="tail" style="width:140px"></span><span class="head"></span></div>
</div>

<style>
  .sky {
    position: fixed;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    background: var(--bg);
    z-index: 0;
  }
  .nebula {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(50% 55% at 78% 30%, rgba(150, 120, 255, 0.22), transparent 70%),
      radial-gradient(45% 40% at 0% 100%, rgba(70, 150, 255, 0.13), transparent 70%),
      radial-gradient(30% 30% at 20% 10%, rgba(255, 190, 110, 0.06), transparent 70%);
  }
  .rotor {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 150vmax;
    height: 150vmax;
    margin-left: -75vmax;
    margin-top: -75vmax;
    will-change: transform;
    animation: igsky 1800s linear infinite;
  }
  .milky {
    position: absolute;
  }
  .milky-glow {
    left: -5%;
    top: 38%;
    width: 110%;
    height: 24%;
    transform: rotate(-30deg);
    background: radial-gradient(closest-side, rgba(205, 195, 255, 0.16), rgba(160, 150, 240, 0.07) 55%, transparent);
    filter: blur(28px);
  }
  .milky-core {
    left: 18%;
    top: 45%;
    width: 64%;
    height: 10%;
    transform: rotate(-30deg);
    background: radial-gradient(closest-side, rgba(255, 236, 210, 0.1), transparent);
    filter: blur(20px);
  }
  .milky-dust {
    left: 10%;
    top: 48.5%;
    width: 80%;
    height: 3%;
    transform: rotate(-31deg);
    background: radial-gradient(closest-side, rgba(6, 8, 24, 0.55), transparent);
    filter: blur(10px);
  }
  .star {
    position: absolute;
    border-radius: 50%;
  }
  .bright {
    position: absolute;
    width: 0;
    height: 0;
  }
  .core {
    position: absolute;
    border-radius: 50%;
  }
  .spike {
    position: absolute;
  }
  .spike.h {
    top: -0.5px;
    height: 1px;
  }
  .spike.v {
    left: -0.5px;
    width: 1px;
  }
  .tw {
    animation-name: igtw;
    animation-timing-function: ease-in-out;
    animation-iteration-count: infinite;
  }
  .shoot {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
    animation: igshoot linear infinite;
  }
  .shoot-a {
    left: 80%;
    top: 7%;
    animation-duration: 23s;
    animation-delay: 5s;
  }
  .shoot-b {
    left: 42%;
    top: 3%;
    animation-duration: 37s;
    animation-delay: 17s;
  }
  .tail {
    position: absolute;
    left: 0;
    top: -0.75px;
    height: 1.5px;
    border-radius: 1px;
    transform-origin: 0 50%;
    transform: rotate(-26.57deg);
    background: linear-gradient(90deg, #ffffff, rgba(255, 255, 255, 0.35) 30%, rgba(255, 255, 255, 0));
  }
  .head {
    position: absolute;
    left: -1.5px;
    top: -1.5px;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 0 8px 2px rgba(255, 255, 255, 0.7);
  }
  .still .rotor,
  .still .tw,
  .still .shoot {
    animation: none;
  }
  .still .shoot {
    display: none;
  }
  @keyframes igsky {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes igtw {
    0%,
    100% {
      opacity: 1;
    }
    18% {
      opacity: 0.4;
    }
    30% {
      opacity: 0.95;
    }
    52% {
      opacity: 0.6;
    }
    70% {
      opacity: 1;
    }
    84% {
      opacity: 0.75;
    }
  }
  @keyframes igshoot {
    0% {
      transform: translate3d(0, 0, 0);
      opacity: 0;
    }
    0.6% {
      opacity: 1;
    }
    4.5% {
      transform: translate3d(-560px, 280px, 0);
      opacity: 0;
    }
    100% {
      transform: translate3d(-560px, 280px, 0);
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .rotor,
    .tw,
    .shoot {
      animation: none;
    }
    .shoot {
      display: none;
    }
  }
</style>
