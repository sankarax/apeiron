import { browserMode } from "../services/browserMode";
import { useEffect, useRef, useState } from "react";
import "./Home.css";

type Frame = {
  tick: number;
  organisms: [number, number, number, number][];
  food: [number, number][];
};

type Replay = {
  config: { width: number; height: number; seed: number };
  fps: number;
  frames: Frame[];
};

export default function Home({ enter }: { enter: () => void }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const timer = useRef<number | undefined>(undefined);
  const [status, setStatus] = useState("Loading engine recording…");
  const [entering, setEntering] = useState(false);

  useEffect(() => {
    const el = canvas.current;
    const ctx = el?.getContext("2d");
    if (!el || !ctx) return;
    const controller = new AbortController();
    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let animation = 0;
    let observer: ResizeObserver | undefined;

    async function load() {
      const response = await fetch(
        `${import.meta.env.BASE_URL}homepage-replay.json`,
        {
          signal: controller.signal,
        },
      );
      if (!response.ok) throw new Error("Recording unavailable");
      const replay: Replay = await response.json();
      if (!replay.frames.length || controller.signal.aborted) return;
      const { width, height, seed } = replay.config;
      let start: number | undefined;
      let lastIndex = -1;

      function draw(now: number) {
        if (!el || !ctx || controller.signal.aborted) return;
        start ??= now;
        const index = reduced
          ? 0
          : Math.floor(((now - start) * replay.fps) / 1000) %
            replay.frames.length;
        const frame = replay.frames[index];
        const w = el.clientWidth;
        const h = el.clientHeight;
        const dpr = Math.min(devicePixelRatio, 2);
        const resized =
          el.width !== Math.round(w * dpr) || el.height !== Math.round(h * dpr);
        if (resized) {
          el.width = Math.round(w * dpr);
          el.height = Math.round(h * dpr);
        }
        if (resized || index !== lastIndex) {
          ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
          ctx.fillStyle = "#202820";
          ctx.fillRect(0, 0, w, h);
          const scale = Math.min(w / width, h / height) * 0.92;
          const ox = (w - width * scale) / 2;
          const oy = (h - height * scale) / 2;
          ctx.fillStyle = "#a6b98b70";
          for (const [x, y] of frame.food) {
            ctx.fillRect(ox + x * scale, oy + y * scale, 1.5, 1.5);
          }
          for (const [x, y, size, predator] of frame.organisms) {
            const px = ox + x * scale;
            const py = oy + y * scale;
            const radius = Math.max(1.5, size * scale);
            ctx.beginPath();
            ctx.arc(px, py, radius, 0, Math.PI * 2);
            ctx.fillStyle = predator ? "#d18e72" : "#a8c58c";
            ctx.fill();
          }
          if (index !== lastIndex) {
            setStatus(
              `SEED ${seed} · TICK ${frame.tick} · POPULATION ${frame.organisms.length}`,
            );
          }
          lastIndex = index;
        }
        if (!reduced) animation = requestAnimationFrame(draw);
      }

      animation = requestAnimationFrame(draw);
      if (reduced) {
        observer = new ResizeObserver(() => draw(performance.now()));
        observer.observe(el!);
      }
    }

    void load().catch(() => {
      if (!controller.signal.aborted)
        setStatus(
          "Engine recording unavailable - generate the replay locally.",
        );
    });
    return () => {
      controller.abort();
      cancelAnimationFrame(animation);
      observer?.disconnect();
      window.clearTimeout(timer.current);
    };
  }, []);

  function initiate() {
    if (entering) return;
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
      enter();
      return;
    }
    setEntering(true);
    timer.current = window.setTimeout(enter, 550);
  }

  return (
    <main className={`home${entering ? " home-entering" : ""}`}>
      <header className="home-top">
        <span>APEIRON / ARTIFICIAL LIFE</span>
        <span>SIMULATION PLATFORM</span>
      </header>
      <div className="home-hero">
        <div className="home-copy">
          <p className="eyebrow">Artificial Life Simulation</p>
          <h1>
            Apeiron<span>.</span>
          </h1>
          <h2>Evolution and Neural Simulation</h2>
          <p>
            Simulate evolution, inspect neural networks, and run controlled
            ecosystem experiments.
          </p>
          <button className="enter-life" onClick={initiate} disabled={entering}>
            Launch Simulation <span aria-hidden="true">↗</span>
          </button>
        </div>
        <figure className="home-figure">
          <div className="home-recording">
            <canvas
              ref={canvas}
              aria-label="Recorded artificial life simulation"
            />
            <span className="home-recording-status">{status}</span>
          </div>
          <figcaption>
            FIG. 01 / RUST ENGINE RECORDING · LOOPED PLAYBACK
          </figcaption>
        </figure>
      </div>
      <footer className="home-footer">
        <span>
          {browserMode
            ? "RUST · NEUROEVOLUTION · WEBASSEMBLY"
            : "RUST · NEUROEVOLUTION · GPU COMPUTE"}
        </span>
        <span>
          {browserMode ? "PUBLIC BROWSER EDITION" : "LOCAL RESEARCH EDITION"}
        </span>
      </footer>
    </main>
  );
}
