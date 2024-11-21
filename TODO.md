
# TODO Gui specific stuff:

- [X] Implement the tab manager
- [X] Implement open tab CTRL-T
- [X] Make async work, so we can init a tab with a spinner, async load favicon, and display favicon once loaded
- [X] Implement url loader in searchbar
- [X] implement closing tab
- [ ] implement pinned tabs (should be working but no way to pin them yet)
- [ ] implement tab dragging and sorting
- [X] change title of window based on the title of the tab
- [ ] implement url history (per tab)
- [X] implement scroller for logging window
- [X] show/hide logging window with CTRL-L
- [X] show raw HTML inside browser tab so we can see things are loaded
- [ ] Address bar should reflect current address

# TODO Gosub engine implementation (assumes async eventloop is implemented in gosub engine):

- [ ] Make sure we can use vello in a window for drawing
- [ ] enable engine to parse html
- [ ] make sure render backend can render html
- [X] scrollable html 
- [X] html per tab
- [ ] make use of the gosub eventloop for tasks