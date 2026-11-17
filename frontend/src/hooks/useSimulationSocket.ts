import { browserMode } from "../services/browserMode";
import { connectBrowser } from "../services/browserClient";

import { useEffect } from "react";

import { WS_URL } from "../services/api";
import { useSimulationStore } from "../store/simulationStore";
import type { SimulationSnapshot } from "../types/simulation";

export function useSimulationSocket() {
  useEffect(() => {
    if (browserMode) return connectBrowser();
    let socket: WebSocket | null = null;
    let reconnectTimer: number | undefined;
    let disposed = false;

    function connect() {
      if (disposed) return;

      socket = new WebSocket(WS_URL);

      socket.onopen = () => {
        if (disposed) return;

        useSimulationStore.getState().setConnected(true);
        console.log("APEIRON WebSocket connected");
      };

      socket.onmessage = (event) => {
        if (disposed) return;

        try {
          const snapshot = JSON.parse(event.data) as SimulationSnapshot;

          if (snapshot.type === "snapshot") {
            useSimulationStore.getState().setSnapshot(snapshot);
          }
        } catch (error) {
          console.error("Invalid simulation snapshot", error);
        }
      };

      socket.onclose = () => {
        if (disposed) return;

        useSimulationStore.getState().setConnected(false);

        reconnectTimer = window.setTimeout(connect, 2000);
      };

      socket.onerror = () => {
        socket?.close();
      };
    }

    connect();

    return () => {
      disposed = true;

      window.clearTimeout(reconnectTimer);

      socket?.close();

      useSimulationStore.getState().setConnected(false);
    };
  }, []);
}
