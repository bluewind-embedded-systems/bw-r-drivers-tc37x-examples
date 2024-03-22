# bw-r-driver-tc37x-examples

The Rust code is structured to work with the [defmt](https://github.com/knurling-rs/defmt) framework
and [tricore-probe](https://github.com/veecle/tricore-probe), but you can use
also other debugging tools, like *UDE* or *MemTool* (see *Run* section below).

Examples in this repo:

| Name  | Description                 | Peripherals |
|-------|-----------------------------|-------------|
| blink | Blinking LED. Button press. | GPIO        |
| can   | CAN bus send and receive    | CAN         |

You can find more information on [bluewind-embedded-systems.github.io/bw-r-drivers-tc37x](https://bluewind-embedded-systems.github.io/bw-r-drivers-tc37x)

## Build

Change the current directory to the example you want to build and run:

```
cargo +tricore build --target=tc162-htc-none
```

This will generate an ELF file in the `target/tc162-htc-none/debug` directory,
which you can load into the target.

## Run

You have many options to flash and run these examples:

- [tricore-probe](https://github.com/veecle/tricore-probe)
- [Infineon MemTool](https://www.infineon.com/cms/en/tools/aurix-tools/free-tools/infineon/)
- [Universal Debug Engine](https://www.pls-mc.com/products/universal-debug-engine/)
