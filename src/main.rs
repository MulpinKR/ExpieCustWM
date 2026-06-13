mod wm;
mod x11;
mod client;
mod layout;
mod config;
mod bindings;
mod wallpaper;
mod version;
mod animation;

use log::info;

fn main() -> anyhow::Result<()> {
    // Handle --version flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("expiecustWM {} \"{}\"", version::VERSION, version::CODENAME);
        return Ok(());
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    info!("expiecustWM {} \"{}\" starting", version::VERSION, version::CODENAME);

    let cfg = config::Config::load()?;
    let mut window_manager = wm::Wm::new(cfg)?;
    window_manager.run()?;
    Ok(())
}
