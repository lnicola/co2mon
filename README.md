# `co2mon`

[![Actions Status]][github actions] [![Latest Version]][crates.io] [![API docs]][docs.rs]

[Build Status]: https://api.travis-ci.com/lnicola/co2mon.svg?branch=master
[Actions Status]: https://github.com/lnicola/sd-notify/workflows/ci/badge.svg
[Latest Version]: https://img.shields.io/crates/v/co2mon.svg
[crates.io]: https://crates.io/crates/co2mon
[API docs]: https://docs.rs/co2mon/badge.svg
[docs.rs]: https://docs.rs/co2mon/

A driver for the Holtek CO₂ USB monitors, tested using a
[TFA-Dostmann AIRCO2TROL MINI][AIRCO2TROL MINI] sensor.

[AIRCO2TROL MINI]: https://www.tfa-dostmann.de/en/produkt/co2-monitor-airco2ntrol-mini/

## Permissions

On Linux, you need to be able to access the USB HID device. For that, you
can save the following `udev` rule to `/etc/udev/rules.d/60-co2mon.rules`:

```text
ACTION=="add|change", SUBSYSTEMS=="usb", ATTRS{idVendor}=="04d9", ATTRS{idProduct}=="a052", MODE:="0666"
```

Then reload the rules and trigger them:

```shell
udevadm control --reload
udevadm trigger
```

Note that the `udev` rule above makes the device accessible to every local user.

## Quick start

```shell
cargo run --example watch
```

## Releases

Release notes are available in [CHANGELOG.md](co2mon/CHANGELOG.md).

## Protocol

The USB HID protocol is not documented, but was [reverse-engineered][had] [before][revspace].

The implementation was inspired by [this one][co2mon].

[co2mon]: https://github.com/dmage/co2mon/
[had]: https://hackaday.io/project/5301/
[revspace]: https://revspace.nl/CO2MeterHacking

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   [http://www.apache.org/licenses/LICENSE-2.0][LICENSE-APACHE])
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
   [http://opensource.org/licenses/MIT][LICENSE-MIT])

at your option.

[LICENSE-APACHE]: http://www.apache.org/licenses/LICENSE-2.0
[LICENSE-MIT]: http://opensource.org/licenses/MIT
