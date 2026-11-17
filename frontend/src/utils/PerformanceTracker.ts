import type { WebGLRenderer } from "three";

import { usePerformanceStore } from "../store/performanceStore";

const REPORT_INTERVAL = 1000;
const MAX_HISTORY = 120;

export class PerformanceTracker {
  private lastFrameAt: number | null = null;

  private lastReportAt = performance.now();

  private frameTimes: number[] = [];

  private frameHistory: number[] = [];

  private renderedFrames = 0;

  private snapshots = 0;

  recordSnapshot(): void {
    this.snapshots++;
  }

  recordFrame(now: number, renderer: WebGLRenderer, rendered: boolean): void {
    if (this.lastFrameAt !== null) {
      const delta = now - this.lastFrameAt;

      this.frameTimes.push(delta);

      this.frameHistory.push(delta);

      if (this.frameHistory.length > MAX_HISTORY) {
        this.frameHistory.shift();
      }
    }

    this.lastFrameAt = now;

    if (rendered) {
      this.renderedFrames++;
    }

    const elapsed = now - this.lastReportAt;

    if (elapsed >= REPORT_INTERVAL) {
      this.publish(elapsed, renderer);

      this.lastReportAt = now;
    }
  }

  private publish(elapsed: number, renderer: WebGLRenderer): void {
    const seconds = elapsed / 1000;

    const sorted = [...this.frameTimes].sort((a, b) => a - b);

    const count = sorted.length;

    const average =
      count > 0 ? sorted.reduce((sum, value) => sum + value, 0) / count : 0;

    const p95Index = Math.max(0, Math.ceil(count * 0.95) - 1);
    console.log("Performance report", {
      elapsed,
      frames: this.renderedFrames,
      snapshots: this.snapshots,
      frameSamples: this.frameTimes.length,
    });
    usePerformanceStore.getState().publish({
      fps: this.renderedFrames / seconds,

      frameTime: average,

      p95FrameTime: sorted[p95Index] ?? 0,

      maxFrameTime: sorted[count - 1] ?? 0,

      snapshotRate: this.snapshots / seconds,

      drawCalls: renderer.info.render.calls,

      triangles: renderer.info.render.triangles,

      frameHistory: [...this.frameHistory],
    });

    this.frameTimes = [];

    this.renderedFrames = 0;

    this.snapshots = 0;
  }
}
