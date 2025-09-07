#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::App;
use eframe::egui::{
    self, Button, Color32, FontId, Grid, Painter, Pos2, Rect, Response, Stroke, Ui, Vec2,
};
use eframe::egui::{FontDefinitions, FontFamily};
use env_logger::fmt::style::Color;
use std::cell::OnceCell;

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

mod miracle_octad_generator_old {
    use std::{default, ops::Index, vec};

    use eframe::egui::{self, Button, Color32, Painter, Pos2, Rect, Response, Stroke, Ui, Vec2};

    use crate::logic::{self, miracle_octad_generator};

    #[derive(Default)]
    pub struct State {
        logic: logic::miracle_octad_generator::BinaryGolayCode,
        mode: Mode,
    }

    #[derive(Debug, Clone)]
    enum Mode {
        PointSelection(LabelledMOGPoints<bool>),
        Sextet {
            ordered_sextet: Vec<logic::miracle_octad_generator::Vector>,
            reordering: Vec<usize>, // a permutation of the 6 sextets
        },
    }

    impl Default for Mode {
        fn default() -> Self {
            Self::PointSelection(LabelledMOGPoints {
                entries: std::array::from_fn(|_| false),
            })
        }
    }

    #[derive(Default, Debug, Clone)]
    struct LabelledMOGPoints<T> {
        entries: [T; 24],
    }

    impl LabelledMOGPoints<bool> {
        fn count(&self) -> usize {
            let mut t = 0;
            for i in 0..24 {
                if self.entries[i] {
                    t += 1;
                }
            }
            t
        }

        fn set_all(&mut self, value: bool) {
            for i in 0..24 {
                self.entries[i] = value;
            }
        }

        fn to_vector(&self) -> logic::miracle_octad_generator::Vector {
            logic::miracle_octad_generator::Vector::from_fn(|p| {
                self.entries[logic::miracle_octad_generator::Vector::point_to_usize(p)]
            })
        }
    }

    impl<T> LabelledMOGPoints<T> {
        fn set(&mut self, i: usize, t: T) {
            self.entries[i] = t;
        }

        fn get(&self, i: usize) -> &T {
            &self.entries[i]
        }

        fn get_mut(&mut self, i: usize) -> &mut T {
            &mut self.entries[i]
        }
    }

    impl State {
        pub fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            #[derive(Default)]
            struct PointVisual {
                hovered: bool,
                selected: bool,
                fill_colour: Option<Color32>,
                stroke_colour: Option<Color32>,
            }
            let mut mog_visuals: LabelledMOGPoints<PointVisual> = LabelledMOGPoints::default();

            egui::SidePanel::left("left_panel")
                .exact_width(200.0)
                .show(ctx, |ui| {
                    if let Some(new_mode) = (|| -> Option<Mode> {
                        match &mut self.mode {
                            Mode::PointSelection(point_selection) => {
                                for i in 0..24 {
                                    if *point_selection.get(i) {
                                        mog_visuals.get_mut(i).selected = true;
                                    }
                                }

                                if point_selection.count() != 0 && ui.button("Clear").clicked() {
                                    point_selection.set_all(false);
                                }

                                ui.heading("Is it a codeword?");
                                if self.logic.is_codeword(point_selection.to_vector()) {
                                    ui.add_enabled(false, Button::new("Yes").fill(Color32::GREEN));
                                } else {
                                    ui.add_enabled(false, Button::new("No").fill(Color32::RED));
                                }

                                // completing sextets
                                if point_selection.count() == 4 {
                                    let complete_sextet_button = ui.button("Complete Sextet");

                                    let mut sextet = self
                                        .logic
                                        .complete_sextet(point_selection.to_vector())
                                        .unwrap()
                                        .into_iter()
                                        .collect::<Vec<_>>();
                                    sextet.sort_unstable();
                                    let ordered_sextet = sextet;

                                    if complete_sextet_button.hovered() {
                                        for (i, vector) in ordered_sextet.iter().enumerate() {
                                            for p in vector.points().map(|p| {
                                                miracle_octad_generator::Vector::point_to_usize(p)
                                            }) {
                                                mog_visuals.get_mut(p).stroke_colour =
                                                    Some(match i {
                                                        0 => Color32::RED,
                                                        1 => Color32::BLUE,
                                                        2 => Color32::GREEN,
                                                        3 => Color32::YELLOW,
                                                        4 => Color32::PURPLE,
                                                        5 => Color32::ORANGE,
                                                        _ => unreachable!(),
                                                    });
                                            }
                                        }
                                    }

                                    if complete_sextet_button.clicked() {
                                        return Some(Mode::Sextet {
                                            ordered_sextet,
                                            reordering: (0..6).collect(),
                                        });
                                    }
                                }

                                // completing octads
                                if point_selection.count() == 5 {
                                    let button = ui.button("Complete Octad");

                                    let octad = self
                                        .logic
                                        .complete_octad(point_selection.to_vector())
                                        .unwrap();

                                    // preview when hovering
                                    if button.hovered() {
                                        for p in (&point_selection.to_vector() + &octad).points() {
                                            mog_visuals
                                        .get_mut(
                                            logic::miracle_octad_generator::Vector::point_to_usize(
                                                p,
                                            ),
                                        )
                                        .hovered = true;
                                        }
                                    }

                                    // complete the selection
                                    if button.clicked() {
                                        for p in octad.points() {
                                            point_selection.set(
                                            logic::miracle_octad_generator::Vector::point_to_usize(
                                                p,
                                            ),
                                            true,
                                        );
                                        }
                                    }
                                }

                                if point_selection.count() == 8
                                    && self.logic.is_octad(point_selection.to_vector())
                                    && ui.add(Button::new("Select Octad")).clicked()
                                {
                                    println!("TODO");
                                }
                            }
                            Mode::Sextet {
                                ordered_sextet,
                                reordering,
                            } => {
                                if ui.button("Reset").clicked() {
                                    return Some(Mode::default());
                                }

                                fn get_colour(i: usize) -> Color32 {
                                    match i {
                                        0 => Color32::RED,
                                        1 => Color32::BLUE,
                                        2 => Color32::GREEN,
                                        3 => Color32::YELLOW,
                                        4 => Color32::PURPLE,
                                        5 => Color32::ORANGE,
                                        _ => unreachable!(),
                                    }
                                }

                                for (i, vector) in ordered_sextet.iter().enumerate() {
                                    for p in vector
                                        .points()
                                        .map(miracle_octad_generator::Vector::point_to_usize)
                                    {
                                        mog_visuals.get_mut(p).fill_colour =
                                            Some(get_colour(reordering[i]));
                                    }
                                }

                                ui.heading("Reorder Foursomes:");
                                egui_dnd::dnd(ui, "foursome_ordering").show_vec(
                                    reordering,
                                    |ui, item, handle, state| {
                                        ui.horizontal(|ui| {
                                            handle.ui(ui, |ui| {
                                                ui.colored_label(
                                                    get_colour(*item),
                                                    format!("Foursome {:?}", state.index + 1),
                                                );
                                            });
                                        });
                                    },
                                );
                            }
                        }

                        None
                    })() {
                        self.mode = new_mode;
                    }
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                let (response, painter) = ui.allocate_painter(
                    {
                        let available = ui.available_size();
                        let mut size = Vec2 {
                            x: available.x,
                            y: (4.0 / 6.0) * available.x,
                        };
                        if size.y > available.y {
                            size = size * available.y / size.y;
                        }
                        size
                    },
                    egui::Sense::click_and_drag(),
                );

                const PAD: f32 = 10.0;
                const NUDGE: f32 = 2.0;

                let unit = response.rect.width() / 6.0;
                let xs: [f32; 6] =
                    std::array::from_fn(|i| response.rect.left() + (i as f32 + 0.5) * unit);
                let ys: [f32; 4] =
                    std::array::from_fn(|j| response.rect.top() + (j as f32 + 0.5) * unit);

                for i in 0..24 {
                    let ix = i % 6;
                    let iy = i / 6;

                    let x = xs[ix] + if ix % 2 == 0 { NUDGE } else { -NUDGE };
                    let y = ys[iy] + if iy % 2 == 0 { NUDGE } else { -NUDGE };

                    let pos = Pos2 { x, y };
                    let rect = Rect::from_center_size(
                        pos,
                        Vec2 {
                            x: unit - PAD,
                            y: unit - PAD,
                        },
                    );

                    let point_visual = mog_visuals.get_mut(i);

                    if let Some(pos) = response.hover_pos()
                        && rect.contains(pos)
                    {
                        point_visual.hovered = true;
                    }

                    if point_visual.selected {
                        painter.rect_filled(rect, 10.0, ui.visuals().selection.bg_fill);
                    } else if let Some(colour) = point_visual.fill_colour {
                        painter.rect_filled(
                            rect,
                            10.0,
                            colour.linear_multiply(0.3)
                                + ui.visuals().faint_bg_color.linear_multiply(0.7),
                        );
                    } else {
                        painter.rect_filled(rect, 10.0, ui.visuals().widgets.inactive.bg_fill);
                    }

                    if let Some(colour) = point_visual.stroke_colour {
                        painter.rect_stroke(
                            rect,
                            10.0,
                            Stroke::new(
                                5.0,
                                colour.linear_multiply(0.7)
                                    + ui.visuals().faint_bg_color.linear_multiply(0.3),
                            ),
                            egui::StrokeKind::Inside,
                        );
                    } else if point_visual.hovered {
                        painter.rect_stroke(
                            rect,
                            10.0,
                            ui.visuals().widgets.hovered.fg_stroke,
                            egui::StrokeKind::Middle,
                        );
                    }

                    if let Mode::PointSelection(grid_cells) = &mut self.mode
                        && response.clicked()
                        && rect.contains(response.interact_pointer_pos().unwrap())
                    {
                        grid_cells.entries[i] = !grid_cells.entries[i];
                    }

                    let label_size = 0.7 * unit;
                    painter.text(
                        rect.center()
                            + Vec2 {
                                x: 0.0,
                                y: -0.8 * label_size,
                            },
                        egui::Align2::CENTER_CENTER,
                        "_",
                        egui::FontId::proportional(label_size),
                        ui.visuals().text_color(),
                    );
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "ω",
                        egui::FontId::proportional(label_size),
                        ui.visuals().text_color(),
                    );
                }
            });
        }
    }
}

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
            state: Box::new(ui::point_toggle::State::default()),
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
