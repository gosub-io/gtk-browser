use crate::tab::TabId;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::gio::{Menu, SimpleAction, SimpleActionGroup};
use gtk4::glib::clone;
use crate::runtime;
use crate::window::message::Message;

/// Simple structure to keep track of tab information. This info is needed in order to enable/disable certain context menu
/// actions.
#[derive(Debug, Clone)]
pub(crate) struct TabInfo {
    /// ID of the tab
    pub(crate) id: TabId,
    /// Tab is pinned or not
    pub(crate) is_pinned: bool,
    /// Tab is at the most left side of the non-pinned tabs
    pub(crate) is_left: bool,
    /// Tab is at the most right side of the non-pinned tabs
    pub(crate) is_right: bool,
    /// Number of total tabs
    pub(crate) tab_count: usize,
}

pub(crate) fn setup_context_menu_actions(
    action_group: &SimpleActionGroup,
    window: &super::BrowserWindow,
    info: TabInfo,
) {
    // New Tab to Right
    let new_tab_right = SimpleAction::new("new_tab_right", None);
    new_tab_right.connect_activate(move |_, _| {
        // Implement the action
        // Example: window.imp().open_new_tab_to_right(tab_id.clone());
    });
    action_group.add_action(&new_tab_right);

    // Reload Tab
    let reload_tab = SimpleAction::new("reload_tab", None);
    reload_tab.connect_activate(move |_, _| {
        // window.imp().reload_tab(tab_id.clone());
    });
    action_group.add_action(&reload_tab);

    // Mute Tab
    let mute_tab = SimpleAction::new("mute_tab", None);
    mute_tab.connect_activate(move |_, _| {
        // window.imp().mute_tab(tab_id.clone());
    });
    action_group.add_action(&mute_tab);

    // Pin Tab
    let pin_tab = SimpleAction::new("pin_tab", None);
    if info.is_pinned {
        pin_tab.set_enabled(false);
    }

    let window_clone = window.clone();
    pin_tab.connect_activate(move |_, _| {
        let sender = window_clone.imp().sender.clone();
        runtime().spawn(clone!(
            #[strong]
            sender,
            async move {
                sender.send(Message::PinTab(info.id.clone())).await.unwrap();
            }
        ));
    });
    action_group.add_action(&pin_tab);

    // Unpin Tab
    let unpin_tab = SimpleAction::new("unpin_tab", None);
    if !info.is_pinned {
        unpin_tab.set_enabled(false);
    }
    let window_clone = window.clone();
    unpin_tab.connect_activate(move |_, _| {
        let sender = window_clone.imp().sender.clone();
        runtime().spawn(clone!(
            #[strong]
            sender,
            async move {
                sender.send(Message::UnpinTab(info.id.clone())).await.unwrap();
            }
        ));
    });
    action_group.add_action(&unpin_tab);

    // Duplicate Tab
    let duplicate_tab = SimpleAction::new("duplicate_tab", None);
    duplicate_tab.connect_activate(move |_, _| {
        // window.imp().duplicate_tab(tab_id.clone());
    });
    action_group.add_action(&duplicate_tab);

    // Close Tab
    let close_tab = SimpleAction::new("close_tab", None);
    let window_clone = window.clone();
    close_tab.connect_activate(move |_, _| {
        window_clone.imp().close_tab(info.id.clone());
    });
    action_group.add_action(&close_tab);

    // Reopen Closed Tab
    let reopen_closed_tab = SimpleAction::new("reopen_closed_tab", None);
    reopen_closed_tab.connect_activate(move |_, _| {
        // window.imp().reopen_closed_tab();
    });
    action_group.add_action(&reopen_closed_tab);

    // Close Tabs to Left
    let close_tabs_left = SimpleAction::new("close_tabs_left", None);
    if info.is_pinned || info.is_left {
        close_tabs_left.set_enabled(false);
    }
    close_tabs_left.connect_activate(move |_, _| {
        // window.imp().close_tabs_to_left(tab_id.clone());
    });
    action_group.add_action(&close_tabs_left);

    // Close Tabs to Right
    let close_tabs_right = SimpleAction::new("close_tabs_right", None);
    if info.is_pinned || info.is_right {
        close_tabs_right.set_enabled(false);
    }
    close_tabs_right.connect_activate(move |_, _| {
        // window.imp().close_tabs_to_right(tab_id.clone());
    });
    action_group.add_action(&close_tabs_right);

    // Close Other Tabs
    let close_other_tabs = SimpleAction::new("close_other_tabs", None);
    if info.tab_count == 1 {
        close_other_tabs.set_enabled(false);
    }
    close_other_tabs.connect_activate(move |_, _| {
        // window.imp().close_other_tabs(tab_id.clone());
    });
    action_group.add_action(&close_other_tabs);
}

pub(crate) fn build_context_menu(tab_info: TabInfo) -> Menu {
    let menu = Menu::new();

    let section = Menu::new();
    section.append(Some("New Tab to Right"), Some("tab.new_tab_right"));
    menu.append_section(None, &section);

    let section = Menu::new();
    section.append(Some("Reload Tab"), Some("tab.reload_tab"));
    section.append(Some("Mute Tab"), Some("tab.mute_tab"));
    if tab_info.is_pinned {
        section.append(Some("Unpin Tab"), Some("tab.unpin_tab"));
    } else {
        section.append(Some("Pin Tab"), Some("tab.pin_tab"));
    }
    section.append(Some("Duplicate Tab"), Some("tab.duplicate_tab"));
    menu.append_section(None, &section);

    let section = Menu::new();
    section.append(Some("Close Tab"), Some("tab.close_tab"));

    let submenu = Menu::new();
    submenu.append(Some("Close Tabs to Left"), Some("tab.close_tabs_left"));
    submenu.append(Some("Close Tabs to Right"), Some("tab.close_tabs_right"));
    submenu.append(Some("Close Other Tabs"), Some("tab.close_other_tabs"));
    section.append_submenu(Some("Close Other Tabs"), &submenu);

    // @todo: we should only be allowed to reopen closed tab, after we have closed one..
    // this functionality is not yet implemented
    section.append(Some("Reopen Closed Tab"), Some("tab.reopen_closed_tab"));
    menu.append_section(None, &section);

    menu
}