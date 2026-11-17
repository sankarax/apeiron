import { memo } from "react";

import { usePerformanceStore } from "../store/performanceStore";

import { useThrottledSimulation } from "../hooks/useThrottledSimulation";

import "./PerformanceMonitor.css";
import type { ComputeBackend } from "../types/simulation";

interface MetricProps {
  label: string;
  value: string;
  unit: string;
}

function Metric({ label, value, unit }: MetricProps) {
  return (
    <div className="telemetry-metric">
      <span className="telemetry-label">{label}</span>

      <strong>{value}</strong>

      <span className="telemetry-unit">{unit}</span>
    </div>
  );
}

function FrameTimeGraph({ history }: { history: number[] }) {
  const width = 600;
  const height = 100;

  const maximum = Math.max(33.33, ...history);

  const points = history
    .map((value, index) => {
      const x = (index / Math.max(1, history.length - 1)) * width;

      const y = height - (value / maximum) * height;

      return `${x},${y}`;
    })
    .join(" ");

  const budgetY = height - (16.67 / maximum) * height;

  return (
    <div className="frame-time-graph">
      <div className="frame-graph-heading">
        <span>FRAME INTERVAL / RECENT SAMPLES</span>

        <span>60 Hz BUDGET: 16.67 ms</span>
      </div>

      <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none">
        <line
          x1={0}
          y1={budgetY}
          x2={width}
          y2={budgetY}
          stroke="#b04d32"
          strokeDasharray="5 5"
          strokeWidth={1}
        />

        <polyline
          points={points}
          fill="none"
          stroke="#4c6946"
          strokeWidth={1.5}
          vectorEffect="non-scaling-stroke"
        />
      </svg>

      <div className="frame-graph-footer">
        <span>OLDER</span>

        <span>NOW</span>
      </div>
    </div>
  );
}

function PerformanceMonitor() {
  const performance = usePerformanceStore();

  const status = useThrottledSimulation(
    (state) => state.snapshot?.status,
    1000,
  );

  const backendNames: Record<ComputeBackend, string> = {
    brute_force: "Brute Force",
    serial_cpu: "Serial CPU",
    parallel_cpu: "Parallel CPU",
    auto_cpu: "Automatic CPU",
    gpu: "GPU (WebGPU)",
  };

  const backend = status ? backendNames[status.active_backend] : "UNKNOWN";

  return (
    <section className="performance-monitor">
      <header className="telemetry-header">
        <div>
          <span className="telemetry-index">04 / SYSTEM TELEMETRY</span>

          <h2>Engine performance</h2>
        </div>

        <span className="telemetry-live">● LIVE / 1 Hz</span>
      </header>

      <div className="telemetry-grid">
        <Metric
          label="Render FPS"
          value={performance.fps.toFixed(1)}
          unit="frames/s"
        />

        <Metric
          label="Frame interval"
          value={performance.frameTime.toFixed(2)}
          unit="ms"
        />

        <Metric
          label="P95 frame interval"
          value={performance.p95FrameTime.toFixed(2)}
          unit="ms"
        />

        <Metric
          label="Simulation throughput"
          value={(status?.actual_tps ?? 0).toFixed(1)}
          unit="ticks/s"
        />

        <Metric
          label="Snapshot rate"
          value={performance.snapshotRate.toFixed(1)}
          unit="Hz"
        />

        <Metric
          label="Draw calls"
          value={String(performance.drawCalls)}
          unit="per rendered frame"
        />

        <Metric
          label="Triangles"
          value={performance.triangles.toLocaleString()}
          unit="per rendered frame"
        />

        <Metric
          label="Peak frame interval"
          value={performance.maxFrameTime.toFixed(2)}
          unit="ms"
        />
      </div>

      <FrameTimeGraph history={performance.frameHistory} />

      <footer className="telemetry-footer">
        <span>BACKEND / {backend}</span>

        <span>RENDERER / WEBGL</span>

        <span>POPULATION / {status?.population ?? 0}</span>
      </footer>
    </section>
  );
}

export default memo(PerformanceMonitor);
