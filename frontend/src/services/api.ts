import { browserMode } from "./browserMode";
import { browserRequest } from "./browserClient";

import type {
  SimulationConfig,
  SimulationStatus,
  ComputeBackend,
} from "../types/simulation";

const API_URL = import.meta.env.VITE_API_URL ?? "http://127.0.0.1:8000";

export const WS_URL =
  import.meta.env.VITE_WS_URL ?? "ws://127.0.0.1:8000/ws/simulation";

async function request<T>(
  endpoint: string,
  method = "GET",
  body?: unknown,
): Promise<T> {
  if (browserMode) {
    const action = endpoint.split("/").pop()!;
    const args = body as
      | {
          ticks?: number;
          ticks_per_second?: number;
          config?: Partial<SimulationConfig>;
        }
      | undefined;
    if (action === "reset") {
      const store = (
        await import("../store/simulationStore")
      ).useSimulationStore.getState();
      store.clearHistory();
      store.selectOrganism(null);
    }
    return browserRequest<T>(
      action,
      action === "step"
        ? args?.ticks
        : action === "speed"
          ? args?.ticks_per_second
          : args?.config,
    );
  }
  const response = await fetch(`${API_URL}${endpoint}`, {
    method,
    headers: {
      "Content-Type": "application/json",
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => null);

    throw new Error(
      typeof error?.detail === "string"
        ? error.detail
        : `Request failed: ${response.status}`,
    );
  }

  return response.json() as Promise<T>;
}

export const api = {
  getStatus: () => request<SimulationStatus>("/api/simulation/status"),

  getConfig: () => request<SimulationConfig>("/api/simulation/config"),

  start: () => request<SimulationStatus>("/api/simulation/start", "POST"),

  pause: () => request<SimulationStatus>("/api/simulation/pause", "POST"),

  step: (ticks = 1) =>
    request<SimulationStatus>("/api/simulation/step", "POST", { ticks }),

  setSpeed: (ticks_per_second: number) =>
    request<SimulationStatus>("/api/simulation/speed", "POST", {
      ticks_per_second,
    }),

  reset: (config?: Partial<SimulationConfig>) =>
    request<SimulationStatus>("/api/simulation/reset", "POST", { config }),

  setBackend: (backend: ComputeBackend) =>
    request<SimulationStatus>("/simulation/backend", "POST", {
      backend,
    }),
};
