use crate::logic;
use crate::logic::finite_field_4::Point as F4Point;
use crate::logic::miracle_octad_generator::*;
use crate::logic::traits::{Enumerated, Labelled};
use crate::ui::mog::sextet_idx_to_colour;
use crate::{
    AppState,
    ui::mog::{draw_f4, row_to_f4},
};
use eframe::egui::Button;
use eframe::{
    Frame,
    egui::{CentralPanel, Color32, Context, Painter, Rect, SidePanel, Ui},
    egui_glow::painter,
};

#[derive(Clone)]
pub struct State {
    selected_points: Labelled<Point, bool>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_points: Labelled::new_constant(false),
        }
    }
}

impl AppState for State {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) -> Option<Box<dyn AppState>> {
        let mut highlight_points = Labelled::<Point, bool>::new_constant(false);
        let mut coloured_highlight_points = Labelled::<Point, Option<Color32>>::new_constant(None);

        if let Some(new_state) = SidePanel::left("left_panel")
            .exact_width(200.0)
            .show(ctx, |ui| {
                let mog = super::mog::mog();

                // Clear selection
                if self.selected_points.weight() != 0 && ui.button("Clear").clicked() {
                    self.selected_points.set_all(false);
                }

                // Is it a codeword?
                ui.heading("Is it a codeword?");

                if mog.is_codeword(&self.selected_points) {
                    ui.add_enabled(false, Button::new("Yes").fill(Color32::GREEN));
                } else {
                    ui.add_enabled(false, Button::new("No").fill(Color32::RED));
                }

                // Complete and octad from 5 points
                if self.selected_points.weight() == 5 {
                    let button = ui.button("Complete Octad");

                    let octad = mog.complete_octad(&self.selected_points).unwrap();

                    // Preview octad when hovering on button
                    if button.hovered() {
                        for p in (&self.selected_points + &octad).points() {
                            highlight_points.set(p, true);
                        }
                    }

                    // complete the selection
                    if button.clicked() {
                        for p in octad.points() {
                            self.selected_points.set(p, true);
                        }
                    }
                }

                // Complete a sextet from 4 points
                if self.selected_points.weight() == 4 {
                    let complete_sextet_button = ui.button("Complete Sextet");

                    let mut sextet = mog
                        .complete_sextet(&self.selected_points)
                        .unwrap()
                        .into_iter()
                        .collect::<Vec<_>>();
                    sextet.sort_unstable();
                    sextet.reverse();
                    let ordered_sextet = sextet;

                    if complete_sextet_button.hovered() {
                        for (i, vector) in ordered_sextet.iter().enumerate() {
                            for p in vector.points() {
                                coloured_highlight_points.set(p, Some(sextet_idx_to_colour(i)));
                            }
                        }
                    }

                    if complete_sextet_button.clicked() {
                        return Some(super::sextet_labelling::State::from_foursome(
                            self.clone(),
                            &self.selected_points.clone(),
                        ));
                    }
                }
                None
            })
            .inner
        {
            return Some(Box::new(new_state));
        }

        struct State<'a> {
            selected_points: &'a mut Labelled<Point, bool>,
            highlight_points: Labelled<Point, bool>,
            coloured_highlight_points: Labelled<Point, Option<Color32>>,
        }

        let mut mog_visuals = super::grid::GridVisuals::<State>::default();

        // The rows labelled by F4
        for r in 0usize..4 {
            mog_visuals.draw(
                (-1, r as isize),
                Box::new(move |ui, response, painter, rect, state| {
                    draw_f4(ui, painter, rect, ui.visuals().text_color(), row_to_f4(r))
                }),
            )
        }

        // The columns labelled by the sum of the F4 values in column
        for c in 0..6 {
            let mut t = F4Point::Zero;
            for r in 0..4 {
                let i = c + 6 * r;
                let p = Point::usize_to_point(i).unwrap();
                if *self.selected_points.get(p) || *highlight_points.get(p) {
                    t = t + row_to_f4(r);
                }
            }
            mog_visuals.draw(
                (c as isize, 4),
                Box::new(move |ui, response, painter, rect, state| {
                    draw_f4(ui, painter, rect, ui.visuals().text_color(), t)
                }),
            )
        }

        // The 6x4 MOG grid
        for p in Point::points() {
            let i = p.point_to_usize();
            mog_visuals.draw(
                (i as isize % 6, i as isize / 6),
                Box::new(move |ui, response, painter, rect, state| {
                    // Draw square
                    if *state.selected_points.get(p) || *state.highlight_points.get(p) {
                        // Selected
                        painter.rect_filled(rect, 10.0, ui.visuals().selection.bg_fill);
                    } else {
                        // Not selected
                        painter.rect_filled(rect, 10.0, ui.visuals().widgets.inactive.bg_fill);
                    }

                    // Highlight if mouse over
                    // or if in highlight_points
                    if *state.highlight_points.get(p) || {
                        if let Some(pos) = response.hover_pos() {
                            rect.contains(pos)
                        } else {
                            false
                        }
                    } {
                        painter.rect_stroke(
                            rect,
                            10.0,
                            ui.visuals().widgets.hovered.fg_stroke,
                            eframe::egui::StrokeKind::Middle,
                        );
                    }

                    // Coloured highlihgts
                    if let Some(colour) = state.coloured_highlight_points.get(p) {
                        painter.rect_stroke(
                            rect,
                            10.0,
                            eframe::egui::Stroke::new(
                                3.0,
                                colour.linear_multiply(0.7)
                                    + ui.visuals().faint_bg_color.linear_multiply(0.3),
                            ),
                            eframe::egui::StrokeKind::Inside,
                        );
                    }

                    // Toggle if clicked
                    if response.clicked() && rect.contains(response.interact_pointer_pos().unwrap())
                    {
                        let b = state.selected_points.get_mut(p);
                        *b = !*b;
                    }
                }),
            );
        }

        CentralPanel::default().show(ctx, |ui| {
            mog_visuals.show(
                ui,
                State {
                    selected_points: &mut self.selected_points,
                    highlight_points,
                    coloured_highlight_points,
                },
            );
        });

        None
    }
}
