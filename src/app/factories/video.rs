use relm4::{
    factory::{AsyncFactoryComponent, AsyncFactorySender, DynamicIndex},
    gtk,
    gtk::gdk_pixbuf::{Colorspace, Pixbuf},
    gtk::glib,
    gtk::pango,
    gtk::prelude::{CheckButtonExt, GestureExt, GestureSingleExt, OrientableExt, WidgetExt},
    loading_widgets::LoadingWidgets,
    view, RelmWidgetExt,
};

use crate::app::models;
use crate::fl;
use core_vimpeg::video::service;

pub struct VideoModel {
    pub video: models::Video,
    pub index: DynamicIndex,
    pixbuf: Option<Pixbuf>,
}

#[derive(Debug)]
pub enum VideoInput {
    Selected(bool),
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
                    set_pixbuf: self.pixbuf.as_ref(),
                    add_controller = gtk::GestureClick {
                        set_button: 3,
                        connect_released[checkbox] => move |gesture, _, _, _| {
                            gesture.set_state(gtk::EventSequenceState::Claimed);
                            let is_active = !checkbox.is_active();
                            checkbox.set_active(is_active);
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
            }
        }
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

        let pixbuf = match service::get_video_thumbnail(filename).await {
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

                    Some(pixbuf)
                } else {
                    None
                }
            }
            Err(err) => {
                tracing::error!("{} {}", fl!("generic-error"), err);
                None
            }
        };

        Self {
            video,
            index: index.clone(),
            pixbuf,
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
            VideoInput::ZoomIn(size) => {
                self.video.thumbnail_size = size;
            }
            VideoInput::ZoomOut(size) => {
                self.video.thumbnail_size = size;
            }
        }
    }
}
