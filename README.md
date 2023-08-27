# unifiers
CLI to discuss with Unifi API, first use case is to enable/disable Ethernet port (e.g. disable Ethernet port for children at night using Cron or other scheduler).

Unifi API CLI developed in Rust.
Copy unifiers.toml.sample to unifiers.toml and change your settings.

Only support changing port profile up and down for the moment.

Usage:
# ./unifiers --port-number 9 --up
