# Storage Notes

- Overlay layout uses lowerdir (image rootfs), upperdir, workdir, and merged mountpoint.
- `mount -t overlay` is used on Linux to create a writable container layer.
- `umount` cleanup is expected during container teardown.
