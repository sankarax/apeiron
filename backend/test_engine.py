
import json
import time

from apeiron_engine import Simulation, default_config


def main():
    config = json.loads(default_config())

    config["seed"] = 42
    config["initial_population"] = 200

    sim = Simulation(json.dumps(config))

    print("APEIRON - Python Interface")
    print("=" * 40)
    print(sim)

    start = time.perf_counter()

    for _ in range(20):
        sim.run(100)

        metrics = json.loads(sim.get_metrics())

        print(
            f"Tick: {sim.tick:5d} | "
            f"Population: {sim.population:4d} | "
            f"Food: {sim.food_count:4d} | "
            f"Speed: {metrics['average_speed']:.3f} | "
            f"Generation: {metrics['max_generation']}"
        )

    elapsed = time.perf_counter() - start

    print(f"\nElapsed: {elapsed:.3f}s")
    print(f"Ticks/sec: {sim.tick / elapsed:.2f}")

    print("\nFinal metrics:")
    print(json.dumps(json.loads(sim.get_metrics()), indent=2))

    state = json.loads(sim.get_state())

    if state["organisms"]:
        print("\nFirst organism:")
        print(json.dumps(state["organisms"][0], indent=2))


if __name__ == "__main__":
    main()
