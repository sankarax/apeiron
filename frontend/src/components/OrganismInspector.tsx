import { memo, useEffect, useState } from "react";
import { useThrottledSimulation } from "../hooks/useThrottledSimulation";

import { useSimulationStore } from "../store/simulationStore";
import { api } from "../services/api";

import type { Organism } from "../types/simulation";

import "./OrganismInspector.css";

interface TraitProps {
  label: string;
  value: number;
  min: number;
  max: number;
  decimals?: number;
}

function Trait({ label, value, min, max, decimals = 2 }: TraitProps) {
  const percentage = Math.max(
    0,
    Math.min(100, ((value - min) / (max - min)) * 100),
  );

  return (
    <div className="trait">
      <div className="trait-heading">
        <span>{label}</span>
        <strong>{value.toFixed(decimals)}</strong>
      </div>

      <div className="trait-track">
        <div className="trait-fill" style={{ width: `${percentage}%` }} />
      </div>

      <div className="trait-range">
        <span>{min}</span>
        <span>{max}</span>
      </div>
    </div>
  );
}

function SpecimenDiagram({ organism }: { organism: Organism }) {
  const { genome, vx, vy } = organism;

  const color = `hsl(${genome.hue}, 45%, 42%)`;

  const heading = (Math.atan2(vy, vx) * 180) / Math.PI;

  const scale = 65 / Math.max(genome.sensor_radius, genome.size + 5);

  const bodyRadius = genome.size * scale;
  const sensorRadius = genome.sensor_radius * scale;

  const isMoving = Math.hypot(vx, vy) > 0.001;

  return (
    <svg
      className="specimen-diagram"
      viewBox="0 0 280 200"
      role="img"
      aria-label="Organism body and sensing radius"
    >
      <defs>
        <pattern
          id="specimen-grid"
          width="16"
          height="16"
          patternUnits="userSpaceOnUse"
        >
          <path
            d="M 16 0 L 0 0 0 16"
            fill="none"
            stroke="#d5d3c9"
            strokeWidth="0.5"
          />
        </pattern>
      </defs>

      <rect width="280" height="200" fill="url(#specimen-grid)" />

      {/* Reference axes */}
      <line
        x1="140"
        y1="15"
        x2="140"
        y2="170"
        stroke="#b9b8ad"
        strokeWidth="0.6"
      />

      <line
        x1="15"
        y1="90"
        x2="265"
        y2="90"
        stroke="#b9b8ad"
        strokeWidth="0.6"
      />

      {/* Sensing field */}
      <circle
        cx="140"
        cy="90"
        r={sensorRadius}
        fill={color}
        fillOpacity="0.06"
        stroke={color}
        strokeWidth="0.8"
        strokeDasharray="3 4"
      />

      {/* Organism body */}
      <circle
        cx="140"
        cy="90"
        r={bodyRadius}
        fill={color}
        stroke="#24241f"
        strokeWidth="0.8"
      />

      {/* Direction of movement */}
      {isMoving && (
        <g transform={`rotate(${heading}, 140, 90)`}>
          <line
            x1="140"
            y1="90"
            x2={140 + sensorRadius + 10}
            y2="90"
            stroke="#24241f"
            strokeWidth="1.2"
          />

          <path
            d={`
              M ${140 + sensorRadius + 10} 90
              l -6 -3
              l 0 6
              z
            `}
            fill="#24241f"
          />
        </g>
      )}

      {/* Sensor annotation */}
      <path
        d={`
          M ${140 - sensorRadius} 90
          L 25 90
          L 25 35
        `}
        fill="none"
        stroke="#646459"
        strokeWidth="0.7"
      />

      <text x="10" y="24" fontSize="8" fill="#646459">
        SENSOR FIELD
      </text>

      {/* Body annotation */}
      <path
        d={`
          M ${140 + bodyRadius} 90
          L 235 135
          L 265 135
        `}
        fill="none"
        stroke="#646459"
        strokeWidth="0.7"
      />

      <text x="265" y="148" textAnchor="end" fontSize="8" fill="#646459">
        BODY
      </text>

      {/* Scale bar: 10 simulation units */}
      <line
        x1={140 - 5 * scale}
        y1="177"
        x2={140 + 5 * scale}
        y2="177"
        stroke="#24241f"
        strokeWidth="1.2"
      />

      <text x="140" y="190" textAnchor="middle" fontSize="8" fill="#646459">
        10 WORLD UNITS
      </text>
    </svg>
  );
}

function OrganismInspector() {
  const selectedId = useSimulationStore((state) => state.selectedOrganismId);

  const snapshot = useThrottledSimulation((state) => state.snapshot, 100);

  const organism = snapshot?.world.organisms.find(
    (item) => item.id === selectedId,
  );

  const selectOrganism = useSimulationStore((state) => state.selectOrganism);

  const [maxEnergy, setMaxEnergy] = useState<number | null>(null);

  const tick = snapshot?.status.tick;

  const isInitialState = tick === 0;

  useEffect(() => {
    let active = true;

    api
      .getConfig()
      .then((config) => {
        if (active) {
          setMaxEnergy(config.max_energy);
        }
      })
      .catch(() => {
        if (active) {
          setMaxEnergy(null);
        }
      });

    return () => {
      active = false;
    };
  }, [isInitialState]);

  if (selectedId === null) {
    return (
      <section className="organism-inspector">
        <span className="inspector-index">03 / ORGANISM ANALYSIS</span>

        <h2>Organism inspector</h2>

        <div className="inspector-empty">
          <svg viewBox="0 0 100 100">
            <circle
              cx="50"
              cy="50"
              r="35"
              fill="none"
              stroke="#b9b8ad"
              strokeDasharray="3 4"
            />

            <circle cx="50" cy="50" r="10" fill="none" stroke="#646459" />

            <path d="M50 5 V95 M5 50 H95" stroke="#b9b8ad" strokeWidth="0.5" />
          </svg>

          <p>No organism selected.</p>

          <span>
            Select an organism in the habitat to examine its physiological state
            and genetic profile.
          </span>
        </div>
      </section>
    );
  }

  if (!organism) {
    return (
      <section className="organism-inspector">
        <span className="inspector-index">03 / ORGANISM ANALYSIS</span>

        <h2>Organism #{selectedId}</h2>

        <p className="specimen-unavailable">
          This organism is no longer present in the living population.
        </p>

        <button
          className="inspector-clear"
          onClick={() => selectOrganism(null)}
        >
          Clear selection
        </button>
      </section>
    );
  }

  const { genome } = organism;

  const energyPercentage =
    maxEnergy !== null && maxEnergy > 0
      ? Math.min(100, (organism.energy / maxEnergy) * 100)
      : 0;

  const reproductionReady =
    organism.energy >= genome.reproduction_threshold && organism.age >= 20;

  const actualSpeed = Math.hypot(organism.vx, organism.vy);

  return (
    <section className="organism-inspector">
      <div className="inspector-topline">
        <span className="inspector-index">03 / ORGANISM ANALYSIS</span>

        <button
          className="inspector-close"
          onClick={() => selectOrganism(null)}
          aria-label="Clear organism selection"
        >
          ×
        </button>
      </div>

      <div className="specimen-columns">
        <div className="specimen-overview">
          <h2>
            Specimen <span>{String(organism.id).padStart(6, "0")}</span>
          </h2>

          <div className="specimen-status">
            <span className="specimen-status-dot" />
            LIVING ORGANISM
          </div>

          <figure className="specimen-figure">
            <SpecimenDiagram organism={organism} />

            <figcaption>FIG. 07 - MORPHOLOGY & SENSING</figcaption>
          </figure>

          <div className="specimen-summary">
            <div>
              <span>Generation</span>
              <strong>{organism.generation}</strong>
            </div>

            <div>
              <span>Age / ticks</span>
              <strong>{organism.age}</strong>
            </div>

            <div>
              <span>Parent</span>
              <strong>
                {organism.parent_id === null ? "-" : `#${organism.parent_id}`}
              </strong>
            </div>
          </div>
        </div>

        <div className="inspector-subsection">
          <h3>Physiological state</h3>

          <div className="trait-heading">
            <span>Stored energy</span>

            <strong>
              {organism.energy.toFixed(1)}
              {maxEnergy !== null && ` / ${maxEnergy}`}
            </strong>
          </div>

          {maxEnergy !== null && (
            <div className="energy-track">
              <div
                className="energy-fill"
                style={{
                  width: `${energyPercentage}%`,
                  background: energyPercentage < 20 ? "#b04d32" : "#4c6946",
                }}
              />
            </div>
          )}

          <dl className="specimen-properties">
            <div>
              <dt>Current speed</dt>
              <dd>{actualSpeed.toFixed(3)}</dd>
            </div>

            <div>
              <dt>Position X</dt>
              <dd>{organism.x.toFixed(1)}</dd>
            </div>

            <div>
              <dt>Position Y</dt>
              <dd>{organism.y.toFixed(1)}</dd>
            </div>

            <div>
              <dt>Reproduction</dt>

              <dd className={reproductionReady ? "reproduction-ready" : ""}>
                {reproductionReady ? "Eligible" : "Not ready"}
              </dd>
            </div>
          </dl>
        </div>

        <div className="inspector-subsection">
          <h3>Genetic profile</h3>

          <Trait
            label="Movement speed"
            value={genome.speed}
            min={0.2}
            max={6}
            decimals={3}
          />

          <Trait
            label="Sensor radius"
            value={genome.sensor_radius}
            min={10}
            max={200}
          />

          <Trait
            label="Metabolism"
            value={genome.metabolism}
            min={0.05}
            max={1}
            decimals={3}
          />

          <Trait label="Body size" value={genome.size} min={1} max={12} />

          <Trait
            label="Reproduction threshold"
            value={genome.reproduction_threshold}
            min={100}
            max={240}
          />

          <div className="genome-color">
            <span>Inherited pigmentation</span>

            <div>
              <span
                className="genome-color-swatch"
                style={{
                  background: `hsl(${genome.hue}, 85%, 60%)`,
                }}
              />

              <strong>{genome.hue.toFixed(1)}°</strong>
            </div>
          </div>
        </div>
      </div>

      <footer className="inspector-footer">
        APEIRON / INDIVIDUAL OBSERVATION
      </footer>
    </section>
  );
}
export default memo(OrganismInspector);
