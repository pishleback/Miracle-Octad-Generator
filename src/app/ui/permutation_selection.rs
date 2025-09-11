use crate::app::logic::miracle_octad_generator::*;
use crate::app::logic::permutation::Permutation;
use crate::app::logic::traits::Enumerated;
use crate::app::ui::grid::GridCell;
use crate::app::ui::mog_permutation_shapes::MogPermutationShapeCache;
use crate::app::{AppState, ui::mog::mog};
use eframe::{
    Frame,
    egui::{CentralPanel, Color32, Context, SidePanel},
};

#[derive(Clone)]
pub struct State<PrevState: AppState + Clone + 'static> {
    prev_state: Option<PrevState>,
    permutation: Permutation<Point>,
    permutation_shapes: MogPermutationShapeCache,
    drag_start: Option<Point>,
    drag_end: Option<Point>,
}

impl<PrevState: AppState + Clone + 'static> Default for State<PrevState> {
    fn default() -> Self {
        Self::new(None, Permutation::identity())
    }
}

impl<PrevState: AppState + Clone + 'static> State<PrevState> {
    pub fn new(prev_state: Option<PrevState>, permutation: Permutation<Point>) -> Self {
        Self {
            prev_state,
            permutation,
            permutation_shapes: MogPermutationShapeCache::default(),
            drag_start: None,
            drag_end: None,
        }
    }
}

impl<PrevState: AppState + Clone + 'static> AppState for State<PrevState> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) -> Option<Box<dyn AppState>> {
        let mog = mog();

        if let Some(new_state) = SidePanel::left("left_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                // Back
                if let Some(prev_state) = self.prev_state.as_ref()
                    && ui.button("Back").clicked()
                {
                    return Some(Box::<dyn AppState>::from(Box::new(prev_state.clone())));
                }

                ui.heading("Automorphism?");
                if mog.is_automorphism(&self.permutation) {
                    ui.label("Yes");
                } else {
                    ui.label("No");
                }

                ui.heading("Invert Permutation");
                if ui.button("Invert").clicked() {
                    self.permutation = self.permutation.clone().inverse();
                }

                None
            })
            .inner
        {
            return Some(new_state);
        }

        let point_to_cell = |p: Point| -> GridCell {
            let i = p.point_to_usize();
            (i as isize % 6, i as isize / 6)
        };

        let mut grid_builder = super::grid::GridBuilder::default();

        // The 6x4 MOG grid
        for p in Point::points() {
            grid_builder.include_cell(point_to_cell(p));
        }

        let mut hovered_point = None;

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, grid) = grid_builder.show(ui);

            for p in Point::points() {
                let rect = grid.cell_to_rect(point_to_cell(p));

                // Draw square
                painter.rect_filled(
                    rect,
                    grid.cell_scalar_to_pos_scalar(0.05),
                    ui.visuals().widgets.inactive.bg_fill,
                );

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
                    && let Some(pos) = response.interact_pointer_pos()
                    && rect.contains(pos)
                {
                    self.drag_end = Some(p);
                }
            }

            let mut drag_permutation = self.permutation.clone();
            if let Some(start_p) = self.drag_start
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

            if let Some(start_p) = self.drag_start
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
                self.permutation = drag_permutation.clone();
            }
            if !response.is_pointer_button_down_on() {
                self.drag_start = None;
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
