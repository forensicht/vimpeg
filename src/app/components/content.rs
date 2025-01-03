use std::path::PathBuf;

use relm4::{
    adw,
    component::{
        AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
        AsyncController,
    },
    gtk,
    gtk::prelude::{OrientableExt, WidgetExt},
};
use relm4_icons::icon_names;

use super::{
    image_list::{ImageListInput, ImageListModel, ImageListOutput},
    video_list::{VideoListInput, VideoListModel, VideoListOutput},
};

use crate::app::models;
use crate::fl;

pub struct ContentModel {
    video_list: AsyncController<VideoListModel>,
    image_list: AsyncController<ImageListModel>,
}

#[derive(Debug)]
pub enum ContentInput {
    StartSearch(PathBuf),
    SearchCompleted(usize),
    FilterCount(usize),
    ExtractVideosToImage(Vec<String>, models::LayoutType, bool, PathBuf),
    ExtractFramesFromVideo(String, u32, u32, u32, bool, PathBuf),
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

            adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Raised,

                add_top_bar = &adw::HeaderBar {
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_show_back_button: false,
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,

                    #[wrap(Some)]
                    set_title_widget = &adw::ViewSwitcher {
                        set_hexpand: true,
                        set_stack: Some(&stack),
                        set_policy: adw::ViewSwitcherPolicy::Wide,
                    },
                },

                #[name(stack)]
                #[wrap(Some)]
                set_content = &adw::ViewStack {
                    add = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        model.video_list.widget(),
                    } -> {
                        set_name: Some("video-page"),
                        set_title: Some(fl!("video-found")),
                        set_icon_name: Some(icon_names::CAMERAS),
                    },

                    add = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        model.image_list.widget(),
                    } -> {
                        set_name: Some("image-page"),
                        set_title: Some(fl!("generated-image")),
                        set_icon_name: Some(icon_names::IMAGE),
                    },
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
                    VideoListOutput::ExtractVideosToImage(
                        video_list,
                        layout_type,
                        show_timestamp,
                        dst_path,
                    ) => ContentInput::ExtractVideosToImage(
                        video_list,
                        layout_type,
                        show_timestamp,
                        dst_path,
                    ),
                    VideoListOutput::ExtractFramesFromVideo(
                        video_path,
                        time_start,
                        time_end,
                        frame_rate,
                        show_timestamp,
                        dst_path,
                    ) => ContentInput::ExtractFramesFromVideo(
                        video_path,
                        time_start,
                        time_end,
                        frame_rate,
                        show_timestamp,
                        dst_path,
                    ),
                    VideoListOutput::Notify(msg, timeout) => ContentInput::Notify(msg, timeout),
                });

        let image_list_controller =
            ImageListModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
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
                self.video_list.emit(VideoListInput::StartSearch(path));
            }
            ContentInput::SearchCompleted(video_found) => {
                sender
                    .output(ContentOutput::SearchCompleted(video_found))
                    .unwrap_or_default();
            }
            ContentInput::FilterCount(count) => {
                sender
                    .output(ContentOutput::FilterCount(count))
                    .unwrap_or_default();
            }
            ContentInput::ExtractVideosToImage(
                video_list,
                layout_type,
                show_timestamp,
                dst_path,
            ) => {
                widgets.stack.set_visible_child_name("image-page");

                self.image_list.emit(ImageListInput::ExtractVideosToImage(
                    video_list,
                    layout_type,
                    show_timestamp,
                    dst_path,
                ));
            }
            ContentInput::ExtractFramesFromVideo(
                video_path,
                time_start,
                time_end,
                frame_rate,
                show_timestamp,
                dst_path,
            ) => {
                widgets.stack.set_visible_child_name("image-page");

                self.image_list.emit(ImageListInput::ExtractFramesFromVideo(
                    video_path,
                    time_start,
                    time_end,
                    frame_rate,
                    show_timestamp,
                    dst_path,
                ));
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

        self.update_view(widgets, sender);
    }
}
