
import pytest
import torch

from neural_reference import DATA_DIR, load_reference, compare


@pytest.mark.parametrize("name", ["small", "deep"])
@pytest.mark.parametrize("device", ["cpu", "cuda"])
def test_rust_equivalence(name, device):
    if device == "cuda" and not torch.cuda.is_available():
        pytest.skip("CUDA unavailable")

    torch.set_float32_matmul_precision("highest")

    reference = load_reference(
        DATA_DIR / f"{name}_reference.json",
        device,
    )

    result = compare(reference)

    assert result["matches"]
    assert result["max_error"] < 1e-6
    assert result["mean_error"] < 1e-6
    assert result["rmse"] < 1e-6
