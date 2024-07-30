use std::path::PathBuf;

use relm4::{
    adw,
    adw::gio,
    component::{
        AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
        AsyncController,
    },
    gtk,
    gtk::prelude::OrientableExt,
};
use relm4_icons::icon_names;

use super::{
    image_list::{ImageListInput, ImageListModel, ImageListOutput},
    video_list::{VideoListInput, VideoListModel, VideoListOutput},
};

use crate::app::models;
use crate::fl;

pub const VIDEO_PAGE: i32 = 0;
pub const IMAGE_PAGE: i32 = 1;

pub struct ContentModel {
    video_list: AsyncController<VideoListModel>,
    image_list: AsyncController<ImageListModel>,
}

#[derive(Debug)]
pub enum ContentInput {
    StartSearch(PathBuf),
    SearchCompleted(usize),
    FilterCount(usize),
    ConvertVideos(Vec<String>, models::LayoutType, PathBuf),
    VideoConversionCompleted,
    ImageCount(usize),
    Notify(String, u32),
}

#[derive(Debug)]
pub enum ContentOutput {
    SearchCompleted(usize),
    FilterCount(usize),
    ImageCount(usize),
    Notify(String, u32),
}

#[relm4::component(pub async)]
impl AsyncComponent for ContentModel {
    type Init = ();
    type Input = ContentInput;
    type Output = ContentOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::TabBar {
                set_view: Some(&tab_view),
                set_autohide: false,
                set_expand_tabs: false,
            },

            #[name(tab_view)]
            adw::TabView {
                append_pinned = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    model.video_list.widget(),
                } -> {
                    set_title: fl!("video-found"),
                    set_icon: Some(&gio::ThemedIcon::new(icon_names::CAMERAS)),
                },

                append_pinned = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    model.image_list.widget(),
                } -> {
                    set_title: fl!("generated-image"),
                    set_icon: Some(&gio::ThemedIcon::new(icon_names::IMAGE)),
                },
            },
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let video_list_controller =
            VideoListModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    VideoListOutput::SearchCompleted(found) => ContentInput::SearchCompleted(found),
                    VideoListOutput::FilterResult(len) => ContentInput::FilterCount(len),
                    VideoListOutput::ExtractVideos(video_list, layout_type, dst_path) => {
                        ContentInput::ConvertVideos(video_list, layout_type, dst_path)
                    }
                    VideoListOutput::Notify(msg, timeout) => ContentInput::Notify(msg, timeout),
                });

        let image_list_controller =
            ImageListModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    ImageListOutput::VideoConversionCompleted => {
                        ContentInput::VideoConversionCompleted
                    }
                    ImageListOutput::ImageCount(count) => ContentInput::ImageCount(count),
                    ImageListOutput::Notify(msg, timeout) => ContentInput::Notify(msg, timeout),
                });

        let model = ContentModel {
            video_list: video_list_controller,
            image_list: image_list_controller,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ContentInput::StartSearch(path) => {
                widgets.tab_view.nth_page(VIDEO_PAGE).set_loading(true);
                self.video_list.emit(VideoListInput::StartSearch(path));
            }
            ContentInput::SearchCompleted(found) => {
                widgets.tab_view.nth_page(VIDEO_PAGE).set_loading(false);
                sender
                    .output(ContentOutput::SearchCompleted(found))
                    .unwrap_or_default();
            }
            ContentInput::FilterCount(count) => {
                sender
                    .output(ContentOutput::FilterCount(count))
                    .unwrap_or_default();
            }
            ContentInput::ConvertVideos(video_list, layout_type, dst_path) => {
                let image_page = &widgets.tab_view.nth_page(IMAGE_PAGE);
                image_page.set_loading(true);
                widgets.tab_view.set_selected_page(image_page);
                self.image_list.emit(ImageListInput::ConvertVideos(
                    video_list,
                    layout_type,
                    dst_path,
                ));
            }
            ContentInput::VideoConversionCompleted => {
                widgets.tab_view.nth_page(IMAGE_PAGE).set_loading(false);
            }
            ContentInput::ImageCount(count) => {
                sender
                    .output(ContentOutput::ImageCount(count))
                    .unwrap_or_default();
            }
            ContentInput::Notify(msg, timeout) => {
                sender
                    .output(ContentOutput::Notify(msg, timeout))
                    .unwrap_or_default();
            }
        }
    }
}
