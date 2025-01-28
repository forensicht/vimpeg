use relm4::{
    factory::{AsyncFactoryComponent, AsyncFactorySender, DynamicIndex},
    gtk::{
        self, gio, pango,
        prelude::{OrientableExt, WidgetExt},
    },
    loading_widgets::LoadingWidgets,
    view, RelmWidgetExt,
};

use crate::app::models;

#[derive(Debug)]
pub enum ImageInput {
    ZoomIn(i32),
    ZoomOut(i32),
}

#[derive(Debug)]
pub struct ImageModel {
    pub image: models::Image,
    pub index: DynamicIndex,
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for ImageModel {
    type Init = models::Image;
    type Input = ImageInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 2,
            set_css_classes: &["card", "activatable", "media-item-box", "border-spacing"],
            set_tooltip_text: Some(&self.image.name),

            gtk::Overlay {
                #[watch]
                set_size_request: (self.image.thumbnail_size, self.image.thumbnail_size),

                add_overlay = &gtk::Picture {
                    set_margin_all: 3,
                    set_content_fit: gtk::ContentFit::Contain,
                    set_can_shrink: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_file: Some(&gio::File::for_parse_name(&self.image.path)),
                },

                add_overlay = &gtk::Label {
                    set_visible: self.image.total_images > 1,
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Start,
                    set_margin_end: 3,
                    set_margin_top: 3,
                    set_css_classes: &["body", "badge"],
                    set_label: &format!("+{}", self.image.total_images - 1),
                },
            },

            gtk::Label {
                set_label: &self.image.name,
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
                set_width_request: models::image::THUMBNAIL_SIZE,
                set_height_request: models::image::THUMBNAIL_SIZE + 14,
                set_css_classes: &["card", "media-item-box", "border-spacing"],

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_height_request: models::image::THUMBNAIL_SIZE + 14,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init_model(
        image: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            image,
            index: index.clone(),
        }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncFactorySender<Self>) {
        match message {
            ImageInput::ZoomIn(size) => {
                self.image.thumbnail_size = size;
            }
            ImageInput::ZoomOut(size) => {
                self.image.thumbnail_size = size;
            }
        }
    }
}
