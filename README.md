# unifiers
CLI to discuss with Unifi API, first use case is to enable/disable Ethernet port (e.g. disable Ethernet port for children at night using Cron or other scheduler).

Unifi API CLI developed in Rust.
Copy unifiers.toml.sample to unifiers.toml and change your settings.

Only support changing port profile up and down for the moment.

Usage:
# ./unifiers --help
Usage: unifiers --config-file-path <CONFIG_FILE_PATH> --port-number <PORT_NUMBER> <PROFILE>

Arguments:
  <PROFILE>  [possible values: up, down]

Options:
  -c, --config-file-path <CONFIG_FILE_PATH>
  -p, --port-number <PORT_NUMBER>            Port number to change profile
  -h, --help                                 Print help
  -V, --version                              Print version
