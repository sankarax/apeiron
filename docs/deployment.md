# Deployment and testing

## Native installation (Windows PowerShell)

Requirements: Rust stable with edition 2024 support, Python 3.10 or newer, Node.js 22, Docker with Compose, Ollama and a compatible GPU driver for native GPU execution. Commands assume the repository root and an activated root virtual environment.

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install -r backend/requirements.txt
cd engine
maturin develop --release
cd ../frontend
npm ci
cd ..
ollama pull qwen3:8b
.\scripts\start.ps1
```

The script starts PostgreSQL and Redis, ensures the Ollama service is available, and opens backend and frontend processes. Open `http://127.0.0.1:5173/`. The API is on port 8000. Stop services with `.\scripts\stop.ps1` and inspect them with `.\scripts\health.ps1`. Docker volumes retain database records. The default database credentials are for local development only; override `APEIRON_DATABASE_URL` and `APEIRON_REDIS_URL` as needed.

## Browser build

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
.\scripts\build-browser.ps1
cd frontend
npm ci
npx tsc -b
npm run lint
npx vite build --mode browser
npx vite preview --host 127.0.0.1 --port 4173
```

Open `http://127.0.0.1:4173/`. The build script copies generated bindings to `frontend/wasm`. The output is `frontend/dist`. Do not commit generated dependencies or compiled outputs. Browser mode is selected at build time.

## Public hosting

Deploy only the static `frontend/dist` directory to a static hosting provider such as Cloudflare Pages. A local build followed by direct upload avoids requiring Rust tooling on the hosting service. The intended deployment uses a free static hosting tier, subject to its current limits. No database, GPU server, model server or API key is required. No live deployment URL has been assigned yet.

## Tests

From the root, with the Python environment active:

```powershell
cd engine
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib
cargo test --tests
cd ..
docker compose up -d --wait
cd backend
python -m pytest tests
cd ../frontend
npm ci
npm audit
npx tsc -b
npm run lint
npx vite build --mode browser
```

The integration suite covers backend equivalence, sensory equivalence, neural behavior and evolution, recurrent integration and checkpoint replay. The Python suite covers health, reset, manual steps, invalid inputs, metrics and WebSocket snapshots. PyTorch experiments require a separate compatible PyTorch installation; CUDA is needed for CUDA benchmarks, not the browser or native Rust engine.

## Release checks

Test the production browser preview with native services stopped. Verify start, pause, step, reset, all controllers, predation, organism selection, switching selections, lineage, charts, multiverse and recorded research. Then test native CPU and GPU execution, checkpoints, replay, live research and persistence. Recheck the panels modified during lint cleanup. CI execution and final runtime tests are pending. Choose a license and verify the repository contains no private credentials before publication.
