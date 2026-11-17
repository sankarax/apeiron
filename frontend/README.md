# Apeiron frontend

React and TypeScript interface for the native and browser engines.

Build the WebAssembly package from the repository root with `./scripts/build-browser.ps1` on Windows before building either frontend mode. Install dependencies with `npm ci` in this directory.

- `npm run dev`: local mode (requires the FastAPI backend)
- `npx vite --mode browser`: standalone browser mode
- `npm run build`: local production build
- `npx tsc -b && npx vite build --mode browser`: browser production build

The browser edition uses Web Workers and the serial Rust WebAssembly engine. Native GPU computation and live Ollama research require the local installation.
