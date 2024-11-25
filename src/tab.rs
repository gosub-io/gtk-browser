use std::collections::HashMap;
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
    /// Tab is sticky and cannot be moved from the leftmost position
    sticky: bool,
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
            .field("sticky", &self.sticky)
            .field("title", &self.title)
            .finish()
    }
}

impl GosubTab {
    pub fn new(url: &str, title: &str) -> Self {
        GosubTab {
            loading: false,
            id: TabId::new(),
            sticky: false,
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

    pub fn set_sticky(&mut self, sticky: bool) {
        self.sticky = sticky;
    }

    pub fn is_sticky(&self) -> bool {
        self.sticky
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
    Close(u32),     // Close index
    #[allow(dead_code)]
    CloseAll,       // Close all
    Move(u32, u32), // Move from index to index
    Update(u32),    // Update index (tab + content)
    Insert(u32),    // Insert index
    Activate(u32),  // Set as active
}

pub struct GosubTabManager {
    // All known tabs in the system
    tabs: HashMap<TabId, GosubTab>,
    // Actual ordering of the tabs in the notebook. Used for converting page_num to tab_id
    tab_order: Vec<TabId>,
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
            tab_order: Vec::new(),
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

    pub(crate) fn get_page_num_by_tab(&self, tab_id: TabId) -> Option<u32> {
        self.tab_order.iter().position(|id| id == &tab_id).map(|pos| pos as u32)
    }

    pub(crate) fn commands(&mut self) -> Vec<TabCommand> {
        self.commands.drain(..).collect()
    }

    pub(crate) fn tab_to_page(&self, tab_id: TabId) -> Option<u32> {
        self.tab_order.iter().position(|id| id == &tab_id).map(|pos| pos as u32)
    }

    pub(crate) fn page_to_tab(&self, page_index: u32) -> Option<TabId> {
        self.tab_order.get(page_index as usize).cloned()
    }

    pub(crate) fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true when the given tab is the leftmost non-sticky tab
    pub(crate) fn is_most_left_nonsticky_tab(&self, tab_id: TabId) -> bool {
        // Find first non-sticky tab
        let mut found = None;
        for id in &self.tab_order {
            if let Some(tab) = self.tabs.get(id) {
                if !tab.is_sticky() {
                    found = Some(*id);
                    break;
                }
            }
        }

        // We match if we found a tab, and the tab-id matches
        found != None && found == Some(tab_id)
    }

    /// Returns true when the given tab is the rightmost tab
    pub(crate) fn is_most_right_tab(&self, tab_id: TabId) -> bool {
        // Find LAST non-sticky tab
        let mut found = None;
        for id in self.tab_order.iter().rev() {
            if let Some(tab) = self.tabs.get(id) {
                if !tab.is_sticky() {
                    found = Some(*id);
                    break;
                }
            }
        }

        // We match if we found a tab, and the tab-id matches
        found != None && found == Some(tab_id)
    }

    pub(crate) fn get_first_nonpinned_tab_index(&self) -> usize {
        for (index, id) in self.tab_order.iter().enumerate() {
            if let Some(tab) = self.tabs.get(id) {
                if !tab.is_sticky() {
                    return index;
                }
            }
        }

        // All tabs are sticky, just return the last index + 1
        self.tab_order.len()
    }

    pub fn set_active(&mut self, tab_id: TabId) {
        if let Some(page_num) = self.tab_order.iter().position(|&id| id == tab_id) {
            self.commands.push(TabCommand::Activate(page_num as u32));
        }
    }

    pub fn mark_tab_updated(&mut self, tab_id: TabId) {
        if let Some(page_num) = self.tab_to_page(tab_id) {
            self.commands.push(TabCommand::Update(page_num));
        }
    }

    pub(crate) fn notify_tab_changed(&mut self, tab_id: TabId) {
        if let Some(page_num) = self.tab_order.iter().position(|&id| id == tab_id) {
            self.commands.push(TabCommand::Update(page_num as u32));
        }
    }

    pub(crate) fn update_tab(&mut self, tab_id: TabId, tab: &GosubTab) {
        self.tabs.insert(tab_id, tab.clone());
        self.notify_tab_changed(tab_id);
    }

    pub fn calculate_position(&self, is_sticky: bool, position: Option<usize>) -> usize {
        // If no position is given, we add it as the last tab
        let position = position.map(|pos| {
            if pos < self.tab_order.len() {
                pos
            } else {
                // if the position is too large, we cannot increase it later on
                usize::MAX / 2
            }
        });

        // calculate 2 ranges: sticky range (ie: 0...4) and non-sticky range (ie: 5...n)
        let npti = self.get_first_nonpinned_tab_index();
        let sticky_range = 0..npti;
        let nonsticky_range = npti..self.tab_order.len();

        // If the tab is sticky, we insert it at the end of the sticky range
        if is_sticky {
            if let Some(pos) = position {
                if sticky_range.contains(&pos) {
                    return pos;
                }
            }
            sticky_range.end
        } else {
            if let Some(pos) = position {
                let pos = pos + npti;
                if nonsticky_range.contains(&pos) {
                    return pos;
                }
            }
            nonsticky_range.end
        }
    }

    pub fn add_tab(&mut self, tab: GosubTab, position: Option<usize>) -> TabId {
        let wanted_pos = self.calculate_position(tab.is_sticky(), position);
        // wanted position could be larger than the current tab_order length

        let real_position = if wanted_pos < self.tab_order.len() {
            self.tab_order.insert(wanted_pos, tab.id);
            wanted_pos
        } else {
            self.tab_order.push(tab.id);
            self.tab_order.len() - 1
        };

        self.commands.push(TabCommand::Insert(real_position as u32));

        let tab_id = tab.id.clone();
        self.tabs.insert(tab_id, tab);
        self.set_active(tab_id);

        tab_id
    }

    pub fn remove_tab(&mut self, tab_id: TabId) {
        if let Some(index) = self.tab_order.iter().position(|id| id == &tab_id) {
            self.tab_order.remove(index);
            self.commands.push(TabCommand::Close(index as u32));

            // Set active tab to the last tab. Assumes there is always one tab
            if index == 0 {
                if let Some(new_active_tab) = self.tab_order.get(0) {
                    self.set_active(*new_active_tab);
                }
            } else {
                if let Some(new_active_tab) = self.tab_order.get(index - 1) {
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
        self.tab_order.clone()
    }

    pub fn reorder(&mut self, tab_id: TabId, position: usize) {
        if let Some(index) = self.tab_order.iter().position(|id| id == &tab_id) {
            self.tab_order.remove(index);
            self.tab_order.insert(position, tab_id);
            self.commands.push(TabCommand::Move(index as u32, position as u32));
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
    fn test_tab_manager_reorder() {
        let mut manager = GosubTabManager::new();
        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");
        let tab3 = GosubTab::new("about:blank", "New tab 3");

        let tab1_id = manager.add_tab(tab1, None);
        let tab2_id = manager.add_tab(tab2, None);
        let tab3_id = manager.add_tab(tab3, None);

        assert_eq!(manager.order(), vec![tab1_id, tab2_id, tab3_id]);

        manager.reorder(tab1_id, 2);
        assert_eq!(manager.order(), vec![tab2_id, tab3_id, tab1_id]);

        manager.reorder(tab1_id, 0);
        assert_eq!(manager.order(), vec![tab1_id, tab2_id, tab3_id]);
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
    fn test_get_first_nonpinned_tab_index() {
        let mut manager = GosubTabManager::new();

        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");
        let mut tab3 = GosubTab::new("about:blank", "New tab 3");
        tab3.set_sticky(true);
        let tab4 = GosubTab::new("about:blank", "New tab 4");
        let mut tab5 = GosubTab::new("about:blank", "New tab 5");
        tab5.set_sticky(true);
        let tab6 = GosubTab::new("about:blank", "New tab 6");

        manager.tab_order = vec![tab3.id, tab5.id, tab1.id, tab2.id, tab4.id, tab6.id];
        manager.tabs.insert(tab1.id, tab1);
        manager.tabs.insert(tab2.id, tab2);
        manager.tabs.insert(tab3.id, tab3);
        manager.tabs.insert(tab4.id, tab4);
        manager.tabs.insert(tab5.id, tab5);
        manager.tabs.insert(tab6.id, tab6);

        assert_eq!(manager.get_first_nonpinned_tab_index(), 2);
    }

    #[test]
    fn test_get_first_nonpinned_tab_index_only_non_sticky() {
        let mut manager = GosubTabManager::new();

        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");

        manager.tab_order = vec![tab1.id, tab2.id];
        manager.tabs.insert(tab1.id, tab1);
        manager.tabs.insert(tab2.id, tab2);

        assert_eq!(manager.get_first_nonpinned_tab_index(), 0);
    }

    #[test]
    fn test_get_first_nonpinned_tab_index_only_sticky() {
        let mut manager = GosubTabManager::new();

        let mut tab1 = GosubTab::new("about:blank", "New tab 1");
        tab1.set_sticky(true);
        let mut tab2 = GosubTab::new("about:blank", "New tab 2");
        tab2.set_sticky(true);

        manager.tab_order = vec![tab1.id, tab2.id];
        manager.tabs.insert(tab1.id, tab1);
        manager.tabs.insert(tab2.id, tab2);

        assert_eq!(manager.get_first_nonpinned_tab_index(), 2);
    }

    #[test]
    fn test_calculate_position() {
        let mut manager = GosubTabManager::new();

        let mut tab1 = GosubTab::new("about:blank", "New tab 1");
        tab1.set_sticky(true);
        let mut tab2 = GosubTab::new("about:blank", "New tab 2");
        tab2.set_sticky(true);
        let tab3 = GosubTab::new("about:blank", "New tab 3");
        let tab4 = GosubTab::new("about:blank", "New tab 4");
        let tab5 = GosubTab::new("about:blank", "New tab 5");
        let tab6 = GosubTab::new("about:blank", "New tab 6");

        manager.tab_order = vec![tab1.id, tab2.id, tab3.id, tab4.id, tab5.id, tab6.id];
        manager.tabs.insert(tab1.id, tab1);
        manager.tabs.insert(tab2.id, tab2);
        manager.tabs.insert(tab3.id, tab3);
        manager.tabs.insert(tab4.id, tab4);
        manager.tabs.insert(tab5.id, tab5);
        manager.tabs.insert(tab6.id, tab6);

        assert_eq!(manager.calculate_position(false, None), 6);
        assert_eq!(manager.calculate_position(false, Some(0)), 2);
        assert_eq!(manager.calculate_position(false, Some(1)), 3);
        assert_eq!(manager.calculate_position(false, Some(2)), 4);
        assert_eq!(manager.calculate_position(false, Some(3)), 5);
        assert_eq!(manager.calculate_position(false, Some(4)), 6);
        assert_eq!(manager.calculate_position(false, Some(5)), 6);
        assert_eq!(manager.calculate_position(false, Some(6)), 6);
        assert_eq!(manager.calculate_position(false, Some(7)), 6);
        assert_eq!(manager.calculate_position(false, Some(21)), 6);

        assert_eq!(manager.calculate_position(true, None), 2);
        assert_eq!(manager.calculate_position(true, Some(0)), 0);
        assert_eq!(manager.calculate_position(true, Some(1)), 1);
        assert_eq!(manager.calculate_position(true, Some(2)), 2);
        assert_eq!(manager.calculate_position(true, Some(3)), 2);
        assert_eq!(manager.calculate_position(true, Some(4)), 2);
        assert_eq!(manager.calculate_position(true, Some(5)), 2);
        assert_eq!(manager.calculate_position(true, Some(6)), 2);
        assert_eq!(manager.calculate_position(true, Some(7)), 2);
    }


    #[test]
    fn test_pinned_tabs() {
        let mut manager = GosubTabManager::new();
        let tab1 = GosubTab::new("about:blank", "New tab 1");
        let tab2 = GosubTab::new("about:blank", "New tab 2");
        let mut tab3 = GosubTab::new("about:blank", "New tab 3");
        tab3.set_sticky(true);
        let tab4 = GosubTab::new("about:blank", "New tab 4");
        let mut tab5 = GosubTab::new("about:blank", "New tab 5");
        tab5.set_sticky(true);
        let tab6 = GosubTab::new("about:blank", "New tab 6");

        let tab1_id = manager.add_tab(tab1, None);
        let tab2_id = manager.add_tab(tab2, None);
        let tab3_id = manager.add_tab(tab3, None);
        let tab4_id = manager.add_tab(tab4, None);
        let tab5_id = manager.add_tab(tab5, None);
        let tab6_id = manager.add_tab(tab6, None);

        // Since some tabs are sticky, this is the ordering:
        // [ 3 5 1 2 4 6 ]
        assert_eq!(manager.tab_order, vec![tab3_id, tab5_id, tab1_id, tab2_id, tab4_id, tab6_id]);

        assert_eq!(manager.get_first_nonpinned_tab_index(), 2);
        assert_eq!(manager.is_most_left_nonsticky_tab(tab1_id), true);
        assert_eq!(manager.is_most_left_nonsticky_tab(tab2_id), false);
        assert_eq!(manager.is_most_right_tab(tab6_id), true);
        assert_eq!(manager.is_most_right_tab(tab5_id), false);
    }
}