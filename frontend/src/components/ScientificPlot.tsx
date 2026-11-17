import { memo, useEffect, useRef } from "react";

import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

import { useSimulationStore } from "../store/simulationStore";

import type { Metrics } from "../types/simulation";

export interface PlotSeries {
  key: keyof Metrics;
  name: string;
  color: string;
}

interface ScientificPlotProps {
  number: string;
  title: string;
  description: string;
  series: PlotSeries[];
}

const UPDATE_INTERVAL = 250;
const PLOT_HEIGHT = 220;

function ScientificPlot({
  number,
  title,
  description,
  series,
}: ScientificPlotProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  const sampleCountRef = useRef<HTMLSpanElement>(null);

  const tickRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const element = containerRef.current;

    if (!element) return;

    const container: HTMLDivElement = element;

    let plot: uPlot | null = null;

    let lastHistory: Metrics[] | null = null;

    function buildData(history: Metrics[]): uPlot.AlignedData {
      const ticks = history.map((sample) => sample.tick);

      const values = series.map((item) =>
        history.map((sample) => Number(sample[item.key])),
      );

      return [ticks, ...values] as uPlot.AlignedData;
    }

    const options: uPlot.Options = {
      width: Math.max(1, container.clientWidth),

      height: PLOT_HEIGHT,

      scales: {
        x: {
          time: false,
        },
      },

      cursor: {
        show: true,
        x: true,
        y: true,

        drag: {
          x: false,
          y: false,
        },

        points: {
          size: 5,
        },
      },

      legend: {
        show: true,
        live: true,
      },

      axes: [
        {
          stroke: "#646459",

          font: "10px IBM Plex Mono",

          grid: {
            show: false,
          },

          ticks: {
            stroke: "#b9b8ad",
            size: 4,
          },

          values: (_plot, ticks) =>
            ticks.map((tick) => Math.round(tick).toLocaleString()),
        },

        {
          stroke: "#646459",

          font: "10px IBM Plex Mono",

          size: 52,

          grid: {
            stroke: "#d5d3c9",
            width: 1,
            dash: [2, 4],
          },

          ticks: {
            show: false,
          },

          values: (_plot, values) =>
            values.map((value) => Number(value.toPrecision(4)).toString()),
        },
      ],

      series: [
        {
          label: "Tick",
        },

        ...series.map((item) => ({
          label: item.name,

          stroke: item.color,

          width: 1.8,

          points: {
            show: false,
          },

          value: (_plot: uPlot, value: number | null) =>
            value == null ? "-" : Number(value.toPrecision(5)).toLocaleString(),
        })),
      ],
    };

    function update() {
      const history = useSimulationStore.getState().history;

      // skip unchanged history
      if (history === lastHistory) return;

      lastHistory = history;

      const data = buildData(history);

      if (!plot) {
        plot = new uPlot(options, data, container);
      } else {
        plot.setData(data);
      }

      if (sampleCountRef.current) {
        sampleCountRef.current.textContent = `${history.length} samples`;
      }

      if (tickRef.current) {
        const lastSample = history[history.length - 1];

        tickRef.current.textContent = lastSample
          ? `Tick ${lastSample.tick.toLocaleString()}`
          : "Awaiting data";
      }
    }

    update();

    // update canvas outside react
    const timer = window.setInterval(update, UPDATE_INTERVAL);

    const observer = new ResizeObserver(() => {
      if (!plot) return;

      const width = Math.max(1, container.clientWidth);

      if (plot.width !== width) {
        plot.setSize({
          width,
          height: PLOT_HEIGHT,
        });
      }
    });

    observer.observe(container);

    return () => {
      window.clearInterval(timer);

      observer.disconnect();

      plot?.destroy();
    };
  }, [series]);

  return (
    <figure className="scientific-chart">
      <figcaption className="chart-heading">
        <span className="figure-number">FIG. {number}</span>

        <h3>{title}</h3>

        <p>{description}</p>
      </figcaption>

      <div ref={containerRef} className="scientific-plot-canvas" />

      <div className="chart-footer">
        <span ref={tickRef}>Awaiting data</span>

        <span ref={sampleCountRef}>0 samples</span>
      </div>
    </figure>
  );
}

export default memo(ScientificPlot);
