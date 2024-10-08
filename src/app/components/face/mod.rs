use std::path::PathBuf;

use crate::app::components::searchbar::{SearchBarInput, SearchBarModel, SearchBarOutput};

use relm4::{
    adw,
    component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender, Controller},
    gtk::prelude::{BoxExt, OrientableExt, WidgetExt},
    prelude::*,
};

pub struct FaceModel {
    searchbar: Controller<SearchBarModel>,
}

impl FaceModel {
    pub fn new(searchbar: Controller<SearchBarModel>) -> Self {
        Self { searchbar }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum FaceInput {
    StartSearch(PathBuf),
    StopSearch,
    SearchCompleted(usize),
    Notify(String, u32),
}

#[relm4::component(pub async)]
impl AsyncComponent for FaceModel {
    type Init = ();
    type Input = FaceInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            append = &adw::HeaderBar {
                set_hexpand: true,
                set_css_classes: &["flat"],
                set_show_start_title_buttons: false,
                set_show_end_title_buttons: true,

                #[wrap(Some)]
                set_title_widget = model.searchbar.widget(),
            },

            append = &adw::ToastOverlay {
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_css_classes: &["view"],

                    append = &gtk::Label {
                        set_label: "Content Face",
                    },
                },
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let searchbar_controller =
            SearchBarModel::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    SearchBarOutput::StartSearch(path) => FaceInput::StartSearch(path),
                    SearchBarOutput::StopSearch => FaceInput::StopSearch,
                    SearchBarOutput::Notify(msg, timeout) => FaceInput::Notify(msg, timeout),
                });

        let model = FaceModel::new(searchbar_controller);
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
            FaceInput::StartSearch(path) => {
                println!("{}", path.display());
            }
            FaceInput::StopSearch => {
                println!("Stop Search");
            }
            FaceInput::SearchCompleted(count) => {
                println!("{}", count);
                self.searchbar.emit(SearchBarInput::SearchCompleted);
            }
            FaceInput::Notify(msg, timeout) => {
                println!("{} - {}", msg, timeout);
            }
        }

        self.update_view(widgets, sender);
    }
}
