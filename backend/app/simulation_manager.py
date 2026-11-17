
import asyncio
import json
import time

from apeiron_engine import Simulation, default_config


class SimulationManager:
    def __init__(self):
        self.config = json.loads(default_config())
        self.simulation = Simulation(json.dumps(self.config))

        self.running = False
        self.ticks_per_second = 60
        self.actual_tps = 0.0

        self.lock = asyncio.Lock()
        self.task: asyncio.Task | None = None

    async def startup(self):
        self.task = asyncio.create_task(self._simulation_loop())

    async def shutdown(self):
        if self.task is not None:
            self.task.cancel()

            try:
                await self.task
            except asyncio.CancelledError:
                pass

            self.task = None
    async def set_backend(self, backend: str):
        async with self.lock:
            await asyncio.to_thread(self.simulation.set_backend, backend)
            return self._status()
    async def _simulation_loop(self):
        while True:
            start = time.perf_counter()

            async with self.lock:
                if self.running:
                    ticks = max(1, self.ticks_per_second // 30)

                    await asyncio.to_thread(
                        self.simulation.run,
                        ticks,
                    )

                    interval = ticks / self.ticks_per_second
                    elapsed = time.perf_counter() - start

                    self.actual_tps = ticks / max(
                        elapsed,
                        interval,
                    )
                else:
                    interval = 1 / 30
                    self.actual_tps = 0.0

            elapsed = time.perf_counter() - start

            await asyncio.sleep(
                max(0.0, interval - elapsed)
            )

    def _status(self):
        return {
            "running": self.running,
            "tick": self.simulation.tick,
            "population": self.simulation.population,
            "food_count": self.simulation.food_count,
            "target_tps": self.ticks_per_second,
            "actual_tps": round(self.actual_tps, 2),
            "backend": self.simulation.get_backend(),
            "active_backend": self.simulation.get_active_backend(),
        }

    async def start(self):
        async with self.lock:
            self.running = True
            return self._status()

    async def pause(self):
        async with self.lock:
            self.running = False
            self.actual_tps = 0.0
            return self._status()

    async def set_speed(self, ticks_per_second: int):
        async with self.lock:
            self.ticks_per_second = ticks_per_second
            return self._status()

    async def step(self, ticks: int = 1):
        async with self.lock:
            if self.running:
                raise ValueError(
                    "Pause the simulation before stepping manually"
                )

            await asyncio.to_thread(
                self.simulation.run,
                ticks,
            )

            return self._status()

    async def reset(self, config: dict | None = None):
        async with self.lock:
            new_config = self.config.copy()

            if config is not None:
                unknown = config.keys() - new_config.keys()

                if unknown:
                    raise ValueError(
                        f"Unknown configuration fields: {unknown}"
                    )

                new_config.update(config)

            backend = self.simulation.get_backend()
            if backend == "gpu" and new_config.get("controller") == "heuristic":
                backend = "auto_cpu"
            new_simulation = await asyncio.to_thread(
                Simulation, json.dumps(new_config), backend
            )

            self.running = False
            self.actual_tps = 0.0
            self.config = new_config
            self.simulation = new_simulation

            return self._status()

    async def get_status(self):
        async with self.lock:
            return self._status()

    async def get_config(self):
        async with self.lock:
            return self.config.copy()

    async def get_metrics(self):
        async with self.lock:
            return json.loads(
                self.simulation.get_metrics()
            )

    async def get_state(self):
        async with self.lock:
            return json.loads(
                self.simulation.get_state()
            )

    async def get_brain(self, organism_id: int):
        async with self.lock:
            return json.loads(self.simulation.get_brain(organism_id))

    async def get_snapshot(self):
        async with self.lock:
            return {
                "type": "snapshot",
                "habitat": {key: self.config[key] for key in ("predation", "resource_zones", "hazard_zones")},
                "status": self._status(),
                "metrics": json.loads(
                    self.simulation.get_metrics()
                ),
                "world": json.loads(
                    self.simulation.get_state()
                ),
            }
    
