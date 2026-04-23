# Runtime Notes

- Namespace setup uses `unshare` flags for PID, UTS, mount, and network.
- `/proc` is mounted inside the container mount namespace when requested.
- Non-Linux platforms intentionally return a clear unsupported error for namespace execution.
