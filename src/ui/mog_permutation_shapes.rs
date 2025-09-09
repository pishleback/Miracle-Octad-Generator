use crate::{
    logic::permutation::Permutation,
    ui::{
        grid::{GridCell, GridCoordinates},
        shape::{Shape, arrowhead_cap},
    },
};
use eframe::egui::Vec2;
use i_overlay::mesh::style::LineCap;

#[derive(Debug, Clone, Default)]
pub struct MogPermutationShapeCache {
    state: Option<(Permutation<GridCell>, GridCoordinates)>,
    cycles_and_shapes: Vec<(Vec<GridCell>, Shape)>,
}

impl MogPermutationShapeCache {
    pub fn set_permutation(
        &mut self,
        permutation: Permutation<GridCell>,
        coordinates: GridCoordinates,
    ) {
        if Some((permutation.clone(), coordinates.clone())) != self.state {
            self.state = Some((permutation.clone(), coordinates.clone()));

            let line_width = coordinates.cell_scalar_to_pos_scalar(0.1) as f64;

            let draw_line = |shape: &mut Shape,
                             start_cell: GridCell,
                             end_cell: GridCell,
                             width: f64,
                             start_cap: LineCap<[f64; 2], f64>,
                             end_cap: LineCap<[f64; 2], f64>| {
                let mut slope_type = (
                    start_cell.0.abs_diff(end_cell.0),
                    start_cell.1.abs_diff(end_cell.1),
                );
                if slope_type.0 < slope_type.1 {
                    slope_type = (slope_type.1, slope_type.0);
                }
                let start_pos = coordinates.cell_to_pos(start_cell);
                let end_pos = coordinates.cell_to_pos(end_cell);
                match slope_type {
                    (2, 0) | (3, 0) | (4, 0) | (5, 0) | (2, 2) | (3, 3) | (4, 2) => {
                        let vec = end_pos - start_pos;
                        let perp = Vec2 {
                            x: vec.y,
                            y: -vec.x,
                        };
                        *shape = &*shape
                            | &Shape::bezier(
                                vec![start_pos, start_pos + 0.5 * vec + 0.17 * perp, end_pos],
                                width,
                                12,
                                start_cap,
                                end_cap,
                            );
                    }
                    _ => {
                        *shape =
                            &*shape | &Shape::line(start_pos, end_pos, width, start_cap, end_cap);
                    }
                }
            };

            self.cycles_and_shapes = vec![];

            for cycle in permutation.disjoint_cycles() {
                let mut shape = Shape::empty();
                let n = cycle.len();
                debug_assert!(n >= 2);

                if n == 2 {
                    // Only draw one line for 2-cycles
                    let mut start = *cycle[0];
                    let mut end = *cycle[1];
                    // Consistently pick an ordering
                    if start > end {
                        (start, end) = (end, start);
                    }
                    // Hand-picked cases to make 2-cycles look nicer
                    let diff = (end.0 - start.0, end.1 - start.1);
                    match (start, diff) {
                        ((_, 1), (0, 2))
                        | ((1, _), (2, 0))
                        | ((3, _), (2, 0))
                        | ((1, _), (3, 0)) => {
                            (start, end) = (end, start);
                        }
                        _ => {}
                    }
                    draw_line(
                        &mut shape,
                        start,
                        end,
                        line_width,
                        LineCap::Round(0.1),
                        LineCap::Round(0.1),
                    );
                    shape = &shape
                        | &Shape::regular_polygon(
                            coordinates.cell_to_pos(start),
                            line_width,
                            12,
                            0.0,
                        );
                    shape = &shape
                        | &Shape::regular_polygon(
                            coordinates.cell_to_pos(end),
                            line_width,
                            12,
                            0.0,
                        );
                } else {
                    // Draw n-cycles for n >= 3 as o--o--o->o
                    // Omit the longest line
                    // If there are multiple equally longest lines, pick one to omit in a systematic way
                    let mut lines = vec![];
                    for i in 0..n {
                        let start = *cycle[i];
                        let end = *cycle[(i + 1) % n];
                        debug_assert_ne!(start, end);
                        lines.push((start, end));
                    }
                    let dist_sq = |x: &GridCell, y: &GridCell| -> usize {
                        let d = (x.0.abs_diff(y.0), x.1.abs_diff(y.1));
                        d.0 * d.0 + d.1 * d.1
                    };
                    let max_dist_sq = lines.iter().map(|(x, y)| dist_sq(x, y)).max().unwrap();
                    let chosen_longest_line_idx = lines
                        .iter()
                        .enumerate()
                        .filter(|(_, (x, y))| dist_sq(x, y) == max_dist_sq)
                        .max_by_key(|(_, (x, _))| x)
                        .map(|(i, _)| i)
                        .unwrap();
                    lines.rotate_left(chosen_longest_line_idx + 1);
                    lines.pop().unwrap();

                    // Draw circles everywhere except the end
                    for (i, (start, _)) in lines.iter().enumerate() {
                        shape = &shape
                            | &Shape::regular_polygon(
                                coordinates.cell_to_pos(*start),
                                if i == 0 {
                                    1.0 * line_width
                                } else {
                                    0.8 * line_width
                                },
                                12,
                                0.0,
                            );
                    }

                    // Draw the last line with the arrow head
                    let (start, end) = lines.pop().unwrap();
                    draw_line(
                        &mut shape,
                        start,
                        end,
                        line_width,
                        LineCap::Round(0.1),
                        arrowhead_cap(1.5),
                    );

                    // Draw all the other lines without arrow heads
                    for (start, end) in lines {
                        draw_line(
                            &mut shape,
                            start,
                            end,
                            line_width,
                            LineCap::Round(0.1),
                            LineCap::Round(0.1),
                        );
                    }
                }

                self.cycles_and_shapes
                    .push((cycle.into_iter().cloned().collect(), shape));
            }
        }
    }

    pub fn shapes(&self) -> &Vec<(Vec<GridCell>, Shape)> {
        &self.cycles_and_shapes
    }
}
