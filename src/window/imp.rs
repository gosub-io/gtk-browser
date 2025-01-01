use crate::engine::GosubEngineConfig;
use crate::eventloop::WindowEventLoopDummy;
use crate::tab::{GosubTab, GosubTabManager, HtmlViewMode, TabCommand, TabId};
use crate::window::message::Message;
use crate::window::tab_context_menu::{build_context_menu, setup_context_menu_actions, TabInfo};
use crate::{fetcher, runtime};
use async_channel::{Receiver, Sender};
use glib::subclass::InitializingObject;
use gosub_engine::prelude::*;
use gtk4::gio::SimpleActionGroup;
use gtk4::glib::subclass::Signal;
use gtk4::glib::Quark;
use gtk4::graphene::Point;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, Button, CompositeTemplate, DrawingArea, Entry, GestureClick, Image, Notebook, PopoverMenu, PopoverMenuFlags, ScrolledWindow, Settings, TemplateChild, TextView, ToggleButton, Widget};
use log::info;
use once_cell::sync::Lazy;
use reqwest::Url;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use sourceview5::prelude::*;
use sourceview5::{LanguageManager, View};

// Create a static Quark as a unique key
static TAB_ID_QUARK: Lazy<Quark> = Lazy::new(|| Quark::from_str("tab_id"));

pub trait WidgetExtTabId {
    fn set_tab_id(&self, tab_id: TabId);
    fn get_tab_id(&self) -> Option<TabId>;
}

impl<T: IsA<Widget>> WidgetExtTabId for T {
    fn set_tab_id(&self, tab_id: TabId) {
        unsafe {
            // - 'tab_id' is of type 'TabId', which is 'Copy' and 'static'.
            // - We ensure that the same type is used when retrieving the data.
            self.set_qdata(*TAB_ID_QUARK, tab_id);
        }
    }

    fn get_tab_id(&self) -> Option<TabId> {
        unsafe { self.qdata::<TabId>(*TAB_ID_QUARK).map(|ptr| *ptr.as_ref()) }
    }
}

#[derive(CompositeTemplate)]
#[template(resource = "/io/gosub/browser-gtk/ui/window.ui")]
pub struct BrowserWindow {
    #[template_child]
    pub searchbar: TemplateChild<Entry>,
    #[template_child]
    pub tab_bar: TemplateChild<Notebook>,
    #[template_child]
    pub log_scroller: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub log: TemplateChild<TextView>,

    // Other stuff that are non-widgets
    pub tab_manager: Arc<Mutex<GosubTabManager>>,
    pub sender: Arc<Sender<Message>>,
    pub receiver: Arc<Receiver<Message>>,
}

impl Default for BrowserWindow {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded::<Message>();
        Self {
            searchbar: TemplateChild::default(),
            tab_bar: TemplateChild::default(),
            log_scroller: TemplateChild::default(),
            log: TemplateChild::default(),

            tab_manager: Arc::new(Mutex::new(GosubTabManager::new())),
            sender: Arc::new(tx),
            receiver: Arc::new(rx),
        }
    }
}

impl BrowserWindow {
    pub(crate) fn get_sender(&self) -> Arc<Sender<Message>> {
        self.sender.clone()
    }

    pub(crate) fn get_receiver(&self) -> Arc<Receiver<Message>> {
        self.receiver.clone()
    }
}

#[glib::object_subclass]
impl ObjectSubclass for BrowserWindow {
    const NAME: &'static str = "BrowserWindow";
    type Type = super::BrowserWindow;
    type ParentType = gtk4::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for BrowserWindow {
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| vec![Signal::builder("update-tabs").build()]);

        SIGNALS.as_ref()
    }

    fn constructed(&self) {
        self.parent_constructed();
        self.log("Browser created...");
    }
}

impl WidgetImpl for BrowserWindow {}
impl WindowImpl for BrowserWindow {}
impl ApplicationWindowImpl for BrowserWindow {}

#[gtk4::template_callbacks]
impl BrowserWindow {
    #[template_callback]
    fn handle_new_tab(&self, _btn: &Button) {
        todo!("not yet implemented");
    }

    #[template_callback]
    fn handle_close_tab(&self, _btn: &Button) {
        todo!("not yet implemented");
    }

    #[template_callback]
    fn handle_prev_clicked(&self, _btn: &Button) {
        todo!("not yet implemented");
    }

    #[template_callback]
    fn handle_toggle_darkmode(&self, btn: &ToggleButton) {
        self.log("Toggling dark mode");

        info!(target: "gtk", "Toggle dark mode action triggered");
        let settings = Settings::default().expect("Failed to get default GtkSettings");
        settings.set_property("gtk-application-prefer-dark-theme", btn.is_active());
    }

    #[template_callback]
    fn handle_refresh_clicked(&self, _btn: &Button) {
        self.log("Refreshing the current page");
    }

    #[template_callback]
    async fn handle_searchbar_clicked(&self, entry: &Entry) {
        let Some(page_num) = self.tab_bar.current_page() else {
            self.log("No active tab to load the URL");
            return;
        };

        match self.tab_bar.nth_page(Some(page_num)) {
            Some(page) => {
                self.log(format!("Visiting the URL {}", entry.text().as_str()).as_str());

                let tab_id = page.get_tab_id().unwrap();
                let url_str = entry.text().to_string();

                self.sender.send(Message::LoadUrl(tab_id, url_str)).await.unwrap();
            }
            None => {
                self.log("No active tab to load the URL");
            }
        }
    }

    fn sanitize_url(&self, url_str: &str) -> (HtmlViewMode, String) {
        let mut view_mode = HtmlViewMode::Rendered;
        let mut url = url_str.to_string();

        if url.starts_with("source:") {
            view_mode = HtmlViewMode::Source;
            url = url.replace("source:", "");
        }
        if url.starts_with("raw:") {
            view_mode = HtmlViewMode::RawSource;
            url = url.replace("raw:", "");
        }
        if url.starts_with("json:") {
            view_mode = HtmlViewMode::Json;
            url = url.replace("json:", "");
        }
        if url.starts_with("xml:") {
            view_mode = HtmlViewMode::Xml;
            url = url.replace("xml:", "");
        }
        if url.starts_with("about:") | url.starts_with("gosub:") {
            url = url.replace("about:", "");
            url = url.replace("gosub:", "");

            return (HtmlViewMode::About, url);
        }

        // Make sure the url starts with a proper scheme or about:. If no scheme is present, we assume https://
        if url.starts_with("http://") || url.starts_with("https://") {
            // URL already has a scheme, we don't need to do anything
        } else {
            // No scheme, we use https:// as a prefix
            url = format!("https://{}", url);
        }

        (view_mode, url)
    }
}

impl BrowserWindow {
    pub fn log(&self, message: &str) {
        let s = format!("[{}] {}\n", chrono::Local::now().format("%X"), message);
        info!(target: "ftk", "Logmessage: {}", s.as_str());

        let buf = self.log.buffer();
        let mut iter = buf.end_iter();
        buf.insert(&mut iter, s.as_str());

        let mark = buf.create_mark(None, &iter, false);
        self.log.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
    }

    pub(crate) fn close_tab(&self, tab_id: TabId) {
        let mut manager = self.tab_manager.lock().unwrap();
        if manager.tab_count() == 1 {
            self.log("Cannot close the last tab");
            return;
        }
        manager.remove_tab(tab_id);
    }

    pub(crate) fn refresh_tabs(&self) {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

        rt.block_on(self.refresh_tabs_async())
    }

    /// Refresh tabs will asynchronously update the tab bar based on the current state of the tab
    /// manager. Any mutations that are done on tabs in the manager, are recorded as commands and
    /// played back here.
    async fn refresh_tabs_async(&self) {
        let mut manager = self.tab_manager.lock().unwrap();
        let commands = manager.commands();
        drop(manager);

        for cmd in commands {
            match cmd {
                TabCommand::Activate(tab_id) => {
                    let page_num = self.get_page_num_for_tab(tab_id);
                    self.tab_bar.set_current_page(page_num);
                }
                TabCommand::Insert(tab_id, position) => {
                    let manager = self.tab_manager.lock().unwrap();
                    let tab = manager.get_tab(tab_id).unwrap().clone();
                    drop(manager);

                    let label = self.create_tab_label(&tab);
                    let default_page = self.generate_default_page();

                    let notebook_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                    notebook_box.append(&default_page);
                    notebook_box.set_tab_id(tab.id());
                    self.tab_bar.insert_page(&notebook_box, Some(&label), Some(position));

                    // We can reorder tab, unless it's pinned/pinned
                    if let Some(page) = self.tab_bar.nth_page(Some(position)) {
                        self.tab_bar.set_tab_reorderable(&page, !tab.is_pinned());
                    }
                }
                TabCommand::Close(tab_id) => {
                    let page_num = self.get_page_num_for_tab(tab_id);
                    self.tab_bar.remove_page(page_num);
                }
                TabCommand::CloseAll => {
                    for _ in 0..self.tab_bar.pages().n_items() {
                        self.tab_bar.remove_page(Some(0));
                    }
                }
                TabCommand::Move(tab_id, position) => {
                    let page_num = self.get_page_num_for_tab(tab_id);
                    let page = self.tab_bar.nth_page(page_num).unwrap();
                    self.tab_bar.reorder_child(&page, Some(position));
                }
                TabCommand::Update(tab_id) => {
                    let manager = self.tab_manager.lock().unwrap();
                    let tab = manager.get_tab(tab_id).unwrap().clone();
                    drop(manager);
                    let page_num = self.get_page_num_for_tab(tab_id).unwrap();

                    let scrolled_window = gtk4::ScrolledWindow::builder()
                        .hscrollbar_policy(gtk4::PolicyType::Never)
                        .vscrollbar_policy(gtk4::PolicyType::Automatic)
                        .vexpand(true)
                        .build();

                    if tab.viewmode() == HtmlViewMode::Source {
                        // @todo: we should be able to do things like json as well. See:
                        // https://github.com/danirod/cartero/blob/4c3a356ccb04be272123126354b91ef707fb7d0e/src/widgets/response_panel.rs#L254C9-L268C19
                        let lang = LanguageManager::default().language("html");
                        let buf = sourceview5::Buffer::builder()
                            .text(tab.content())
                            .highlight_syntax(true)
                            .highlight_matching_brackets(true)
                            .build();

                        match lang {
                            Some(lang) => {
                                buf.set_language(Some(&lang));
                            }
                            None => {
                                buf.set_language(None);
                            }
                        }

                        let view = View::new();
                        view.set_editable(false);
                        view.set_show_line_marks(true);
                        view.set_show_line_numbers(true);
                        view.set_auto_indent(true);
                        view.set_cursor_visible(false);
                        view.set_buffer(Some(&buf));
                        view.set_monospace(true);
                        scrolled_window.set_child(Some(&view));
                    } else if tab.has_drawer() {
                        let tab_clone = tab.clone();

                        let area = DrawingArea::default();
                        area.set_draw_func(move |_area, cr, width, height| {
                            let binding = tab_clone.drawer();
                            let mut drawer_lock = binding.lock().unwrap();

                            if let Some(drawer) = drawer_lock.as_mut() {
                                let mut render_backend = <GosubEngineConfig as HasRenderBackend>::RenderBackend::new();
                                let size = SizeU32::new(width as u32, height as u32);

                                // Drawer.draw will populate the scene with elements from the tree
                                let mut win_data = WindowData {
                                    scene: Scene::new(),
                                    cr: Some(cr.clone()),
                                };
                                drawer.draw(&mut render_backend, &mut win_data, size, &WindowEventLoopDummy);

                                let mut active_win_data = ActiveWindowData { cr: cr.clone() };
                                _ = render_backend.render(&mut win_data, &mut active_win_data);

                                return;
                            }

                            drop(drawer_lock);
                        });

                        scrolled_window.set_child(Some(&area));
                    } else {
                        // No drawer is (yet) created, so we display a default page (with the gosub logo)
                        let content = self.generate_default_page();
                        scrolled_window.set_child(Some(&content));
                    };

                    // let content = TextView::builder().editable(false).wrap_mode(gtk4::WrapMode::Word).build();
                    // scrolled_window.set_child(Some(&content));
                    scrolled_window.set_tab_id(tab.id());

                    // Since a tab contains a box, we just update the child inside the box. This way
                    // we do not need to remove the actual page from the notebook, which results in all
                    // kind of issues.
                    let page = self.tab_bar.nth_page(Some(page_num)).unwrap();
                    let notebox_box = page.downcast_ref::<gtk4::Box>().unwrap();
                    notebox_box.remove(&notebox_box.first_child().unwrap());
                    notebox_box.append(&scrolled_window);

                    // We update the tab label as well
                    let tab_label = self.create_tab_label(&tab);
                    self.tab_bar.set_tab_label(notebox_box, Some(&tab_label));

                    // self.tab_bar.set_current_page(Some(page_num));
                }
            }
        }
    }

    fn create_pinned_tab_label(&self, tab: &GosubTab) -> Widget {
        if let Some(favicon) = &tab.favicon() {
            let img = Image::from_paintable(Some(&favicon.clone()));
            img.set_margin_top(5);
            img.set_margin_bottom(5);
            return img.into();
        }

        // No favicon for this pinned tab, so use a default icon
        let img = Image::from_resource("/io/gosub/browser-gtk/assets/pin.svg");
        img.set_margin_top(5);
        img.set_margin_bottom(5);
        img.into()
    }

    fn create_normal_tab_label(&self, tab: &GosubTab) -> Widget {
        let label_vbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);

        // When the tab is loading, we show a spinner
        if tab.is_loading() {
            let spinner = gtk4::Spinner::new();
            spinner.start();
            label_vbox.append(&spinner);
        } else if let Some(favicon) = &tab.favicon() {
            label_vbox.append(&Image::from_paintable(Some(&favicon.clone())));
        }

        let mut title = tab.title().to_string();
        title.truncate(20);
        let tab_label = gtk4::Label::new(Some(title.as_str()));
        label_vbox.append(&tab_label);

        let tab_close_button = Button::builder()
            .halign(gtk4::Align::End)
            .has_frame(false)
            .margin_bottom(0)
            .margin_end(0)
            .margin_start(0)
            .margin_top(0)
            .build();
        let img = Image::from_icon_name("window-close-symbolic");
        tab_close_button.set_child(Some(&img));
        label_vbox.append(&tab_close_button);

        let window_clone = self.obj().clone();
        let tab_id = tab.id();
        tab_close_button.connect_clicked(move |_| {
            info!(target: "gtk", "Clicked close button for tab {}", tab_id);
            window_clone.imp().close_tab(tab_id);
            _ = window_clone.imp().get_sender().send_blocking(Message::RefreshTabs());
        });

        label_vbox.into()
    }

    /// generates a tab label based on the tab info
    fn create_tab_label(&self, tab: &GosubTab) -> gtk4::Widget {
        let tab_label = match tab.is_pinned() {
            true => self.create_pinned_tab_label(tab),
            false => self.create_normal_tab_label(tab),
        };

        let gesture = GestureClick::builder()
            .button(0) // 0 means all buttons
            .build();

        let window_clone = self.obj().clone();
        let tab_id = tab.id();
        let tab_is_pinned = tab.is_pinned();

        gesture.connect_pressed(move |gesture, _n_press, x, y| {
            if gesture.current_button() == gdk::BUTTON_SECONDARY {
                // Refresh the tab info based on the current state
                let tab_manager = window_clone.imp().tab_manager.lock().unwrap();
                let tab_count = tab_manager.tab_count();
                let tab_info = TabInfo {
                    id: tab_id,
                    is_pinned: tab_is_pinned,
                    is_left: tab_manager.is_most_left_unpinned_tab(tab_id),
                    is_right: tab_manager.is_most_right_tab(tab_id),
                    tab_count,
                };
                drop(tab_manager);

                let menu_model = build_context_menu(tab_info.clone());
                let popover = PopoverMenu::builder()
                    .menu_model(&menu_model)
                    .halign(gtk4::Align::Start)
                    .has_arrow(false)
                    .flags(PopoverMenuFlags::NESTED)
                    .build();

                let action_group = SimpleActionGroup::new();
                setup_context_menu_actions(&action_group, &window_clone, tab_info.clone());
                popover.insert_action_group("tab", Some(&action_group));

                if let Some(widget) = gesture.widget() {
                    // We need to use the window as a parent, not the parent widget. Since X/Y coordinates
                    // are relative from the widget, we need to convert them X/Y positions based on the window.
                    popover.set_parent(&window_clone);
                    if let Some(p) = widget.compute_point(&window_clone, &Point::new(x as f32, y as f32)) {
                        popover.set_pointing_to(Some(&gdk::Rectangle::new(p.x() as i32, p.y() as i32, 0, 0)));
                        popover.popup();
                    }
                }
            }
        });
        tab_label.add_controller(gesture);

        tab_label
    }

    fn generate_default_page(&self) -> gtk4::Box {
        let img = Image::from_resource("/io/gosub/browser-gtk/assets/submarine.svg");
        img.set_visible(true);
        img.set_focusable(false);
        img.set_valign(gtk4::Align::Center);
        img.set_margin_top(64);
        img.set_pixel_size(500);
        img.set_hexpand(true);

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        vbox.set_visible(true);
        vbox.set_can_focus(false);
        vbox.set_halign(gtk4::Align::Center);
        vbox.set_vexpand(true);
        vbox.set_hexpand(true);

        vbox.append(&img);

        vbox
    }

    fn load_favicon_async(&self, tab_id: TabId) {
        info!(target: "gtk", "Fetching favicon for tab: {}", tab_id);

        let manager = self.tab_manager.lock().unwrap();
        let tab = manager.get_tab(tab_id).unwrap();
        let url = tab.url().to_string();
        drop(manager);

        let sender_clone = self.get_sender().clone();
        runtime().spawn(async move {
            let favicon = if url.starts_with("about:") {
                // about: pages do not have a favicon (or maybe a default one?)
                Vec::new()
            } else {
                fetcher::fetch_favicon(url.as_str()).await
            };
            sender_clone.send(Message::FaviconLoaded(tab_id, favicon)).await.unwrap();
        });
    }

    fn load_url_async(&self, tab_id: TabId) {
        let manager = self.tab_manager.lock().unwrap();
        let tab = manager.get_tab(tab_id).unwrap();
        let url = tab.url().to_string();
        let view_mode = tab.viewmode();
        drop(manager);

        let sender_clone = self.get_sender().clone();
        runtime().spawn(async move {
            if view_mode == HtmlViewMode::About {
                let html_content = load_about_url(url);
                sender_clone.send(Message::UrlLoaded(tab_id, html_content)).await.unwrap();
                return;
            }

            match fetcher::fetch_url_body(&url).await {
                Ok(content) => {
                    let html_content = String::from_utf8_lossy(content.as_slice());
                    // we get a Cow.. and we clone it into the url?
                    sender_clone
                        .send(Message::UrlLoaded(tab_id, html_content.to_string()))
                        .await
                        .unwrap();
                }
                Err(e) => {
                    log::error!("Failed to fetch URL: {}", e);
                    sender_clone
                        .send(Message::Log(format!("Failed to fetch URL: {}", e)))
                        .await
                        .unwrap();
                }
            }
        });
    }

    /// Handles all message coming from the async (tokio) tasks
    pub async fn handle_message(&self, message: Message) {
        info!(target: "gtk", "Received a message: {:?}", message);

        match message {
            Message::RefreshTabs() => {
                self.refresh_tabs();
            }
            Message::OpenTab(url, title) => {
                self.open_tab(None, &url, &title);
            }
            Message::OpenTabRight(target_tab_id, url, title) => {
                for page_num in 0..self.tab_bar.pages().n_items() {
                    let page = self.tab_bar.nth_page(Some(page_num)).unwrap();
                    if page.get_tab_id().unwrap() == target_tab_id {
                        self.open_tab(Some(page_num as usize + 1), &url, &title);
                        return;
                    }
                }
            }

            Message::LoadUrl(tab_id, url_str) => {
                self.log(format!("Loading URL: {}", url_str).as_str());

                // Make sure the URL is correct (starts with scheme://, and find the view_mode (source: xml: raw: etc)
                let (view_mode, url) = self.sanitize_url(&url_str);
                dbg!(&view_mode);
                dbg!(&url);

                // Update information in the given tab with the new url
                let mut manager = self.tab_manager.lock().unwrap();
                let mut tab = manager.get_tab(tab_id).unwrap().clone();

                tab.set_favicon(None);
                tab.set_title(url.as_str());
                tab.set_url(url.as_str());
                tab.set_loading(true);
                tab.set_viewmode(view_mode);

                manager.update_tab(tab_id, &tab);
                drop(manager);

                self.refresh_tabs();

                // Now, load favicon and url content
                self.load_favicon_async(tab_id);
                self.load_url_async(tab_id);
            }
            Message::FaviconLoaded(tab_id, buf) => {
                if buf.is_empty() {
                    self.log(format!("no favicon found for tab {}", tab_id).as_str());
                    return;
                }

                let manager = self.tab_manager.lock().unwrap();
                let mut tab = manager.get_tab(tab_id).unwrap().clone();
                drop(manager);

                let bytes = glib::Bytes::from(buf.as_slice());
                match gdk::Texture::from_bytes(&bytes) {
                    Ok(texture) => {
                        tab.set_favicon(Some(texture));
                    }
                    Err(e) => {
                        log::error!("Failed to load favicon into buffer: {}", e);
                        self.log(format!("Failed to load favicon into buffer: {}", e).as_str());
                    }
                }

                let mut manager = self.tab_manager.lock().unwrap();
                tab.set_loading(false);
                manager.update_tab(tab_id, &tab);
                drop(manager);

                self.refresh_tabs();
            }
            Message::UrlLoaded(tab_id, html_content) => {
                let mut manager = self.tab_manager.lock().unwrap();
                let mut tab = manager.get_tab(tab_id).unwrap().clone();
                tab.set_content(&html_content);

                let url = if tab.viewmode() == HtmlViewMode::About {
                    // @todo: this needs to be changed
                    Url::from_str(format!("about:{}", tab.url()).as_str()).unwrap()
                } else {
                    match Url::from_str(tab.url()) {
                        Ok(url) => url,
                        Err(e) => {
                            log::error!("Failed to parse URL: {}", e);
                            self.log(format!("Failed to parse URL: {}", e).as_str());
                            return
                        }
                    }
                };
                let d = <GosubEngineConfig as HasTreeDrawer>::TreeDrawer::from_source(url, &html_content, TaffyLayouter, false).unwrap();
                tab.set_drawer(d);

                // Fetch title from HTML content... poorly..
                if let Some(title) = fetch_title_from_html(&html_content) {
                    tab.set_title(title.as_str());
                } else {
                    let url = tab.url().to_string();
                    tab.set_title(url.as_str());
                }

                tab.set_loading(false);
                manager.update_tab(tab_id, &tab);
                drop(manager);

                self.refresh_tabs();
            }
            Message::Log(msg) => {
                self.log(msg.as_str());
            }
            Message::PinTab(tab_id) => {
                let mut manager = self.tab_manager.lock().unwrap();
                manager.pin_tab(tab_id);
                drop(manager);

                // Update tab-bar
                self.refresh_tabs();
            }
            Message::UnpinTab(tab_id) => {
                let mut manager = self.tab_manager.lock().unwrap();
                manager.unpin_tab(tab_id);
                drop(manager);

                // Update tab-bar
                self.refresh_tabs();
            }
        }
    }

    /// Retrieves the page number for the given TabID
    fn get_page_num_for_tab(&self, tab_id: TabId) -> Option<u32> {
        for i in 0..self.tab_bar.pages().n_items() {
            let page = self.tab_bar.nth_page(Some(i)).unwrap();
            if page.get_tab_id().unwrap() == tab_id {
                return Some(i);
            }
        }

        None
    }

    /// Opens a new tab at the given position, with the given URL and title. If the position is None,
    /// the tab will be added at the end of the tab-bar.
    fn open_tab(&self, position: Option<usize>, url_str: &str, title: &str) {
        let (view_mode, url) = self.sanitize_url(&url_str);
        dbg!(&view_mode);
        dbg!(&url_str);

        let mut tab = GosubTab::new(&url, title);
        let tab_id = tab.id();

        // add tab to manager, and notify the tab has changed. This will update the
        // tab-bar during a refresh-tabs call.
        let mut manager = self.tab_manager.lock().unwrap();
        tab.set_viewmode(view_mode);
        tab.set_loading(true);
        manager.add_tab(tab, position);
        manager.notify_tab_changed(tab_id);
        drop(manager);
        self.refresh_tabs();

        // Async load the favicon and the url contents
        self.load_favicon_async(tab_id);
        self.load_url_async(tab_id);
    }
}

fn load_about_url(url: String) -> String {
    match url.as_str() {
        "blank" => r#"
            <html>
                <head>
                    <title>Blank page</title>
                </head>
                <body>
                    <h1>Blank page</h1>
                    <p>This is a blank page</p>
                </body>
            </html>
            "#
        .to_string(),
        _ => r#"
            <html>
                <head>
                    <title>Unknown about: page</title>
                </head>
                <body>
                    <h1>Unknown about: page</h1>
                    <p>This is an unknown about: page</p>
                </body>
            </html>
            "#
        .to_string(),
    }
}

/// Fetches the title from a HTML code snippet, or returns None if no title is found
fn fetch_title_from_html(html: &str) -> Option<String> {
    let start_tag = "<title>";
    let end_tag = "</title>";

    let start_index = html.find(start_tag)? + start_tag.len();
    let end_index = html[start_index..].find(end_tag)? + start_index;
    let title = &html[start_index..end_index];

    Some(title.to_string())
}
