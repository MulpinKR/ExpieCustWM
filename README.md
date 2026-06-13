# ExpieCustWM

**ExpieCustWM** — an X11 window manager built for the fans of the *Scav Prototype* game series. Currently in active prototyping phase, evolving toward a fully customizable desktop environment with Wayland support on the roadmap.

## Philosophy

Everything that works can and should be customizable. No hardcoded behavior — if it's visible or functional, you can change it through configuration. The window manager adapts to you, not the other way around.

## Features

- **Tiling, floating, monocle** layout modes
- **Highly customizable** via Rhai scripting language (more languages planned)
- **Native wallpaper engine** — scales to fill, no external tools required
- **Keyboard-driven** — full bindings system with configurable modifiers
- **Dynamic workspace management** with EWMH compliance
- **Polybar integration** — automatic strut detection and layout adjustment
- **Minimal resource footprint** — written in Rust for performance and safety

## Customization

ExpieCustWM aims to exceed the flexibility of established window managers like Awesome WM, while remaining easier to configure than River. The entire UI — gaps, borders, colors, layouts, keybindings, wallpaper — is driven by a single configuration file with no recompilation required.

Future releases will support additional configuration languages, allowing users to write their setup in Lua, Python, JavaScript, or any other language via embedded runtimes.

## Roadmap

1. **Current phase** — X11 prototyping: core window management, wallpaper engine, EWMH, screenshot support
2. **Stabilization** — polish X11 implementation, fix edge cases, add window decorations, multi-monitor support
3. **Wayland** — full compositor implementation using `smithay` or equivalent, once the X11 version is mature
4. **Alpha release** — publicly available with both X11 and Wayland backends

## Building

```sh
git clone https://github.com/MulpinKR/ExpieCustWM.git
cd ExpieCustWM
cargo build --release
cp target/release/expiecustwm ~/.local/bin/
```

## Configuration

Configuration lives at `~/.config/expiecustwm/config.rhai`. See the [examples](./examples) directory for reference configurations.

## Dependencies

- Rust (latest stable)
- X11 server
- Optional: `maim` for screenshots, `polybar` for status bar, `rofi` for launcher

## License

MIT
