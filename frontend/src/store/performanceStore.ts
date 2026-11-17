import { create } from "zustand";

export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  p95FrameTime: number;
  maxFrameTime: number;

  snapshotRate: number;

  drawCalls: number;
  triangles: number;

  frameHistory: number[];
}

interface PerformanceStore extends PerformanceMetrics {
  publish: (metrics: PerformanceMetrics) => void;
}

export const usePerformanceStore = create<PerformanceStore>((set) => ({
  fps: 0,
  frameTime: 0,
  p95FrameTime: 0,
  maxFrameTime: 0,

  snapshotRate: 0,

  drawCalls: 0,
  triangles: 0,

  frameHistory: [],

  publish: (metrics) => set(metrics),
}));
