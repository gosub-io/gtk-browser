use gtk4::{gio, glib};
use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use crate::application::Application;

mod imp;
mod db;

glib::wrapper! {
    pub struct BookmarkWindow(ObjectSubclass<imp::BookmarkWindow>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl BookmarkWindow {
    pub fn new(app: &Application) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();

        window.set_transient_for(app.active_window().as_ref());
        window.set_modal(true);
        window.set_resizable(true);
        window.set_decorated(true);
        window.set_default_size(1024, 768);

        window.imp().load_bookmarks("");

        window
    }
}
