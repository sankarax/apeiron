import { memo } from "react";
import { useSimulationStore } from "../store/simulationStore";

import ScientificPlot from "./ScientificPlot";

import type { PlotSeries } from "./ScientificPlot";

import "./EvolutionCharts.css";

interface ChartDefinition {
  number: string;
  title: string;
  description: string;
  series: PlotSeries[];
}

const MOSS = "#4c6946";
const TERRACOTTA = "#b04d32";
const INK = "#24241f";

const CHARTS: ChartDefinition[] = [
  {
    number: "01",

    title: "Population dynamics",

    description: "Living organisms and available environmental resources.",

    series: [
      {
        key: "population",
        name: "Organisms",
        color: MOSS,
      },
      {
        key: "food_count",
        name: "Food",
        color: TERRACOTTA,
      },
    ],
  },

  {
    number: "02",

    title: "Locomotion",

    description: "Mean inherited movement speed across the living population.",

    series: [
      {
        key: "average_speed",
        name: "Mean speed",
        color: MOSS,
      },
    ],
  },

  {
    number: "03",

    title: "Metabolic adaptation",

    description: "Evolution of the mean inherited metabolic rate.",

    series: [
      {
        key: "average_metabolism",
        name: "Metabolism",
        color: TERRACOTTA,
      },
    ],
  },

  {
    number: "04",

    title: "Generational depth",

    description: "Average and maximum generation among living organisms.",

    series: [
      {
        key: "average_generation",
        name: "Mean generation",
        color: MOSS,
      },
      {
        key: "max_generation",
        name: "Max generation",
        color: INK,
      },
    ],
  },

  {
    number: "05",

    title: "Energetic equilibrium",

    description: "Mean energy reserves of the living population.",

    series: [
      {
        key: "average_energy",
        name: "Mean energy",
        color: MOSS,
      },
    ],
  },

  {
    number: "06",

    title: "Birth and mortality",

    description: "Cumulative reproductive events and organism deaths.",

    series: [
      {
        key: "total_births",
        name: "Births",
        color: MOSS,
      },
      {
        key: "total_deaths",
        name: "Deaths",
        color: TERRACOTTA,
      },
    ],
  },
];

const ECOLOGY_CHARTS: ChartDefinition[] = [
  {
    number: "07",
    title: "Species dynamics",
    description: "Living herbivores and predators.",
    series: [
      { key: "herbivores", name: "Herbivores", color: MOSS },
      { key: "predators", name: "Predators", color: TERRACOTTA },
    ],
  },
  {
    number: "08",
    title: "Predation events",
    description: "Cumulative successful hunts.",
    series: [{ key: "total_kills", name: "Kills", color: TERRACOTTA }],
  },
  {
    number: "09",
    title: "Trophic energy transfer",
    description: "Cumulative energy gained through predation.",
    series: [{ key: "predation_energy", name: "Energy", color: INK }],
  },
];

function EvolutionCharts() {
  const predation = useSimulationStore(
    (state) => state.snapshot?.habitat?.predation ?? false,
  );
  const metrics = useSimulationStore((state) => state.history.at(-1));
  const charts = predation ? [...CHARTS, ...ECOLOGY_CHARTS] : CHARTS;
  return (
    <section className="evolution-charts">
      <header className="observations-header">
        <div>
          <span className="section-index">02 / OBSERVATIONS</span>

          <h2>Evolutionary dynamics</h2>
        </div>

        <span className="observations-status">LIVE SIMULATIONS / 4 Hz MAX</span>
      </header>

      {predation && metrics && (
        <p>
          Herbivores: {metrics.herbivores} · Predators: {metrics.predators} ·
          Kills: {metrics.total_kills} · Energy transferred:{" "}
          {metrics.predation_energy.toFixed(2)}
        </p>
      )}
      <div className="charts-grid">
        {charts.map((chart) => (
          <ScientificPlot
            key={chart.number}
            number={chart.number}
            title={chart.title}
            description={chart.description}
            series={chart.series}
          />
        ))}
      </div>
    </section>
  );
}

export default memo(EvolutionCharts);
