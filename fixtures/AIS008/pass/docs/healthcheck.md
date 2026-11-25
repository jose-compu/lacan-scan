# Port probe (documentation)

Describes the usual container health check:

```sh
while ! (echo >/dev/tcp/127.0.0.1/"$port") 2>/dev/null; do sleep 1; done
```

Loopback-only; not an interactive reverse shell.