use gtk4::{gio, glib};
use gtk4::prelude::{GtkApplicationExt, GtkWindowExt};
use crate::application::Application;

mod imp;

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

        // Self::connect_actions(app, &window);
        // Self::connect_accelerators(app, &window);

        window
    }

    // fn connect_accelerators(app: &Application, _window: &Self) {
    //     app.set_accels_for_action("app.open-new-tab", &["<Primary>T"]);
    //     app.set_accels_for_action("app.close-tab", &["<Primary>W"]);
    //     app.set_accels_for_action("app.toggle-log", &["<Primary>L"]);
    // }
    //
    // fn connect_actions(app: &Application, window: &Self) {
    //     let logwindow_action = SimpleAction::new("toggle-log", None);
    //     logwindow_action.connect_activate({
    //         let window_clone = window.clone();
    //         move |_, _| {
    //             window_clone
    //                 .imp()
    //                 .log_scroller
    //                 .set_visible(!window_clone.imp().log_scroller.get_visible());
    //         }
    //     });
    //     app.add_action(&logwindow_action);
    //
    //     // Create new tab
    //     let window_clone = window.clone();
    //     let new_tab_action = SimpleAction::new("open-new-tab", None);
    //     new_tab_action.connect_activate(move |_, _| {
    //         let sender = window_clone.imp().sender.clone();
    //         runtime().spawn(clone!(
    //             #[strong]
    //             sender,
    //             async move {
    //                 sender.send(Message::OpenTab("about:blank".into(), "New Tab".into())).await.unwrap();
    //             }
    //         ));
    //     });
    //     app.add_action(&new_tab_action);
    //
    //     let tab_bar = window.imp().tab_bar.clone();
    //     tab_bar.connect_page_added({
    //         let window_clone = window.clone();
    //         move |_notebook, _, page_num| {
    //             window_clone
    //                 .imp()
    //                 .log(format!("[result] added a tab on page {}", page_num).as_str());
    //         }
    //     });
    //
    //     tab_bar.connect_page_removed({
    //         let window_clone = window.clone();
    //         move |_notebook, _widget, page_num| {
    //             window_clone.imp().log(format!("[result] removed tab: {}", page_num).as_str());
    //         }
    //     });
    //
    //     tab_bar.connect_page_reordered({
    //         let window_clone = window.clone();
    //         move |_notebook, page, page_num| {
    //             window_clone
    //                 .imp()
    //                 .log(format!("[result] reordered tab: [{:?}] to {}", page.get_tab_id(), page_num).as_str());
    //         }
    //     });
    //
    //     tab_bar.connect_switch_page({
    //         let window_clone = window.clone();
    //         move |_notebook, page, page_num| {
    //             window_clone.imp().log(format!("[result] switched to tab: {}", page_num).as_str());
    //
    //             if let Some(tab_id) = page.get_tab_id() {
    //                 let manager = window_clone.imp().tab_manager.lock().unwrap();
    //                 let tab = manager.get_tab(tab_id).unwrap();
    //                 window_clone.imp().searchbar.set_text(tab.url());
    //                 drop(manager);
    //             }
    //         }
    //     });
    // }
}
