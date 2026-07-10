#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="alex-fsw0-quicksend"
OUTPUT_DIR="${1:-"$ROOT_DIR/release/quicksend"}"

if [[ -z "$OUTPUT_DIR" || "$OUTPUT_DIR" == "/" || "$OUTPUT_DIR" == "$ROOT_DIR" ]]; then
  echo "Refusing unsafe release output directory: ${OUTPUT_DIR:-<empty>}" >&2
  exit 64
fi

echo "Building frontend assets..."
npm --prefix "$ROOT_DIR/frontend" ci
npm --prefix "$ROOT_DIR/frontend" run build

echo "Building release binary..."
cargo build --release

echo "Assembling release directory at $OUTPUT_DIR..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/bin" "$OUTPUT_DIR/public"

cp "$ROOT_DIR/target/release/$APP_NAME" "$OUTPUT_DIR/bin/$APP_NAME"
cp -R "$ROOT_DIR/frontend/dist/." "$OUTPUT_DIR/public/"

cat > "$OUTPUT_DIR/run.sh" <<'RUN_SCRIPT'
#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export HOST="${HOST:-0.0.0.0}"
export PORT="${PORT:-8080}"
export FRONTEND_DIST_DIR="${FRONTEND_DIST_DIR:-$SCRIPT_DIR/public}"

exec "$SCRIPT_DIR/bin/alex-fsw0-quicksend"
RUN_SCRIPT

chmod +x "$OUTPUT_DIR/run.sh"
cp "$ROOT_DIR/.env.example" "$OUTPUT_DIR/.env.example"

echo "Release bundle ready: $OUTPUT_DIR"
echo "Run it with: cd $OUTPUT_DIR && ./run.sh"
