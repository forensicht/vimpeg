use relm4::{
    factory::{AsyncFactoryComponent, AsyncFactorySender, DynamicIndex},
    gtk::{
        self, gdk,
        gdk_pixbuf::{Colorspace, Pixbuf},
        glib, pango,
        prelude::{
            BoxExt, CheckButtonExt, GestureExt, GestureSingleExt, OrientableExt, PaintableExt,
            PopoverExt, WidgetExt,
        },
    },
    loading_widgets::LoadingWidgets,
    view, RelmWidgetExt,
};

use crate::app::models;
use crate::fl;
use core_vimpeg::video::service;

pub struct VideoModel {
    pub video: models::Video,
    pub index: DynamicIndex,
    paintable: Option<gdk::Paintable>,
    is_visible: bool,
}

impl VideoModel {
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

#[derive(Debug)]
pub enum VideoInput {
    Selected(bool),
    SetVisible(bool),
    ZoomIn(i32),
    ZoomOut(i32),
}

#[derive(Debug)]
pub enum VideoOutput {
    Selected(bool),
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for VideoModel {
    type Init = models::Video;
    type Input = VideoInput;
    type Output = VideoOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 2,
            set_css_classes: &["card", "activatable", "media-item-box", "border-spacing"],
            set_tooltip_text: Some(&self.video.name),

            gtk::Overlay {
                #[watch]
                set_size_request: (self.video.thumbnail_size, self.video.thumbnail_size),

                add_overlay = &gtk::Picture {
                    set_margin_all: 3,
                    set_content_fit: gtk::ContentFit::Contain,
                    set_can_shrink: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_paintable: self.paintable.as_ref(),
                    add_controller = gtk::GestureClick {
                        set_button: 3,
                        connect_released[popover] => move |gesture, _, x, y| {
                            gesture.set_state(gtk::EventSequenceState::Claimed);
                            popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                            popover.popup();
                        }
                    }
                },

                #[name(checkbox)]
                add_overlay = &gtk::CheckButton {
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,
                    set_css_classes: &["border-spacing"],
                    #[watch]
                    set_active: self.video.is_selected,
                    connect_toggled[sender] => move |checkbox| {
                        sender.input(VideoInput::Selected(checkbox.is_active()));
                    }
                },
            },

            gtk::Label {
                set_label: &self.video.name,
                set_margin_all: 2,
                set_hexpand: true,
                set_halign: gtk::Align::Fill,
                set_max_width_chars: 25,
                set_ellipsize: pango::EllipsizeMode::End,
            },

            #[name(popover)]
            gtk::Popover {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Frame {
                        gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            set_show_separators: false,

                            gtk::ListBoxRow {
                                gtk::Box {
                                    set_spacing: 40,
                                    set_margin_all: 2,

                                    gtk::Label {
                                        set_label: fl!("duration"),
                                        set_halign: gtk::Align::Start,
                                    },

                                    gtk::Label {
                                        set_label: &self.video.duration,
                                        set_halign: gtk::Align::End,
                                        set_hexpand: true,
                                    },
                                }
                            },

                            gtk::ListBoxRow {
                                gtk::Box {
                                    set_spacing: 40,
                                    set_margin_all: 2,

                                    gtk::Label {
                                        set_label: fl!("frame-rate"),
                                        set_halign: gtk::Align::Start,
                                    },

                                    gtk::Label {
                                        set_label: &format!("{:.2} fps", self.video.rate),
                                        set_halign: gtk::Align::End,
                                        set_hexpand: true,
                                    },
                                }
                            },

                            gtk::ListBoxRow {
                                gtk::Box {
                                    set_spacing: 40,
                                    set_margin_all: 2,

                                    gtk::Label {
                                        set_label: fl!("total-frames"),
                                        set_halign: gtk::Align::Start,
                                    },

                                    gtk::Label {
                                        set_label: &self.video.nb_frames.to_string(),
                                        set_halign: gtk::Align::End,
                                        set_hexpand: true,
                                    },
                                }
                            },

                            gtk::ListBoxRow {
                                gtk::Box {
                                    set_spacing: 40,
                                    set_margin_all: 2,

                                    gtk::Label {
                                        set_label: fl!("frame-width"),
                                        set_halign: gtk::Align::Start,
                                    },

                                    gtk::Label {
                                        set_label: &self.video.width.to_string(),
                                        set_halign: gtk::Align::End,
                                        set_hexpand: true,
                                    },
                                }
                            },

                            gtk::ListBoxRow {
                                gtk::Box {
                                    set_spacing: 40,
                                    set_margin_all: 2,

                                    gtk::Label {
                                        set_label: fl!("frame-height"),
                                        set_halign: gtk::Align::Start,
                                    },

                                    gtk::Label {
                                        set_label: &self.video.height.to_string(),
                                        set_halign: gtk::Align::End,
                                        set_hexpand: true,
                                    },
                                }
                            },
                        },
                    },
                }
            }
        },
    }

    fn init_loading_widgets(root: Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 2,
                set_width_request: models::video::THUMBNAIL_SIZE,
                set_height_request: models::video::THUMBNAIL_SIZE + 14,
                set_css_classes: &["card", "media-item-box", "border-spacing"],

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_height_request: models::video::THUMBNAIL_SIZE + 14,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init_model(
        video: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        let filename = video.path.as_str();
        let paintable = match service::get_video_thumbnail(filename).await {
            Ok(thumb) => {
                if let Some(data) = thumb.data.as_ref() {
                    let bytes = glib::Bytes::from(data);
                    let rowstride = Pixbuf::calculate_rowstride(
                        Colorspace::Rgb,
                        true,
                        8,
                        thumb.width as i32,
                        thumb.height as i32,
                    );

                    let pixbuf = Pixbuf::from_bytes(
                        &bytes,
                        Colorspace::Rgb,
                        true,
                        8,
                        thumb.width as i32,
                        thumb.height as i32,
                        rowstride,
                    );

                    Some(gdk::Texture::for_pixbuf(&pixbuf).current_image())
                } else {
                    None
                }
            }
            Err(err) => {
                tracing::error!("{} {}", fl!("generic-error"), err);
                while let Some(e) = err.source() {
                    tracing::error!("Caused by: {}", e);
                }

                None
            }
        };

        Self {
            video,
            index: index.clone(),
            paintable,
            is_visible: true,
        }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncFactorySender<Self>) {
        match message {
            VideoInput::Selected(is_selected) => {
                self.video.is_selected = is_selected;
                sender
                    .output(VideoOutput::Selected(is_selected))
                    .unwrap_or_default();
            }
            VideoInput::SetVisible(is_visible) => {
                self.is_visible = is_visible;
            }
            VideoInput::ZoomIn(size) => {
                self.video.thumbnail_size = size;
            }
            VideoInput::ZoomOut(size) => {
                self.video.thumbnail_size = size;
            }
        }
    }
}
