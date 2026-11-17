import { useEffect, useRef, useState } from "react";
import type {
  SimulationSnapshot,
  SimulationConfig,
  SimulationStatus,
  Metrics,
  ControllerKind,
} from "../types/simulation";
import "./Multiverse.css";
import { browserMode } from "../services/browserMode";
import {
  browserMultiverseRequest,
  disposeBrowserWorlds,
} from "../services/browserMultiverse";

const worldLimit = browserMode ? 4 : 8;

type Universe = {
  id: string;
  name: string;
  config: SimulationConfig;
  status: SimulationStatus;
  metrics: Metrics;
};
const base = `${(import.meta.env.VITE_API_URL || "http://127.0.0.1:8000").replace(/\/$/, "")}/api/multiverse`;
async function request<T>(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  if (browserMode) return browserMultiverseRequest<T>(path, method, body);
  const response = await fetch(base + path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}

function Habitat({ snapshot }: { snapshot?: SimulationSnapshot }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = ref.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx || !snapshot) return;
    const { world, habitat } = snapshot;
    ctx.setTransform(
      canvas.width / world.width,
      0,
      0,
      canvas.height / world.height,
      0,
      0,
    );
    ctx.fillStyle = "#202820";
    ctx.fillRect(0, 0, world.width, world.height);
    ctx.fillStyle = "#70965b";
    for (const food of world.food) ctx.fillRect(food.x, food.y, 3, 3);
    for (const organism of world.organisms) {
      ctx.fillStyle = habitat?.predation
        ? organism.species === "predator"
          ? "#ed925a"
          : "#a4cf85"
        : `hsl(${organism.genome.hue}, 65%, 65%)`;
      ctx.beginPath();
      ctx.arc(organism.x, organism.y, organism.genome.size, 0, Math.PI * 2);
      ctx.fill();
    }
  }, [snapshot]);
  return (
    <canvas ref={ref} width={800} height={450} aria-label="Live ecosystem" />
  );
}

export default function Multiverse() {
  const [worlds, setWorlds] = useState<Universe[]>([]);
  const [snapshots, setSnapshots] = useState<
    Record<string, SimulationSnapshot>
  >({});
  const [name, setName] = useState("Universe");
  const [seed, setSeed] = useState(42);
  const [population, setPopulation] = useState(1000);
  const [controller, setController] = useState<ControllerKind>("recurrent");
  const [predation, setPredation] = useState(false);
  const [foodRate, setFoodRate] = useState(10);
  const [ticks, setTicks] = useState(1000);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [connection, setConnection] = useState("");
  useEffect(() => {
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      try {
        const list = await request<Universe[]>("/worlds");
        const results = await Promise.allSettled(
          list.map(
            async (w) =>
              [
                w.id,
                await request<SimulationSnapshot>(`/worlds/${w.id}/snapshot`),
              ] as const,
          ),
        );
        if (stopped) return;
        setWorlds(list);
        setSnapshots(
          Object.fromEntries(
            results.flatMap((r) => (r.status === "fulfilled" ? [r.value] : [])),
          ),
        );
        setConnection(
          results.some((r) => r.status === "rejected")
            ? "Some snapshots unavailable"
            : "Connected",
        );
      } catch (e) {
        if (!stopped) setConnection(String(e));
      } finally {
        if (!stopped) timer = setTimeout(poll, 500);
      }
    }
    void poll();
    return () => {
      stopped = true;
      clearTimeout(timer);
      if (browserMode) disposeBrowserWorlds();
    };
  }, []);
  async function command(action: () => Promise<unknown>) {
    setBusy(true);
    setError("");
    try {
      await action();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="multiverse">
      <header>
        <small>02 / PARALLEL EXPERIMENTS</small>
        <h2>Multiverse</h2>
        <p>
          {worlds.length}/{worldLimit} worlds · {connection}
        </p>
      </header>
      {browserMode && (
        <p>
          Browser worlds run independently on the CPU and are cleared when you
          leave this page.
        </p>
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void command(() =>
            request("/worlds", "POST", {
              name,
              backend: "auto_cpu",
              config: {
                seed,
                initial_population: population,
                controller,
                predation,
                food_spawn_rate: foodRate,
              },
            }),
          );
        }}
      >
        <label>
          Name
          <input
            required
            maxLength={80}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <label>
          Seed
          <input
            type="number"
            min={0}
            max={2147483647}
            required
            value={seed}
            onChange={(e) => setSeed(Number(e.target.value))}
          />
        </label>
        <label>
          Population
          <input
            type="number"
            min={1}
            max={5000}
            required
            value={population}
            onChange={(e) => setPopulation(Number(e.target.value))}
          />
        </label>
        <label>
          Controller
          <select
            value={controller}
            onChange={(e) => {
              const value = e.target.value as ControllerKind;
              setController(value);
              if (value === "small" || value === "heuristic")
                setPredation(false);
            }}
          >
            <option value="heuristic">Heuristic</option>
            <option value="small">Small MLP</option>
            <option value="deep">Deep MLP</option>
            <option value="recurrent">GRU</option>
          </select>
        </label>
        <label>
          Food / tick
          <input
            type="number"
            min={0}
            max={2000}
            required
            value={foodRate}
            onChange={(e) => setFoodRate(Number(e.target.value))}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={predation}
            disabled={controller === "small" || controller === "heuristic"}
            onChange={(e) => setPredation(e.target.checked)}
          />{" "}
          Predation
        </label>
        <button disabled={busy || worlds.length >= worldLimit}>
          Create world
        </button>
      </form>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void command(async () => {
            const results = await request<Array<Universe & { error?: string }>>(
              "/run",
              "POST",
              { ticks },
            );
            const failures = results.filter((r) => r.error);
            if (failures.length)
              throw new Error(
                failures.map((r) => `${r.name}: ${r.error}`).join("; "),
              );
          });
        }}
      >
        <label>
          Batch ticks
          <input
            type="number"
            min={1}
            max={10000}
            required
            value={ticks}
            onChange={(e) => setTicks(Number(e.target.value))}
          />
        </label>
        <button disabled={busy || !worlds.length}>
          {busy ? "Working…" : "Run all worlds"}
        </button>
        <span>
          Pauses live worlds before advancing each by the requested ticks.
        </span>
      </form>
      {error && <p role="alert">{error}</p>}
      {worlds.length === 0 && (
        <div className="multiverse-empty">
          <small>PARALLEL SIMULATIONS</small>
          <h3>No worlds created yet.</h3>
          <p>
            Create a world above, then vary its seed or controller to compare
            evolutionary outcomes.
          </p>
        </div>
      )}
      <div className="multiverse-grid">
        {worlds.map((w) => {
          const s = snapshots[w.id];
          const status = s?.status ?? w.status;
          const metrics = s?.metrics ?? w.metrics;
          return (
            <article key={w.id}>
              <h3>{w.name}</h3>
              <p>
                Seed {w.config.seed} · {w.config.controller} ·{" "}
                {status.active_backend}
              </p>
              <Habitat snapshot={s} />
              <p>
                Tick {status.tick.toLocaleString()} · Population{" "}
                {metrics.population.toLocaleString()} · Generation{" "}
                {metrics.max_generation}
              </p>
              <p>
                Births {metrics.total_births} · Deaths {metrics.total_deaths} ·
                Energy {metrics.average_energy.toFixed(2)}
              </p>
              {w.config.predation && (
                <p>
                  Herbivores {metrics.herbivores} · Predators{" "}
                  {metrics.predators} · Kills {metrics.total_kills}
                </p>
              )}
              <div className="multiverse-actions">
                {[status.running ? "pause" : "start", "reset", "delete"].map(
                  (action) => (
                    <button
                      key={action}
                      disabled={busy}
                      onClick={() => {
                        if (
                          (action === "delete" || action === "reset") &&
                          !window.confirm(`${action} ${w.name}?`)
                        )
                          return;
                        void command(() =>
                          request(
                            `/worlds/${w.id}${action === "delete" ? "" : `/${action}`}`,
                            action === "delete" ? "DELETE" : "POST",
                          ),
                        );
                      }}
                    >
                      {action}
                    </button>
                  ),
                )}
              </div>
            </article>
          );
        })}
      </div>
      <h3>Population comparison</h3>
      {worlds.map((w) => {
        const m = snapshots[w.id]?.metrics ?? w.metrics;
        const max = Math.max(
          1,
          ...worlds.map(
            (v) => (snapshots[v.id]?.metrics ?? v.metrics).population,
          ),
        );
        return (
          <div className="multiverse-bar" key={w.id}>
            <span>{w.name}</span>
            <div>
              <i style={{ width: `${(m.population / max) * 100}%` }} />
            </div>
            <strong>{m.population}</strong>
          </div>
        );
      })}
      <div className="multiverse-table">
        <table>
          <thead>
            <tr>
              <th>World</th>
              <th>Tick</th>
              <th>Population</th>
              <th>Mean speed</th>
              <th>Mean metabolism</th>
              <th>Max generation</th>
            </tr>
          </thead>
          <tbody>
            {worlds.map((w) => {
              const m = snapshots[w.id]?.metrics ?? w.metrics;
              return (
                <tr key={w.id}>
                  <td>{w.name}</td>
                  <td>{m.tick}</td>
                  <td>{m.population}</td>
                  <td>{m.average_speed.toFixed(3)}</td>
                  <td>{m.average_metabolism.toFixed(3)}</td>
                  <td>{m.max_generation}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
