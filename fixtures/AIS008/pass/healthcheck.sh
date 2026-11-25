#!/bin/sh
# OpenHands-style container port readiness probe (loopback only).
port="${PORT:-8765}"
while ! (echo >/dev/tcp/127.0.0.1/"$port") 2>/dev/null; do
  sleep 1
done