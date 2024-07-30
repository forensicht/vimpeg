use std::path::PathBuf;

use relm4::{
    adw,
    component::Controller,
    factory::{FactoryVecDeque, FactoryVecDequeGuard},
    gtk,
    gtk::prelude::{
        ButtonExt, EditableExt, EntryExt, FlowBoxChildExt, GtkWindowExt, OrientableExt, WidgetExt,
    },
    Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
};
use relm4_components::open_dialog::*;
use relm4_icons::icon_names;

use crate::app::{factories::layout::LayoutModel, models};
use crate::fl;

pub struct ExtractDialogModel {
    layout_list_factory: FactoryVecDeque<LayoutModel>,
    open_dialog: Controller<OpenDialog>,
    layout_type: Option<models::LayoutType>,
    file_path: PathBuf,
}

#[derive(Debug)]
pub enum ExtractDialogInput {
    SelectLayout(usize),
    OpenFileRequest,
    OpenFileResponse(PathBuf),
    Convert,
    Cancel,
    Ignore,
}

#[derive(Debug)]
pub enum ExtractDialogOutput {
    Response(models::LayoutType, PathBuf),
}

#[relm4::component(pub)]
impl Component for ExtractDialogModel {
    type Init = ();
    type Input = ExtractDialogInput;
    type Output = ExtractDialogOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name(dialog)]
        adw::Window {
            set_default_size: (500, 350),
            set_hide_on_close: true,
            set_modal: true,
            set_resizable: false,
            set_deletable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    set_hexpand: true,
                    set_show_start_title_buttons: true,
                    set_show_end_title_buttons: true,

                    pack_start = &gtk::Button {
                        set_label: fl!("cancel"),
                        connect_clicked => ExtractDialogInput::Cancel,
                    },

                    pack_end = &gtk::Button {
                        set_label: fl!("extract"),
                        set_css_classes: &["suggested-action"],
                        connect_clicked => ExtractDialogInput::Convert,
                    },

                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                        set_label: fl!("extract-frames"),
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Frame {
                        set_margin_end: 6,
                        set_margin_start: 6,
                        set_margin_top: 6,
                        set_css_classes: &["view"],

                        gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            set_hexpand: true,
                            set_vexpand: true,

                            #[local_ref]
                            layout_grid_widget -> gtk::FlowBox {
                                set_valign: gtk::Align::Start,
                                set_max_children_per_line: 4,
                                set_selection_mode: gtk::SelectionMode::Single,
                                set_activate_on_single_click: true,
                                connect_child_activated[sender] => move |_, child| {
                                    let index = child.index() as usize;
                                    sender.input(ExtractDialogInput::SelectLayout(index));
                                },
                            },
                        }
                    },

                    gtk::Entry {
                        #[watch]
                        set_text?: &model.file_path.to_str(),
                        set_margin_all: 6,
                        set_hexpand: true,
                        set_halign: gtk::Align::Fill,
                        set_placeholder_text: Some(fl!("destination-directory")),
                        set_secondary_icon_name: Some(icon_names::FOLDER_OPEN_FILLED),
                        set_secondary_icon_tooltip_text: Some(fl!("select-directory")),
                        connect_icon_release[sender] => move |_, icon_position| {
                            if icon_position == gtk::EntryIconPosition::Secondary {
                                sender.input(ExtractDialogInput::OpenFileRequest);
                            }
                        },
                    },
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut layout_list_factory = FactoryVecDeque::builder().launch_default().detach();

        {
            let mut layout_guard = layout_list_factory.guard();
            init_layout_list_factory(&mut layout_guard);
        }

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
                OpenDialogResponse::Accept(path) => ExtractDialogInput::OpenFileResponse(path),
                OpenDialogResponse::Cancel => ExtractDialogInput::Ignore,
            });

        let model = ExtractDialogModel {
            layout_list_factory,
            open_dialog,
            layout_type: None,
            file_path: PathBuf::default(),
        };

        let layout_grid_widget = model.layout_list_factory.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ExtractDialogInput::SelectLayout(index) => {
                let layouts_guard = self.layout_list_factory.guard();
                if let Some(layout_model) = layouts_guard.get(index) {
                    self.layout_type = Some(layout_model.layout.layout_type);
                }
            }
            ExtractDialogInput::OpenFileRequest => self.open_dialog.emit(OpenDialogMsg::Open),
            ExtractDialogInput::OpenFileResponse(path) => self.file_path = path,
            ExtractDialogInput::Convert => {
                if !self.file_path.exists() {
                    return;
                }
                if let Some(layout_type) = self.layout_type {
                    let file_path = self.file_path.clone();
                    sender
                        .output(ExtractDialogOutput::Response(layout_type, file_path))
                        .unwrap_or_default();
                    widgets.dialog.close();
                }
            }
            ExtractDialogInput::Cancel => widgets.dialog.close(),
            ExtractDialogInput::Ignore => {}
        }

        self.update_view(widgets, sender);
    }
}

fn init_layout_list_factory(layout_guard: &mut FactoryVecDequeGuard<LayoutModel>) {
    use models::{Layout, LayoutType};

    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_2x2.png".to_string(),
        LayoutType::L2x2,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_2x3.png".to_string(),
        LayoutType::L2x3,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_2x4.png".to_string(),
        LayoutType::L2x4,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_3x2.png".to_string(),
        LayoutType::L3x2,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_3x3.png".to_string(),
        LayoutType::L3x3,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_3x4.png".to_string(),
        LayoutType::L3x4,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_4x2.png".to_string(),
        LayoutType::L4x2,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_4x3.png".to_string(),
        LayoutType::L4x3,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_4x4.png".to_string(),
        LayoutType::L4x4,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_4x5.png".to_string(),
        LayoutType::L4x5,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_5x4.png".to_string(),
        LayoutType::L5x4,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_5x5.png".to_string(),
        LayoutType::L5x5,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_6x6.png".to_string(),
        LayoutType::L6x6,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_8x8.png".to_string(),
        LayoutType::L8x8,
    ));
    layout_guard.push_back(Layout::new(
        "/com/github/forensicht/vimpeg/images/layout_10x10.png".to_string(),
        LayoutType::L10x10,
    ));
}
