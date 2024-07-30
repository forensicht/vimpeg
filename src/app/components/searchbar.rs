use std::path::PathBuf;

use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender, Controller},
    gtk,
    gtk::prelude::{BoxExt, ButtonExt, EditableExt, EntryExt, OrientableExt, WidgetExt},
    Component, ComponentController,
};
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_icons::icon_names;

use crate::fl;

pub struct SearchBarModel {
    open_dialog: Controller<OpenDialog>,
    stopped: bool,
    file_path: PathBuf,
}

#[derive(Debug)]
pub enum SearchBarInput {
    StartSearch,
    SearchCompleted,
    OpenFileRequest,
    OpenFileResponse(PathBuf),
    Ignore,
}

#[derive(Debug)]
pub enum SearchBarOutput {
    StartSearch(PathBuf),
    Notify(String, u32),
}

#[relm4::component(pub async)]
impl AsyncComponent for SearchBarModel {
    type Init = ();
    type Input = SearchBarInput;
    type Output = SearchBarOutput;
    type CommandOutput = ();

    view! {
       gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_hexpand: true,
            set_spacing: 6,
            set_margin_start: 24,
            set_margin_end: 24,

            append = &gtk::Entry {
                #[watch]
                set_text: &model.file_path.to_str().unwrap_or_default(),
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_placeholder_text: Some(fl!("directory")),
                set_secondary_icon_name: Some(icon_names::FOLDER_OPEN_FILLED),
                set_secondary_icon_tooltip_text: Some(fl!("select-directory")),
                connect_icon_release[sender] => move |_, icon_position| {
                    if icon_position == gtk::EntryIconPosition::Secondary {
                        sender.input(SearchBarInput::OpenFileRequest);
                    }
                },
            },

            append = &gtk::Button {
                #[watch]
                set_sensitive: model.stopped,
                set_icon_name: icon_names::LOUPE_LARGE,
                set_tooltip_text: Some(fl!("search")),
                set_css_classes: &["suggested-action"],
                connect_clicked => SearchBarInput::StartSearch,
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let open_dialog_settings = OpenDialogSettings {
            folder_mode: true,
            accept_label: String::from(fl!("open")),
            cancel_label: String::from(fl!("cancel")),
            create_folders: false,
            is_modal: true,
            filters: Vec::new(),
        };

        let open_dialog = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(open_dialog_settings)
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => SearchBarInput::OpenFileResponse(path),
                OpenDialogResponse::Cancel => SearchBarInput::Ignore,
            });

        let model = SearchBarModel {
            open_dialog,
            stopped: true,
            file_path: PathBuf::default(),
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            SearchBarInput::StartSearch => {
                if self.file_path.exists() {
                    self.stopped = false;
                    let file_path = self.file_path.clone();
                    sender
                        .output(SearchBarOutput::StartSearch(file_path))
                        .unwrap_or_default();
                } else {
                    let msg = fl!("invalid-directory").to_string();
                    sender
                        .output(SearchBarOutput::Notify(msg, 3))
                        .unwrap_or_default();
                }
            }
            SearchBarInput::SearchCompleted => {
                self.stopped = true;
            }
            SearchBarInput::OpenFileRequest => {
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            SearchBarInput::OpenFileResponse(path) => {
                self.file_path = path;
            }
            SearchBarInput::Ignore => {}
        }
    }
}
