import { browserMode } from "./services/browserMode";

import { lazy, Suspense, useState } from "react";
import Home from "./components/Home";
import EvolutionCharts from "./components/EvolutionCharts";
import BrainInspector from "./components/BrainInspector";
import OrganismInspector from "./components/OrganismInspector";
import { useThrottledSimulation } from "./hooks/useThrottledSimulation";
import PerformanceMonitor from "./components/PerformanceMonitor";
import EnvironmentConfig from "./components/EnvironmentConfig";
const Multiverse = lazy(() => import("./components/Multiverse"));
const History = lazy(() => import("./components/History"));
const Scientist = lazy(() => import("./components/Scientist"));

const EcosystemViewport = lazy(() => import("./components/EcosystemViewport"));
import { useSimulationSocket } from "./hooks/useSimulationSocket";
import { useSimulationStore } from "./store/simulationStore";
import { api } from "./services/api";

import { BackendSelector } from "./components/BackendSelector";

import "./App.css";
import "./Workspace.css";

function Workspace({ goHome }: { goHome: () => void }) {
  useSimulationSocket();

  const snapshot = useThrottledSimulation((state) => state.snapshot, 100);

  const connected = useSimulationStore((state) => state.connected);

  const [error, setError] = useState("");

  const selectedId = useSimulationStore((state) => state.selectedOrganismId);
  const status = snapshot?.status;
  const metrics = snapshot?.metrics;
  const [page, setPage] = useState("ecosystem");
  const nav = (
    <header className="header">
      <button className="brand" onClick={goHome}>
        Apeiron<span>.</span>
      </button>
      <nav aria-label="Research navigation">
        {[
          ["ecosystem", "Ecosystem"],
          ["multiverse", "Multiverse"],
          ["history", "History"],
          ["scientist", "Research"],
        ].map(([key, label]) => (
          <button
            key={key}
            aria-current={page === key ? "page" : undefined}
            onClick={() => setPage(key)}
          >
            {label}
          </button>
        ))}
      </nav>
      <span className={connected ? "online" : "offline"}>
        ●{" "}
        {connected
          ? browserMode
            ? "BROWSER CPU"
            : "CONNECTED"
          : "DISCONNECTED"}
      </span>
    </header>
  );

  async function command(action: () => Promise<unknown>) {
    try {
      setError("");
      await action();
    } catch (error) {
      setError(String(error));
    }
  }

  if (page !== "ecosystem")
    return (
      <main className="app">
        {nav}
        <Suspense fallback={<p>Loading page…</p>}>
          {page === "history" ? (
            <History />
          ) : page === "multiverse" ? (
            <Multiverse />
          ) : (
            <Scientist />
          )}
        </Suspense>
      </main>
    );

  return (
    <main className="app">
      {nav}
      <div className="workspace-heading">
        <span className="eyebrow">01 / LIVE SIMULATION</span>
        <h1>Ecosystem</h1>
        <p>Live simulation and population statistics.</p>
      </div>
      <section className="metrics">
        <div className="metric">
          <span>POPULATION</span>
          <strong>{status?.population ?? 0}</strong>
        </div>

        <div className="metric">
          <span>SIMULATION TICK</span>
          <strong>{status?.tick ?? 0}</strong>
        </div>

        <div className="metric">
          <span>MAX GENERATION</span>
          <strong>{metrics?.max_generation ?? 0}</strong>
        </div>

        <div className="metric">
          <span>TICKS / SEC</span>
          <strong>{status?.actual_tps ?? 0}</strong>
        </div>
      </section>

      <section className="workspace">
        <div className="viewport-panel">
          <Suspense fallback={<p>Loading simulation…</p>}>
            <EcosystemViewport />
          </Suspense>
        </div>

        <aside className="sidebar">
          <h2>Simulation</h2>

          <button
            disabled={!connected}
            onClick={() => command(status?.running ? api.pause : api.start)}
          >
            {status?.running ? "Pause" : "Start"}
          </button>

          <button
            disabled={!connected || status?.running}
            onClick={() => command(() => api.step(1))}
          >
            Step
          </button>

          <button
            disabled={!connected}
            onClick={() => command(() => api.reset())}
          >
            Reset
          </button>

          <label>
            Simulation speed: {status?.target_tps ?? 60} TPS
            <input
              type="range"
              min="1"
              max="500"
              value={status?.target_tps ?? 60}
              disabled={!connected}
              onChange={(event) =>
                command(() => api.setSpeed(Number(event.target.value)))
              }
            />
          </label>

          <hr />

          <h2>Evolution</h2>

          <p>Births: {metrics?.total_births ?? 0}</p>
          <p>Deaths: {metrics?.total_deaths ?? 0}</p>
          <p>Avg speed: {metrics?.average_speed.toFixed(3) ?? "-"}</p>
          <p>Avg energy: {metrics?.average_energy.toFixed(2) ?? "-"}</p>

          {error && <p className="error">{error}</p>}
        </aside>
      </section>
      <section
        className="specimen-workspace"
        aria-label="Individual observation"
      >
        <OrganismInspector />
        {selectedId !== null && <BrainInspector />}
      </section>
      <EnvironmentConfig />
      <EvolutionCharts />
      {!browserMode && (
        <BackendSelector
          backend={snapshot?.status.backend ?? "auto_cpu"}
          activeBackend={snapshot?.status.active_backend ?? "serial_cpu"}
        />
      )}
      <PerformanceMonitor />
    </main>
  );
}

export default function App() {
  const [entered, setEntered] = useState(false);
  return entered ? (
    <Workspace goHome={() => setEntered(false)} />
  ) : (
    <Home enter={() => setEntered(true)} />
  );
}
