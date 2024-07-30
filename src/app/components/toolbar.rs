use relm4::{
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    gtk,
    gtk::prelude::{BoxExt, ButtonExt, CheckButtonExt, EditableExt, OrientableExt, WidgetExt},
    RelmWidgetExt,
};
use relm4_icons::icon_names;

use crate::fl;

#[derive(Debug)]
pub struct ToolBarModel {
    selected_count: usize,
}

#[derive(Debug)]
pub enum SizeOption {
    Size0KB,
    Size30KB,
    Size100KB,
    Size500KB,
    SizeGreater500KB,
}

#[derive(Debug)]
pub enum ToolBarInput {
    CleanFilters,
    AllSizeOptions(bool),
    SelectedVideo(bool),
    CheckButtonToggled(SizeOption, bool),
}

#[derive(Debug)]
pub enum ToolBarOutput {
    ZoomIn,
    ZoomOut,
    SelectAll(bool),
    SearchEntry(String),
    SizeFilter0KB(bool),
    SizeFilter30KB(bool),
    SizeFilter100KB(bool),
    SizeFilter500KB(bool),
    SizeFilterGreater500KB(bool),
}

#[relm4::component(pub async)]
impl AsyncComponent for ToolBarModel {
    type Init = ();
    type Input = ToolBarInput;
    type Output = ToolBarOutput;
    type CommandOutput = ();

    view! {
        #[name = "toolbar"]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_hexpand: true,
            set_margin_bottom: 4,
            set_spacing: 6,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_width_request: 324,
                set_hexpand: false,
                set_halign: gtk::Align::Start,
                set_spacing: 6,

                gtk::Button {
                    set_icon_name: icon_names::MINUS,
                    set_tooltip: fl!("deselect-all"),
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.output(ToolBarOutput::SelectAll(false)).unwrap();
                    },
                },

                gtk::Button {
                    set_icon_name: icon_names::CHECKMARK,
                    set_tooltip: fl!("select-all"),
                    add_css_class: "flat",
                    connect_clicked[sender] => move |_| {
                        sender.output(ToolBarOutput::SelectAll(true)).unwrap();
                    },
                },

                gtk::Label {
                    set_label: fl!("selected-count"),
                    set_xalign: 0.0,
                    set_margin_start: 6,
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.selected_count.to_string(),
                    set_xalign: 0.0,
                    add_css_class: "dim-label",
                    set_margin_start: 4,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_margin_start: 12,
                set_margin_end: 12,
                set_halign: gtk::Align::Center,
                set_spacing: 12,

                gtk::Button {
                    set_icon_name: icon_names::MINUS,
                    set_tooltip: fl!("zoom-out"),
                    add_css_class: "circular",
                    connect_clicked[sender] => move |_| {
                        sender.output(ToolBarOutput::ZoomOut).unwrap_or_default();
                    },
                },

                gtk::Button {
                    set_icon_name: icon_names::PLUS,
                    set_tooltip: fl!("zoom-in"),
                    add_css_class: "circular",
                    connect_clicked[sender] => move |_| {
                        sender.output(ToolBarOutput::ZoomIn).unwrap_or_default();
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_width_request: 324,
                set_hexpand: false,
                set_halign: gtk::Align::End,
                set_spacing: 6,

                append = &gtk::Button {
                    set_icon_name: icon_names::FILTER_DISMISS_FILLED,
                    set_tooltip: fl!("clean-filters"),
                    add_css_class: "flat",
                    connect_clicked => ToolBarInput::CleanFilters,
                },

                append = &gtk::MenuButton {
                    set_label: fl!("filter"),
                    set_always_show_arrow: true,
                    set_css_classes: &["flat"],

                    #[wrap(Some)]
                    set_popover: popover = &gtk::Popover {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 1,

                            gtk::Label {
                                set_label: fl!("size"),
                                set_xalign: 0.0,
                                set_margin_bottom: 3,
                                set_css_classes: &["title-4"],
                            },

                            gtk::Frame {
                                gtk::ListBox {
                                    set_selection_mode: gtk::SelectionMode::None,
                                    set_show_separators: false,
                                    set_css_classes: &["rich-list"],

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_all_size)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::AllSizeOptions(checkbox.is_active()));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: fl!("all-sizes"),
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_size_0kb)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::CheckButtonToggled(
                                                        SizeOption::Size0KB,
                                                        checkbox.is_active(),
                                                    ));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: "0 KB",
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_size_30kb)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::CheckButtonToggled(
                                                        SizeOption::Size30KB,
                                                        checkbox.is_active(),
                                                    ));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: "1 - 30 KB",
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_size_100kb)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::CheckButtonToggled(
                                                        SizeOption::Size100KB,
                                                        checkbox.is_active(),
                                                    ));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: "31 - 100 KB",
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_size_500kb)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::CheckButtonToggled(
                                                        SizeOption::Size500KB,
                                                        checkbox.is_active(),
                                                    ));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: "101 - 500 KB",
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },

                                    gtk::ListBoxRow {
                                        gtk::Box {
                                            #[name(chk_size_greater_500kb)]
                                            gtk::CheckButton {
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_margin_start: 0,
                                                set_margin_end: 0,
                                                set_active: true,
                                                connect_toggled[sender] => move |checkbox| {
                                                    sender.input(ToolBarInput::CheckButtonToggled(
                                                        SizeOption::SizeGreater500KB,
                                                        checkbox.is_active(),
                                                    ));
                                                },
                                            },
                                            gtk::Label {
                                                set_label: fl!("greater-than-500-kb"),
                                                set_xalign: 0.0,
                                                set_halign: gtk::Align::Start,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                            },
                                        }
                                    },
                                }
                            },
                        },
                    },
                },

                #[name(search_entry)]
                append = &gtk::SearchEntry {
                    set_placeholder_text: Some(fl!("search")),
                    set_search_delay: 500,
                    connect_search_changed[sender] => move |entry| {
                        let query = entry.text().as_str().to_string();
                        sender.output(ToolBarOutput::SearchEntry(query)).unwrap_or_default();
                    }
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = ToolBarModel { selected_count: 0 };
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
            ToolBarInput::AllSizeOptions(is_selected) => {
                if is_selected {
                    widgets.chk_size_0kb.set_active(true);
                    widgets.chk_size_30kb.set_active(true);
                    widgets.chk_size_100kb.set_active(true);
                    widgets.chk_size_500kb.set_active(true);
                    widgets.chk_size_greater_500kb.set_active(true);
                }
            }
            ToolBarInput::CleanFilters => {
                widgets.chk_all_size.set_active(true);
                widgets.search_entry.set_text("");
            }
            ToolBarInput::SelectedVideo(is_selected) => {
                if is_selected {
                    self.selected_count += 1;
                } else if self.selected_count > 0 {
                    self.selected_count -= 1;
                } else {
                    self.selected_count = 0;
                }
            }
            ToolBarInput::CheckButtonToggled(size_option, is_selected) => {
                if !is_selected {
                    widgets.chk_all_size.set_active(false);
                }

                match size_option {
                    SizeOption::Size0KB => {
                        sender
                            .output(ToolBarOutput::SizeFilter0KB(is_selected))
                            .unwrap_or_default();
                    }
                    SizeOption::Size30KB => {
                        sender
                            .output(ToolBarOutput::SizeFilter30KB(is_selected))
                            .unwrap_or_default();
                    }
                    SizeOption::Size100KB => {
                        sender
                            .output(ToolBarOutput::SizeFilter100KB(is_selected))
                            .unwrap_or_default();
                    }
                    SizeOption::Size500KB => {
                        sender
                            .output(ToolBarOutput::SizeFilter500KB(is_selected))
                            .unwrap_or_default();
                    }
                    SizeOption::SizeGreater500KB => {
                        sender
                            .output(ToolBarOutput::SizeFilterGreater500KB(is_selected))
                            .unwrap_or_default();
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
