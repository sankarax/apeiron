import { create } from "zustand";

import type { Metrics, SimulationSnapshot } from "../types/simulation";

interface SimulationStore {
  snapshot: SimulationSnapshot | null;
  history: Metrics[];
  connected: boolean;
  selectedOrganismId: number | null;

  setSnapshot: (snapshot: SimulationSnapshot) => void;
  setConnected: (connected: boolean) => void;
  selectOrganism: (id: number | null) => void;
  clearHistory: () => void;
}

const MAX_HISTORY = 300;
const HISTORY_INTERVAL = 10;

export const useSimulationStore = create<SimulationStore>((set) => ({
  snapshot: null,
  history: [],
  connected: false,
  selectedOrganismId: null,

  setSnapshot: (snapshot) =>
    set((state) => {
      let history = state.history;
      let selectedOrganismId = state.selectedOrganismId;

      const previousTick = state.snapshot?.status.tick;
      const currentTick = snapshot.status.tick;

      if (previousTick !== undefined && currentTick < previousTick) {
        history = [];
        selectedOrganismId = null;
      }

      const last = history[history.length - 1];

      if (!last || snapshot.metrics.tick - last.tick >= HISTORY_INTERVAL) {
        history = [...history, snapshot.metrics].slice(-MAX_HISTORY);
      }

      return {
        snapshot,
        history,
        selectedOrganismId,
      };
    }),

  setConnected: (connected) => set({ connected }),

  selectOrganism: (selectedOrganismId) => set({ selectedOrganismId }),

  clearHistory: () => set({ history: [] }),
}));
