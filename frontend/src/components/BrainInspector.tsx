import { browserMode } from "../services/browserMode";
import { browserRequest } from "../services/browserClient";
import { useEffect, useState } from "react";
import { useSimulationStore } from "../store/simulationStore";
import type { ControllerKind } from "../types/simulation";
import "./BrainInspector.css";

type Layer = {
  name: string;
  weights: number[][];
  biases: number[];
  inputs: number[];
  outputs: number[];
};
type Brain = {
  id: number;
  tick: number;
  generation: number;
  controller: ControllerKind;
  parameters: number;
  inputs: number[];
  layers: Layer[];
  memory: { current: number[]; next: number[] } | null;
  action: {
    turn: number;
    speed: number;
    reproduce: boolean;
    aggression: number;
  } | null;
};
const smallLabels = [
  "Energy",
  "Age",
  "Food distance",
  "Food forward",
  "Food lateral",
  "Speed",
];
const deepLabels = [
  "Energy",
  "Age",
  "Speed",
  "Heading X",
  "Heading Y",
  "Food distance",
  "Food X",
  "Food Y",
  "Food detected",
  "Food density",
  "Organism density",
  "Organism distance",
  "Organism X",
  "Organism Y",
  "Threat",
  "Energy change",
];
const apiUrl = import.meta.env.VITE_API_URL || "http://127.0.0.1:8000";

function color(value: number) {
  return value >= 0 ? "#4c6946" : "#b04d32";
}

function Values({ values, labels }: { values: number[]; labels?: string[] }) {
  return (
    <div className="brain-values">
      {values.map((value, i) => (
        <div key={i} title={`${labels?.[i] ?? i}: ${value.toFixed(5)}`}>
          <span>{labels?.[i] ?? i}</span>
          <span style={{ color: color(value) }}>{value.toFixed(3)}</span>
        </div>
      ))}
    </div>
  );
}

function LayerView({ layer }: { layer: Layer }) {
  const y = (i: number, count: number) =>
    18 + (i * 240) / Math.max(1, count - 1);
  return (
    <div className="brain-layer">
      <h4>
        {layer.name} · {layer.inputs.length} → {layer.outputs.length}
      </h4>
      <svg
        viewBox="0 0 260 276"
        role="img"
        aria-label={`${layer.name} network weights`}
      >
        {layer.weights.flatMap((weights, j) =>
          weights.map((w, i) => (
            <line
              key={`${j}-${i}`}
              x1={28}
              y1={y(i, layer.inputs.length)}
              x2={232}
              y2={y(j, layer.outputs.length)}
              stroke={color(w)}
              strokeOpacity={Math.min(0.85, 0.08 + Math.abs(w) * 0.65)}
              strokeWidth={0.7}
            >
              <title>{`Weight ${i} → ${j}: ${w.toFixed(5)}`}</title>
            </line>
          )),
        )}
        {layer.inputs.map((v, i) => (
          <circle
            key={`i${i}`}
            cx={28}
            cy={y(i, layer.inputs.length)}
            r={3.5}
            fill={color(v)}
            opacity={0.35 + 0.65 * Math.min(1, Math.abs(v))}
          >
            <title>{`Input ${i}: ${v.toFixed(5)}`}</title>
          </circle>
        ))}
        {layer.outputs.map((v, i) => (
          <circle
            key={`o${i}`}
            cx={232}
            cy={y(i, layer.outputs.length)}
            r={4}
            fill={color(v)}
            opacity={0.35 + 0.65 * Math.min(1, Math.abs(v))}
          >
            <title>{`Output ${i}: ${v.toFixed(5)} | Bias: ${layer.biases[i].toFixed(5)}`}</title>
          </circle>
        ))}
      </svg>
      <details>
        <summary>Activations and biases</summary>
        <Values values={layer.outputs} />
        <Values values={layer.biases} />
      </details>
    </div>
  );
}

export default function BrainInspector() {
  const id = useSimulationStore((s) => s.selectedOrganismId);
  return <BrainTelemetry key={id} id={id} />;
}

function BrainTelemetry({ id }: { id: number | null }) {
  const connected = useSimulationStore((s) => s.connected);
  const [brain, setBrain] = useState<Brain | null>(null);
  const [status, setStatus] = useState("Waiting for organism data");

  useEffect(() => {
    if (id == null || !connected) return;
    const abort = new AbortController();
    let timer: ReturnType<typeof setTimeout> | undefined;

    async function refresh() {
      try {
        const data: Brain | null = browserMode
          ? await browserRequest<Brain | null>("brain", id)
          : await (async () => {
              const response = await fetch(
                `${apiUrl}/api/simulation/brain/${id}`,
                {
                  signal: abort.signal,
                  cache: "no-store",
                },
              );
              if (!response.ok) throw new Error(`HTTP ${response.status}`);
              return await response.json();
            })();
        if (abort.signal.aborted) return;

        if (data) {
          setBrain(data);
          setStatus("Live inference preview");
        } else {
          setStatus("Organism no longer present · last available state");
        }
      } catch (err) {
        if (!abort.signal.aborted) {
          setStatus(`Telemetry unavailable · retrying (${String(err)})`);
        }
      } finally {
        if (!abort.signal.aborted) timer = setTimeout(refresh, 500);
      }
    }

    void refresh();
    return () => {
      abort.abort();
      clearTimeout(timer);
    };
  }, [id, connected]);

  const selected = brain?.id === id ? brain : null;
  return (
    <section className="brain-inspector">
      <h2>Neural inspector</h2>
      <p>Read-only inference preview · up to 2 Hz</p>
      {id != null && (
        <p role="status">
          {connected ? status : "Disconnected · last available state"}
        </p>
      )}
      {id == null ? (
        <p>Select an organism to inspect its brain.</p>
      ) : !selected ? (
        <p>No brain snapshot available yet.</p>
      ) : (
        <>
          <h3>
            #{selected.id} · {selected.controller} ·{" "}
            {selected.parameters.toLocaleString()} parameters
          </h3>
          <p>
            Generation {selected.generation} · Tick {selected.tick}
          </p>
          {selected.controller === "heuristic" ? (
            <p>This organism uses heuristic behaviour, not a neural network.</p>
          ) : (
            <>
              <h4>Sensory inputs</h4>
              <Values
                values={selected.inputs}
                labels={
                  selected.controller === "small" ? smallLabels : deepLabels
                }
              />
              <div className="brain-layers">
                {selected.layers.map((layer) => (
                  <LayerView key={layer.name} layer={layer} />
                ))}
              </div>
              {selected.memory && (
                <>
                  <h4>Current memory</h4>
                  <Values values={selected.memory.current} />
                  <h4>Predicted next memory</h4>
                  <Values values={selected.memory.next} />
                </>
              )}
              {selected.action && (
                <>
                  <h4>Predicted action</h4>
                  <p>
                    Turn {selected.action.turn.toFixed(3)} rad · Speed{" "}
                    {(selected.action.speed * 100).toFixed(1)}% · Reproduce{" "}
                    {selected.action.reproduce ? "yes" : "no"} · Aggression{" "}
                    {selected.action.aggression.toFixed(3)}
                  </p>
                </>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
