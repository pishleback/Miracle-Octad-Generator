use crate::AppState;
use crate::logic::miracle_octad_generator::*;
use crate::logic::permutation::Permutation;
use crate::logic::traits::Enumerated;
use crate::ui::grid::GridCell;
use crate::ui::mog_permutation_shapes::MogPermutationShapeCache;
use eframe::{
    Frame,
    egui::{CentralPanel, Color32, Context, SidePanel},
};

#[derive(Clone)]
pub struct State {
    permutation: Permutation<Point>,
    permutation_shapes: MogPermutationShapeCache,
    drag_start: Option<Point>,
    drag_end: Option<Point>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            permutation: Permutation::identity(),
            permutation_shapes: MogPermutationShapeCache::default(),
            drag_start: None,
            drag_end: None,
        }
    }
}

impl AppState for State {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) -> Option<Box<dyn AppState>> {
        if let Some(new_state) = SidePanel::left("left_panel")
            .exact_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Foo");
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

        let mut hovered_point = None;

        struct StateWrapper<'a> {
            hovered_point: &'a mut Option<Point>,
            state: &'a mut State,
        }

        let mut grid = super::grid::GridVisuals::<StateWrapper>::default();

        // The 6x4 MOG grid
        for p in Point::points() {
            let i = p.point_to_usize();
            grid.draw_cell(
                (i as isize % 6, i as isize / 6),
                Box::new(move |ui, response, painter, rect, state| {
                    // Draw square
                    painter.rect_filled(rect, 10.0, ui.visuals().widgets.inactive.bg_fill);

                    // Check if the mouse is over this point
                    if let Some(pos) = response.hover_pos()
                        && rect.contains(pos)
                    {
                        *state.hovered_point = Some(p);
                    }

                    // Start dragging
                    if response.drag_started()
                        && let Some(pos) = response.interact_pointer_pos()
                        && rect.contains(pos)
                    {
                        state.state.drag_start = Some(p);
                    }

                    // Dragging
                    if response.dragged()
                        && let Some(pos) = response.interact_pointer_pos()
                        && rect.contains(pos)
                    {
                        state.state.drag_end = Some(p);
                    }
                }),
            );
        }

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, state, coordinates) = grid.show(
                ui,
                StateWrapper {
                    hovered_point: &mut hovered_point,
                    state: self,
                },
            );

            let mut drag_permutation = state.state.permutation.clone();
            if (response.dragged() || response.drag_stopped())
                && let Some(start_p) = state.state.drag_start
                && let Some(end_p) = state.state.drag_end
            {
                drag_permutation = &Permutation::new_swap(&start_p, &end_p) * &drag_permutation;
            }

            // Stop dragging
            if response.drag_stopped() {
                state.state.permutation = drag_permutation.clone();
                state.state.drag_start = None;
                state.state.drag_end = None;
            }

            let cell_permutation = drag_permutation
                .clone()
                .map_injective_unchecked(point_to_cell);

            state
                .state
                .permutation_shapes
                .set_permutation(cell_permutation, coordinates);

            for (cycle, shape) in state.state.permutation_shapes.shapes() {
                let colour = if let Some(p) = state.hovered_point
                    && cycle.contains(&point_to_cell(*p))
                {
                    Color32::BLACK
                } else {
                    Color32::BLACK * Color32::from_white_alpha(96)
                };

                painter.add(shape.to_egui_mesh(colour));
            }
        });

        None
    }
}
