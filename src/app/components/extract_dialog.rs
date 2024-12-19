use std::path::PathBuf;

use relm4::{
    adw,
    component::Controller,
    factory::{FactoryVecDeque, FactoryVecDequeGuard},
    gtk::{
        self,
        prelude::{
            BoxExt, ButtonExt, EditableExt, EntryExt, FlowBoxChildExt, GtkWindowExt, OrientableExt,
            WidgetExt,
        },
    },
    Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt,
};
use relm4_components::open_dialog::*;
use relm4_icons::icon_names;

use crate::app::{factories::layout::LayoutModel, models, utils};
use crate::fl;

pub struct ExtractDialogModel {
    layout_list_factory: FactoryVecDeque<LayoutModel>,
    open_dialog: Controller<OpenDialog>,
    layout_type: Option<models::LayoutType>,
    file_path: PathBuf,
    video: Option<models::Video>,
}

#[derive(Debug)]
pub enum ExtractDialogInput {
    Show(ExtractDialogType),
    Hide,
    Extract,
    SelectLayout(usize),
    OpenFileRequest,
    OpenFileResponse(PathBuf),
    Ignore,
}

#[derive(Debug)]
pub enum ExtractDialogOutput {
    Response(models::LayoutType, PathBuf),
}

#[derive(Debug)]
pub enum ExtractDialogType {
    Multi,
    Single(models::Video),
}

#[relm4::component(pub)]
impl Component for ExtractDialogModel {
    type Init = ();
    type Input = ExtractDialogInput;
    type Output = ExtractDialogOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::Window {
            set_default_size: (500, 362),
            set_hide_on_close: true,
            set_modal: true,
            set_resizable: false,
            set_deletable: false,

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_hexpand: true,
                    set_show_start_title_buttons: true,
                    set_show_end_title_buttons: true,

                    pack_start = &gtk::Button {
                        set_label: fl!("cancel"),
                        connect_clicked => ExtractDialogInput::Hide,
                    },

                    pack_end = &gtk::Button {
                        set_label: fl!("extract"),
                        set_css_classes: &["suggested-action"],
                        connect_clicked => ExtractDialogInput::Extract,
                    },

                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcher {
                        set_stack: Some(&stack),
                        set_policy: adw::ViewSwitcherPolicy::Wide,
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_hexpand: true,
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_bottom: 3,

                        #[name(banner)]
                        adw::Banner {
                            set_button_label: Some(fl!("close")),
                            connect_button_clicked => |button| {
                                button.set_revealed(false);
                            },
                        },

                        #[name(stack)]
                        adw::ViewStack {
                            add = &gtk::Frame {
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
                            } -> {
                                set_name: Some("extract-to-image"),
                                set_title: Some(fl!("extract-to-image")),
                                set_icon_name: Some(icon_names::IMAGE),
                            },

                            add = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_start: 6,
                                set_margin_end: 6,

                                gtk::Label {
                                    set_margin_all: 2,

                                    #[watch]
                                    set_label: match model.video.as_ref() {
                                        Some(video) => video.name.as_str(),
                                        None => "",
                                    },
                                },

                                gtk::Frame {
                                    gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        set_show_separators: false,

                                        gtk::ListBoxRow {
                                            gtk::Box {
                                                set_spacing: 40,
                                                set_margin_top: 1,
                                                set_margin_bottom: 1,
                                                set_margin_start: 6,
                                                set_margin_end: 6,

                                                gtk::Label {
                                                    set_label: fl!("duration"),
                                                    set_halign: gtk::Align::Start,
                                                },

                                                gtk::Label {
                                                    #[watch]
                                                    set_label: match model.video.as_ref() {
                                                        Some(video) => video.duration.as_str(),
                                                        None => "",
                                                    },
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                },
                                            }
                                        },

                                        gtk::ListBoxRow {
                                            gtk::Box {
                                                set_spacing: 40,
                                                set_margin_top: 1,
                                                set_margin_bottom: 1,
                                                set_margin_start: 6,
                                                set_margin_end: 6,

                                                gtk::Label {
                                                    set_label: fl!("frame-rate"),
                                                    set_halign: gtk::Align::Start,
                                                },

                                                gtk::Label {
                                                    #[watch]
                                                    set_label: &match model.video.as_ref() {
                                                        Some(video) => format!("{:.2} fps", video.rate),
                                                        None => "".to_owned(),
                                                    },
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                },
                                            }
                                        },

                                        gtk::ListBoxRow {
                                            gtk::Box {
                                                set_spacing: 40,
                                                set_margin_top: 1,
                                                set_margin_bottom: 1,
                                                set_margin_start: 6,
                                                set_margin_end: 6,

                                                gtk::Label {
                                                    set_label: fl!("total-frames"),
                                                    set_halign: gtk::Align::Start,
                                                },

                                                gtk::Label {
                                                    #[watch]
                                                    set_label: &match model.video.as_ref() {
                                                        Some(video) => video.nb_frames.to_string(),
                                                        None => String::new(),
                                                    },
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                },
                                            }
                                        },

                                        gtk::ListBoxRow {
                                            gtk::Box {
                                                set_spacing: 40,
                                                set_margin_top: 1,
                                                set_margin_bottom: 1,
                                                set_margin_start: 6,
                                                set_margin_end: 6,

                                                gtk::Label {
                                                    set_label: fl!("frame-width"),
                                                    set_halign: gtk::Align::Start,
                                                },

                                                gtk::Label {
                                                    #[watch]
                                                    set_label: &match model.video.as_ref() {
                                                        Some(video) => video.width.to_string(),
                                                        None => String::new(),
                                                    },
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                },
                                            }
                                        },

                                        gtk::ListBoxRow {
                                            gtk::Box {
                                                set_spacing: 40,
                                                set_margin_top: 1,
                                                set_margin_bottom: 1,
                                                set_margin_start: 6,
                                                set_margin_end: 6,

                                                gtk::Label {
                                                    set_label: fl!("frame-height"),
                                                    set_halign: gtk::Align::Start,
                                                },

                                                gtk::Label {
                                                    #[watch]
                                                    set_label: &match model.video.as_ref() {
                                                        Some(video) => video.height.to_string(),
                                                        None => String::new(),
                                                    },
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                },
                                            }
                                        },
                                    },
                                },

                                gtk::ListBox {
                                    set_margin_top: 6,
                                    set_selection_mode: gtk::SelectionMode::None,
                                    set_show_separators: false,
                                    set_css_classes: &["rich-list", "boxed-list"],

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            gtk::Label {
                                                set_label: fl!("initial-duration"),
                                                set_xalign: 0f32,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },

                                            #[name(entry_initial)]
                                            gtk::Entry {
                                                set_halign: gtk::Align::End,
                                                set_valign: gtk::Align::Center,
                                                set_max_length: 8,
                                                set_max_width_chars: 8,
                                                set_placeholder_text: Some("00:00:00"),
                                                set_text: "00:00:00",
                                                connect_changed => move |entry| {
                                                    utils::on_format_timestamp(entry);
                                                },
                                            },
                                        },
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            gtk::Label {
                                                set_label: fl!("final-duration"),
                                                set_xalign: 0f32,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },

                                            #[name(entry_final)]
                                            gtk::Entry {
                                                set_halign: gtk::Align::End,
                                                set_valign: gtk::Align::Center,
                                                set_max_length: 8,
                                                set_max_width_chars: 8,
                                                set_placeholder_text: Some("00:00:00"),
                                                #[watch]
                                                set_text: match model.video.as_ref() {
                                                    Some(video) => video.duration.as_str(),
                                                    None => "00:00:00",
                                                },
                                                connect_changed => move |entry| {
                                                    utils::on_format_timestamp(entry);
                                                },
                                            },
                                        },
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            gtk::Label {
                                                set_label: fl!("frame-rate"),
                                                set_xalign: 0f32,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },

                                            #[name(spin_rate)]
                                            gtk::SpinButton {
                                                set_halign: gtk::Align::End,
                                                set_valign: gtk::Align::Center,
                                                set_adjustment: &gtk::Adjustment::new(25f64, 1f64, 25f64, 1f64, 1f64, 0f64),
                                            },
                                        },
                                    },
                                },
                            } -> {
                                set_name: Some("extract-frames"),
                                set_title: Some(fl!("extract-frames")),
                                set_icon_name: Some(icon_names::VIDEO_REEL),
                            },
                        },
                    },
                },

                #[name(entry_dir)]
                add_bottom_bar = &gtk::Entry {
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
            video: None,
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
        root: &Self::Root,
    ) {
        match message {
            ExtractDialogInput::Show(dialog_type) => {
                let mut is_single = false;
                match dialog_type {
                    ExtractDialogType::Multi => {
                        self.video = None;
                    }
                    ExtractDialogType::Single(video) => {
                        let spin_rate = &widgets.spin_rate;
                        spin_rate.set_adjustment(&gtk::Adjustment::new(
                            video.rate, 1f64, video.rate, 1f64, 1f64, 0f64,
                        ));

                        self.video = Some(video);
                        is_single = true;
                    }
                };

                if let Some(page) = widgets.stack.child_by_name("extract-frames") {
                    widgets.stack.page(&page).set_visible(is_single);
                }
                widgets.stack.set_visible_child_name("extract-to-image");
                self.clear_errors(widgets);
                root.present();
            }
            ExtractDialogInput::Hide => root.close(),
            ExtractDialogInput::Extract => {
                if self.video.is_some() {
                    self.extract_frames(widgets, sender.clone());
                } else {
                    self.extract_to_image(widgets, sender.clone());
                }
            }
            ExtractDialogInput::SelectLayout(index) => {
                let layouts_guard = self.layout_list_factory.guard();
                if let Some(layout_model) = layouts_guard.get(index) {
                    self.layout_type = Some(layout_model.layout.layout_type);
                }
            }
            ExtractDialogInput::OpenFileRequest => self.open_dialog.emit(OpenDialogMsg::Open),
            ExtractDialogInput::OpenFileResponse(path) => self.file_path = path,
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

impl ExtractDialogModel {
    fn clear_errors(&self, widgets: &mut ExtractDialogModelWidgets) {
        widgets.entry_initial.set_class_active("input-error", false);
        widgets.entry_final.set_class_active("input-error", false);
        widgets.entry_dir.set_class_active("input-error", false);
    }

    fn extract_to_image(
        &self,
        widgets: &mut ExtractDialogModelWidgets,
        sender: ComponentSender<Self>,
    ) {
        let banner = &widgets.banner;

        let entry_dir = &widgets.entry_dir;
        if !self.file_path.exists() {
            entry_dir.set_class_active("input-error", true);
            banner.set_title(fl!("message-invalid-dst-dir"));
            banner.set_revealed(true);
            return;
        } else {
            entry_dir.set_class_active("input-error", false);
            banner.set_revealed(false);
        }

        if let Some(layout_type) = self.layout_type {
            let file_path = self.file_path.clone();
            sender
                .output(ExtractDialogOutput::Response(layout_type, file_path))
                .unwrap_or_default();
            sender.input(ExtractDialogInput::Hide);
        }
    }

    fn extract_frames(
        &self,
        widgets: &mut ExtractDialogModelWidgets,
        _sender: ComponentSender<Self>,
    ) {
        let banner = &widgets.banner;

        let entry_initial = &widgets.entry_initial;
        let initial_duration = entry_initial
            .text()
            .split(':')
            .map(|v| v.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();
        let initial_duration = if initial_duration.len() == 3 {
            initial_duration[0] * 3600 + initial_duration[1] * 60 + initial_duration[2]
        } else {
            0
        };

        let entry_final = &widgets.entry_final;
        let final_duration = entry_final
            .text()
            .split(':')
            .map(|v| v.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();
        let final_duration = if final_duration.len() == 3 {
            final_duration[0] * 3600 + final_duration[1] * 60 + final_duration[2]
        } else {
            0
        };

        if initial_duration >= final_duration {
            entry_initial.set_class_active("input-error", true);
            banner.set_title(fl!("message-invalid-duration"));
            banner.set_revealed(true);
            return;
        } else {
            entry_initial.set_class_active("input-error", false);
            banner.set_revealed(false);
        }

        let entry_dir = &widgets.entry_dir;
        if !self.file_path.exists() {
            entry_dir.set_class_active("input-error", true);
            banner.set_title(fl!("message-invalid-dst-dir"));
            banner.set_revealed(true);
        } else {
            entry_dir.set_class_active("input-error", false);
            banner.set_revealed(false);
        }
    }
}
