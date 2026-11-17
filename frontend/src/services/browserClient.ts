import { useSimulationStore } from "../store/simulationStore";

let worker: Worker | undefined;
let nextId = 0;
const pending = new Map<
  number,
  { resolve: (value: unknown) => void; reject: (error: Error) => void }
>();

function fail(message: string) {
  for (const request of pending.values()) request.reject(new Error(message));
  pending.clear();
  useSimulationStore.getState().setConnected(false);
}

export function connectBrowser() {
  worker = new Worker(new URL("./browser.worker.ts", import.meta.url), {
    type: "module",
  });
  worker.onmessage = ({ data }) => {
    if (data.fatal) {
      fail(data.fatal);
      console.error(data.fatal);
      return;
    }
    if (data.snapshot) {
      useSimulationStore.getState().setSnapshot(data.snapshot);
      useSimulationStore.getState().setConnected(true);
    }
    const request = pending.get(data.id);
    if (!request) return;
    pending.delete(data.id);
    if (data.error) request.reject(new Error(data.error));
    else request.resolve(data.result);
  };
  worker.onerror = (event) => fail(event.message || "Browser engine failed");
  return () => {
    worker?.terminate();
    worker = undefined;
    fail("Browser engine stopped");
  };
}

export function browserRequest<T>(action: string, value?: unknown): Promise<T> {
  return new Promise((resolve, reject) => {
    if (!worker) {
      reject(new Error("Browser engine unavailable"));
      return;
    }
    const id = ++nextId;
    pending.set(id, { resolve: (result) => resolve(result as T), reject });
    worker.postMessage({ id, action, value });
  });
}
