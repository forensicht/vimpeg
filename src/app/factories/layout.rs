use relm4::{
    factory::{FactoryComponent, FactorySender},
    gtk,
    gtk::prelude::{OrientableExt, WidgetExt},
    RelmWidgetExt,
};

use crate::app::models;

pub struct LayoutModel {
    pub layout: models::Layout,
}

#[relm4::factory(pub)]
impl FactoryComponent for LayoutModel {
    type Init = models::Layout;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 2,
            set_css_classes: &["card", "activatable", "media-item-box", "border-spacing"],

            gtk::Image {
                set_width_request: 100,
                set_height_request: 100,
                set_margin_all: 3,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                #[watch]
                set_resource: Some(&self.layout.resource),
            },
        }
    }

    fn init_model(layout: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self { layout }
    }
}
