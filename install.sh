#!/usr/bin/env bash
set -e

echo "Installing ArcAI APT repository..."

curl -fsSL https://arcnaboo.github.io/arc/arc-archive-keyring.asc \
| sudo gpg --dearmor -o /usr/share/keyrings/arc.gpg

echo "deb [signed-by=/usr/share/keyrings/arc.gpg] https://arcnaboo.github.io/arc stable main" \
| sudo tee /etc/apt/sources.list.d/arc.list >/dev/null

sudo apt update || true
sudo apt install -y arc-ai

echo "ArcAI installed."
arc-ai --version
