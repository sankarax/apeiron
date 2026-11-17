import { memo, useEffect, useState } from "react";

import { api } from "../services/api";

import {
  DEFAULT_CONFIG,
  EXPERIMENT_PRESETS,
} from "../config/experimentPresets";

import { useSimulationStore } from "../store/simulationStore";

import type { ControllerKind, SimulationConfig } from "../types/simulation";

import "./EnvironmentConfig.css";

type NumericConfigKey = {
  [K in keyof SimulationConfig]: SimulationConfig[K] extends number ? K : never;
}[keyof SimulationConfig];

type HabitatToggle = "predation" | "resource_zones" | "hazard_zones";

interface ParameterDefinition {
  key: NumericConfigKey;
  label: string;
  min: number;
  max: number;
  step: number;
}

const CONTROLLERS = {
  heuristic: {
    name: "Heuristic",
    description: "Rule-based behaviour without neural inference.",
    parameters: 0,
  },
  small: {
    name: "Small neural network",
    description: "6 → 8 → 3 MLP with egocentric perception.",
    parameters: 83,
  },
  deep: {
    name: "Deep neural network",
    description: "16 → 32 → 16 → 4 MLP with extended perception.",
    parameters: 1140,
  },
  recurrent: {
    name: "Recurrent GRU",
    description: "A recurrent neural network with 16 memory units.",
    parameters: 1652,
  },
} satisfies Record<
  ControllerKind,
  {
    name: string;
    description: string;
    parameters: number;
  }
>;

const PARAMETER_GROUPS: {
  title: string;
  parameters: ParameterDefinition[];
}[] = [
  {
    title: "World dimensions",

    parameters: [
      {
        key: "width",
        label: "World width",
        min: 400,
        max: 4000,
        step: 100,
      },
      {
        key: "height",
        label: "World height",
        min: 300,
        max: 3000,
        step: 100,
      },
    ],
  },

  {
    title: "Population",

    parameters: [
      {
        key: "initial_population",
        label: "Initial population",
        min: 1,
        max: 5000,
        step: 1,
      },
      {
        key: "max_population",
        label: "Population capacity",
        min: 1,
        max: 10000,
        step: 1,
      },
      {
        key: "max_age",
        label: "Maximum lifespan",
        min: 100,
        max: 10000,
        step: 100,
      },
    ],
  },

  {
    title: "Environmental resources",

    parameters: [
      {
        key: "initial_food",
        label: "Initial food",
        min: 0,
        max: 5000,
        step: 1,
      },
      {
        key: "max_food",
        label: "Food capacity",
        min: 0,
        max: 10000,
        step: 1,
      },
      {
        key: "food_spawn_rate",
        label: "Food regeneration / tick",
        min: 0,
        max: 100,
        step: 1,
      },
      {
        key: "food_energy",
        label: "Energy per food",
        min: 1,
        max: 100,
        step: 1,
      },
    ],
  },

  {
    title: "Energy dynamics",

    parameters: [
      {
        key: "initial_energy",
        label: "Initial energy",
        min: 1,
        max: 250,
        step: 1,
      },
      {
        key: "max_energy",
        label: "Maximum energy",
        min: 1,
        max: 500,
        step: 1,
      },
    ],
  },

  {
    title: "Genetic variation",

    parameters: [
      {
        key: "mutation_rate",
        label: "Mutation probability",
        min: 0,
        max: 1,
        step: 0.01,
      },
      {
        key: "mutation_strength",
        label: "Mutation strength",
        min: 0,
        max: 1,
        step: 0.01,
      },
    ],
  },

  {
    title: "Neural evolution",

    parameters: [
      {
        key: "neural_mutation_rate",
        label: "Neural mutation probability",
        min: 0,
        max: 1,
        step: 0.01,
      },
      {
        key: "neural_mutation_strength",
        label: "Neural mutation strength",
        min: 0,
        max: 1,
        step: 0.01,
      },
    ],
  },
];

const ECOLOGY_PARAMETERS: ParameterDefinition[] = [
  {
    key: "predator_fraction",
    label: "Initial predator fraction",
    min: 0,
    max: 1,
    step: 0.01,
  },
  {
    key: "predator_metabolism_scale",
    label: "Predator energy cost multiplier",
    min: 0,
    max: 2,
    step: 0.01,
  },
  {
    key: "attack_cost",
    label: "Attack energy cost",
    min: 0,
    max: 10,
    step: 0.1,
  },
  {
    key: "attack_range_bonus",
    label: "Attack range bonus",
    min: 0,
    max: 50,
    step: 0.1,
  },
  {
    key: "predation_efficiency",
    label: "Energy transfer efficiency",
    min: 0,
    max: 1,
    step: 0.01,
  },
  {
    key: "hazard_damage",
    label: "Hazard energy loss / tick",
    min: 0,
    max: 10,
    step: 0.1,
  },
];
PARAMETER_GROUPS.push({
  title: "Ecological parameters",
  parameters: ECOLOGY_PARAMETERS,
});

function EnvironmentConfig() {
  const [expanded, setExpanded] = useState(false);

  const [config, setConfig] = useState<SimulationConfig>({
    ...DEFAULT_CONFIG,
  });

  const [presetId, setPresetId] = useState("custom");

  const [error, setError] = useState("");

  const [message, setMessage] = useState("");

  const [busy, setBusy] = useState(false);

  const connected = useSimulationStore((state) => state.connected);

  useEffect(() => {
    if (!connected) return;

    let active = true;

    api
      .getConfig()
      .then((currentConfig) => {
        if (active) {
          setConfig(currentConfig);
        }
      })
      .catch((error: unknown) => {
        if (active) {
          setError(String(error));
        }
      });

    return () => {
      active = false;
    };
  }, [connected]);

  function updateParameter(key: NumericConfigKey, value: number) {
    setConfig((previous) => ({
      ...previous,
      [key]: value,
    }));

    setPresetId("custom");
    setError("");
    setMessage("");
  }

  function updateController(controller: ControllerKind) {
    setConfig((previous) => ({
      ...previous,
      controller,
    }));

    setPresetId("custom");
    setError("");
    setMessage("");
  }

  function applyPreset(id: string) {
    const preset = EXPERIMENT_PRESETS.find((item) => item.id === id);

    if (!preset) return;

    setConfig({
      ...DEFAULT_CONFIG,
      ...preset.config,
    });

    setPresetId(id);

    setError("");
    setMessage("");
  }

  function toggleHabitat(key: HabitatToggle, value: boolean) {
    setConfig((previous) => ({
      ...previous,
      [key]: value,
      ...(key === "predation" &&
      value &&
      (previous.controller === "small" || previous.controller === "heuristic")
        ? { controller: "recurrent" as const }
        : {}),
    }));
    setPresetId("custom");
    setError("");
    setMessage("");
  }

  function validate(): string | null {
    if (
      config.predation &&
      config.controller !== "deep" &&
      config.controller !== "recurrent"
    ) {
      return "Predation requires the deep or recurrent controller.";
    }
    for (const group of PARAMETER_GROUPS) {
      for (const parameter of group.parameters) {
        const value = config[parameter.key];

        if (
          !Number.isFinite(value) ||
          value < parameter.min ||
          value > parameter.max
        ) {
          return `${parameter.label} is outside its allowed range.`;
        }

        if (parameter.step >= 1 && !Number.isInteger(value)) {
          return `${parameter.label} must be an integer.`;
        }
      }
    }

    if (config.initial_population > config.max_population) {
      return "Initial population exceeds population capacity.";
    }

    if (config.initial_food > config.max_food) {
      return "Initial food exceeds food capacity.";
    }

    if (config.initial_energy > config.max_energy) {
      return "Initial energy exceeds maximum energy.";
    }

    if (!Number.isSafeInteger(config.seed) || config.seed < 0) {
      return "Seed must be a non-negative safe integer.";
    }

    return null;
  }

  async function applyConfiguration() {
    const validationError = validate();

    if (validationError) {
      setError(validationError);

      return;
    }

    const confirmed = window.confirm(
      "Apply these conditions? The current ecosystem and its live history will be reset.",
    );

    if (!confirmed) return;

    setBusy(true);
    setError("");
    setMessage("");

    try {
      await api.reset(config);

      const store = useSimulationStore.getState();

      store.clearHistory();
      store.selectOrganism(null);

      setMessage(
        "Configuration applied. Start the simulation to begin the experiment.",
      );
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }

  const selectedPreset = EXPERIMENT_PRESETS.find(
    (item) => item.id === presetId,
  );

  return (
    <section className="environment-config">
      <header className="environment-header">
        <div>
          <span className="environment-index">
            05 / EXPERIMENTAL CONDITIONS
          </span>

          <h2>Environment configuration</h2>
        </div>

        <button
          type="button"
          className="environment-toggle"
          aria-expanded={expanded}
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? "Close configuration −" : "Configure experiment +"}
        </button>
      </header>

      {expanded && (
        <div className="environment-content">
          <div className="preset-section">
            <label htmlFor="experiment-preset">Experimental preset</label>

            <select
              id="experiment-preset"
              value={presetId}
              onChange={(event) => {
                if (event.target.value === "custom") {
                  setPresetId("custom");
                } else {
                  applyPreset(event.target.value);
                }
              }}
            >
              <option value="custom">Custom configuration</option>

              {EXPERIMENT_PRESETS.map((preset) => (
                <option key={preset.id} value={preset.id}>
                  {preset.name}
                </option>
              ))}
            </select>

            <p>
              {selectedPreset?.description ??
                "Define your own environmental conditions."}
            </p>
          </div>

          <div className="parameter-grid">
            <fieldset className="parameter-group">
              <legend>Neural architecture</legend>

              <label className="parameter-control">
                <span>Behaviour controller</span>

                <select
                  value={config.controller}
                  onChange={(event) =>
                    updateController(event.target.value as ControllerKind)
                  }
                >
                  {Object.entries(CONTROLLERS).map(([kind, controller]) => (
                    <option key={kind} value={kind}>
                      {controller.name}
                    </option>
                  ))}
                </select>
              </label>

              <p className="parameter-note">
                {CONTROLLERS[config.controller].description}
              </p>

              <p className="parameter-note">
                Parameters per organism:{" "}
                {CONTROLLERS[config.controller].parameters.toLocaleString()}
              </p>

              <p className="parameter-note">
                Changing the controller initializes a new population with the
                selected architecture.
              </p>
            </fieldset>
            <fieldset className="parameter-group">
              <legend>Habitat mechanics</legend>
              {(
                [
                  ["predation", "Predator-prey dynamics"],
                  ["resource_zones", "Resource clusters"],
                  ["hazard_zones", "Central hazard"],
                ] as const
              ).map(([key, label]) => (
                <label key={key} className="parameter-control">
                  <span>{label}</span>
                  <input
                    type="checkbox"
                    checked={config[key]}
                    onChange={(event) =>
                      toggleHabitat(key, event.target.checked)
                    }
                  />
                </label>
              ))}
              <p className="parameter-note">
                Predation requires a deep or recurrent neural controller.
              </p>
            </fieldset>
            {PARAMETER_GROUPS.map((group) => (
              <fieldset key={group.title} className="parameter-group">
                <legend>{group.title}</legend>

                {group.parameters.map((parameter) => (
                  <label key={parameter.key} className="parameter-control">
                    <span>{parameter.label}</span>

                    <input
                      type="number"
                      min={parameter.min}
                      max={parameter.max}
                      step={parameter.step}
                      value={config[parameter.key]}
                      onChange={(event) =>
                        updateParameter(
                          parameter.key,
                          Number(event.target.value),
                        )
                      }
                    />
                  </label>
                ))}
              </fieldset>
            ))}

            <fieldset className="parameter-group">
              <legend>Reproducibility</legend>

              <label className="parameter-control">
                <span>Random seed</span>

                <input
                  type="number"
                  min={0}
                  step={1}
                  value={config.seed}
                  onChange={(event) =>
                    updateParameter("seed", Number(event.target.value))
                  }
                />
              </label>

              <p className="parameter-note">
                Identical configurations and seeds reproduce identical serial
                CPU simulation trajectories.
              </p>

              <button
                type="button"
                className="secondary-action"
                onClick={() =>
                  updateParameter(
                    "seed",
                    Math.floor(Math.random() * 2147483647),
                  )
                }
              >
                Generate random seed
              </button>
            </fieldset>
          </div>

          <div className="environment-actions">
            <button
              type="button"
              className="secondary-action"
              disabled={busy}
              onClick={() => applyPreset("balanced")}
            >
              Restore defaults
            </button>

            <button
              type="button"
              className="primary-action"
              disabled={!connected || busy}
              onClick={applyConfiguration}
            >
              {busy ? "Applying..." : "Initialize experiment →"}
            </button>
          </div>

          {error && (
            <p className="environment-error" role="alert">
              {error}
            </p>
          )}

          {message && (
            <p className="environment-message" role="status">
              {message}
            </p>
          )}
        </div>
      )}
    </section>
  );
}

export default memo(EnvironmentConfig);
