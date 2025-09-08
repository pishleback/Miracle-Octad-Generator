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
        let mut preview_select_points = Labelled::<Point, Option<bool>>::new_constant(None);
        let mut coloured_highlight_points = Labelled::<Point, Option<Color32>>::new_constant(None);

        if let Some(new_state) = SidePanel::left("left_panel")
            .exact_width(200.0)
            .show(ctx, |ui| {
                let mog = super::mog::mog();

                // Clear selection
                if self.selected_points.weight() != 0 {
                    ui.heading("Clear selection");
                    let button = ui.button("Clear");

                    if button.hovered() {
                        for p in self.selected_points.points() {
                            preview_select_points.set(p, Some(false));
                        }
                    }

                    if button.clicked() {
                        self.selected_points.set_all(false);
                    }
                }

                // The nearest codeword(s)
                let nearest = mog.nearest_codeword(&self.selected_points);
                match nearest {
                    NearestCodewordsResult::Unique { codeword, distance } => {
                        if distance == 0 {
                            ui.heading("It's a Codeword");
                        } else {
                            ui.heading("Nearest Codeword");
                            ui.label(format!("Distance = {}", distance));

                            let button = ui.button("Select");
                            // Preview octad when hovering on button
                            if button.hovered() {
                                for p in (&self.selected_points + &codeword).points() {
                                    preview_select_points.set(p, Some(*codeword.get(p)));
                                }
                            }
                            // Complete the selection
                            if button.clicked() {
                                for p in (&self.selected_points + &codeword).points() {
                                    self.selected_points.set(p, *codeword.get(p));
                                }
                            }
                        }
                    }
                    NearestCodewordsResult::Six { codewords } => {
                        ui.heading("Nearest Codewords");
                        ui.label("Distance = 4");
                        for (num, codeword) in codewords.iter().enumerate() {
                            let button = ui.button(format!("Select {}", num + 1));
                            // Preview octad when hovering on button
                            if button.hovered() {
                                for p in (&self.selected_points + codeword).points() {
                                    preview_select_points.set(p, Some(*codeword.get(p)));
                                }
                            }
                            // Complete the selection
                            if button.clicked() {
                                for p in (&self.selected_points + codeword).points() {
                                    self.selected_points.set(p, *codeword.get(p));
                                }
                            }
                        }

                        // Complete a sextet from 4 points
                        ui.heading("Complete Sextet");
                        if self.selected_points.weight() == 4 {
                            ui.label("The unique sextet containing these 4 points");
                        } else {
                            ui.label(
                                "\
The sextet whose foursomes are the differences between these points and the nearest 6 codewords",
                            );
                        }
                        let complete_sextet_button = ui.button("Select");

                        let mut sextet = codewords
                            .iter()
                            .map(|codeword| &self.selected_points + codeword)
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
                                &(&self.selected_points + &codewords[0]),
                            ));
                        }
                    }
                }

                // Complete and octad from 5 points
                if self.selected_points.weight() == 5 {
                    ui.heading("Complete Octad");
                    ui.label("The unique octad containing these 5 points");
                    let button = ui.button("Complete");

                    let octad = mog.complete_octad(&self.selected_points).unwrap();

                    // Preview octad when hovering on button
                    if button.hovered() {
                        for p in (&self.selected_points + &octad).points() {
                            preview_select_points.set(p, Some(true));
                        }
                    }
                    // complete the selection
                    if button.clicked() {
                        for p in octad.points() {
                            self.selected_points.set(p, true);
                        }
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
            preview_select_points: Labelled<Point, Option<bool>>,
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
                if preview_select_points
                    .get(p)
                    .unwrap_or(*self.selected_points.get(p))
                {
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
                    if state
                        .preview_select_points
                        .get(p)
                        .unwrap_or(*state.selected_points.get(p))
                    {
                        // Selected
                        painter.rect_filled(rect, 10.0, ui.visuals().selection.bg_fill);
                    } else {
                        // Not selected
                        painter.rect_filled(rect, 10.0, ui.visuals().widgets.inactive.bg_fill);
                    }

                    // Highlight if mouse over
                    // or if in highlight_points
                    if state.preview_select_points.get(p).is_some() || {
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
                    preview_select_points,
                    coloured_highlight_points,
                },
            );
        });

        None
    }
}
