# QNS Web Dashboard Guide

This guide explains how to run the QNS Web Dashboard, which consists of a Rust backend and a Next.js frontend.

## Prerequisites

- Rust (cargo)
- Node.js (npm)

## 1. Start the Backend Server

The backend runs as a subcommand of the `qns` CLI. It exposes a REST API at `http://127.0.0.1:3000`.

1. Open a terminal in the project root (`d:\SynProject\Engine\QC\nqc\qns`).
2. Run the server:

   ```bash
   cargo run --bin qns -- serve
   ```

   You should see: `Server listening on http://127.0.0.1:3000`

## 2. Start the Frontend Application

You have two options for the frontend:

### Option A: Single HTML File (Recommended for simplicity)

1. Navigate to `web/` directory.
2. Open `dashboard.html` in your browser.
   - Ensure the backend is running on `http://localhost:3000`.

### Option B: Next.js Application (For development)

1. Open a **new** terminal.
2. Navigate to the dashboard directory:

   ```bash
   cd web/dashboard
   ```

3. Start the development server:

   ```bash
   npm run dev
   ```

4. Open your browser and visit: `http://localhost:3000` (Note: Next.js might pick port 3001 if 3000 is taken by the backend).

**Important:** If the backend is running on port 3000, Next.js will likely default to port 3001. The frontend code currently points to `http://localhost:3000/api/simulate`. If the backend is on 3000 and frontend on 3001, it should work fine due to CORS.

## Usage

1. **QASM Editor:** Enter your QASM 2.0 code in the left pane.
   - Note: `include` statements are currently not supported.
   - Ensure explicit measurement indices if needed (e.g., `measure q[0] -> c[0]`).
2. **Run Simulation:** Click the "Run Simulation" button.
3. **Results:** View the measurement probabilities and execution metrics in the right pane.

## Troubleshooting

- **Server Error:** Check the backend terminal for panic messages or error logs.
- **Network Error:** Ensure the backend is running and accessible at `http://localhost:3000`.
- **Parse Error:** Verify your QASM syntax. The current parser has limited support for complex QASM features.
