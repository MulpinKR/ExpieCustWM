# Changelog

## 0.0.2 "Dune" (2026-06-13)

### Added
- Settings menu via rofi (`OpenMenu` action, `Super+Shift+d`)
- `WM_CHANGE_STATE` handler (iconify/deiconify)
- `_NET_ACTIVE_WINDOW` handler (window activation from tray)
- `withdrawn` flag for Client — windows can be hidden without destroying frame
- GPU module placeholder for polybar

### Fixed
- Wallpaper: switched to Desktop window (`override_redirect` + `_NET_WM_WINDOW_TYPE_DESKTOP`) — works under picom
- Tray show/hide: UnmapNotify no longer destroys client window, re-map on MapRequest works
- Telegram show-from-tray: handled via `WM_CHANGE_STATE` and `_NET_ACTIVE_WINDOW`
- `eprintln!` replaced with `log::info!` — no buffered output in log
- Rhai config syntax: nested maps use `#{ }` instead of `{ }`

### Changed
- Keybindings: `Super+d` → app launcher, `Super+Shift+d` → settings menu
- Polybar: separators between all right modules, bluetooth script shows BT/ON/BT: name
- Autostart: added `wireplumber` (PipeWire session manager)
- Version bumped to 0.0.2, codename "Dune"

## 0.0.1_5 (2026-06-13)

### Added
- Focus-follows-mouse (auto-focus on hover)

### Fixed
- Rofi monochrome theme: inverted to white bg with black text, removed transparency
- Launcher changed to `-show drun` (desktop entries instead of PATH)
- Font changed to DejaVu Sans Mono (Fira Code wasn't installed)
- Polybar tray reordered: next to BT module with separator

## 0.0.1 "Cave" (2026-06-13)

### Added
- Workspace slide animation (left/right, 12 frames)
- Window open animation (slide-down from -1000px)
- Window close animation (slide-up)
- Rofi black & white theme (monochrome.rasi)
- EWMH: _NET_CURRENT_DESKTOP now updates live, _NET_DESKTOP_NAMES set
- Fastfetch integration: WM name shown as "expiecustwm 0.0.1"

### Fixed
- Workspace highlighting lag (switched to xprop -spy event-based polling)
- Polybar bluetooth text (removed icon fonts, use "BT"/"off")
- Separator between BT and wifi modules in polybar

### Added
- Polybar workspace module (custom/script, works with any EWMH WM)
- Polybar separator module |
- Rofi theme (examples/rofi/monochrome.rasi)

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
- External notify-send dependency removed

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
