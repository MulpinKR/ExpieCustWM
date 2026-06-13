use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub mods: Vec<String>,
    pub key: String,
    pub action: BindingAction,
}

#[derive(Debug, Clone)]
pub enum BindingAction {
    FocusNext,
    FocusPrev,
    MoveToMaster,
    SwapWithMaster,
    ToggleFloating,
    CloseWindow,
    LayoutTiling,
    LayoutFloating,
    LayoutMonocle,
    Spawn(String),
    ReloadConfig,
    Quit,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub config_path: Option<PathBuf>,
    pub mod_key: String,
    pub terminal: String,
    pub launcher: String,
    pub gap: u32,
    pub border_width: u32,
    pub border_focus: String,
    pub border_normal: String,
    pub master_count: usize,
    pub master_ratio: f64,
    pub autostart: Vec<String>,
    pub keybindings: Vec<KeyBinding>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs_config_dir()
            .map(|p| p.join("expiecustwm/config.rhai"))
            .filter(|p| p.exists());

        match config_path {
            Some(path) => Self::from_file(&path),
            None => {
                log::info!("No config file found, using defaults");
                Ok(Self::default())
            }
        }
    }

    pub fn reload(&self) -> anyhow::Result<Self> {
        match &self.config_path {
            Some(path) if path.exists() => Self::from_file(path),
            _ => {
                log::info!("Config file not found, keeping current config");
                Ok(self.clone())
            }
        }
    }

    fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let _script = std::fs::read_to_string(path)?;
        let mut engine = rhai::Engine::new();
        let mut cfg = Self::default();

        engine.register_fn("set_mod_key", {
            let _cfg_key = cfg.mod_key.clone();
            move |_k: &str| { /* handled below */ }
        });

        match engine.eval_file::<rhai::Dynamic>(path.into()) {
            Ok(val) => {
                if let Some(map) = val.try_cast::<rhai::Map>() {
                    if let Some(v) = map.get("mod_key").and_then(|v| v.clone().try_cast::<String>()) {
                        cfg.mod_key = v;
                    }
                    if let Some(v) = map.get("terminal").and_then(|v| v.clone().try_cast::<String>()) {
                        cfg.terminal = v;
                    }
                    if let Some(v) = map.get("launcher").and_then(|v| v.clone().try_cast::<String>()) {
                        cfg.launcher = v;
                    }
                    if let Some(v) = map.get("gap").and_then(|v| v.clone().try_cast::<i64>()) {
                        cfg.gap = v as u32;
                    }
                    if let Some(v) = map.get("border_width").and_then(|v| v.clone().try_cast::<i64>()) {
                        cfg.border_width = v as u32;
                    }
                    if let Some(v) = map.get("master_count").and_then(|v| v.clone().try_cast::<i64>()) {
                        cfg.master_count = v as usize;
                    }
                    if let Some(v) = map.get("master_ratio").and_then(|v| v.clone().try_cast::<f64>()) {
                        cfg.master_ratio = v;
                    }
                    if let Some(v) = map.get("autostart").and_then(|v| v.clone().try_cast::<rhai::Array>()) {
                        cfg.autostart = v.into_iter().filter_map(|e| e.try_cast::<String>()).collect();
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to eval config script: {}", e);
            }
        }

        // Use terminal and launcher from config
        for kb in &mut cfg.keybindings {
            if let BindingAction::Spawn(ref cmd) = kb.action {
                if cmd == "xterm" {
                    kb.action = BindingAction::Spawn(cfg.terminal.clone());
                } else if cmd == "dmenu_run" {
                    kb.action = BindingAction::Spawn(cfg.launcher.clone());
                }
            }
        }

        cfg.config_path = Some(path.clone());
        log::info!("Loaded config from: {:?}", path);
        Ok(cfg)
    }

    pub fn default() -> Self {
        Self {
            config_path: None,
            mod_key: "Mod4".into(),
            terminal: "xterm".into(),
            launcher: "dmenu_run".into(),
            gap: 4,
            border_width: 2,
            border_focus: "#5294e2".into(),
            border_normal: "#222222".into(),
            master_count: 1,
            master_ratio: 0.55,
            autostart: vec![],
            keybindings: default_bindings(),
        }
    }
}

fn default_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding { mods: vec!["Mod4".into()], key: "Return".into(), action: BindingAction::Spawn("xterm".into()) },
        KeyBinding { mods: vec!["Mod4".into()], key: "d".into(), action: BindingAction::Spawn("dmenu_run".into()) },
        KeyBinding { mods: vec!["Mod4".into()], key: "j".into(), action: BindingAction::FocusNext },
        KeyBinding { mods: vec!["Mod4".into()], key: "k".into(), action: BindingAction::FocusPrev },
        KeyBinding { mods: vec!["Mod4".into()], key: "Tab".into(), action: BindingAction::FocusNext },
        KeyBinding { mods: vec!["Mod4".into(), "Shift".into()], key: "Return".into(), action: BindingAction::SwapWithMaster },
        KeyBinding { mods: vec!["Mod4".into()], key: "f".into(), action: BindingAction::ToggleFloating },
        KeyBinding { mods: vec!["Mod4".into()], key: "q".into(), action: BindingAction::CloseWindow },
        KeyBinding { mods: vec!["Mod4".into()], key: "t".into(), action: BindingAction::LayoutTiling },
        KeyBinding { mods: vec!["Mod4".into()], key: "m".into(), action: BindingAction::LayoutMonocle },
        KeyBinding { mods: vec!["Mod4".into()], key: "s".into(), action: BindingAction::LayoutFloating },
        KeyBinding { mods: vec!["Mod4".into(), "Shift".into()], key: "c".into(), action: BindingAction::ReloadConfig },
        KeyBinding { mods: vec!["Mod4".into(), "Shift".into()], key: "q".into(), action: BindingAction::Quit },
    ]
}

fn dirs_config_dir() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME").ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME").ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })
}
