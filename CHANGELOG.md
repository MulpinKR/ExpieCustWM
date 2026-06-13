# Changelog

## 0.0.0 "Cave" (2026-06-13)

First prototype release.

### Added
- X11 window manager core (Rust + Rhai config)
- Tiling, floating, monocle layout modes
- Keyboard-driven bindings system
- Native wallpaper engine with scale-to-fill
- Multi-workspace support (5 workspaces, Mod4+N to switch)
- EWMH compliance (_NET_WM_, _NET_CLIENT_LIST, etc.)
- Bluetooth module (toggle + rofi menu)
- Volume/ Mic mute keys (XF86Audio*)
- Brightness control via xrandr (XF86MonBrightness*)
- `maim` screenshot integration (Print / Mod4+Shift+P)
- Polybar integration (automatic strut detection)
- Config reload at runtime (Mod4+Shift+C)
- Dunst notification daemon autostart
- Version check script (startup update notification)

### Fixed
- Wallpaper scaling (now fills screen, feh --bg-fill style)
- Print key binding (no longer requires Mod4)
- Screenshot save path (now ~/Pictures/Screenshots/)
- Scripts use full paths (no PATH dependency)
- No external notify-send dependency

### Config
- Default polybar config included in repository
- Example config.rhai with annotations
- Helper scripts: expiecustwm-brightness, expiecustwm-bluetooth

### Planned
- Wayland compositor (post-stabilization)
- Additional configuration languages (Lua, Python, JS)
- Window decorations and title bars
- Multi-monitor support
- Alpha release with both X11 and Wayland backends
