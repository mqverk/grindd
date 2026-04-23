# Networking Notes

- Bridge device and veth pairs are orchestrated using the `ip` command.
- Container-side veth is moved into a dedicated network namespace.
- NAT setup relies on host iptables masquerading rules.
