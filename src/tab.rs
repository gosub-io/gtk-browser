use std::collections::{HashMap, VecDeque};
use uuid::Uuid;
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use gtk4::gdk::Texture;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct TabId(Uuid);

impl TabId {
    pub fn new() -> Self {
        TabId(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        TabId(uuid)
    }
}

impl FromStr for TabId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(TabId)
    }
}

// Optional: Implement `Display` for easier printing
impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct GosubTab {
    /// Tab is currently loading
    loading: bool,
    /// Id of the tab
    id: TabId,
    /// Tab is pinned and cannot be moved from the leftmost position
    pinned: bool,
    /// Tab content is private and not saved in history
    private: bool,
    /// URL that is loaded into the tab
    url: String,
    /// History of the tab
    history: Vec<String>,
    /// Title of the tab
    title: String,
    /// Loaded favicon of the tab
    favicon: Option<Texture>,
    /// Actual content (HTML) of the tab
    content: String,
}

impl Debug for GosubTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GosubTab")
            .field("id", &self.id)

            .field("title", &self.title)
            .finish()
    }
}

impl GosubTab {
    pub fn new(url: &str, title: &str) -> Self {
        GosubTab {
            loading: false,
            id: TabId::new(),
            pinned: false,
            private: false,
            url: url.to_string(),
            history: Vec::new(),
            title: title.to_string(),
            favicon: None,
            content: String::new(),
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn id(&self) -> TabId {
        self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_pinned(&mut self, pinned: bool) {
        self.pinned = pinned;
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    pub fn set_private(&mut self, private: bool) {
        self.private = private;
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn add_to_history(&mut self, url: &str) {
        self.history.push(url.to_string());
    }

    pub fn pop_history(&mut self) -> Option<String> {
        self.history.pop()
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub(crate) fn favicon(&self) -> Option<Texture> {
        self.favicon.clone()
    }

    pub fn set_favicon(&mut self, favicon: Option<Texture>) {
        self.favicon = favicon;
    }
}

#[derive(Debug)]
pub enum TabCommand {
    Close(TabId),       // Close index
    #[allow(dead_code)]
    CloseAll,           // Close all
    Move(TabId, u32),   // tab has been moved to given position
    Update(TabId),      // Update tab (tab + content)
    Insert(TabId, u32), // Insert new tab at given position
    Activate(TabId),    // Set as active
}

pub struct GosubTabManager {
    // All known tabs in the system
    tabs: HashMap<TabId, GosubTab>,
    // Actual ordering of the pinned tabs in the notebook.
    pinned_tab_order: VecDeque<TabId>,
    // Actual ordering of the ubpinned tabs in the notebook.
    unpinned_tab_order: VecDeque<TabId>,
    // list of commands to execute on the next tab notebook update
    commands: Vec<TabCommand>,
}

impl Default for GosubTabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GosubTabManager {
    pub fn new() -> Self {
        let manager = GosubTabManager {
            tabs: HashMap::new(),
            unpinned_tab_order: VecDeque::new(),
            pinned_tab_order: VecDeque::new(),
            commands: Vec::new(),
        };

        // // Always add an initial tab
        // let mut tab = GosubTab::new("about:blank", "New tab");
        // tab.set_loading(false);
        // let tab_id = manager.add_tab(tab, None);
        // manager.mark_tab_updated(tab_id);   // This will take care of removing the "loading" spinner.

        manager
    }

    #[allow(dead_code)]
    pub(crate) fn get_by_tab(&self, tab_id: TabId) -> Option<&GosubTab> {
        self.tabs.get(&tab_id)
    }

    // pub(crate) fn get_page_num_by_tab(&self, tab_id: TabId) -> Option<u32> {
    //     self.tab_order.iter().position(|id| id == &tab_id).map(|pos| pos as u32)
    // }

    pub(crate) fn commands(&mut self) -> Vec<TabCommand> {
        self.commands.drain(..).collect()
    }

    pub(crate) fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true when the given tab is the leftmost unpinned tab
    pub(crate) fn is_most_left_unpinned_tab(&self, tab_id: TabId) -> bool {
        self.unpinned_tab_order.front() == Some(&tab_id)
    }

    /// Returns true when the given tab is the rightmost tab
    pub(crate) fn is_most_right_tab(&self, tab_id: TabId) -> bool {
        self.unpinned_tab_order.back() == Some(&tab_id)
    }

    pub fn set_active(&mut self, tab_id: TabId) {
        self.commands.push(TabCommand::Activate(tab_id));
    }

    pub(crate) fn notify_tab_changed(&mut self, tab_id: TabId) {
        self.commands.push(TabCommand::Update(tab_id));
    }

    pub(crate) fn update_tab(&mut self, tab_id: TabId, tab: &GosubTab) {
        self.tabs.insert(tab_id, tab.clone());
        self.notify_tab_changed(tab_id);
    }

    pub fn pin_tab(&mut self, tab_id: TabId) {
        let tab = self.tabs.get_mut(&tab_id).unwrap();
        tab.set_pinned(true);

        self.unpinned_tab_order.retain(|id| id != &tab_id);
        self.pinned_tab_order.push_back(tab_id);

        // Tab has been moved to end of pinned tabs
        self.commands.push(TabCommand::Update(tab_id));
        self.commands.push(TabCommand::Move(tab_id, (self.pinned_tab_order.len() - 1) as u32));
    }

    pub fn unpin_tab(&mut self, tab_id: TabId) {
        let tab = self.tabs.get_mut(&tab_id).unwrap();
        tab.set_pinned(false);

        self.pinned_tab_order.retain(|id| id != &tab_id);
        self.unpinned_tab_order.push_front(tab_id);

        // Tab has been moved to begin of unpinned tabs
        self.commands.push(TabCommand::Update(tab_id));
        self.commands.push(TabCommand::Move(tab_id, self.pinned_tab_order.len() as u32));
    }

    pub fn add_tab(&mut self, tab: GosubTab, position: Option<usize>) -> TabId {
        let mut real_position = position.unwrap_or(usize::MAX);

        if tab.is_pinned() {
            if real_position > self.pinned_tab_order.len() {
                self.pinned_tab_order.push_back(tab.id());
                real_position = self.pinned_tab_order.len() - 1;
            } else {
                self.pinned_tab_order.insert(real_position, tab.id());
            }
        } else {
            if real_position > self.unpinned_tab_order.len() {
                self.unpinned_tab_order.push_back(tab.id());
                real_position = self.unpinned_tab_order.len() - 1;
            } else {
                self.unpinned_tab_order.insert(real_position, tab.id());
            }
        }

        self.commands.push(TabCommand::Insert(tab.id(), real_position as u32));

        let tab_id = tab.id.clone();
        self.tabs.insert(tab_id, tab);
        // self.set_active(tab_id);

        tab_id
    }

    pub fn remove_tab(&mut self, tab_id: TabId) {
        if let Some(index) = self.unpinned_tab_order.iter().position(|id| id == &tab_id) {
            self.unpinned_tab_order.remove(index);
            self.commands.push(TabCommand::Close(tab_id));

            // Set active tab to the last tab. Assumes there is always one tab
            if index == 0 {
                if let Some(new_active_tab) = self.unpinned_tab_order.get(0) {
                    self.set_active(*new_active_tab);
                }
            } else {
                if let Some(new_active_tab) = self.unpinned_tab_order.get(index - 1) {
                    self.set_active(*new_active_tab);
                }
            }
        }

        self.tabs.remove(&tab_id);
    }

    pub fn get_tab(&self, tab_id: TabId) -> Option<GosubTab> {
        if let Some(tab) = self.tabs.get(&tab_id) {
            return Some(tab.clone())
        }
        None
    }

    pub fn order(&self) -> Vec<TabId> {
        let mut order = Vec::with_capacity(self.pinned_tab_order.len() + self.unpinned_tab_order.len());
        order.extend_from_slice(&self.pinned_tab_order.iter().cloned().collect::<Vec<TabId>>());
        order.extend_from_slice(&self.unpinned_tab_order.iter().cloned().collect::<Vec<TabId>>());

        order
    }

    pub fn reorder(&mut self, tab_id: TabId, position: usize) {
        let tab = self.tabs.get(&tab_id).unwrap();

        if tab.is_pinned() {
            if let Some(index) = self.unpinned_tab_order.iter().position(|id| id == &tab_id) {
                if index > position {
                    self.unpinned_tab_order.remove(index);
                    self.unpinned_tab_order.insert(position, tab_id);
                } else if index < position {
                    self.unpinned_tab_order.insert(position, tab_id);
                    self.unpinned_tab_order.remove(index);
                } else {
                    // Nothing to do, as index and post are the same
                }
                self.commands.push(TabCommand::Move(tab_id, position as u32));
            }
        } else {
            if let Some(index) = self.pinned_tab_order.iter().position(|id| id == &tab_id) {
                if index > position {
                    self.pinned_tab_order.remove(index);
                    self.pinned_tab_order.insert(position, tab_id);
                } else if index < position {
                    self.pinned_tab_order.insert(position, tab_id);
                    self.pinned_tab_order.remove(index);
                } else {
                    // Nothing to do, as index and post are the same
                }
                self.commands.push(TabCommand::Move(tab_id, position as u32));
            }

        }
    }
}


#[cfg(test)]
mod test {
    use super::{TabId, GosubTab, GosubTabManager};

    #[test]
    fn test_tab_id() {
        use std::str::FromStr;

        let id = TabId::new();
        let id_str = id.to_string();
        let id_parsed = TabId::from_str(&id_str).unwrap();

        assert_eq!(id, id_parsed);
    }

    #[test]
    fn test_tab_manager() {
        let mut manager = GosubTabManager::new();
        let tab = GosubTab::new("about:blank", "New tab");
        let tab_id = manager.add_tab(tab, None);

        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.get_tab(tab_id).unwrap().url(), "about:blank");
        assert_eq!(manager.get_tab(tab_id).unwrap().title(), "New tab");

        manager.remove_tab(tab_id);
        assert_eq!(manager.tab_count(), 0);
    }

    #[test]
    fn test_tab_manager_remove() {
        let mut manager = GosubTabManager::new();
        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");
        let tab3 = GosubTab::new("about:blank", "New tab 3");

        let tab1_id = manager.add_tab(tab1, None);
        let tab2_id = manager.add_tab(tab2, None);
        let tab3_id = manager.add_tab(tab3, None);

        assert_eq!(manager.tab_count(), 3);

        manager.remove_tab(tab2_id);
        assert_eq!(manager.tab_count(), 2);
        assert_eq!(manager.order(), vec![tab1_id, tab3_id]);
    }

    #[test]
    fn test_pinned_tabs() {
        let mut manager = GosubTabManager::new();
        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");
        let mut tab3 = GosubTab::new("about:blank", "New tab 3");
        tab3.set_pinned(true);
        let tab4 = GosubTab::new("about:blank", "New tab 4");
        let mut tab5 = GosubTab::new("about:blank", "New tab 5");
        tab5.set_pinned(true);
        let tab6 = GosubTab::new("about:blank", "New tab 6");

        let tab1_id = manager.add_tab(tab1, None);
        let tab2_id = manager.add_tab(tab2, None);
        let tab3_id = manager.add_tab(tab3, None);
        let tab4_id = manager.add_tab(tab4, None);
        let tab5_id = manager.add_tab(tab5, None);
        let tab6_id = manager.add_tab(tab6, None);

        // Since some tabs are pinned, this is the ordering:
        // [ 3 5 1 2 4 6 ]
        assert_eq!(manager.pinned_tab_order, vec![tab3_id, tab5_id]);
        assert_eq!(manager.unpinned_tab_order, vec![tab1_id, tab2_id, tab4_id, tab6_id]);

        assert_eq!(manager.is_most_left_unpinned_tab(tab1_id), true);
        assert_eq!(manager.is_most_left_unpinned_tab(tab2_id), false);
        assert_eq!(manager.is_most_right_tab(tab6_id), true);
        assert_eq!(manager.is_most_right_tab(tab5_id), false);
    }
}