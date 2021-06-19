# `defmt-persist-print`

> A version of [`defmt-print`] that prints [`defmt`] logs saved by [`defmt-persist`] by supporting its framing encoding.

### Usage

Use `probe-rs-cli` to read a dump of persisted logs from the device and then pass them to `defmt-persist-print`.

1. Install `probe-rs-cli`:

```bash
cargo install probe-rs-cli --force
```

2. Install this tool via cargo:

```bash
cargo install
```

3. Read persisted logs (example):

```bash
export ELF_PATH=/path/to/the/binary.elf
export LOGS_FLASH_BASE_ADDR=08040000
export LOGS_FLASH_SIZE_BYTES=4096
export CHIP_NAME=STM32WB55CGUx
probe-rs-cli dump --chip $CHIP_NAME $LOGS_FLASH_BASE_ADDR $LOGS_FLASH_SIZE_BYTES | defmt-persist-print $ELF_PATH
```

### How It Works

First, the `probe-rs-cli` produces the following dump of the FLASH memory area where logs were saved:

```text
Addr 0x08040000: 0xefbeedfe
Addr 0x08040004: 0xbebafeca
Addr 0x08040008: 0xfdffecfd

------- >% SNIP %< -------

Addr 0x0804001c: 0xfcfe05fd
Addr 0x08040020: 0xffffffff
Addr 0x08040024: 0xffffffff
Read 10 words in 778.346µs
```

The dump above is then decoded into binary blob which is then decoded by [`defmt-persist`] into [`defmt`] frames.

These frames are then displayed as the actual log messages by [`defmt`] with the help of data
from an ELF firmware binary passed into the tool. The binary contains crucial information about log strings format.

Output example:

```text
────────────────────────────────────────────────────────────────────────────────
 INFO  Starting
└─ ble_quaternions::__cortex_m_rt_main @ src/bin/ble_quaternions.rs:598
 INFO  Running main
└─ ble_quaternions::run_main::task::{{closure}} @ src/bin/ble_quaternions.rs:102
 INFO  Waiting for PMIC
└─ tracksb::pmic::wait_init_pmic::{{closure}} @ src/pmic.rs:267
 INFO  Setting IMU enabled: true
└─ tracksb::pmic::{{impl}}::set_imu_power::{{closure}} @ src/pmic.rs:155
 INFO  Resetting IMU
└─ tracksb::imu::{{impl}}::reset_imu::{{closure}} @ src/imu.rs:127
 INFO  Updating battery level 95%
└─ ble_quaternions::update_battery_level::{{closure}} @ src/bin/ble_quaternions.rs:370
...
────────────────────────────────────────────────────────────────────────────────
```

[`defmt-persist`]: https://github.com/BlackbirdHQ/defmt-persist
[`defmt`]: https://github.com/knurling-rs/defmt
[`defmt-print`]: https://github.com/knurling-rs/defmt/tree/main/print

## Credits

This tool is based on [Knurling](https://github.com/knurling-rs)'s [`defmt-print`].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
