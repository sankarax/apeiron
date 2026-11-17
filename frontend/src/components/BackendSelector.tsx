import { memo, useState } from "react";
import { Cpu } from "lucide-react";
import { api } from "../services/api";
import type { ComputeBackend } from "../types/simulation";

interface Props {
  backend: ComputeBackend;
  activeBackend: ComputeBackend;
}

const labels: Record<ComputeBackend, string> = {
  auto_cpu: "Automatic CPU",
  gpu: "GPU (wgpu)",
  serial_cpu: "Serial CPU",
  parallel_cpu: "Parallel CPU",
  brute_force: "Brute force",
};

export const BackendSelector = memo(function BackendSelector({
  backend,
  activeBackend,
}: Props) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");

  async function changeBackend(value: ComputeBackend) {
    setPending(true);
    setError("");

    try {
      await api.setBackend(value);
    } catch {
      setError("Could not change backend");
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="backend-selector">
      <div className="backend-label">
        <Cpu size={15} />
        <span>Compute backend</span>
      </div>

      <select
        aria-label="Compute backend"
        value={backend}
        disabled={pending}
        onChange={(event) =>
          changeBackend(event.target.value as ComputeBackend)
        }
      >
        {Object.entries(labels).map(([value, label]) => (
          <option key={value} value={value}>
            {label}
          </option>
        ))}
      </select>

      <span className="backend-active">Active: {labels[activeBackend]}</span>

      {error && <span role="alert">{error}</span>}
    </div>
  );
});
