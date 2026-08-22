use crate::app::ui::grid::GridCell;
use crate::app::ui::mog_arrow_shapes::{MogPermutationShapeCache, ShapeSource, point_to_grid_cell};
use crate::app::{
    AppState,
    ui::mog::{draw_f4, row_to_f4},
};
use algebraeon::combinatorics::golay_codes::extended_binary_golay_code::Point;
use algebraeon::structures::MetaCountableSetSignature;
use algebraeon::structures::MetaOrderedFiniteSetSignature;
use eframe::{
    Frame,
    egui::{CentralPanel, Color32, Context, SidePanel},
};

#[derive(Clone)]
pub struct State {
    selected_mappings: [Option<Point>; 24],
    mappings_shapes: MogPermutationShapeCache,
    drag_start: Option<Point>, // Set as soon as mouse is pressed
    is_dragging: bool, // Set only once the mouse has moved far enough to be considered dragging
    drag_end: Option<Point>, // Set at the end of the drag
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_mappings: [const { None }; 24],
            mappings_shapes: MogPermutationShapeCache::default(),
            drag_start: None,
            is_dragging: false,
            drag_end: None,
        }
    }
}

impl AppState for State {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) -> Option<Box<dyn AppState>> {
        if let Some(new_state) = SidePanel::left("left_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                if ui.button("Back").clicked() {
                    return Some(
                        Box::new(super::point_toggle::State::default()) as Box<dyn AppState>
                    );
                }

                ui.separator();

                ui.label("hiyaa");

                None
            })
            .inner
        {
            return Some(new_state);
        }

        let mut grid_builder = super::grid::GridBuilder::default();

        let row_label_to_cell = |r: usize| -> GridCell { (-1, r as isize) };
        let col_label_to_cell = |c: usize| -> GridCell { (c as isize, 4) };

        // The rows labelled by F4
        for r in 0usize..4 {
            grid_builder.include_cell(row_label_to_cell(r));
        }

        // The columns labelled by the sum of the F4 values in column
        for c in 0..6 {
            grid_builder.include_cell(col_label_to_cell(c));
        }

        // The 6x4 MOG grid
        for p in Point::generate_all_elements() {
            grid_builder.include_cell(point_to_grid_cell(&p));
        }

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, grid) = grid_builder.show(ui);

            // The rows labelled by F4
            for r in 0usize..4 {
                let rect = grid.cell_to_rect(row_label_to_cell(r));
                draw_f4(ui, &painter, rect, ui.visuals().text_color(), row_to_f4(r))
            }

            // The 6x4 MOG grid
            for p in Point::generate_all_elements() {
                let rect = grid.cell_to_rect(point_to_grid_cell(&p));

                // Not selected
                painter.rect_filled(
                    rect,
                    grid.cell_scalar_to_pos_scalar(0.05),
                    ui.visuals().widgets.inactive.bg_fill,
                );
            }

            let mut hovered_point = None;

            for p in Point::generate_all_elements() {
                let rect = grid.cell_to_rect(point_to_grid_cell(&p));

                // Check if the mouse is over this point
                if let Some(pos) = response.hover_pos()
                    && rect.contains(pos)
                {
                    hovered_point = Some(p.clone());
                }

                // Start dragging
                if response.is_pointer_button_down_on()
                    && self.drag_start.is_none()
                    && let Some(pos) = response.interact_pointer_pos()
                    && rect.contains(pos)
                {
                    self.drag_start = Some(p.clone());
                }

                // Dragging
                if response.dragged()
                    && self.is_dragging
                    && let Some(pos) = response.interact_pointer_pos()
                    && rect.contains(pos)
                {
                    self.drag_end = Some(p.clone());
                }
            }

            if response.drag_started() {
                self.is_dragging = true;
            }

            let mut drag_mappings = self.selected_mappings.clone();
            if self.is_dragging
                && let Some(start_p) = &self.drag_start
                && let Some(end_p) = &self.drag_end
                && (response.dragged() || response.drag_stopped())
            {
                let start_i: usize = start_p.element_to_enumeration().try_into().unwrap();
                if start_p == end_p {
                    drag_mappings[start_i] = None;
                } else {
                    drag_mappings[start_i] = Some(end_p.clone());
                }
            }

            let colour = Color32::CYAN;

            if self.is_dragging
                && let Some(start_p) = &self.drag_start
                && start_p == self.drag_end.as_ref().unwrap_or(start_p)
                && response.is_pointer_button_down_on()
            {
                painter.circle_filled(
                    grid.cell_to_pos(point_to_grid_cell(&start_p)),
                    grid.cell_scalar_to_pos_scalar(self.mappings_shapes.small_radius()),
                    colour,
                );
            }

            self.mappings_shapes.set_arrows(
                &drag_mappings
                    .iter()
                    .enumerate()
                    .filter_map(|(start_i, end_p)| {
                        let start_p = Point::enumeration_to_element(&start_i.into()).unwrap();
                        end_p.clone().map(|end_p| (start_p, end_p))
                    })
                    .collect::<Vec<_>>(),
                grid,
            );

            // Stop dragging
            if !response.is_pointer_button_down_on() {
                self.selected_mappings = drag_mappings;
                self.drag_start = None;
                self.is_dragging = false;
                self.drag_end = None;
            }

            for (source, shape) in self.mappings_shapes.sources_and_shapes() {
                let colour = if let Some(p) = &hovered_point
                    && let ShapeSource::Arrow(start, end) = source
                    && (point_to_grid_cell(p) == *start || point_to_grid_cell(p) == *end)
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
