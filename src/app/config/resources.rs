use anyhow::Ok;
use relm4::{
    adw::{gdk, gio, prelude::ApplicationExt},
    gtk,
    gtk::glib,
    main_adw_application,
};

use super::info::{APP_ID, APP_NAME};

pub(crate) fn init() -> anyhow::Result<()> {
    glib::set_application_name(APP_NAME);
    gio::resources_register_include!("resources.gresource")?;

    if let Some(display) = gdk::Display::default() {
        gtk::IconTheme::for_display(&display)
            .add_resource_path("/com/github/forensicht/vimpeg/icons");
    }
    gtk::Window::set_default_icon_name(APP_ID);

    let app = main_adw_application();
    app.set_resource_base_path(Some("/com/github/forensicht/vimpeg/app"));

    Ok(())
}
