#{
    // Key modifier (Mod1=Alt, Mod4=Win/Super)
    mod_key: "Mod4",

    // Terminal emulator
    terminal: "konsole",

    // Application launcher
    launcher: "rofi -show run",

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
    autostart: [
        "setxkbmap -layout us,ru,ua -option grp:win_space_toggle",
        "polybar main",
        "dunst",
    ],

    // Default keybindings (see src/config.rs for the full list)
    // keybindings: [],
}
