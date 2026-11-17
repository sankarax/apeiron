
import asyncio
from contextlib import asynccontextmanager

from fastapi import (
    FastAPI,
    HTTPException,
    WebSocket,
    WebSocketDisconnect,
)
from fastapi.middleware.cors import CORSMiddleware

from app.schemas import (
    ResetRequest,
    SpeedRequest,
    StepRequest,
    BackendRequest
)
from app.simulation_manager import SimulationManager


from app.multiverse import router as multiverse_router, shutdown_worlds

from app.persistence import initialize, router as archive_router

manager = SimulationManager()


@asynccontextmanager
async def lifespan(app: FastAPI):
    await asyncio.to_thread(initialize)
    await manager.startup()
    yield
    await shutdown_worlds()
    await manager.shutdown()


app = FastAPI(
    title="APEIRON API",
    description="Artificial life and evolutionary simulation engine",
    version="0.1.0",
    lifespan=lifespan,
)


app.include_router(multiverse_router)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:5173",
        "http://127.0.0.1:5173",
    ],
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
async def root():
    return {
        "name": "APEIRON",
        "version": "0.1.0",
        "status": "online",
    }


@app.get("/api/health")
async def health():
    return {"status": "healthy"}


@app.get("/api/simulation/status")
async def simulation_status():
    return await manager.get_status()


@app.get("/api/simulation/config")
async def simulation_config():
    return await manager.get_config()


@app.get("/api/simulation/metrics")
async def simulation_metrics():
    return await manager.get_metrics()


@app.get("/api/simulation/brain/{organism_id}")
async def simulation_brain(organism_id: int):
    if not 0 <= organism_id <= 2**64 - 1:
        raise HTTPException(status_code=400, detail="Invalid organism ID")
    return await manager.get_brain(organism_id)


@app.get("/api/simulation/state")
async def simulation_state():
    return await manager.get_state()


@app.post("/api/simulation/start")
async def simulation_start():
    return await manager.start()


@app.post("/api/simulation/pause")
async def simulation_pause():
    return await manager.pause()


@app.post("/api/simulation/speed")
async def simulation_speed(request: SpeedRequest):
    return await manager.set_speed(
        request.ticks_per_second
    )

@app.post("/simulation/backend")
async def set_backend(request: BackendRequest):
    try:
        return await manager.set_backend(request.backend)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc

@app.post("/api/simulation/step")
async def simulation_step(request: StepRequest):
    try:
        return await manager.step(request.ticks)
    except ValueError as exc:
        raise HTTPException(
            status_code=409,
            detail=str(exc),
        ) from exc


@app.post("/api/simulation/reset")
async def simulation_reset(request: ResetRequest):
    try:
        return await manager.reset(request.config)
    except ValueError as exc:
        raise HTTPException(
            status_code=400,
            detail=str(exc),
        ) from exc


@app.websocket("/ws/simulation")
async def simulation_websocket(websocket: WebSocket):
    await websocket.accept()

    try:
        while True:
            snapshot = await manager.get_snapshot()

            await websocket.send_json(snapshot)

            await asyncio.sleep(1 / 30)

    except WebSocketDisconnect:
        pass


from app.history import history_router

app.include_router(history_router(manager))

from app.scientist import router as scientist_router

app.include_router(scientist_router)

app.include_router(archive_router)
