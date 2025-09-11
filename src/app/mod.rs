pub mod logic;
mod ui;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
// Add #[serde(skip)] to fields to opt-out of serialization of a field
pub struct MyApp {
    #[serde(skip)]
    state: Box<dyn AppState>,

    // pixels per point i.e. zoom level
    ppp: f32,
}

pub trait AppState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>>;
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: Box::new(ui::point_toggle::State::default()),
            // state: Box::new(ui::permutation_selection::State::default()),
            ppp: 2.5,
        }
    }
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        setup_custom_fonts(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        app
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    use egui::{FontDefinitions, FontFamily};

    // Start with default fonts
    let mut fonts = FontDefinitions::default();

    // Add Computer Modern (regular)
    fonts.font_data.insert(
        "cmun".to_owned(),
        egui::FontData::from_static(include_bytes!("../../cm-unicode-0.7.0/cmunrm.ttf")).into(),
    );

    // Put our font first in the proportional family (used by default labels)
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "cmun".to_owned());

    ctx.set_fonts(fonts);
}

impl eframe::App for MyApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Allow changing the zoom with ctrl + scroll
        ctx.set_pixels_per_point(self.ppp);

        ctx.input(|input| {
            let scroll_y = input.raw_scroll_delta.y;
            if input.modifiers.ctrl && scroll_y != 0.0 {
                let step = 1.003f32;
                let mut new_scale = self.ppp * step.powf(scroll_y);
                new_scale = new_scale.clamp(0.2, 12.0);
                self.ppp = new_scale;
            }
        });

        // Global Settings
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        if let Some(new_state) = self.state.update(ctx, frame) {
            self.state = new_state;
            ctx.request_discard("Changed State");
        }
    }
}
