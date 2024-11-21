use adw::gio::SimpleAction;
use adw::gtk;
use gtk4::glib::{clone, spawn_future_local};
use gtk::glib;

mod imp;
mod message;

use crate::application::Application;
use gtk::gio;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use crate::runtime;
use crate::window::message::Message;

// This wrapper must be in a different module than the implementation, because both will define a
// `struct BrowserWindow` and they would clash. In this case, the browser window is a subclass of
// its implementation.
glib::wrapper! {
    pub struct BrowserWindow(ObjectSubclass<imp::BrowserWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl BrowserWindow {
    pub fn new(app: &Application) -> Self {
        let window: Self = glib::Object::builder().property("application", app).build();

        window.set_resizable(true);
        window.set_decorated(true);
        window.set_default_size(1024, 768);

        let builder = gtk::Builder::from_resource("/io/gosub/browser-gtk/ui/main_menu.ui");
        let menubar = builder
            .object::<gio::MenuModel>("app-menu")
            .expect("Could not find app-menu");

        app.set_menubar(Some(&menubar));
        window.set_show_menubar(true);

        Self::connect_actions(app, &window);
        Self::connect_accelerators(app, &window);

        // Spawn handler
        let window_clone = window.clone();
        spawn_future_local(async move {
            loop {
                match window_clone.imp().get_receiver().recv().await {
                    Ok(message) => {
                        window_clone.imp().handle_message(message).await;
                    }
                    Err(e) => {
                        log::error!("Error receiving message: {:?}", e);
                        return;
                    }
                }
            }
        });

        // Refresh tabs on startup
        let window_clone = window.clone();
        spawn_future_local(async move {
            // Refresh tabs on startup
            window_clone.imp().get_sender().send(Message::RefreshTabs()).await.unwrap();
        });

        window
    }

    fn connect_accelerators(app: &Application, _window: &Self) {
        app.set_accels_for_action("app.open-new-tab", &["<Primary>T"]);
        app.set_accels_for_action("app.close-tab", &["<Primary>W"]);
        app.set_accels_for_action("app.toggle-log", &["<Primary>L"]);
    }

    fn connect_actions(app: &Application, window: &Self) {
        let logwindow_action = SimpleAction::new("toggle-log", None);
        logwindow_action.connect_activate({
            let window_clone = window.clone();
            move |_, _| {
                window_clone
                    .imp()
                    .log_scroller
                    .set_visible(!window_clone.imp().log_scroller.get_visible());
            }
        });
        app.add_action(&logwindow_action);

        // Create new tab
        let window_clone = window.clone();
        let new_tab_action = SimpleAction::new("open-new-tab", None);
        new_tab_action.connect_activate(move | _, _ |{
            let sender = window_clone.imp().sender.clone();
            runtime().spawn(clone!(
                #[strong]
                sender,
                async move {
                    sender.send(Message::OpenTab("about:blank".into())).await.unwrap();
                }
            ));
        });
        app.add_action(&new_tab_action);

        let tab_bar = window.imp().tab_bar.clone();
        tab_bar.connect_page_added({
            let window_clone = window.clone();
            move |_notebook, _, page_num| {
                window_clone
                    .imp()
                    .log(format!("[result] added a tab on page {}", page_num).as_str());
            }
        });

        tab_bar.connect_page_removed({
            let window_clone = window.clone();
            move |_notebook, _, page_num| {
                window_clone
                    .imp()
                    .log(format!("[result] removed tab: {}", page_num).as_str());
            }
        });

        tab_bar.connect_page_reordered({
            let window_clone = window.clone();
            move |_notebook, _, page_num| {
                window_clone
                    .imp()
                    .log(format!("[result] reordered tab: {}", page_num).as_str());
            }
        });

        tab_bar.connect_switch_page({
            let window_clone = window.clone();
            move |_notebook, _, page_num| {
                window_clone
                    .imp()
                    .log(format!("[result] switched to tab: {}", page_num).as_str());

                let tab_manager = window_clone.imp().tab_manager.clone();
                let binding = tab_manager.clone();
                let mut manager = binding.lock().unwrap();
                if let Some(tab_id) = manager.page_to_tab(page_num) {
                    manager.set_active(tab_id);
                }
                drop(manager);
            }
        });
    }
}
