#!/bin/sh
set -e

wasm-pack build --target web

echo "Build complete. Run a local server to view index.html"
echo "Example: python3 -m http.server"
