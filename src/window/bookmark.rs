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

        // let items = StringList::new(&["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"]);
        // let selection_model = SingleSelection::new(Some(items));
        //
        // let factory = SignalListItemFactory::new();
        // factory.connect_setup(move |_, list_item| {
        //     let label = gtk4::Label::new(None);
        //     label.set_margin_top(5);
        //     label.set_margin_bottom(5);
        //     label.set_margin_start(10);
        //     label.set_margin_end(10);
        //
        //     let list_item = list_item
        //         .downcast_ref::<ListItem>()
        //         .expect("list_item not a ListItem");
        //
        //     list_item.set_child(Some(&label));
        //
        //     list_item
        //         .property_expression("item")
        //         .chain_property::<StringObject>("string")
        //         .bind(&label, "label", Widget::NONE);
        // });
        //
        // // factory.connect_bind(move |_, list_item| {
        // //     // Update the label text for each list item
        // //     let label = list_item.child().and_then(|c| c.downcast::<gtk4::Label>().ok());
        // //     let item = list_item.item().and_then(|i| i.downcast::<String>().ok());
        // //     if let (Some(label), Some(item)) = (label, item) {
        // //         label.set_text(item.as_str());
        // //     }
        // // });
        //
        // window.imp().gtk_list_view.set_factory(Some(&factory));
        // window.imp().gtk_list_view.set_model(Some(&selection_model));

        // Self::connect_actions(app, &window);
        // Self::connect_accelerators(app, &window);


        window.imp().load_bookmarks();

        window
    }
}
