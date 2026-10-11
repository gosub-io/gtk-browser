//! `gosub://dive` as the egui frontend shows it: the game's frame as a texture in place of
//! the page, ticked at the game's own pace, flapped with a tap, a click, or space.
//!
//! The game itself is `beacon_core::dive`, shared with GTK; this file only moves its pixels
//! into egui and its one input out of egui.

use beacon_core::dive::paint::PixelOrder;
use beacon_core::dive::{Dive, FRAME};

/// Past this many device pixels the game paints at half resolution and the texture doubles
/// it, as GTK does: a frame is painted on the CPU every tick, and a maximised window on a
/// wide display does not fit that in a frame. Pixel art doubles cleanly.
const COARSE_ABOVE: u32 = 3_000_000;

/// A tab playing the game.
#[derive(Default)]
pub struct DiveView {
    game: Option<Dive>,
    texture: Option<egui::TextureHandle>,
    /// Game time owed since the last frame, in seconds.
    owed: f64,
    /// When the game was last ticked. Off screen (another tab, the tab list) it is not, and
    /// the cap on owed time below means it does not catch up on return either.
    last: Option<f64>,
}

impl DiveView {
    /// The last frame shown, for the tab list's card.
    pub fn texture(&self) -> Option<egui::TextureId> {
        self.texture.as_ref().map(|t| t.id())
    }

    /// Play a frame into `rect`: tick, take the input, paint.
    pub fn show(&mut self, ui: &mut egui::Ui, rect: egui::Rect, keys: bool) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let (w, h) = (
            (rect.width() * ppp).round().max(1.0) as u32,
            (rect.height() * ppp).round().max(1.0) as u32,
        );
        let coarse = if w * h > COARSE_ABOVE { 2 } else { 1 };
        let (pw, ph, scale) = ((w / coarse).max(1), (h / coarse).max(1), ppp / coarse as f32);
        let game = self.game.get_or_insert_with(|| Dive::new(pw, ph, scale, PixelOrder::Rgba));
        game.resize(pw, ph, scale);
        game.set_dark(ui.visuals().dark_mode);
        game.set_touch(cfg!(target_os = "android"));

        // A press, not a click: the sub should rise when the finger lands, not when it
        // lifts. egui-winit turns a finger into a pointer press as well.
        ui.allocate_rect(rect, egui::Sense::click());
        let pressed = ctx.input(|i| i.pointer.any_pressed() && i.pointer.press_origin().is_some_and(|pos| rect.contains(pos)));
        let key = keys
            && ctx.input_mut(|i| {
                // Not Enter, which GTK also takes: here the Enter that submits
                // `gosub://dive` in the address bar is still in the frame that starts the
                // game, and the sub would rise before anyone pressed anything.
                [egui::Key::Space, egui::Key::ArrowUp]
                    .into_iter()
                    .any(|key| i.consume_key(egui::Modifiers::NONE, key))
            });
        if pressed || key {
            game.flap();
        }

        // Fixed steps of one game frame, the remainder carried, so the game runs at its own
        // speed whatever the display's rate; at most a handful at once, so a stall is not a
        // fast-forward.
        let now = ctx.input(|i| i.time);
        let frame = FRAME.as_secs_f64();
        if let Some(last) = self.last {
            self.owed = (self.owed + (now - last).max(0.0)).min(frame * 4.0);
            while self.owed >= frame {
                game.tick();
                self.owed -= frame;
            }
        }
        self.last = Some(now);

        // Every pixel is opaque, so premultiplied is a straight copy.
        let image = egui::ColorImage::from_rgba_premultiplied([pw as usize, ph as usize], game.frame());
        match &mut self.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => self.texture = Some(ctx.load_texture("dive", image, egui::TextureOptions::NEAREST)),
        }
        if let Some(texture) = &self.texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, egui::Color32::WHITE);
        }
        ctx.request_repaint();
    }
}
