
from typing import Any
from typing import Literal

from pydantic import BaseModel, Field

class BackendRequest(BaseModel):
    backend: Literal[
        "brute_force",
        "serial_cpu",
        "parallel_cpu",
        "auto_cpu",
        "gpu"
    ]

class SpeedRequest(BaseModel):
    ticks_per_second: int = Field(
        default=60,
        ge=1,
        le=1000,
    )


class StepRequest(BaseModel):
    ticks: int = Field(
        default=1,
        ge=1,
        le=1000,
    )


class ResetRequest(BaseModel):
    config: dict[str, Any] | None = None
