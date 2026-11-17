import type { SimulationConfig, SimulationSnapshot } from "../types/simulation";

type Pending = {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
};

class World {
  readonly worker = new Worker(
    new URL("./browser.worker.ts", import.meta.url),
    { type: "module" },
  );
  private pending = new Map<number, Pending>();
  private nextId = 0;
  private failure: Error | undefined;
  latest?: SimulationSnapshot;

  readonly id: string;
  readonly name: string;
  readonly config: Partial<SimulationConfig>;

  constructor(id: string, name: string, config: Partial<SimulationConfig>) {
    this.id = id;
    this.name = name;
    this.config = config;
    this.worker.onmessage = ({ data }) => {
      if (data.fatal) {
        this.close(String(data.fatal));
        return;
      }
      if (data.snapshot) this.latest = data.snapshot;
      const pending = this.pending.get(data.id);
      if (!pending) return;
      this.pending.delete(data.id);
      if (data.error) pending.reject(new Error(data.error));
      else pending.resolve(data.result);
    };
    this.worker.onerror = (event) =>
      this.close(event.message || "World worker failed");
  }

  call<T>(action: string, value?: unknown): Promise<T> {
    return new Promise((resolve, reject) => {
      if (this.failure) {
        reject(this.failure);
        return;
      }
      const id = ++this.nextId;
      this.pending.set(id, {
        resolve: (result) => resolve(result as T),
        reject,
      });
      this.worker.postMessage({ id, action, value });
    });
  }

  describe() {
    if (!this.latest) throw new Error("World not initialized");
    return {
      id: this.id,
      name: this.name,
      config: this.latest.habitat ?? this.config,
      status: this.latest.status,
      metrics: this.latest.metrics,
    };
  }

  close(message = "World closed") {
    this.failure = new Error(message);
    this.worker.terminate();
    for (const pending of this.pending.values()) pending.reject(this.failure);
    this.pending.clear();
  }
}

const worlds = new Map<string, World>();
let nextWorld = 0;

export function disposeBrowserWorlds() {
  for (const world of worlds.values()) world.close();
  worlds.clear();
}

async function dispatch(
  path: string,
  method: string,
  body?: unknown,
): Promise<unknown> {
  if (path === "/worlds" && method === "GET") {
    return [...worlds.values()]
      .filter((world) => world.latest)
      .map((world) => world.describe());
  }
  if (path === "/worlds" && method === "POST") {
    if (worlds.size >= 4) throw new Error("Maximum of four browser worlds");
    const { name, config } = body as {
      name: string;
      config: Partial<SimulationConfig>;
    };
    const id = String(++nextWorld);
    const world = new World(id, name, config);
    worlds.set(id, world);
    try {
      await world.call("reset", config);
      return world.describe();
    } catch (error) {
      world.close();
      worlds.delete(id);
      throw error;
    }
  }
  if (path === "/run" && method === "POST") {
    const { ticks } = body as { ticks: number };
    if (!Number.isInteger(ticks) || ticks < 1 || ticks > 10000)
      throw new Error("Invalid batch size");
    const entries = [...worlds.values()];
    return Promise.all(
      entries.map(async (world) => {
        try {
          await world.call("pause");
          for (let remaining = ticks; remaining > 0; remaining -= 1000) {
            await world.call("step", Math.min(remaining, 1000));
          }
          return world.describe();
        } catch (error) {
          return { id: world.id, name: world.name, error: String(error) };
        }
      }),
    );
  }
  const match = /^\/worlds\/([^/]+)(?:\/(snapshot|start|pause|reset))?$/.exec(
    path,
  );
  if (!match) throw new Error("Unknown multiverse command");
  const world = worlds.get(match[1]);
  if (!world) throw new Error("World not found");
  if (method === "DELETE") {
    world.close();
    worlds.delete(world.id);
    return { deleted: world.id };
  }
  if (match[2] === "snapshot") return world.latest;
  if (method === "POST" && match[2]) return world.call(match[2]);
  throw new Error("Unsupported multiverse command");
}

export async function browserMultiverseRequest<T>(
  path: string,
  method: string,
  body?: unknown,
): Promise<T> {
  return (await dispatch(path, method, body)) as T;
}
