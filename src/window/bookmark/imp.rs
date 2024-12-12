use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, TreeExpander};

#[derive(CompositeTemplate)]
#[template(resource = "/io/gosub/browser-gtk/ui/bookmark-window.ui")]
pub struct BookmarkWindow {
    #[template_child]
    pub tree_expander: TemplateChild<TreeExpander>,
    // #[template_child]
    // pub tab_bar: TemplateChild<Notebook>,
    // #[template_child]
    // pub log_scroller: TemplateChild<ScrolledWindow>,
    // #[template_child]
    // pub log: TemplateChild<TextView>,
}

impl Default for BookmarkWindow {
    fn default() -> Self {
        Self {
            tree_expander: Default::default(),
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

#[gtk4::template_callbacks]
impl BookmarkWindow {
    // #[template_callback]
    // fn handle_new_tab(&self, _btn: &Button) {
    //     todo!("not yet implemented");
    // }
    //
    // #[template_callback]
    // fn handle_close_tab(&self, _btn: &Button) {
    //     todo!("not yet implemented");
    // }
    //
    // #[template_callback]
    // fn handle_prev_clicked(&self, _btn: &Button) {
    //     todo!("not yet implemented");
    // }
    //
    // #[template_callback]
    // fn handle_toggle_darkmode(&self, btn: &ToggleButton) {
    //     self.log("Toggling dark mode");
    //
    //     info!("Toggle dark mode action triggered");
    //     let settings = Settings::default().expect("Failed to get default GtkSettings");
    //     settings.set_property("gtk-application-prefer-dark-theme", btn.is_active());
    // }
    //
    // #[template_callback]
    // fn handle_refresh_clicked(&self, _btn: &Button) {
    //     self.log("Refreshing the current page");
    // }
}
