use std::cell::RefCell;
use std::collections::HashMap;
use chrono::{DateTime, NaiveDateTime, Utc};
use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, ListItem, NoSelection, SignalListItemFactory};
use gtk4::gdk::Paintable;
use gtk4::gio::{ListStore};
use gtk4::prelude::{BoxExt, Cast, ListItemExt, WidgetExt};

#[derive(CompositeTemplate)]
#[template(resource = "/io/gosub/browser-gtk/ui/bookmark-window.ui")]
pub struct BookmarkWindow {
    #[template_child]
    pub bookmarks_list: TemplateChild<gtk4::ColumnView>,
}

impl Default for BookmarkWindow {
    fn default() -> Self {
        Self {
            bookmarks_list: Default::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for BookmarkWindow {
    const NAME: &'static str = "BookmarkWindow";
    type Type = super::BookmarkWindow;
    type ParentType = gtk4::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for BookmarkWindow {
}

impl WidgetImpl for BookmarkWindow {}
impl WindowImpl for BookmarkWindow {}
impl ApplicationWindowImpl for BookmarkWindow {}


glib::wrapper! {
    pub struct RowObject(ObjectSubclass<RowObjectImpl>);
}

#[derive(Default)]
pub struct RowObjectImpl {
    pub favicon: RefCell<Option<Vec<u8>>>,
    pub name: RefCell<String>,
    pub tags: RefCell<String>,
    pub url: RefCell<String>,
    pub last_accessed: RefCell<Option<DateTime<Utc>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for RowObjectImpl {
    const NAME: &'static str = "RowObject";
    type Type = RowObject;
    type ParentType = glib::Object;
}

impl ObjectImpl for RowObjectImpl {}

impl RowObject {
    pub fn new(icon: Option<Vec<u8>>, name: &str, tags: &str, url: &str, last_accessed: Option<DateTime<Utc>>) -> Self {
        let obj: Self = glib::Object::new::<Self>();

        // Now set the fields
        obj.imp().favicon.replace(icon);
        obj.imp().name.replace(name.to_string());
        obj.imp().tags.replace(tags.to_string());
        obj.imp().url.replace(url.to_string());
        obj.imp().last_accessed.replace(last_accessed);

        obj
    }

    pub fn icon(&self) -> Option<Vec<u8>> {
        self.imp().favicon.borrow().clone()
    }

    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }

    pub fn tags(&self) -> String {
        self.imp().tags.borrow().clone()
    }

    pub fn url(&self) -> String {
        self.imp().url.borrow().clone()
    }

    pub fn last_accessed(&self) -> Option<DateTime<Utc>> {
        self.imp().last_accessed.borrow().clone()
    }
}

impl BookmarkWindow {
    pub fn load_mock_data(&self) {
        let list_store = ListStore::new::<RowObject>();

        let icon = Some(include_bytes!("../../../resources/favicon.png").to_vec());

        let connection = sqlite::open("bookmarks.db").unwrap();
        let query = "SELECT * FROM bookmarks";

        _ = connection.iterate(query, |pairs| {
            let row: HashMap<_, _> = pairs
                .iter()
                .filter_map(|&(column, value)| Some((column, value?))) // Ignore `None` values
                .collect();

            list_store.append(&RowObject::new(
                icon.clone(),
                row.get("title").unwrap_or(&""),
                row.get("tags").unwrap_or(&""),
                row.get("url").unwrap_or(&""),
                row.get("last_accessed").and_then(|s| {
                    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
                }),
            ));

            true
        });

        let selection_model = NoSelection::new(Some(list_store));
        self.bookmarks_list.set_model(Some(&selection_model));
    }
}

#[gtk4::template_callbacks]
impl BookmarkWindow {
    #[template_callback]
    fn setup_name_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);

        let image = gtk4::Image::new();
        hbox.append(&image);
        let label = gtk4::Label::new(None);
        hbox.append(&label);

        list_item.set_child(Some(&hbox));
    }

    #[template_callback]
    fn bind_name_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {

        if let Some(hbox) = list_item.child().and_then(|c| c.downcast::<gtk4::Box>().ok()) {
            let fc = hbox.first_child().expect("fc");
            if let Ok(image) = fc.downcast::<gtk4::Image>() {
                if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowObject>().ok()) {
                    if let Some(icon_bytes) = row.icon() {
                        if let Ok(texture) = gtk4::gdk::Texture::from_bytes(&glib::Bytes::from(icon_bytes.as_slice())) {
                            image.set_paintable(Some(&Paintable::from(texture)));
                        }
                    }
                }
            }

            let fc = hbox.first_child().expect("fc");
            let nc = fc.next_sibling().expect("nc");
            if let Ok(label) = nc.downcast::<gtk4::Label>() {
                if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowObject>().ok()) {
                    label.set_text(&row.name());
                }
            }
        }
    }

    #[template_callback]
    fn setup_tags_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        let label = gtk4::Label::new(None);
        label.set_halign(gtk4::Align::Start);
        list_item.set_child(Some(&label));
    }

    #[template_callback]
    fn bind_tags_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<gtk4::Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowObject>().ok()) {
                label.set_text(&row.tags());
            }
        }
    }

    #[template_callback]
    fn setup_url_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        let label = gtk4::Label::new(None);
        label.set_halign(gtk4::Align::Start);
        list_item.set_child(Some(&label));
    }

    #[template_callback]
    fn bind_url_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<gtk4::Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowObject>().ok()) {
                label.set_text(&row.url());
            }
        }
    }

    #[template_callback]
    fn setup_last_accessed_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        let label = gtk4::Label::new(None);
        label.set_halign(gtk4::Align::Start);
        list_item.set_child(Some(&label));
    }

    #[template_callback]
    fn bind_last_accessed_cb(_factory: &SignalListItemFactory, list_item: &ListItem) {
        if let Some(label) = list_item.child().and_then(|c| c.downcast::<gtk4::Label>().ok()) {
            if let Some(row) = list_item.item().and_then(|i| i.downcast::<RowObject>().ok()) {
                match row.last_accessed() {
                    Some(date) => {
                        let formatter = timeago::Formatter::default();
                        let human_readable = formatter.convert_chrono(date, Utc::now());
                        label.set_text(&human_readable);
                    }
                    None => {
                        label.set_text("Never");
                    }
                }
            }
        }
    }

}

