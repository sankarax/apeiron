
from fastapi.testclient import TestClient

from app.main import app


def test_health():
    with TestClient(app) as client:
        response = client.get("/api/health")

        assert response.status_code == 200
        assert response.json()["status"] == "healthy"


def test_reset():
    with TestClient(app) as client:
        response = client.post(
            "/api/simulation/reset",
            json={"config": {"seed": 42}},
        )

        assert response.status_code == 200
        assert response.json()["tick"] == 0
        assert response.json()["population"] == 200


def test_manual_step():
    with TestClient(app) as client:
        client.post(
            "/api/simulation/reset",
            json={},
        )

        response = client.post(
            "/api/simulation/step",
            json={"ticks": 100},
        )

        assert response.status_code == 200
        assert response.json()["tick"] == 100
        assert response.json()["population"] == 387


def test_invalid_speed():
    with TestClient(app) as client:
        response = client.post(
            "/api/simulation/speed",
            json={"ticks_per_second": -10},
        )

        assert response.status_code == 422


def test_invalid_config():
    with TestClient(app) as client:
        response = client.post(
            "/api/simulation/reset",
            json={"config": {"width": -100}},
        )

        assert response.status_code == 400


def test_metrics():
    with TestClient(app) as client:
        response = client.get(
            "/api/simulation/metrics"
        )

        assert response.status_code == 200
        assert "average_speed" in response.json()


def test_websocket():
    with TestClient(app) as client:
        with client.websocket_connect(
            "/ws/simulation"
        ) as websocket:
            data = websocket.receive_json()

            assert data["type"] == "snapshot"
            assert "world" in data
            assert "metrics" in data
            assert "status" in data
