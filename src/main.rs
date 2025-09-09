#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui::{self};
use eframe::egui::{FontDefinitions, FontFamily};

mod logic;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Drawable Canvas with Colors",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_pixels_per_point(1.6);
            Ok(Box::<MyApp>::default())
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with default fonts
    let mut fonts = FontDefinitions::default();

    // Add Computer Modern (regular)
    fonts.font_data.insert(
        "cmun".to_owned(),
        egui::FontData::from_static(include_bytes!("../cm-unicode-0.7.0/cmunrm.ttf")).into(),
    );

    // Put our font first in the proportional family (used by default labels)
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "cmun".to_owned());

    ctx.set_fonts(fonts);
}

mod ui;

pub trait AppState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>>;
}

struct MyApp {
    state: Box<dyn AppState>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // state: Box::new(ui::point_toggle::State::default()),
            state: Box::new(ui::permutation_selection::State::default()),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // ctx.set_visuals(egui::Visuals::dark());
        ctx.set_visuals(egui::Visuals::light());
        if let Some(new_state) = self.state.update(ctx, frame) {
            self.state = new_state;
            ctx.request_discard("Changed State");
        }
    }
}
