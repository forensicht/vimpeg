use anyhow::{self, Context};
use relm4::gtk;

use super::{actions, resources, settings};

pub fn init() -> anyhow::Result<()> {
    gtk::init()?;

    // Enable logging
    let directory_log = std::env::current_dir()
        .context("Could not get current directory.")?
        .join("log");
    let file_appender = tracing_appender::rolling::daily(directory_log, "vimpeg.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_file(false)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(non_blocking)
        .init();

    resources::init()?;
    relm4_icons::initialize_icons();
    actions::init();
    settings::init()?;

    Ok(())
}
