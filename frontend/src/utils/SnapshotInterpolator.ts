import type { Organism, SimulationSnapshot } from "../types/simulation";

import { clamp, lerpWrapped } from "./interpolation";

interface Position {
  x: number;
  y: number;
}

export class SnapshotInterpolator {
  private previousPositions = new Map<number, Position>();

  private currentSnapshot: SimulationSnapshot | null = null;

  private receivedAt = 0;

  private interval = 1000 / 30;

  push(snapshot: SimulationSnapshot, now: number): void {
    const previous = this.currentSnapshot;

    const shouldReset =
      !previous ||
      snapshot.status.tick < previous.status.tick ||
      snapshot.world.width !== previous.world.width ||
      snapshot.world.height !== previous.world.height;

    if (shouldReset) {
      this.previousPositions.clear();

      for (const organism of snapshot.world.organisms) {
        this.previousPositions.set(organism.id, {
          x: organism.x,
          y: organism.y,
        });
      }

      this.currentSnapshot = snapshot;
      this.receivedAt = now;
      this.interval = 1000 / 30;

      return;
    }

    // ignore duplicate snapshots
    if (snapshot.status.tick === previous.status.tick) {
      this.currentSnapshot = snapshot;
      return;
    }

    const elapsed = now - this.receivedAt;

    const nextPositions = new Map<number, Position>();

    for (const organism of previous.world.organisms) {
      nextPositions.set(organism.id, this.getPosition(organism, now));
    }

    this.previousPositions = nextPositions;

    this.interval = clamp(this.interval * 0.8 + elapsed * 0.2, 16, 150);

    this.currentSnapshot = snapshot;
    this.receivedAt = now;
  }

  getPosition(organism: Organism, now: number): Position {
    const previous = this.previousPositions.get(organism.id);

    if (!previous || !this.currentSnapshot) {
      return {
        x: organism.x,
        y: organism.y,
      };
    }

    const alpha = clamp((now - this.receivedAt) / this.interval, 0, 1);

    const { width, height } = this.currentSnapshot.world;

    return {
      x: lerpWrapped(previous.x, organism.x, alpha, width),

      y: lerpWrapped(previous.y, organism.y, alpha, height),
    };
  }

  getSnapshot(): SimulationSnapshot | null {
    return this.currentSnapshot;
  }
}
