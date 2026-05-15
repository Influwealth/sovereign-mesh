#!/usr/bin/env bash
set -e

echo "Installing PQC Runtime..."

if ! command -v devbox &> /dev/null; then
  curl -fsSL https://get.jetpack.io/devbox | bash
fi

cd "$(dirname "$0")/../devbox"

echo "Loading Devbox environment..."
devbox shell -- echo "PQC environment ready."

echo "Installing liboqs..."
git clone https://github.com/open-quantum-safe/liboqs.git || true
cd liboqs
mkdir -p build && cd build
cmake -GNinja -DOQS_USE_OPENSSL=ON ..
ninja
sudo ninja install

echo "PQC Bootstrap Complete."

