# alex-fsw0-quicksend

A small Axum + React message sender. The backend serves the React build, accepts
`POST /api/send`, validates the payload, applies a per-IP rate limit, and sends
messages through the myClawTeam platform email proxy when credentials are
configured.

## Development

Install frontend dependencies:

```bash
npm --prefix frontend ci
```

Build the frontend:

```bash
npm --prefix frontend run build
```

Run the backend from the repository root:

```bash
cargo run
```

The server listens on `0.0.0.0:8080` by default and serves the built frontend
from `frontend/dist`.

Useful checks:

```bash
npm --prefix frontend run build
npm --prefix frontend run e2e
cargo test
```

## API

`GET /health` returns a JSON health response with email-proxy and rate-limit
configuration status.

`POST /api/send` accepts:

```json
{
  "recipient_email": "person@example.com",
  "subject": "Hello",
  "message": "Message body"
}
```

Successful requests return `202 Accepted`. Validation failures return `400` with
field-level errors. Rate-limited requests return `429` and include a
`Retry-After` header. Email proxy failures return a structured error without
exposing credentials.

## Environment

Copy `.env.example` for local reference and provide values through the process
environment in production.

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `HOST` | No | `0.0.0.0` | IP address the Axum server binds to. |
| `PORT` | No | `8080` | Port the Axum server listens on. |
| `FRONTEND_DIST_DIR` | No | `frontend/dist` | Directory containing `index.html` and static frontend assets. |
| `MCTAI_EMAIL_URL` | For delivery | none | myClawTeam email proxy endpoint. If absent, sends are accepted but delivery is skipped. |
| `MCTAI_EMAIL_APP_TOKEN` | For delivery | none | Server-side bearer token for the myClawTeam email proxy. Never expose this to browser code. |
| `RATE_LIMIT_REQUESTS_PER_WINDOW` | No | `5` | Number of send attempts allowed per client IP per window. Must be greater than zero. |
| `RATE_LIMIT_WINDOW_SECONDS` | No | `60` | Rate-limit window length in seconds. Must be greater than zero. |

The backend fails startup if `FRONTEND_DIST_DIR/index.html` is missing. Build the
frontend first or point `FRONTEND_DIST_DIR` at the deployed static asset
directory.

## Self-Hosted Build

Create a self-contained release directory:

```bash
./scripts/build-release.sh
```

The default output is `release/quicksend`:

```text
release/quicksend/
  bin/alex-fsw0-quicksend
  public/
  .env.example
  run.sh
```

Run the bundle:

```bash
cd release/quicksend
HOST=0.0.0.0 PORT=8080 MCTAI_EMAIL_URL=... MCTAI_EMAIL_APP_TOKEN=... ./run.sh
```

`run.sh` sets `FRONTEND_DIST_DIR` to the bundled `public` directory unless it is
already provided. It does not load `.env.example`; export real environment
values through your process manager or shell.
