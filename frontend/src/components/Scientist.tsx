import { browserMode } from "../services/browserMode";
import { useState, useEffect } from "react";
import "./Scientist.css";

type Sample = {
  tick: number;
  population: number;
  predators: number;
  herbivores: number;
};
type Run = Sample & { seed: number; history: Sample[] };
type Experiment = {
  id: string;
  name: string;
  config: Record<string, unknown>;
  ticks: number;
  results: Run[];
};
type Summary = {
  name: string;
  seeds: number[];
  mean_population: number;
  mean_predators: number;
  coexistence_count: number;
  mean_metabolism: number;
};
type Archive = { id: string; name: string; created_at: string };
type Result = {
  archive_id?: string;
  archive_error?: string;
  request?: unknown;
  model?: string;
  report: string;
  complete: boolean;
  summary: Summary[];
  experiments: Experiment[];
  trace: unknown[];
};
const base = import.meta.env.VITE_API_URL || "http://127.0.0.1:8000";
const initialQuestion =
  "Investigate whether reducing predator metabolism improves predator-prey coexistence. Compare final populations and survival trajectories.";

function Trajectory({ run }: { run: Run }) {
  const history = run.history;
  const max = Math.max(1, ...history.map((p) => p.population));
  const end = Math.max(1, ...history.map((p) => p.tick));
  const line = (key: "predators" | "herbivores") =>
    history
      .map((p) => `${30 + (p.tick / end) * 260},${110 - (p[key] / max) * 90}`)
      .join(" ");
  return (
    <figure className="scientist-figure">
      <figcaption>
        Seed {run.seed} · final {run.population} organisms
      </figcaption>
      <svg
        viewBox="0 0 310 140"
        role="img"
        aria-label={`Population trajectories for seed ${run.seed}`}
      >
        <path
          d="M30 20 V110 H290"
          fill="none"
          stroke="currentColor"
          opacity=".35"
        />
        <text x="26" y="23" textAnchor="end">
          {max}
        </text>
        <text x="26" y="113" textAnchor="end">
          0
        </text>
        <text x="30" y="128">
          0
        </text>
        <text x="290" y="128" textAnchor="end">
          {end} ticks
        </text>
        <polyline
          points={line("herbivores")}
          fill="none"
          stroke="#4c6946"
          strokeWidth="2"
        />
        <polyline
          points={line("predators")}
          fill="none"
          stroke="#b04d32"
          strokeWidth="2"
        />
      </svg>
    </figure>
  );
}

export default function Scientist() {
  const [question, setQuestion] = useState(initialQuestion);
  const [controlled, setControlled] = useState(true);
  const [config, setConfig] = useState(
    '{"predation":true,"controller":"recurrent","initial_population":1000}',
  );
  const [factor, setFactor] = useState("predator_metabolism_scale");
  const [values, setValues] = useState("1.0, 0.4");
  const [seeds, setSeeds] = useState("42, 123, 456");
  const [ticks, setTicks] = useState(5000);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState<Result | null>(null);
  const [archive, setArchive] = useState<Archive[]>([]);
  const [archiveError, setArchiveError] = useState("");
  const [archiveBusy, setArchiveBusy] = useState(false);
  async function refreshArchive() {
    setArchiveError("");
    try {
      const response = await fetch(`${base}/api/archive/research`);
      if (!response.ok) throw new Error("Could not load research archive");
      setArchive(await response.json());
    } catch (err) {
      setArchiveError(String(err));
    }
  }
  useEffect(() => {
    const abort = new AbortController();
    if (!browserMode) {
      fetch(`${base}/api/archive/research`, { signal: abort.signal })
        .then((response) => {
          if (!response.ok) throw new Error("Could not load research archive");
          return response.json();
        })
        .then((data) => {
          if (!abort.signal.aborted) setArchive(data);
        })
        .catch((err) => {
          if (!abort.signal.aborted) setArchiveError(String(err));
        });
      return () => abort.abort();
    }
    fetch(`${import.meta.env.BASE_URL}research-demo.json`, {
      signal: abort.signal,
    })
      .then((response) => {
        if (!response.ok) throw new Error("Recorded research unavailable");
        return response.json();
      })
      .then((data) => {
        if (!abort.signal.aborted) setResult(data);
      })
      .catch((err) => {
        if (!abort.signal.aborted) setError(String(err));
      });
    return () => abort.abort();
  }, []);
  async function openArchive(id: string) {
    setArchiveBusy(true);
    setArchiveError("");
    try {
      const response = await fetch(`${base}/api/archive/research/${id}`);
      if (!response.ok) throw new Error("Could not open research session");
      setResult({ ...(await response.json()), archive_id: id });
    } catch (err) {
      setArchiveError(String(err));
    } finally {
      setArchiveBusy(false);
    }
  }
  async function deleteArchive(id: string) {
    if (!window.confirm("Permanently delete this research session?")) return;
    setArchiveBusy(true);
    setArchiveError("");
    try {
      const response = await fetch(`${base}/api/archive/research/${id}`, {
        method: "DELETE",
      });
      if (!response.ok) throw new Error("Could not delete research session");
      if (result?.archive_id === id) setResult(null);
      await refreshArchive();
    } catch (err) {
      setArchiveError(String(err));
    } finally {
      setArchiveBusy(false);
    }
  }
  async function research() {
    setBusy(true);
    setError("");
    try {
      const body = controlled
        ? {
            question,
            fixed_config: JSON.parse(config),
            factor,
            values: values.split(",").map(Number),
            seeds: seeds.split(",").map(Number),
            ticks,
          }
        : { question };
      const response = await fetch(`${base}/api/scientist/research`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      const data = await response.json();
      if (!response.ok)
        throw new Error(
          typeof data.detail === "string"
            ? data.detail
            : JSON.stringify(data.detail),
        );
      setResult(data);
      await refreshArchive();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }
  function download() {
    const url = URL.createObjectURL(
      new Blob([JSON.stringify(result, null, 2)], { type: "application/json" }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = "apeiron-research.json";
    a.click();
    URL.revokeObjectURL(url);
  }
  return (
    <section className="scientist">
      <header>
        <span className="scientist-kicker">APEIRON / RESEARCH</span>
        <h1>Research results</h1>
        <p>
          {browserMode
            ? "Recorded Qwen3 research · six independent CPU experiments"
            : "Local Qwen3 · independent CPU experiments · measured evidence"}
        </p>
      </header>
      {browserMode ? (
        <section>
          <h2>Recorded research session</h2>
          <p>{initialQuestion}</p>
          <p>
            Two treatments · three seeds each · 5,000 ticks per run. Live
            research requires the local Ollama installation.
          </p>
          <button disabled>Conduct research</button>
        </section>
      ) : (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void research();
          }}
        >
          <fieldset disabled={busy || archiveBusy}>
            <label className="scientist-question">
              Research question
              <textarea
                required
                maxLength={2000}
                value={question}
                onChange={(e) => setQuestion(e.target.value)}
              />
            </label>
            <label className="scientist-check">
              <input
                type="checkbox"
                checked={controlled}
                onChange={(e) => setControlled(e.target.checked)}
              />{" "}
              Enforce controlled experiment
            </label>
            {controlled && (
              <>
                <label className="scientist-config">
                  Fixed configuration (JSON)
                  <textarea
                    value={config}
                    onChange={(e) => setConfig(e.target.value)}
                  />
                </label>
                <div className="scientist-fields">
                  <label>
                    Factor
                    <input
                      required
                      value={factor}
                      onChange={(e) => setFactor(e.target.value)}
                    />
                  </label>
                  <label>
                    Treatment values
                    <input
                      required
                      value={values}
                      onChange={(e) => setValues(e.target.value)}
                    />
                  </label>
                  <label>
                    Seeds
                    <input
                      required
                      value={seeds}
                      onChange={(e) => setSeeds(e.target.value)}
                    />
                  </label>
                  <label>
                    Ticks
                    <input
                      type="number"
                      min={1}
                      max={5000}
                      required
                      value={ticks}
                      onChange={(e) => setTicks(Number(e.target.value))}
                    />
                  </label>
                </div>
              </>
            )}
            <button type="submit" disabled={busy}>
              {busy ? "Research in progress…" : "Conduct research"}
            </button>
          </fieldset>
        </form>
      )}
      {busy && (
        <p role="status">
          The local model is planning or executing experiments. This may take
          several minutes. Previous results remain visible.
        </p>
      )}
      {error && <p role="alert">{error}</p>}
      {!browserMode && (
        <section className="scientist-archive">
          <div className="scientist-heading">
            <h2>Research archive</h2>
            <button
              disabled={busy || archiveBusy}
              onClick={() => void refreshArchive()}
            >
              Refresh
            </button>
          </div>
          {archiveError && <p role="alert">{archiveError}</p>}
          {archive.length === 0 && !archiveError && <p>No saved sessions.</p>}
          {archive.map((entry) => (
            <div className="scientist-archive-entry" key={entry.id}>
              <div>
                <strong>{entry.name}</strong>
                <p>
                  {new Date(entry.created_at).toLocaleString()} ·{" "}
                  {entry.id.slice(0, 8)}
                  {result?.archive_id === entry.id ? " · Open" : ""}
                </p>
              </div>
              <div className="scientist-archive-actions">
                <button
                  disabled={busy || archiveBusy}
                  onClick={() => void openArchive(entry.id)}
                >
                  Open
                </button>
                <button
                  disabled={busy || archiveBusy}
                  onClick={() => void deleteArchive(entry.id)}
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </section>
      )}
      {result && (
        <>
          {result.archive_error && <p role="alert">{result.archive_error}</p>}
          {result.archive_id && (
            <p className="scientist-kicker">
              ARCHIVED SESSION / {result.archive_id}
            </p>
          )}
          <div className="scientist-heading">
            <h2>Measured evidence</h2>
            <button onClick={download}>Export JSON</button>
          </div>
          <p>
            {result.complete
              ? "Required execution complete"
              : "Execution incomplete"}{" "}
            · {result.experiments.length} completed treatments
          </p>
          <div className="scientist-table">
            <table>
              <thead>
                <tr>
                  <th>Treatment</th>
                  <th>Mean population</th>
                  <th>Mean predators</th>
                  <th>Mean herbivores</th>
                  <th>Coexistence</th>
                </tr>
              </thead>
              <tbody>
                {result.summary.map((s, i) => (
                  <tr key={i}>
                    <td>{s.name}</td>
                    <td>{s.mean_population.toFixed(2)}</td>
                    <td>{s.mean_predators.toFixed(2)}</td>
                    <td>{(s.mean_population - s.mean_predators).toFixed(2)}</td>
                    <td>
                      {s.coexistence_count}/{s.seeds.length}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="scientist-legend">
            {" "}
            <span>● Herbivores</span> <span>● Predators</span> · independent
            axis scales
          </p>
          {result.experiments.map((exp) => (
            <section key={exp.id}>
              <h3>{exp.name}</h3>
              <div className="scientist-plots">
                {exp.results.map((run) => (
                  <Trajectory key={run.seed} run={run} />
                ))}
              </div>
              <details>
                <summary>Configuration and raw measurements</summary>
                <pre>{JSON.stringify(exp, null, 2)}</pre>
              </details>
            </section>
          ))}
          <h2>AI interpretation</h2>
          <p>
            Model-generated interpretation may contain errors. Endpoint
            coexistence is not proof of stability; no significance test is
            performed.
          </p>
          <article className="scientist-report">{result.report}</article>
          <details>
            <summary>Tool execution trace ({result.trace.length})</summary>
            <pre>{JSON.stringify(result.trace, null, 2)}</pre>
          </details>
        </>
      )}
    </section>
  );
}
