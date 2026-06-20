# Mandala-Smart-Book

## Build

```bash
cross build --target aarch64-unknown-linux-gnu --release
```

Copy `target/aarch64-unknown-linux-gnu/release/Mandala-Smart-Book` to the Raspberry Pi, make it
executable with `chmod +x Mandala-Smart-Book` and run. If run through `ssh` it will be necessary to
run `export WAYLAND_DISPLAY=wayland-0` or you get an error:
`Misc("neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.")`.
