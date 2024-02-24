pub mod toolbar;

use crate::fl;
use core_chasam as service;
use core_chasam::csam::StateMedia;
use crate::app::{
    models,
    components::searchbar::{
        SearchBarModel,
        SearchBarOutput,
    },
    factories::media_item::MediaItem,
};
use toolbar::{
    ToolbarModel,
    ToolbarOutput,
};

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

use relm4::{
    prelude::*,
    gtk::prelude::*,
    adw, 
    binding::Binding, 
    component::{
        AsyncComponent, 
        AsyncComponentController, 
        AsyncComponentParts, 
        AsyncComponentSender, 
        AsyncController,
    },
    typed_view::grid::TypedGridView,
};
use anyhow::Result;

pub struct CsamModel {
    searchbar: AsyncController<SearchBarModel>,
    toolbar: AsyncController<ToolbarModel>,
    media_list_wrapper: TypedGridView<MediaItem, gtk::NoSelection>,
    media_list_filter: Rc<RefCell<models::MediaFilter>>,
    thumbnail_size: i32,
    service: service::csam::SearchMedia,
}

impl CsamModel {
    pub fn new(
        searchbar: AsyncController<SearchBarModel>,
        toolbar: AsyncController<ToolbarModel>,
        media_list_wrapper: TypedGridView<MediaItem, gtk::NoSelection>,
        service: service::csam::SearchMedia,
    ) -> Self {
        Self {
            searchbar,
            toolbar,
            media_list_wrapper,
            media_list_filter: Rc::new(RefCell::new(models::MediaFilter::default())),
            thumbnail_size: models::media::THUMBNAIL_SIZE,
            service,
        }
    }
}

#[derive(Debug)]
pub enum CsamInput {
    // Searchbar
    StartSearch(PathBuf),
    SearchCompleted(usize),

    // Toolbar
    ZoomIn,
    ZoomOut,
    SelectAllMedias(bool),
    SizeFilter0KB(bool),
    SizeFilter30KB(bool),
    SizeFilter100KB(bool),
    SizeFilter500KB(bool),
    SizeFilterA500KB(bool),
    SearchEntry(String),

    MediaListSelect(u32),
    Notify(String, u32),
}

#[derive(Debug)]
pub enum CsamCommandOutput {
    SearchCompleted,
    AddMedia(Result<models::Media>),
    MediaFound(usize),
}

#[relm4::component(pub async)]
impl AsyncComponent for CsamModel {
    type Init = ();
    type Input = CsamInput;
    type Output = ();
    type CommandOutput = CsamCommandOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,

            append = &adw::HeaderBar {
                set_hexpand: true,
                set_css_classes: &["flat"],
                set_show_start_title_buttons: false,
                set_show_end_title_buttons: true,

                #[wrap(Some)]
                set_title_widget = model.searchbar.widget(),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_css_classes: &["view"],

                append = model.toolbar.widget(),

                append = &adw::ToastOverlay {
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,

                        append = &gtk::Paned {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_hexpand: true,
                            set_vexpand: true,
                            set_resize_start_child: true,
                            set_resize_end_child: true,
                            set_shrink_start_child: false,
                            set_shrink_end_child: false,
                            set_margin_bottom: 6,
                            set_margin_end: 6,
                            set_margin_start: 6,

                            #[wrap(Some)]
                            set_start_child = &gtk::Frame {
                                set_width_request: 800,
                                set_vexpand: true,
                                set_margin_end: 6,

                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_hexpand: true,
                                    set_vexpand: true,

                                    #[local_ref]
                                    media_list_widget -> gtk::GridView {
                                        set_vexpand: true,
                                        set_single_click_activate: true,
                                        set_enable_rubberband: false,
                                        set_max_columns: 10,
                                        connect_activate[sender] => move |_, position| {
                                            sender.input(CsamInput::MediaListSelect(position));
                                        },
                                    },
                                },
                            },

                            #[wrap(Some)]
                            set_end_child = &gtk::Frame {
                                set_width_request: 300,
                                set_vexpand: true,
                                set_margin_start: 6,
                            },
                        },
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
        let searchbar_controller = SearchBarModel::builder()
            .launch(())
            .forward(sender.input_sender(), |output| match output {
                SearchBarOutput::StartSearch(path) => CsamInput::StartSearch(path),
                SearchBarOutput::Notify(msg, timeout) => CsamInput::Notify(msg, timeout),
            });

        let toolbar_controller = ToolbarModel::builder()
            .launch_with_broker((), &toolbar::SELECT_BROKER)
            .forward(sender.input_sender(), |output| match output {
                ToolbarOutput::ZoomIn => CsamInput::ZoomIn,
                ToolbarOutput::ZoomOut => CsamInput::ZoomOut,
                ToolbarOutput::SelectAll(is_selected) => CsamInput::SelectAllMedias(is_selected),
                ToolbarOutput::SearchEntry(query) => CsamInput::SearchEntry(query),
                ToolbarOutput::SizeFilter0KB(is_active) => CsamInput::SizeFilter0KB(is_active),
                ToolbarOutput::SizeFilter30KB(is_active) => CsamInput::SizeFilter30KB(is_active),
                ToolbarOutput::SizeFilter100KB(is_active) => CsamInput::SizeFilter100KB(is_active),
                ToolbarOutput::SizeFilter500KB(is_active) => CsamInput::SizeFilter500KB(is_active),
                ToolbarOutput::SizeFilterGreater500KB(is_active) => CsamInput::SizeFilterA500KB(is_active),
            });

        let media_list_wrapper: TypedGridView<MediaItem, gtk::NoSelection> =
            TypedGridView::new();

        let service = service::csam::SearchMedia::new();
        let mut model = CsamModel::new(
            searchbar_controller,
            toolbar_controller,
            media_list_wrapper,
            service,
        );

        let filter = model.media_list_filter.clone();
        model.media_list_wrapper.add_filter(on_filter(filter));
        model.media_list_wrapper.set_filter_status(0, false);

        let media_list_widget = &model.media_list_wrapper.view;
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
            CsamInput::ZoomIn => {
                self.apply_media_zoom(true).await;
            }
            CsamInput::ZoomOut => {
                self.apply_media_zoom(false).await;
            }
            CsamInput::StartSearch(path) => {
                self.media_list_wrapper.clear();
                self.on_search(path, &sender).await;
            }
            CsamInput::SearchCompleted(count) => {
                println!("{}", count);
            }
            CsamInput::SelectAllMedias(is_selected) => {
                self.on_select_all_medias(is_selected).await;
            }
            CsamInput::SearchEntry(query) => {
                self.media_list_filter.borrow_mut().search_entry = Some(query);
                self.apply_media_filters().await;
            }
            CsamInput::SizeFilter0KB(is_active) => {
                self.media_list_filter.borrow_mut().size_0 = is_active;
                self.apply_media_filters().await;
            }
            CsamInput::SizeFilter30KB(is_active) => {
                self.media_list_filter.borrow_mut().size_30 = is_active;
                self.apply_media_filters().await;
            }
            CsamInput::SizeFilter100KB(is_active) => {
                self.media_list_filter.borrow_mut().size_100 = is_active;
                self.apply_media_filters().await;
            }
            CsamInput::SizeFilter500KB(is_active) => {
                self.media_list_filter.borrow_mut().size_500 = is_active;
                self.apply_media_filters().await;
            }
            CsamInput::SizeFilterA500KB(is_active) => {
                self.media_list_filter.borrow_mut().size_greater_500 = is_active;
                self.apply_media_filters().await;
            }
            CsamInput::MediaListSelect(position) => {
                if let Some(item) = self.media_list_wrapper.get(position) {
                    let media = &item.borrow().media;
                    println!("Select item: {}", media.name);
                }
            }
            CsamInput::Notify(msg, timeout) => {
                println!("{} - {}", msg, timeout);
            }
        }   

        self.update_view(widgets, sender);
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CsamCommandOutput::SearchCompleted => {
                println!("Search Completed");
            }
            CsamCommandOutput::MediaFound(count) => {
                println!("Media Found: {}", count);
            }
            CsamCommandOutput::AddMedia(result) => {
                match result {
                    Ok(media) => {
                        self.media_list_wrapper.append(MediaItem::new(media));
                    }
                    Err(error) => tracing::error!("{}: {}", fl!("generic-error"), error),
                }
            }
        }
    }
}

impl CsamModel {
    async fn on_search(
        &mut self, 
        path: PathBuf,
        sender: &AsyncComponentSender<CsamModel>,
    ) {
        let (tx, mut rx) = relm4::tokio::sync::mpsc::channel(100);

        sender.command(|out, shutdown| {
            shutdown.register(async move {
                while let Some(state) = rx.recv().await {
                    match state {
                        StateMedia::Completed => {
                            out.send(CsamCommandOutput::SearchCompleted)
                                .unwrap_or_default();
                        }
                        StateMedia::Found(count) => {
                            out.send(CsamCommandOutput::MediaFound(count))
                                .unwrap_or_default();
                        }
                        StateMedia::Ok(media) => {
                            let media = models::Media::from(&media);
                            out.send(CsamCommandOutput::AddMedia(Ok(media)))
                                .unwrap_or_default();
                        }
                        StateMedia::Err(error) => {
                            out.send(CsamCommandOutput::AddMedia(Err(error)))
                                .unwrap_or_default();
                        }
                    }
                }
            })
            .drop_on_shutdown()
        });

        match self.service.search(path, tx).await {
            Err(error) => tracing::error!("{error}"),
            _ => (),
        }
    }

    async fn on_select_all_medias(
        &mut self,
        is_selected: bool,
    ) {
        // let len = self.media_list_wrapper.len();
        for position in 0..160 {
            let item = self.media_list_wrapper.get(position).unwrap();
            let binding = &mut item.borrow_mut().active;
            let mut guard = binding.guard();
            *guard = is_selected;
        }
    }

    async fn apply_media_filters(&mut self) {
        self.media_list_wrapper.set_filter_status(0, false);
        self.media_list_wrapper.set_filter_status(0, true);
    }

    async fn apply_media_zoom(&mut self, is_zoom_in: bool) {
        use models::media::THUMBNAIL_SIZE;
        use models::media::ZOOM_SIZE;

        if is_zoom_in {
            if self.thumbnail_size < 320 {
                self.thumbnail_size += ZOOM_SIZE;
            }
        } else {
            if self.thumbnail_size > THUMBNAIL_SIZE {
                let mut thumb_size = self.thumbnail_size - ZOOM_SIZE;
                if thumb_size < THUMBNAIL_SIZE {
                    thumb_size = THUMBNAIL_SIZE;
                }
                self.thumbnail_size = thumb_size;
            }
        }

        let len = self.media_list_wrapper.len();
        for position in 0..len {
            let item = self.media_list_wrapper.get(position).unwrap();
            let binding = &mut item.borrow_mut().thumbnail_size;
            let mut guard = binding.guard();
            *guard = self.thumbnail_size;
        }
    }
}

fn on_filter(filter: Rc<RefCell<models::MediaFilter>>) -> impl Fn(&MediaItem) -> bool {
    move |item: &MediaItem| -> bool {
        let filter = filter.borrow();
        let media = &item.media;
        let mut is_visible = true;

        if let Some(query) = &filter.search_entry {
            is_visible = media.name.to_lowercase().contains(&query.to_lowercase());  
        }

        if !filter.size_0 && media.size == 0 {
            is_visible = false;
        } else if !filter.size_30 && (media.size > 0 && media.size <= 30) {
            is_visible = false;
        } else if !filter.size_100 && (media.size > 30 && media.size <= 100) {
            is_visible = false;
        } else if !filter.size_500 && (media.size > 100 && media.size <= 500) {
            is_visible = false;
        } else if !filter.size_greater_500 && media.size > 500 {
            is_visible = false;
        }

        is_visible
    }
}
