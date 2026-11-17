import asyncio
import json
from uuid import uuid4

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field
from apeiron_engine import Simulation, default_config

from app.simulation_manager import SimulationManager


router = APIRouter(prefix='/api/multiverse')
worlds: dict[str, tuple[str, SimulationManager]] = {}
registry_lock = asyncio.Lock()


class WorldRequest(BaseModel):
    name: str = Field(min_length=1, max_length=80)
    config: dict = Field(default_factory=dict)
    backend: str = 'auto_cpu'


class RunRequest(BaseModel):
    ticks: int = Field(default=100, ge=1, le=10000)


async def lookup(world_id: str):
    if world_id not in worlds:
        raise HTTPException(404, 'World not found')
    return worlds[world_id][1]


async def describe(world_id, name, manager):
    async with manager.lock:
        return {'id': world_id, 'name': name, 'status': manager._status(),
                'config': manager.config.copy(),
                'metrics': json.loads(manager.simulation.get_metrics())}


@router.get('/worlds')
async def list_worlds():
    return await asyncio.gather(*(describe(key, name, manager)
                                  for key, (name, manager) in list(worlds.items())))


@router.post('/worlds', status_code=201)
async def create_world(request: WorldRequest):
    async with registry_lock:
        if len(worlds) >= 8:
            raise HTTPException(409, 'Maximum of eight worlds')
        config = json.loads(default_config())
        if request.config.keys() - config.keys():
            raise HTTPException(400, 'Unknown configuration fields')
        config.update(request.config)
        try:
            manager = SimulationManager()
            manager.simulation = await asyncio.to_thread(Simulation, json.dumps(config), request.backend)
            manager.config = config
        except ValueError as exc:
            raise HTTPException(400, str(exc)) from exc
        key = uuid4().hex
        await manager.startup()
        worlds[key] = (request.name, manager)
        return await describe(key, request.name, manager)


@router.get('/worlds/{world_id}/snapshot')
async def snapshot(world_id: str):
    return await (await lookup(world_id)).get_snapshot()


@router.post('/worlds/{world_id}/start')
async def start(world_id: str):
    return await (await lookup(world_id)).start()


@router.post('/worlds/{world_id}/pause')
async def pause(world_id: str):
    return await (await lookup(world_id)).pause()


@router.post('/worlds/{world_id}/reset')
async def reset(world_id: str):
    return await (await lookup(world_id)).reset()


@router.delete('/worlds/{world_id}')
async def delete(world_id: str):
    async with registry_lock:
        manager = await lookup(world_id)
        await manager.pause()
        await manager.shutdown()
        del worlds[world_id]
    return {'deleted': world_id}


@router.post('/run')
async def run_all(request: RunRequest):
    async with registry_lock:
        entries = list(worlds.items())
        # pause live loops before running controlled batches
        await asyncio.gather(*(manager.pause() for _, (_, manager) in entries))
        async def run_one(key, name, manager):
            try:
                await manager.step(request.ticks)
                return await describe(key, name, manager)
            except ValueError as exc:
                return {'id': key, 'name': name, 'error': str(exc)}
        return await asyncio.gather(*(run_one(key, name, manager) for key, (name, manager) in entries))


async def shutdown_worlds():
    async with registry_lock:
        for _, manager in worlds.values():
            await manager.pause()
            await manager.shutdown()
        worlds.clear()
