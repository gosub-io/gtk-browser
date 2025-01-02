use crate::engine::GosubEngineConfig;
use gosub_engine::prelude::*;
use log::info;
use reqwest::Url;

#[derive(Clone)]
pub struct WindowEventLoopDummy;

impl WindowedEventLoop<GosubEngineConfig> for WindowEventLoopDummy {
    fn redraw(&mut self) {
        info!(target: "eventloop", "Redraw needed");
    }

    fn add_img_cache(
        &mut self,
        url: String,
        _buf: ImageBuffer<<GosubEngineConfig as HasRenderBackend>::RenderBackend>,
        _size: Option<SizeU32>,
    ) {
        info!(target: "eventloop", "Add image to cache: {}", url);
    }

    fn reload_from(&mut self, _rt: <GosubEngineConfig as HasRenderTree>::RenderTree) {
        info!(target: "eventloop", "reload from")
    }

    fn open_tab(&mut self, _url: Url) {
        info!(target: "eventloop", "open tab")
    }
}
