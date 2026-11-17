import { useEffect, useRef, useState } from "react";

import { useSimulationStore } from "../store/simulationStore";

type SimulationStore = ReturnType<typeof useSimulationStore.getState>;

export function useThrottledSimulation<T>(
  selector: (state: SimulationStore) => T,
  interval = 100,
): T {
  const selectorRef = useRef(selector);

  useEffect(() => {
    selectorRef.current = selector;
  }, [selector]);

  const [value, setValue] = useState<T>(() =>
    selector(useSimulationStore.getState()),
  );

  useEffect(() => {
    let lastUpdate = performance.now();

    let timeout: ReturnType<typeof setTimeout> | null = null;

    function publish() {
      timeout = null;

      lastUpdate = performance.now();

      const nextValue = selectorRef.current(useSimulationStore.getState());

      setValue((previous) =>
        Object.is(previous, nextValue) ? previous : nextValue,
      );
    }

    const unsubscribe = useSimulationStore.subscribe(() => {
      const elapsed = performance.now() - lastUpdate;

      if (elapsed >= interval) {
        if (timeout !== null) {
          clearTimeout(timeout);
          timeout = null;
        }

        publish();
      } else if (timeout === null) {
        timeout = setTimeout(publish, interval - elapsed);
      }
    });

    publish();

    return () => {
      unsubscribe();

      if (timeout !== null) {
        clearTimeout(timeout);
      }
    };
  }, [interval]);

  return value;
}
