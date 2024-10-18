# Simple socks5 proxy

A standalone, lightweight socks5 proxy client (without `openssl`) that requires no additional server-side installations (based on `sshd`). It allows you to quickly set up a local socks5 listener and forward traffic through the target server.

Built on [russh](https://github.com/Eugeny/russh) and [socks5-impl](https://github.com/tun2proxy/socks5-impl).

### Usage:

```
cargo run -- ${USERNAME}@${HOST} -k ${PATH_TO_LOCAL_PRIVATE_KEY} -l 127.0.0.1:${PORT}
```

This is equivalent to:
```
ssh -D ${PORT} -i ${PATH_TO_LOCAL_PRIVATE_KEY} ${USERNAME}@${HOST}
```