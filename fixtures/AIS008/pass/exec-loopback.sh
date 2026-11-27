#!/bin/bash
if (exec 3<>"/dev/tcp/127.0.0.1/${port}") 2>/dev/null; then :; fi
if (exec 3<>/dev/tcp/127.0.0.1/8080) 2>/dev/null; then :; fi
exec 3<>/dev/tcp/localhost/8080
exec 3<>/dev/tcp/::1/8080
exec 3>/dev/tcp/127.0.0.1/8080
exec 3</dev/tcp/127.0.0.1/8080
exec 3<>/dev/tcp/127.0.0.1/8080 4>&3