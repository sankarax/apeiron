import init, { BrowserSimulation } from "../../wasm/apeiron_engine.js";
import type { SimulationConfig, SimulationSnapshot } from "../types/simulation";

let simulation: BrowserSimulation | undefined;
let config: SimulationConfig;
let running = false;
let target = 60;
let actual = 0;
let timer: ReturnType<typeof setTimeout> | undefined;

function snapshot(): SimulationSnapshot {
  if (!simulation) throw new Error("Engine not initialized");
  const data = JSON.parse(simulation.snapshot());
  config = data.config;
  return {
    type: "snapshot",
    world: data.world,
    metrics: data.metrics,
    habitat: config,
    status: {
      running,
      tick: data.metrics.tick,
      population: data.metrics.population,
      food_count: data.metrics.food_count,
      target_tps: target,
      actual_tps: actual,
      backend: "serial_cpu",
      active_backend: "serial_cpu",
    },
  };
}

function publish() {
  const value = snapshot();
  self.postMessage({ snapshot: value });
  return value.status;
}

function schedule() {
  clearTimeout(timer);
  if (!running) return;
  const ticks = Math.max(1, Math.floor(target / 20));
  const interval = (ticks / target) * 1000;
  const start = performance.now();
  try {
    simulation!.run(ticks);
    actual = Math.round(
      (ticks * 1000) / Math.max(interval, performance.now() - start),
    );
    publish();
    timer = setTimeout(
      schedule,
      Math.max(0, interval - (performance.now() - start)),
    );
  } catch (error) {
    running = false;
    actual = 0;
    self.postMessage({ fatal: String(error) });
  }
}

const ready = init().then(() => {
  simulation = new BrowserSimulation("{}");
  publish();
});

self.onmessage = async (event: MessageEvent) => {
  const { id, action, value } = event.data;
  try {
    await ready;
    let result: unknown;
    switch (action) {
      case "brain":
        result = JSON.parse(simulation!.get_brain(value));
        break;
      case "lineage":
        result = JSON.parse(simulation!.lineage());
        break;
      case "status":
        result = snapshot().status;
        break;
      case "config":
        result = config;
        break;
      case "start":
        if (!running) {
          running = true;
          schedule();
        }
        result = publish();
        break;
      case "pause":
        running = false;
        actual = 0;
        clearTimeout(timer);
        result = publish();
        break;
      case "step":
        if (running) throw new Error("Pause before stepping");
        simulation!.run(value);
        result = publish();
        break;
      case "speed":
        if (!Number.isInteger(value) || value < 1 || value > 500)
          throw new Error("Invalid speed");
        target = value;
        result = publish();
        break;
      case "reset": {
        const next = new BrowserSimulation(
          JSON.stringify({ ...config, ...value }),
        );
        running = false;
        actual = 0;
        clearTimeout(timer);
        simulation!.free();
        simulation = next;
        result = publish();
        break;
      }
      default:
        throw new Error("Unsupported browser command");
    }
    self.postMessage({ id, result });
  } catch (error) {
    self.postMessage({ id, error: String(error) });
  }
};
void ready.catch((error) => self.postMessage({ fatal: String(error) }));
