# LEDSRACE firmware

[![Instagram](https://img.shields.io/badge/Instagram-%23E4405F.svg?style=for-the-badge&logo=Instagram&logoColor=white)](https://www.instagram.com/leds.race/)
[![Licence](https://img.shields.io/github/license/Ileriayo/markdown-badges?style=for-the-badge)](./LICENSE)

This is the firmware repository for the LEDSRACE circuit projects. LEDSRACE is based on the ESP32-C3-mini. Official firmware is written in Rust and makes use of the [Embassy ecosystem](https://embassy.dev).

![](img/ledsrace.jpg)

## Crates

The project is organized in two separate crates:

Crate | Description
--- | ---
[ledsrace](/ledsrace-application/) | Application crate that compiles to a binary to run on the hardware
[ledsrace-core](/ledsrace-core/) | Hardware independent core crate. For logic and shared functionality and tests.

The `ledsrace` crate depends on `ledsrace-core` and has the actual programs that can be run on the LEDSRACE board.

## What can it do?

### Play back the F1 Grand Prix

We can play the 2023 and 2024 Formula 1 races using datasets from openf1.org. The datasets provide 3.7Hz positions updates for all cars on the track. We have downloaded and processed the dataset so that it can be included in the firmware in an efficient binary format.

https://github.com/user-attachments/assets/71ff5e01-a4fb-4412-a468-66177dd372a8

### Run other animations

We provide a set of other animations you can run on the board.

https://github.com/user-attachments/assets/ec318ba4-40ef-46bc-8056-d427e40b6a43

### Program it yourself.

You can use this repository as a starting point, or write it yourself from scratch in Rust or C.

## Functionality

- 216 Smart LEDS HD108-2020.
- 1x temperature sensor for safety
- Wifi (currently not implemented in firmware)
- USB Connection (currently not implemented in firmware)

## How to load a new program?

See [FLASH GUIDE BROWSER](docs/FLASH_GUIDE_BROWSER.md) for instructions.


## Create bin file for distribution

Binary file including bootloader and partition table (in case of bricked board)

```bash
espflash save-image --chip esp32c3 target/riscv32imc-unknown-none-elf/release/ledsrace firmware-ledsrace.bin --merge

espflash save-image --chip esp32c3 target/riscv32imc-unknown-none-elf/release/kingsday firmware-kingsday.bin --merge

## Flash this binary file at 0x0 address (since it includes bootloader and partition table)
espflash write-bin 0x0 ./firmware-ledsrace.bin
```

Binary file including application only

```bash
espflash save-image --chip esp32c3 target/riscv32imc-unknown-none-elf/debug/f1-hardware firmware.bin

## Flash this binary file at 0x10000 address
espflash write-bin 0x10000 ./firmware.bin
```

## License

LEDSRACE firmware is licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

## Disclaimer

LEDSRACE is an unofficial project and is not associated in any way with the Formula 1 companies. F1, FORMULA ONE, FORMULA 1, FIA FORMULA ONE WORLD CHAMPIONSHIP, GRAND PRIX and related marks are trade marks of Formula One Licensing B.V.
