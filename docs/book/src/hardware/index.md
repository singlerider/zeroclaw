# Hardware & Peripherals Docs

For board integration, firmware flow, and peripheral architecture.

ZeroClaw's hardware subsystem enables direct control of microcontrollers and peripherals via the `Peripheral` trait. Each board exposes tools for GPIO, ADC, and sensor operations, allowing agent-driven hardware interaction on boards like STM32 Nucleo, Raspberry Pi, and ESP32. See [hardware-peripherals-design.md](hardware-peripherals-design.md) for the full architecture.

## Entry Points

- Architecture and peripheral model: [hardware-peripherals-design.md](hardware-peripherals-design.md)
- Add a new board/tool: [adding-boards-and-tools.md](adding-boards-and-tools.md)
- Nucleo setup: [nucleo-setup.md](nucleo-setup.md)
- Arduino Uno Q setup: [arduino-uno-q-setup.md](arduino-uno-q-setup.md)

## Datasheets

For per-board pin maps and electrical characteristics, see the vendor docs:

- STM32 Nucleo-F401RE: <https://www.st.com/en/evaluation-tools/nucleo-f401re.html>
- Arduino Uno: <https://docs.arduino.cc/hardware/uno-rev3>
- ESP32: <https://www.espressif.com/sites/default/files/documentation/esp32_datasheet_en.pdf>
