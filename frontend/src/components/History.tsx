import { browserMode } from "../services/browserMode";
import { browserRequest } from "../services/browserClient";
import { useEffect, useRef, useState } from "react";
import type { World, Metrics } from "../types/simulation";
import "./History.css";

type RecordEntry = {
  id: number;
  parent_id: number | null;
  generation: number;
  species: string;
  birth_tick: number;
  death_tick: number | null;
  genome: Record<string, number>;
};
type Checkpoint = { id: string; name: string; tick: number };
type Frame = { tick: number; world: World; metrics: Metrics };
const base = `${(import.meta.env.VITE_API_URL || "http://127.0.0.1:8000").replace(/\/$/, "")}/api/history`;
async function request<T>(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  const response = await fetch(base + path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}
export default function History() {
  const [records, setRecords] = useState<RecordEntry[]>([]);
  const [checkpoints, setCheckpoints] = useState<Checkpoint[]>([]);
  const [selected, setSelected] = useState("");
  const [name, setName] = useState("Observation 01");
  const [frames, setFrames] = useState<Frame[]>([]);
  const [index, setIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const canvas = useRef<HTMLCanvasElement>(null);
  const frame = frames[index];
  async function refresh() {
    if (browserMode) {
      setRecords(await browserRequest<RecordEntry[]>("lineage"));
      return;
    }
    const [r, c] = await Promise.all([
      request<RecordEntry[]>("/lineage"),
      request<Checkpoint[]>("/checkpoints"),
    ]);
    setRecords(r);
    setCheckpoints(c);
  }
  useEffect(() => {
    let active = true;
    const recordsRequest = browserMode
      ? browserRequest<RecordEntry[]>("lineage")
      : request<RecordEntry[]>("/lineage");
    const checkpointsRequest = browserMode
      ? Promise.resolve<Checkpoint[]>([])
      : request<Checkpoint[]>("/checkpoints");
    void Promise.all([recordsRequest, checkpointsRequest])
      .then(([r, c]) => {
        if (!active) return;
        setRecords(r);
        setCheckpoints(c);
      })
      .catch((err) => {
        if (active) setError(String(err));
      });
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    if (!playing) return;
    const timer = setInterval(
      () => setIndex((i) => Math.min(i + 1, frames.length - 1)),
      150,
    );
    return () => clearInterval(timer);
  }, [playing, frames.length]);
  useEffect(() => {
    const ctx = canvas.current?.getContext("2d");
    if (!ctx || !frame) return;
    const w = frame.world;
    ctx.setTransform(800 / w.width, 0, 0, 450 / w.height, 0, 0);
    ctx.fillStyle = "#202820";
    ctx.fillRect(0, 0, w.width, w.height);
    ctx.fillStyle = "#70965b";
    for (const f of w.food) ctx.fillRect(f.x, f.y, 3, 3);
    for (const o of w.organisms) {
      ctx.fillStyle =
        o.species === "predator" ? "#ed925a" : `hsl(${o.genome.hue},65%,65%)`;
      ctx.beginPath();
      ctx.arc(o.x, o.y, o.genome.size, 0, Math.PI * 2);
      ctx.fill();
    }
  }, [frame]);
  async function command(action: () => Promise<unknown>) {
    setBusy(true);
    setError("");
    try {
      await action();
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  const byId = new Map(records.map((r) => [r.id, r]));
  const current =
    selected.trim() === "" ? undefined : byId.get(Number(selected));
  const ancestry: RecordEntry[] = [];
  const seen = new Set<number>();
  let node = current;
  while (node && !seen.has(node.id)) {
    ancestry.unshift(node);
    seen.add(node.id);
    node = node.parent_id === null ? undefined : byId.get(node.parent_id);
  }
  const children = current
    ? records.filter((r) => r.parent_id === current.id)
    : [];
  return (
    <section className="history-view">
      <header>
        <small>03 / ARCHIVE</small>
        <h1>Evolutionary history</h1>
        <button disabled={busy} onClick={() => void command(refresh)}>
          Refresh records
        </button>
      </header>
      {error && <p role="alert">{error}</p>}
      <section>
        <h2>Lineage</h2>
        <p>
          {records.length} recorded organisms ·{" "}
          {records.filter((r) => r.death_tick === null).length} living
        </p>
        <label>
          Organism ID{" "}
          <input
            type="number"
            value={selected}
            onChange={(e) => setSelected(e.target.value)}
          />
        </label>
        <div className="history-tree">
          {ancestry.map((r) => (
            <button key={r.id} onClick={() => setSelected(String(r.id))}>
              #{r.id} · G{r.generation} ·{" "}
              {r.death_tick === null ? "alive" : "deceased"}
            </button>
          ))}
        </div>
        {current && (
          <>
            <p>
              Born at {current.birth_tick} · Death: {current.death_tick ?? "-"}{" "}
              · {current.species}
            </p>
            <dl className="history-genome">
              {Object.entries(current.genome).map(([k, v]) => (
                <div key={k}>
                  <dt>{k.replaceAll("_", " ")}</dt>
                  <dd>{v.toFixed(3)}</dd>
                </div>
              ))}
            </dl>
            <h3>Direct descendants</h3>
            <div className="history-children history-branches">
              {children.length === 0 && <p>No direct descendants recorded.</p>}
              {children.map((r) => (
                <button key={r.id} onClick={() => setSelected(String(r.id))}>
                  #{r.id}
                </button>
              ))}
            </div>
          </>
        )}
        {!current && <p>Enter an organism ID or choose a recent record.</p>}
        <h3>Recent records</h3>
        <div className="history-children">
          {records.slice(-20).map((r) => (
            <button key={r.id} onClick={() => setSelected(String(r.id))}>
              #{r.id}
            </button>
          ))}
        </div>
      </section>
      {browserMode ? (
        <section>
          <h2>Checkpoints and replay</h2>
          <p>
            Persistent checkpoints and replay require the local installation.
            Lineage records are available for the current browser ecosystem.
          </p>
        </section>
      ) : (
        <>
          <section>
            <h2>Checkpoints</h2>
            {checkpoints.length === 0 && (
              <p>No saved checkpoints. Save the current world to begin.</p>
            )}
            <input
              value={name}
              maxLength={100}
              onChange={(e) => setName(e.target.value)}
            />
            <button
              disabled={busy || !name.trim()}
              onClick={() =>
                void command(() => request("/checkpoints", "POST", { name }))
              }
            >
              Save checkpoint
            </button>
            {checkpoints.map((c) => (
              <div className="history-checkpoint" key={c.id}>
                <span>
                  {c.name} / tick {c.tick}
                </span>
                <button
                  disabled={busy}
                  onClick={() => {
                    if (
                      confirm(
                        "Replace the live simulation with this checkpoint?",
                      )
                    )
                      void command(() =>
                        request(`/checkpoints/${c.id}/restore`, "POST"),
                      );
                  }}
                >
                  Restore
                </button>
                <button
                  disabled={busy}
                  onClick={() =>
                    void command(async () => {
                      setPlaying(false);
                      setFrames(
                        await request<Frame[]>(
                          `/checkpoints/${c.id}/replay`,
                          "POST",
                          { ticks: 500, interval: 10 },
                        ),
                      );
                      setIndex(0);
                    })
                  }
                >
                  Replay 500 ticks
                </button>
                <button
                  disabled={busy}
                  onClick={() =>
                    void command(() =>
                      request(`/checkpoints/${c.id}`, "DELETE"),
                    )
                  }
                >
                  Delete
                </button>
              </div>
            ))}
          </section>
          <section>
            <h2>Replay</h2>
            <p>
              Independent reconstruction from a checkpoint; the live world is
              unchanged.
            </p>
            {frame && (
              <>
                <canvas ref={canvas} width={800} height={450} />
                <p>
                  Tick {frame.tick} / Population {frame.metrics.population}
                </p>
                <input
                  aria-label="Replay frame"
                  type="range"
                  min={0}
                  max={frames.length - 1}
                  value={index}
                  onChange={(e) => {
                    setPlaying(false);
                    setIndex(Number(e.target.value));
                  }}
                />
                <button
                  onClick={() => {
                    if (index === frames.length - 1) setIndex(0);
                    setPlaying(!playing);
                  }}
                >
                  {playing ? "Pause" : "Play"}
                </button>
              </>
            )}
          </section>
        </>
      )}
    </section>
  );
}
