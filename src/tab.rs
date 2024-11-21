use std::collections::HashMap;
use gtk4::gdk_pixbuf::Pixbuf;
use uuid::Uuid;

#[derive(Clone)]
pub struct GosubTab {
    /// Tab is currently loading
    loading: bool,
    /// Id of the tab
    id: Uuid,
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
    favicon: Option<Pixbuf>,
    /// Actual content (HTML) of the tab
    content: String,
}

impl GosubTab {
    pub fn new(url: &str, title: &str) -> Self {
        GosubTab {
            loading: false,
            id: Uuid::new_v4(),
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

    pub fn id(&self) -> Uuid {
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

    pub(crate) fn favicon(&self) -> Option<Pixbuf> {
        self.favicon.clone()
    }

    pub fn set_favicon(&mut self, favicon: Option<Pixbuf>) {
        self.favicon = favicon;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TabCommand {
    Close(u32),     // Close indexUpdate
    CloseAll,       // Close all
    Move(u32, u32), // Move from index to index
    Pin(u32),       // Pin index
    Unpin(u32),     // Unpin index
    Private(u32),   // Make private tab
    Update(u32),    // Update index (tab + content)
    Insert(u32),    // Insert index
    Activate(u32),  // set as active
}

pub struct GosubTabManager {
    // All known tabs in the system
    tabs: HashMap<Uuid, GosubTab>,
    // Actual ordering of the tabs in the notebook. Used for converting page_num to tab_id
    tab_order: Vec<Uuid>,
    // Currently active tab, if any
    active_tab: Uuid,
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
        let mut manager = GosubTabManager {
            tabs: HashMap::new(),
            tab_order: Vec::new(),
            active_tab: Uuid::new_v4(),
            commands: Vec::new(),
        };

        // Always add an initial tab
        let mut tab = GosubTab::new("about:blank", "New tab");
        tab.set_loading(false);
        let tab_id = manager.add_tab(tab, None);
        manager.mark_tab_updated(tab_id);   // This will take care of removing the "loading" spinner.

        manager
    }

    #[allow(dead_code)]
    pub(crate) fn get_by_tab(&self, tab_id: Uuid) -> Option<&GosubTab> {
        self.tabs.get(&tab_id)
    }

    pub(crate) fn get_page_num_by_tab(&self, tab_id: Uuid) -> Option<u32> {
        self.tab_order.iter().position(|id| id == &tab_id).map(|pos| pos as u32)
    }

    pub(crate) fn commands(&mut self) -> Vec<TabCommand> {
        self.commands.drain(..).collect()
    }

    pub(crate) fn tab_to_page(&self, tab_id: Uuid) -> Option<u32> {
        self.tab_order.iter().position(|id| id == &tab_id).map(|pos| pos as u32)
    }

    pub(crate) fn page_to_tab(&self, page_index: u32) -> Option<Uuid> {
        self.tab_order.get(page_index as usize).cloned()
    }

    pub(crate) fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub(crate) fn get_active_tab(&self) -> Option<GosubTab> {
        let tab_id = self.active_tab;
        self.get_tab(tab_id)
    }

    pub fn set_active(&mut self, tab_id: Uuid) {
        if let Some(page_num) = self.tab_order.iter().position(|&id| id == tab_id) {
            println!("Setting active tab to page {} / {}", page_num, tab_id);
            self.active_tab = tab_id;

            self.commands.push(TabCommand::Activate(page_num as u32));
        }
    }

    pub fn mark_tab_updated(&mut self, tab_id: Uuid) {
        if let Some(page_num) = self.tab_to_page(tab_id) {
            self.commands.push(TabCommand::Update(page_num));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn notify_tab_changed(&mut self, tab_id: Uuid) {
        if let Some(page_num) = self.tab_order.iter().position(|&id| id == tab_id) {
            self.commands.push(TabCommand::Update(page_num as u32));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn update_tab(&mut self, tab_id: Uuid, tab: &GosubTab) {
        self.tabs.insert(tab_id, tab.clone());
        self.notify_tab_changed(tab_id);
    }

    pub fn add_tab(&mut self, tab: GosubTab, position: Option<usize>) -> Uuid {
        let pos = if let Some(pos) = position {
            self.tab_order.insert(pos, tab.id);
            pos
        } else {
            self.tab_order.push(tab.id);
            self.tab_order.len() - 1
        };

        self.commands.push(TabCommand::Insert(pos as u32));

        let tab_id = tab.id.clone();
        self.tabs.insert(tab_id, tab);
        self.set_active(tab_id);

        tab_id
    }

    pub fn remove_tab(&mut self, tab_id: Uuid) {
        if let Some(index) = self.tab_order.iter().position(|id| id == &tab_id) {
            println!("removing tab at index {}", index);
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

    pub fn get_tab(&self, tab_id: Uuid) -> Option<GosubTab> {
        if let Some(tab) = self.tabs.get(&tab_id) {
            return Some(tab.clone())
        }
        None
    }

    pub fn order(&self) -> Vec<Uuid> {
        self.tab_order.clone()
    }
}