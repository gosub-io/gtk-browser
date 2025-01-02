use gosub_engine::prelude::*;

/**
 * Defines the gosub engine. It consists of a set of modules that each define a part of the engine.
 * Each module has an implementation that can either be a generic implementation as found in the
 * engine itself, or a custom implementation that you can provide.
 */

#[derive(Clone, Debug, PartialEq)]
pub struct GosubEngineConfig;

impl HasCssSystem for GosubEngineConfig {
    type CssSystem = Css3System;
}
impl HasDocument for GosubEngineConfig {
    type Document = DocumentImpl<Self>;
    type DocumentFragment = DocumentFragmentImpl<Self>;
    type DocumentBuilder = DocumentBuilderImpl;
}

impl HasHtmlParser for GosubEngineConfig {
    type HtmlParser = Html5Parser<'static, Self>;
}

impl HasLayouter for GosubEngineConfig {
    type Layouter = TaffyLayouter;
    type LayoutTree = RenderTree<Self>;
}

impl HasRenderTree for GosubEngineConfig {
    type RenderTree = RenderTree<Self>;
}

impl HasTreeDrawer for GosubEngineConfig {
    type TreeDrawer = TreeDrawerImpl<Self>;
}

impl HasRenderBackend for GosubEngineConfig {
    type RenderBackend = CairoBackend;
}

impl ModuleConfiguration for GosubEngineConfig {}
