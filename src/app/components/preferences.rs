use std::str::FromStr;

use relm4::{
    adw,
    adw::prelude::{
        ActionRowExt, AdwWindowExt, BoxExt, CheckButtonExt, ComboRowExt, GtkWindowExt, IsA,
        MessageDialogExt, OrientableExt, PreferencesGroupExt, PreferencesPageExt,
        PreferencesRowExt, WidgetExt,
    },
    component::{AsyncComponent, AsyncComponentParts},
    gtk, AsyncComponentSender,
};

use crate::app::{config::settings, models};
use crate::fl;

#[derive(Debug)]
pub struct PreferencesModel {
    preference: models::Preference,
}

#[derive(Debug)]
pub enum PreferencesInput {
    SetColorScheme(models::ColorScheme),
    SetLanguage(models::Language),
}

#[relm4::component(pub async)]
impl AsyncComponent for PreferencesModel {
    type Init = gtk::Window;
    type Input = PreferencesInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesWindow {
            set_title: Some(fl!("preferences")),
            set_hide_on_close: true,
            set_default_size: (400, 600),
            set_resizable: false,
            set_transient_for: Some(&main_window),

            #[wrap(Some)]
            #[name = "overlay"]
            set_content = &adw::ToastOverlay {
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    append = &adw::HeaderBar {
                        set_show_end_title_buttons: true,
                    },
                    append = &adw::Clamp {
                        #[wrap(Some)]
                        set_child = &adw::PreferencesPage {
                            set_vexpand: true,
                            add = &adw::PreferencesGroup {
                                set_title: fl!("appearance"),
                                adw::ComboRow {
                                    set_title: fl!("color-scheme"),
                                    set_model: Some(&gtk::StringList::new(&[
                                        fl!("color-scheme-light"),
                                        fl!("color-scheme-dark"),
                                        fl!("color-scheme-default"),
                                    ])),
                                    set_selected: match model.preference.color_scheme {
                                        models::ColorScheme::Light => 0,
                                        models::ColorScheme::Dark => 1,
                                        models::ColorScheme::Default => 2,
                                    },
                                    connect_selected_notify[sender] => move |combo_row| {
                                        match combo_row.selected() {
                                            0 => sender.input_sender().send(
                                                PreferencesInput::SetColorScheme(models::ColorScheme::Light)
                                            ).unwrap_or_default(),
                                            1 => sender.input_sender().send(
                                                PreferencesInput::SetColorScheme(models::ColorScheme::Dark)
                                            ).unwrap_or_default(),
                                            _ => sender.input_sender().send(
                                                PreferencesInput::SetColorScheme(models::ColorScheme::Default)
                                            ).unwrap_or_default(),
                                        }
                                    },
                                }
                            },

                            add = &adw::PreferencesGroup {
                                set_title: fl!("language"),
                                adw::ActionRow {
                                    set_title: fl!("english"),
                                    add_prefix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        append = &gtk::Image {
                                            set_width_request: 64,
                                            set_height_request: 44,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            set_resource: Some("/com/github/forensicht/vimpeg/icons/en.png"),
                                        }
                                    },
                                    add_suffix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        #[name = "chk_language"]
                                        append = &gtk::CheckButton {
                                            set_active: matches!(model.preference.language, models::Language::English),
                                            connect_toggled[sender] => move |chk_button| {
                                                if chk_button.is_active() {
                                                    sender
                                                        .input_sender()
                                                        .send(PreferencesInput::SetLanguage(models::Language::English))
                                                        .unwrap_or_default();
                                                }
                                            },
                                        }
                                    },
                                },
                                adw::ActionRow {
                                    set_title: fl!("portuguese"),
                                    add_prefix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        append = &gtk::Image {
                                            set_width_request: 64,
                                            set_height_request: 44,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            set_resource: Some("/com/github/forensicht/vimpeg/icons/pt.png"),
                                        }
                                    },
                                    add_suffix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        append = &gtk::CheckButton {
                                            set_group: Some(&chk_language),
                                            set_active: matches!(model.preference.language, models::Language::Portuguese),
                                            connect_toggled[sender] => move |chk_button| {
                                                if chk_button.is_active() {
                                                    sender
                                                        .input_sender()
                                                        .send(PreferencesInput::SetLanguage(models::Language::Portuguese))
                                                        .unwrap_or_default();
                                                }
                                            },
                                        }
                                    },
                                },
                                adw::ActionRow {
                                    set_title: fl!("spanish"),
                                    add_prefix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        append = &gtk::Image {
                                            set_width_request: 64,
                                            set_height_request: 44,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            set_resource: Some("/com/github/forensicht/vimpeg/icons/es.png"),
                                        }
                                    },
                                    add_suffix = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        append = &gtk::CheckButton {
                                            set_group: Some(&chk_language),
                                            set_active: matches!(model.preference.language, models::Language::Spanish),
                                            connect_toggled[sender] => move |chk_button| {
                                                if chk_button.is_active() {
                                                    sender
                                                        .input_sender()
                                                        .send(PreferencesInput::SetLanguage(models::Language::Spanish))
                                                        .unwrap_or_default();
                                                }
                                            },
                                        }
                                    },
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(
        main_window: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut preference = models::Preference::default();

        if let Ok(settings_toml) = settings::get_settings() {
            let color_scheme = settings_toml.theme;
            let language = models::Language::from_str(settings_toml.language.as_str()).unwrap();
            preference = models::Preference::new(color_scheme, language);
        }

        let model = PreferencesModel { preference };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            PreferencesInput::SetColorScheme(color_scheme) => {
                settings::set_color_scheme(color_scheme);
                self.preference.color_scheme = color_scheme;
            }
            PreferencesInput::SetLanguage(language) => {
                self.preference.language = language;
                self.show_dialog(root);
            }
        }

        if let Err(error) = settings::save_preferences(&self.preference).await {
            tracing::error!("{error}");
        }

        self.update_view(widgets, sender);
    }
}

impl PreferencesModel {
    fn show_dialog(&self, root: &impl IsA<gtk::Window>) {
        let dialog = adw::MessageDialog::new(
            Some(root),
            Some(fl!("preferences")),
            Some(fl!("message-dialog")),
        );
        dialog.set_transient_for(Some(root));
        dialog.set_modal(true);
        dialog.set_destroy_with_parent(false);
        dialog.add_response("cancel", "_OK");
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        dialog.present();
    }
}
