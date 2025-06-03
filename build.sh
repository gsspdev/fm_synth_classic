#!/bin/bash

# FM Synthesizer WebAssembly Build Script

set -e  # Exit on error

echo "🎵 FM Synthesizer WASM Builder"
echo "=============================="

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed!"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg/

# Build the WebAssembly module
echo "🔨 Building WebAssembly module..."n
wasm-pack build --target web --out-dir pkg

echo "✅ Build complete!"
echo ""
echo "📁 Output files:"
echo "  - pkg/fm_synth.js (WASM module)"
echo "  - pkg/fm_synth_bg.wasm (WASM binary)"
echo ""
echo "🚀 To run the web version:"
echo "  1. Start a local server:"
echo "     python3 -m http.server 8000"
echo "  2. Open http://localhost:8000/index.html"
echo ""
echo "🎹 Happy synthesizing!"
