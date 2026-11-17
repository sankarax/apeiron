import asyncio
import json
from app.persistence import save_record, list_records, get_record, delete_record

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field


class SaveRequest(BaseModel):
    name: str = Field(min_length=1, max_length=100)


class ReplayRequest(BaseModel):
    ticks: int = Field(default=100, ge=1, le=1000)
    interval: int = Field(default=10, ge=1, le=1000)


def history_router(manager):
    router = APIRouter(prefix='/api/history')
    @router.get('/lineage')
    async def lineage():
        async with manager.lock:
            return json.loads(await asyncio.to_thread(manager.simulation.get_lineage))

    @router.get('/checkpoints')
    async def list_checkpoints():
        return await asyncio.to_thread(list_records, 'checkpoint')

    @router.post('/checkpoints')
    async def save(request: SaveRequest):
        async with manager.lock:
            data = await asyncio.to_thread(manager.simulation.checkpoint)
            if len(data.encode()) > 128 * 1024 * 1024:
                raise HTTPException(413, 'Checkpoint exceeds 128 MiB')
            tick = manager.simulation.tick
        return await asyncio.to_thread(save_record, 'checkpoint', request.name, data, tick)

    @router.delete('/checkpoints/{key}')
    async def delete(key: str):
        return await asyncio.to_thread(delete_record, 'checkpoint', key)

    @router.post('/checkpoints/{key}/restore')
    async def restore(key: str):
        _, data = await asyncio.to_thread(get_record, 'checkpoint', key)
        async with manager.lock:
            await asyncio.to_thread(manager.simulation.restore_checkpoint, data)
            manager.config = json.loads(manager.simulation.get_config())
            manager.running = False
            manager.actual_tps = 0.0
            return manager._status()

    @router.post('/checkpoints/{key}/replay')
    async def replay(key: str, request: ReplayRequest):
        from apeiron_engine import Simulation
        _, data = await asyncio.to_thread(get_record, 'checkpoint', key)
        if (request.ticks + request.interval - 1) // request.interval > 100:
            raise HTTPException(400, 'Limit replay to 100 intervals')

        def run():
            sim = Simulation()
            sim.restore_checkpoint(data)
            if sim.population > 5000:
                raise ValueError('Replay preview limited to 5000 initial organisms')
            frames = []
            def capture():
                world = json.loads(sim.get_state())
                frames.append({'tick': sim.tick, 'world': world, 'metrics': json.loads(sim.get_metrics())})
            capture()
            remaining = request.ticks
            while remaining:
                n = min(remaining, request.interval)
                sim.run(n)
                remaining -= n
                if sim.population > 5000:
                    raise ValueError('Replay preview population exceeds 5000')
                capture()
            return frames
        try:
            return await asyncio.to_thread(run)
        except ValueError as exc:
            raise HTTPException(400, str(exc)) from exc

    return router
