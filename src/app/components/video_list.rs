use anyhow;
use std::path::PathBuf;

use relm4::{
    adw,
    component::{
        AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
        AsyncController,
    },
    factory::AsyncFactoryVecDeque,
    gtk,
    gtk::prelude::{ButtonExt, FlowBoxChildExt, OrientableExt, WidgetExt},
    Component, ComponentController, Controller, RelmIterChildrenExt,
};
use relm4_icons::icon_names;

use super::toolbar::{ToolBarInput, ToolBarModel, ToolBarOutput};
use crate::app::{
    components::extract_dialog::{
        ExtractDialogInput, ExtractDialogModel, ExtractDialogOutput, ExtractDialogType,
    },
    factories::video::{VideoInput, VideoModel, VideoOutput},
    models,
};
use crate::fl;
use core_vimpeg::video::service;

pub struct VideoListModel {
    toolbar: AsyncController<ToolBarModel>,
    video_list_factory: AsyncFactoryVecDeque<VideoModel>,
    convert_dialog: Controller<ExtractDialogModel>,
    video_list_filter: models::VideoFilter,
    thumbnail_size: i32,
}

impl VideoListModel {
    pub fn new(
        toolbar: AsyncController<ToolBarModel>,
        video_list_factory: AsyncFactoryVecDeque<VideoModel>,
        convert_dialog: Controller<ExtractDialogModel>,
    ) -> Self {
        Self {
            toolbar,
            video_list_factory,
            convert_dialog,
            video_list_filter: models::VideoFilter::default(),
            thumbnail_size: models::video::THUMBNAIL_SIZE,
        }
    }
}

#[derive(Debug)]
pub enum VideoListInput {
    StartSearch(PathBuf),
    SearchCompleted(usize),
    PlayVideo(usize),
    ZoomIn,
    ZoomOut,
    OpenExtractDialog,
    OpenExtractResponse(models::LayoutType, PathBuf),
    SelectAllVideos(bool),
    SelectedVideo(bool),
    SearchEntry(String),
    SizeFilter0KB(bool),
    SizeFilter30KB(bool),
    SizeFilter100KB(bool),
    SizeFilter500KB(bool),
    SizeFilterA500KB(bool),
}

#[derive(Debug)]
pub enum VideoListOutput {
    SearchCompleted(usize),
    FilterResult(usize),
    ExtractVideos(Vec<String>, models::LayoutType, PathBuf),
    Notify(String, u32),
}

#[derive(Debug)]
pub enum VideoListCommandOutput {
    SearchCompleted(anyhow::Result<Vec<models::Video>>),
}

#[relm4::component(pub async)]
impl AsyncComponent for VideoListModel {
    type Init = ();
    type Input = VideoListInput;
    type Output = VideoListOutput;
    type CommandOutput = VideoListCommandOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_bottom: 3,
            set_margin_end: 6,
            set_margin_start: 6,
            set_margin_top: 4,

            model.toolbar.widget(),

            gtk::Overlay {
                set_hexpand: true,
                set_vexpand: true,

                add_overlay = &gtk::Frame {
                    set_css_classes: &["view"],

                    gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_hexpand: true,
                        set_vexpand: true,

                        #[local_ref]
                        video_list_widget -> gtk::FlowBox {
                            set_css_classes: &["list-padding-bottom"],
                            set_valign: gtk::Align::Start,
                            set_max_children_per_line: 16,
                            set_selection_mode: gtk::SelectionMode::None,
                            set_activate_on_single_click: false,
                            connect_child_activated[sender] => move |_, child| {
                                let index = child.index() as usize;
                                sender.input(VideoListInput::PlayVideo(index));
                            },
                        },
                    }
                },

                add_overlay = &gtk::Button {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::End,
                    set_margin_bottom: 36,
                    set_margin_end: 36,
                    set_css_classes: &["pill", "suggested-action"],

                    #[wrap(Some)]
                    set_child = &adw::ButtonContent {
                        set_icon_name: icon_names::ENCODE,
                        set_label: fl!("extract"),
                        set_use_underline: true,
                    },

                    connect_clicked => VideoListInput::OpenExtractDialog,
                },

                #[name(spinner)]
                add_overlay = &gtk::Spinner {
                    set_size_request: (30, 30),
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    stop: (),
                },
            },
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let tool_bar_controller = ToolBarModel::builder().launch(()).forward(
            sender.input_sender(),
            |output| match output {
                ToolBarOutput::ZoomIn => VideoListInput::ZoomIn,
                ToolBarOutput::ZoomOut => VideoListInput::ZoomOut,
                ToolBarOutput::SelectAll(is_selected) => {
                    VideoListInput::SelectAllVideos(is_selected)
                }
                ToolBarOutput::SearchEntry(query) => VideoListInput::SearchEntry(query),
                ToolBarOutput::SizeFilter0KB(is_active) => VideoListInput::SizeFilter0KB(is_active),
                ToolBarOutput::SizeFilter30KB(is_active) => {
                    VideoListInput::SizeFilter30KB(is_active)
                }
                ToolBarOutput::SizeFilter100KB(is_active) => {
                    VideoListInput::SizeFilter100KB(is_active)
                }
                ToolBarOutput::SizeFilter500KB(is_active) => {
                    VideoListInput::SizeFilter500KB(is_active)
                }
                ToolBarOutput::SizeFilterGreater500KB(is_active) => {
                    VideoListInput::SizeFilterA500KB(is_active)
                }
            },
        );

        let video_list_factory = AsyncFactoryVecDeque::builder().launch_default().forward(
            sender.input_sender(),
            |output| match output {
                VideoOutput::Selected(is_selected) => VideoListInput::SelectedVideo(is_selected),
            },
        );

        let convert_dialog_controller = ExtractDialogModel::builder()
            .transient_for(&root)
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                ExtractDialogOutput::Response(layout_type, dst_path) => {
                    VideoListInput::OpenExtractResponse(layout_type, dst_path)
                }
            });

        let model = VideoListModel::new(
            tool_bar_controller,
            video_list_factory,
            convert_dialog_controller,
        );
        let video_list_widget = model.video_list_factory.widget();
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
            VideoListInput::ZoomIn => {
                self.apply_video_zoom(true).await;
            }
            VideoListInput::ZoomOut => {
                self.apply_video_zoom(false).await;
            }
            VideoListInput::StartSearch(path) => {
                widgets.spinner.start();
                self.on_search(path, &sender).await;
            }
            VideoListInput::SearchCompleted(videos_found) => {
                widgets.spinner.stop();
                sender
                    .output(VideoListOutput::SearchCompleted(videos_found))
                    .unwrap_or_default();
            }
            VideoListInput::PlayVideo(index) => {
                self.on_play_video(index, &sender).await;
            }
            VideoListInput::OpenExtractDialog => {
                let guard = self.video_list_factory.guard();
                let selected_videos: Vec<&VideoModel> = guard
                    .iter()
                    .filter(|&video_model| video_model.unwrap().video.is_selected)
                    .map(|video_model| video_model.unwrap())
                    .collect();

                match selected_videos.len() {
                    1 => {
                        if let Some(&video_model) = selected_videos.first() {
                            let video = video_model.video.clone();
                            self.convert_dialog
                                .emit(ExtractDialogInput::Show(ExtractDialogType::Single(video)));
                        }
                    }
                    2.. => {
                        self.convert_dialog
                            .emit(ExtractDialogInput::Show(ExtractDialogType::Multi));
                    }
                    _ => {
                        sender
                            .output(VideoListOutput::Notify(
                                fl!("select-one-video").to_string(),
                                3,
                            ))
                            .unwrap_or_default();
                    }
                }
            }
            VideoListInput::OpenExtractResponse(layout_type, dst_path) => {
                self.on_open_convert_response(layout_type, dst_path, &sender)
                    .await;
            }
            VideoListInput::SelectAllVideos(is_selected) => {
                self.on_select_all_videos(is_selected).await;
            }
            VideoListInput::SelectedVideo(is_selected) => {
                self.toolbar.emit(ToolBarInput::SelectedVideo(is_selected));
            }
            VideoListInput::SearchEntry(query) => {
                self.video_list_filter.search_entry = Some(query);
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
            VideoListInput::SizeFilter0KB(is_active) => {
                self.video_list_filter.is_size_0 = is_active;
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
            VideoListInput::SizeFilter30KB(is_active) => {
                self.video_list_filter.is_size_30 = is_active;
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
            VideoListInput::SizeFilter100KB(is_active) => {
                self.video_list_filter.is_size_100 = is_active;
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
            VideoListInput::SizeFilter500KB(is_active) => {
                self.video_list_filter.is_size_500 = is_active;
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
            VideoListInput::SizeFilterA500KB(is_active) => {
                self.video_list_filter.is_size_greater_500 = is_active;
                let affected = self.apply_video_filters().await;
                sender
                    .output(VideoListOutput::FilterResult(affected))
                    .unwrap_or_default();
            }
        }

        self.update_view(widgets, sender);
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            VideoListCommandOutput::SearchCompleted(result) => {
                let mut videos_found: usize = 0;

                match result {
                    Ok(videos) => {
                        videos_found = videos.len();
                        let mut guard = self.video_list_factory.guard();
                        guard.clear();

                        for video in videos {
                            guard.push_back(video);
                        }
                    }
                    Err(err) => {
                        tracing::error!("{}: {}", fl!("generic-error"), err);
                        while let Some(e) = err.source() {
                            tracing::error!("Caused by: {}", e);
                        }
                    }
                }

                sender.input(VideoListInput::SearchCompleted(videos_found));
            }
        }
    }
}

impl VideoListModel {
    async fn on_search(&mut self, path: PathBuf, sender: &AsyncComponentSender<VideoListModel>) {
        sender.oneshot_command(async move {
            match service::search_videos(path).await {
                Ok(videos) => {
                    let videos = videos.iter().map(models::Video::from).collect();
                    VideoListCommandOutput::SearchCompleted(Ok(videos))
                }
                Err(err) => VideoListCommandOutput::SearchCompleted(Err(err)),
            }
        });
    }

    async fn on_play_video(&mut self, index: usize, sender: &AsyncComponentSender<VideoListModel>) {
        if let Some(video_model) = self.video_list_factory.guard().get(index) {
            let video_name = video_model.video.path.as_str();

            if open::that(video_name).is_err() {
                let msg = format!(
                    "{} {}",
                    fl!("open-video-error"),
                    video_model.video.name.as_str()
                );
                sender
                    .output(VideoListOutput::Notify(msg, 3))
                    .unwrap_or_default();
            }
        }
    }

    async fn on_open_convert_response(
        &mut self,
        layout_type: models::LayoutType,
        dst_path: PathBuf,
        sender: &AsyncComponentSender<VideoListModel>,
    ) {
        let videos_list: Vec<String> = self
            .video_list_factory
            .guard()
            .iter()
            .filter(|&video_model| video_model.unwrap().video.is_selected)
            .map(|video_model| video_model.unwrap().video.path.clone())
            .collect();

        if !videos_list.is_empty() {
            sender
                .output(VideoListOutput::ExtractVideos(
                    videos_list,
                    layout_type,
                    dst_path,
                ))
                .unwrap_or_default();
        } else {
            sender
                .output(VideoListOutput::Notify(
                    fl!("select-one-video").to_string(),
                    5,
                ))
                .unwrap_or_default();
        }
    }

    async fn on_select_all_videos(&mut self, is_selected: bool) {
        self.video_list_factory
            .guard()
            .iter_mut()
            .for_each(|video_model| {
                let video_model = video_model.unwrap();
                if !is_selected || video_model.is_visible() {
                    video_model.video.is_selected = is_selected;
                }
            });
    }

    async fn apply_video_filters(&mut self) -> usize {
        let video_widget = self.video_list_factory.widget();
        let filter = &self.video_list_filter;

        for video_model in self.video_list_factory.iter() {
            let video_model = video_model.unwrap();
            let video = &video_model.video;
            let mut is_visible = true;

            if let Some(query) = &filter.search_entry {
                is_visible = video.name.to_lowercase().contains(&query.to_lowercase());
            }

            #[allow(clippy::nonminimal_bool)]
            if (!filter.is_size_0 && video.size == 0)
                || (!filter.is_size_30 && (video.size > 0 && video.size <= 30))
                || (!filter.is_size_100 && (video.size > 30 && video.size <= 100))
                || (!filter.is_size_100 && (video.size > 30 && video.size <= 100))
                || (!filter.is_size_500 && (video.size > 100 && video.size <= 500))
                || (!filter.is_size_greater_500 && video.size > 500)
            {
                is_visible = false;
            }

            let index = video_model.index.current_index();
            self.video_list_factory
                .send(index, VideoInput::SetVisible(is_visible));
            video_widget
                .child_at_index(index as i32)
                .as_ref()
                .unwrap()
                .set_visible(is_visible);
        }

        video_widget
            .iter_children()
            .filter(|c| c.is_visible())
            .count()
    }

    async fn apply_video_zoom(&mut self, is_zoom_in: bool) {
        use models::video::THUMBNAIL_SIZE;
        use models::video::ZOOM_SIZE;

        if is_zoom_in {
            if self.thumbnail_size < 320 {
                self.thumbnail_size += ZOOM_SIZE;
            }
        } else if self.thumbnail_size > THUMBNAIL_SIZE {
            let mut thumb_size = self.thumbnail_size - ZOOM_SIZE;
            if thumb_size < THUMBNAIL_SIZE {
                thumb_size = THUMBNAIL_SIZE;
            }
            self.thumbnail_size = thumb_size;
        }

        for video_model in self.video_list_factory.iter() {
            let index = video_model.unwrap().index.current_index();
            if is_zoom_in {
                self.video_list_factory
                    .send(index, VideoInput::ZoomIn(self.thumbnail_size));
            } else {
                self.video_list_factory
                    .send(index, VideoInput::ZoomOut(self.thumbnail_size));
            }
        }
    }
}
