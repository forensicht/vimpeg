use anyhow;
use relm4::gtk;

use super::{actions, resources, settings};

pub fn init() -> anyhow::Result<()> {
    gtk::init()?;

    // Enable logging
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::ERROR)
        .init();
    resources::init()?;
    relm4_icons::initialize_icons();
    actions::init();
    settings::init()?;

    Ok(())
}
