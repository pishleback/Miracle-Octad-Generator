use crate::app::logic::finite_field_4::Point as F4Point;
use crate::app::logic::miracle_octad_generator::*;
use crate::app::logic::permutation::Permutation;
use crate::app::logic::traits::{Enumerated, Labelled};
use crate::app::ui::grid::GridCell;
use crate::app::ui::mog::sextet_idx_to_colour;
use crate::app::ui::mog_permutation_shapes::MogPermutationShapeCache;
use crate::app::{
    AppState,
    ui::mog::{draw_f4, row_to_f4},
};
use eframe::{
    Frame,
    egui::{CentralPanel, Color32, Context, SidePanel},
};

#[derive(Clone)]
pub struct State {
    selected_points: Labelled<Point, bool>,
    selected_permutation: Permutation<Point>,
    permutation_shapes: MogPermutationShapeCache,
    drag_start: Option<Point>, // Set as soon as mouse is pressed
    is_dragging: bool, // Set only once the mouse has moved far enough to be considered dragging
    drag_end: Option<Point>, // Set at the end of the drag
}

impl Default for State {
    fn default() -> Self {
        Self::new(Labelled::new_constant(false), Permutation::identity())
    }
}

impl State {
    pub fn new(
        selected_points: Labelled<Point, bool>,
        selected_permutation: Permutation<Point>,
    ) -> Self {
        Self {
            selected_points,
            selected_permutation,
            permutation_shapes: MogPermutationShapeCache::default(),
            drag_start: None,
            is_dragging: false,
            drag_end: None,
        }
    }
}

impl AppState for State {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) -> Option<Box<dyn AppState>> {
        let mut preview_select_points = Labelled::<Point, Option<bool>>::new_constant(None);
        let mut coloured_highlight_points = Labelled::<Point, Option<Color32>>::new_constant(None);

        let mog = super::mog::mog();

        if let Some(new_state) = SidePanel::left("left_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                // Clear selection
                if self.selected_points.weight() != 0
                    || self.selected_permutation != Permutation::identity()
                {
                    ui.heading("Clear selection");
                    let button = ui.button("Clear");

                    if button.hovered() {
                        for p in self.selected_points.points() {
                            preview_select_points.set(p, Some(false));
                        }
                    }

                    if button.clicked() {
                        self.selected_points.set_all(false);
                        self.selected_permutation = Permutation::identity();
                    }
                }

                if self.selected_permutation != Permutation::identity() {
                    ui.heading("Permutation");
                    if mog.is_automorphism(&self.selected_permutation) {
                        ui.label("Automorphism");
                    } else {
                        ui.label("Not Automorphism");
                    }

                    if ui.button("Invert").clicked() {
                        self.selected_permutation = self.selected_permutation.clone().inverse();
                    }

                    if ui.button("Apply").clicked() {
                        self.selected_points =
                            self.selected_points.permute(&self.selected_permutation);
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
                            return Some(Box::new(super::sextet_labelling::State::from_foursome(
                                self.clone(),
                                &(&self.selected_points + &codewords[0]),
                            )));
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
            return Some(new_state);
        }

        let mut grid_builder = super::grid::GridBuilder::default();

        let row_label_to_cell = |r: usize| -> GridCell { (-1, r as isize) };
        let col_label_to_cell = |c: usize| -> GridCell { (c as isize, 4) };
        let point_to_cell = |p: Point| -> GridCell {
            let i = p.point_to_usize();
            (i as isize % 6, i as isize / 6)
        };

        // The rows labelled by F4
        for r in 0usize..4 {
            grid_builder.include_cell(row_label_to_cell(r));
        }

        // The columns labelled by the sum of the F4 values in column
        for c in 0..6 {
            grid_builder.include_cell(col_label_to_cell(c));
        }

        // The 6x4 MOG grid
        for p in Point::points() {
            grid_builder.include_cell(point_to_cell(p));
        }

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, grid) = grid_builder.show(ui);

            // The rows labelled by F4
            for r in 0usize..4 {
                let rect = grid.cell_to_rect(row_label_to_cell(r));
                draw_f4(ui, &painter, rect, ui.visuals().text_color(), row_to_f4(r))
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
                let rect = grid.cell_to_rect(col_label_to_cell(c));
                draw_f4(ui, &painter, rect, ui.visuals().text_color(), t);
            }

            // The 6x4 MOG grid
            for p in Point::points() {
                let rect = grid.cell_to_rect(point_to_cell(p));

                // Draw square
                if preview_select_points
                    .get(p)
                    .unwrap_or(*self.selected_points.get(p))
                {
                    // Selected
                    painter.rect_filled(
                        rect,
                        grid.cell_scalar_to_pos_scalar(0.05),
                        ui.visuals().selection.bg_fill,
                    );
                } else {
                    // Not selected
                    painter.rect_filled(
                        rect,
                        grid.cell_scalar_to_pos_scalar(0.05),
                        ui.visuals().widgets.inactive.bg_fill,
                    );
                }

                // Highlight
                if preview_select_points.get(p).is_some() || {
                    if self.is_dragging {
                        // Don't highlight when dragging
                        false
                    } else if let Some(pos) = response.hover_pos() {
                        // Highlight when hovering on this pos
                        rect.contains(pos)
                    } else {
                        // Don't highlight otherwise
                        false
                    }
                } {
                    painter.rect_stroke(
                        rect,
                        grid.cell_scalar_to_pos_scalar(0.05),
                        ui.visuals().widgets.hovered.fg_stroke,
                        eframe::egui::StrokeKind::Middle,
                    );
                }

                // Coloured highlihgts
                if let Some(colour) = coloured_highlight_points.get(p) {
                    painter.rect_stroke(
                        rect,
                        grid.cell_scalar_to_pos_scalar(0.05),
                        eframe::egui::Stroke::new(
                            3.0,
                            colour.lerp_to_gamma(ui.visuals().faint_bg_color, 0.4),
                        ),
                        eframe::egui::StrokeKind::Inside,
                    );
                }

                // Toggle if clicked
                if response.clicked() && rect.contains(response.interact_pointer_pos().unwrap()) {
                    let b = self.selected_points.get_mut(p);
                    *b = !*b;
                }
            }

            let mut hovered_point = None;

            for p in Point::points() {
                let rect = grid.cell_to_rect(point_to_cell(p));

                // Check if the mouse is over this point
                if let Some(pos) = response.hover_pos()
                    && rect.contains(pos)
                {
                    hovered_point = Some(p);
                }

                // Start dragging
                if response.is_pointer_button_down_on()
                    && self.drag_start.is_none()
                    && let Some(pos) = response.interact_pointer_pos()
                    && rect.contains(pos)
                {
                    self.drag_start = Some(p);
                }

                // Dragging
                if response.dragged()
                    && self.is_dragging
                    && let Some(pos) = response.interact_pointer_pos()
                    && rect.contains(pos)
                {
                    self.drag_end = Some(p);
                }
            }

            if response.drag_started() {
                self.is_dragging = true;
            }

            let mut drag_permutation = self.selected_permutation.clone();
            if self.is_dragging
                && let Some(start_p) = self.drag_start
                && let Some(end_p) = self.drag_end
                && (response.dragged() || response.drag_stopped())
            {
                drag_permutation = &Permutation::new_swap(&start_p, &end_p) * &drag_permutation;
            }

            let colour = if mog.is_automorphism(&drag_permutation) {
                Color32::GREEN
            } else {
                Color32::RED
            };

            if self.is_dragging
                && let Some(start_p) = self.drag_start
                && start_p == self.drag_end.unwrap_or(start_p)
                && response.is_pointer_button_down_on()
            {
                painter.circle_filled(
                    grid.cell_to_pos(point_to_cell(start_p)),
                    grid.cell_scalar_to_pos_scalar(self.permutation_shapes.small_radius()),
                    colour,
                );
            }

            // Stop dragging
            if response.drag_stopped() {
                self.selected_permutation = drag_permutation.clone();
            }
            if !response.is_pointer_button_down_on() {
                self.drag_start = None;
                self.is_dragging = false;
                self.drag_end = None;
            }

            let cell_permutation = drag_permutation
                .clone()
                .map_injective_unchecked(point_to_cell);

            self.permutation_shapes
                .set_permutation(Some(cell_permutation), grid);

            for (cycle, shape) in self.permutation_shapes.shapes() {
                let colour = if let Some(p) = hovered_point
                    && cycle.contains(&point_to_cell(p))
                {
                    colour
                } else {
                    colour * Color32::from_white_alpha(128)
                };

                painter.add(shape.to_egui_mesh(colour));
            }
        });
        None
    }
}
