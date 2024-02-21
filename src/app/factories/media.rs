use crate::app::models;

use relm4::{
    prelude::*,
    gtk,
    gtk::prelude::*,
    gtk::pango,
    // gtk::gdk_pixbuf::{
    //     Pixbuf,
    //     Colorspace,
    // },
    // gtk::glib,
    view,
    factory::{
        AsyncFactoryComponent,
        AsyncFactorySender,
        DynamicIndex,
    },
    loading_widgets::LoadingWidgets,
};

pub struct MediaModel {
    pub media: models::Media,
    pub index: DynamicIndex,
    // pixbuf: Option<Pixbuf>,
}

#[derive(Debug)]
pub enum MediaInput {
    Selected(bool),
    ZoomIn(i32),
    ZoomOut(i32),
}

#[derive(Debug)]
pub enum MediaOutput {
    Selected(bool),
}

#[relm4::factory(pub async)]
impl AsyncFactoryComponent for MediaModel {
    type Init = models::Media;
    type Input = MediaInput;
    type Output = MediaOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 2,
            set_css_classes: &["card", "activatable", "media-item-box", "border-spacing"],
            set_tooltip_text: Some(&self.media.name),

            gtk::Overlay {
                #[watch]
                set_size_request: (self.media.thumbnail_size, self.media.thumbnail_size),

                add_overlay = &gtk::Picture {
                    set_margin_all: 3,
                    set_content_fit: gtk::ContentFit::Contain,
                    set_can_shrink: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    // set_file: Some(&gtk::gio::File::for_path(self.media.thumb_path.as_str())),
                    // set_pixbuf: self.pixbuf.as_ref(),
                },

                #[name(checkbox)]
                add_overlay = &gtk::CheckButton {
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,
                    set_css_classes: &["border-spacing"],
                    #[watch]
                    set_active: self.media.is_selected,
                    connect_toggled[sender] => move |checkbox| {
                        sender.input(MediaInput::Selected(checkbox.is_active()));
                    }
                },
            },

            gtk::Label {
                set_label: &self.media.name,
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
                set_width_request: models::media::THUMBNAIL_SIZE,
                set_height_request: models::media::THUMBNAIL_SIZE + 14,
                set_css_classes: &["card", "media-item-box", "border-spacing"],

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_height_request: models::media::THUMBNAIL_SIZE + 14,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init_model(
        media: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        // let filename = media.path.as_str();

        // let pixbuf = match core::video::thumbnail(filename).await {
        //     Ok(thumb) => {
        //         if let Some(data) = thumb.data.as_ref() {
        //             let bytes = glib::Bytes::from(data);
        //             let rowstride = Pixbuf::calculate_rowstride(
        //                 Colorspace::Rgb, 
        //                 true, 
        //                 8, 
        //                 thumb.width as i32, 
        //                 thumb.height as i32,
        //             );

        //             let pixbuf = Pixbuf::from_bytes(
        //                 &bytes, 
        //                 Colorspace::Rgb, 
        //                 true, 
        //                 8, 
        //                 thumb.width as i32, 
        //                 thumb.height as i32, 
        //                 rowstride,
        //             );

        //             Some(pixbuf)
        //         } else {
        //             None
        //         }
        //     }
        //     Err(err) => {
        //         tracing::error!("{} {}", fl!("generic-error"), err);
        //         None
        //     }
        // };

        Self {
            media,
            index: index.clone(),
            // pixbuf: None,
        }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncFactorySender<Self>,
    ) {
        match message {
            MediaInput::Selected(is_selected) => {
                self.media.is_selected = is_selected;
                sender.output(MediaOutput::Selected(is_selected)).unwrap_or_default();
            }
            MediaInput::ZoomIn(size) => {
                self.media.thumbnail_size = size;
            }
            MediaInput::ZoomOut(size) => {
                self.media.thumbnail_size = size;
            }
        }   
    }
}
