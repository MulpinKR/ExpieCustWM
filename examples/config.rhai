#{
    // Key modifier (Mod1=Alt, Mod4=Win/Super)
    mod_key: "Mod4",

    // Terminal emulator
    terminal: "konsole",

    // Application launcher
    launcher: "rofi -show drun",

    // Gaps and borders (pixels)
    gap: 4,
    border_width: 2,

    // Master-stack layout
    master_count: 1,
    master_ratio: 0.55,

    // Wallpaper (PNG path, or use wallpaper_color for solid)
    wallpaper: "/home/mulpin/Projects/expiecustwm/assets/expiecustWM.png",
    // wallpaper_color: "#1a1a2e",

    // Auto-start programs
    // System tray: polybar has built-in tray, or use stalonetray/trayer
    // Background apps (Telegram, etc.) sit in the system tray
    autostart: [
        // Keyboard layout (US + RU + UA, toggle with Win+Space)
        "setxkbmap -layout us,ru,ua -option grp:win_space_toggle",

        // Status bar with system tray and bluetooth module
        "polybar main",

        // Notification daemon (install: sudo apt-get install dunst libnotify-bin)
        "dunst",

        // Background apps with tray icons (uncomment as needed)
        // "telegram-desktop -startintray",
        // "discord --start-minimized",
        // "thunderbird",

        // Bluetooth tray icon (install: sudo apt-get install blueman)
        // "blueman-applet",

        // Version update check
        "/home/mulpin/.local/bin/expiecustwm-check-update",
    ],

    // Status notifier / system tray
    // Polybar's built-in tray (tray-position = right in polybar config.ini)
    // handles icons from Telegram, Discord, Bluetooth, etc.

    // Default keybindings (see src/config.rs for the full list)
    // keybindings: [],
}
