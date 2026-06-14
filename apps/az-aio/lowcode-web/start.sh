#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."

echo "=== 启动 Lowcode API (8081) ==="
cd "$ROOT/plugins"
cargo run -p lowcode --bin lowcode-api &
API_PID=$!
sleep 3

echo "=== 启动 Lowcode 前端 (3000) ==="
cd "$SCRIPT_DIR"
TRUNK=/tmp/trunk
if [ ! -x "$TRUNK" ]; then
    echo "下载 trunk..."
    curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v0.21.13/trunk-x86_64-apple-darwin.tar.gz -o /tmp/trunk.tar.gz
    tar xzf /tmp/trunk.tar.gz -C /tmp trunk
fi
"$TRUNK" serve --port 3000 &
WEB_PID=$!

echo ""
echo "Lowcode 开发环境已启动:"
echo "  前端: http://localhost:3000"
echo "  API:  http://localhost:8081"
echo ""
echo "按 Ctrl+C 停止所有服务"

trap "kill $API_PID $WEB_PID 2>/dev/null" EXIT
wait
