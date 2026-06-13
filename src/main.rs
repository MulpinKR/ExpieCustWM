mod wm;
mod x11;
mod client;
mod layout;
mod config;
mod bindings;

use log::info;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    info!("expiecustWM starting");

    let cfg = config::Config::load()?;
    let mut window_manager = wm::Wm::new(cfg)?;
    window_manager.run()?;
    Ok(())
}
