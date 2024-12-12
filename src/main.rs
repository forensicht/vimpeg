#![windows_subsystem = "windows"]

mod app;

use anyhow::Result;
use app::config::{info::APP_ID, setup};
use relm4::RelmApp;

use app::App;

fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let _ = relm4::RELM_THREADS.set(num_cpus::get());

    let app = RelmApp::new(APP_ID);
    setup::init()?;
    app.run_async::<App>(());
    Ok(())
}
