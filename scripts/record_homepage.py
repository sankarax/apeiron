import json
from pathlib import Path

from apeiron_engine import Simulation


def main():
    config = {
        "seed": 42,
        "width": 900,
        "height": 900,
        "controller": "recurrent",
        "initial_population": 350,
        "max_population": 700,
        "initial_food": 350,
        "max_food": 700,
        "food_spawn_rate": 5,
        "predation": True,
        "predator_metabolism_scale": 0.4,
    }
    sim = Simulation(json.dumps(config), "auto_cpu")
    frames = []
    for index in range(240):
        if index:
            sim.run(4)
        world = json.loads(sim.get_state())
        metrics = json.loads(sim.get_metrics())
        frames.append({
            "tick": metrics["tick"],
            "organisms": [
                [round(o["x"], 2), round(o["y"], 2), round(o["genome"]["size"], 2),
                 int(o["species"] == "predator")]
                for o in world["organisms"]
            ],
            "food": [[round(f["x"], 2), round(f["y"], 2)] for f in world["food"]],
        })
    target = Path(__file__).resolve().parents[1] / "frontend/public/homepage-replay.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps({"config": config, "fps": 24, "frames": frames}, separators=(",", ":")), encoding="utf-8")
    print(f"Recorded {len(frames)} frames to {target} ({target.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
