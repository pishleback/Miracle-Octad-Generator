use crate::app::ui::{
    grid::{GridCell, GridShower},
    shape::{Shape, arrowhead_cap},
};
use algebraeon::structures::{MetaOrderedFiniteSetSignature, MetaPermutationsSignature};
use algebraeon::{
    combinatorics::golay_codes::extended_binary_golay_code::Point, sets::sets::ConstSizePermutation,
};
use eframe::egui::Vec2;
use i_overlay::mesh::style::LineCap;

pub fn point_to_grid_cell(pt: &Point) -> GridCell {
    (
        pt.col.element_to_enumeration().try_into().unwrap(),
        pt.row.element_to_enumeration().try_into().unwrap(),
    )
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Permutation(ConstSizePermutation<24, Point>),
    Arrows(Vec<(Point, Point)>),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ShapeSource {
    Cycle(Vec<GridCell>),
    Arrow(GridCell, GridCell),
}

#[derive(Debug, Clone)]
pub struct MogPermutationShapeCache {
    state: Option<(State, GridShower)>,
    sources_and_shapes: Vec<(ShapeSource, Shape)>,
    line_width: f32,
    small_radius: f32,
    large_radius: f32,
}

impl Default for MogPermutationShapeCache {
    fn default() -> Self {
        Self {
            state: Default::default(),
            sources_and_shapes: Default::default(),
            line_width: 0.1,
            small_radius: 0.08,
            large_radius: 0.1,
        }
    }
}

impl MogPermutationShapeCache {
    pub fn small_radius(&self) -> f32 {
        self.small_radius
    }
}

impl MogPermutationShapeCache {
    fn clear(&mut self) {
        self.sources_and_shapes = vec![];
    }

    fn set_state(&mut self, new_state: Option<State>, coordinates: GridShower) {
        if new_state.as_ref().map(|state| (state, &coordinates))
            != self.state.as_ref().map(|(x, y)| (x, y))
        {
            self.clear();

            if let Some(new_state) = new_state {
                let line_width = coordinates.cell_scalar_to_pos_scalar(self.line_width) as f64;
                let small_radius = coordinates.cell_scalar_to_pos_scalar(self.small_radius) as f64;
                let large_radius = coordinates.cell_scalar_to_pos_scalar(self.large_radius) as f64;

                let draw_line =
                    |shape: &mut Shape,
                     start: &Point,
                     end: &Point,
                     width: f64,
                     mut start_cap: LineCap<[f64; 2], f64>,
                     mut end_cap: LineCap<[f64; 2], f64>| {
                        let mut start_cell = point_to_grid_cell(&start);
                        let mut end_cell: GridCell = point_to_grid_cell(&end);

                        if start_cell > end_cell {
                            (start_cell, end_cell) = (end_cell, start_cell);
                            (start_cap, end_cap) = (end_cap, start_cap);
                        }
                        let cell_vec = (end_cell.0 - start_cell.0, end_cell.1 - start_cell.1);
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
                                let pos_vec = end_pos - start_pos;
                                let mut perp = Vec2 {
                                    x: pos_vec.y,
                                    y: -pos_vec.x,
                                };
                                // Hand-picked curve directions
                                match (start_cell, cell_vec) {
                                    ((_, 1), (0, 2))
                                    | ((1, _), (2, 0))
                                    | ((3, _), (2, 0))
                                    | ((1, _), (3, 0))
                                    | ((1, _), (4, 0))
                                    | ((_, 1), (2, 2))
                                    | ((_, 3), (2, -2)) => perp = -perp,
                                    _ => {}
                                }

                                *shape = &*shape
                                    | &Shape::bezier(
                                        vec![
                                            start_pos,
                                            start_pos + 0.5 * pos_vec + 0.17 * perp,
                                            end_pos,
                                        ],
                                        width,
                                        12,
                                        start_cap,
                                        end_cap,
                                    );
                            }
                            _ => {
                                *shape = &*shape
                                    | &Shape::line(start_pos, end_pos, width, start_cap, end_cap);
                            }
                        }
                    };

                match &new_state {
                    State::Permutation(permutation) => {
                        for cycle in permutation.disjoint_cycles() {
                            let mut shape = Shape::empty();
                            let n = cycle.len();
                            debug_assert!(n >= 2);
                            if n == 2 {
                                // Only draw one line for 2-cycles
                                let start = &cycle[0];
                                let end = &cycle[1];

                                draw_line(
                                    &mut shape,
                                    &start,
                                    &end,
                                    line_width,
                                    LineCap::Round(0.1),
                                    LineCap::Round(0.1),
                                );
                                shape = &shape
                                    | &Shape::regular_polygon(
                                        coordinates.cell_to_pos(point_to_grid_cell(&start)),
                                        small_radius,
                                        12,
                                        0.0,
                                    );
                                shape = &shape
                                    | &Shape::regular_polygon(
                                        coordinates.cell_to_pos(point_to_grid_cell(&end)),
                                        small_radius,
                                        12,
                                        0.0,
                                    );
                            } else {
                                // Draw n-cycles for n >= 3 as o--o--o->o
                                // Omit the longest line
                                // If there are multiple equally longest lines, pick one to omit in a systematic way
                                let mut lines = vec![];
                                for i in 0..n {
                                    let start = &cycle[i];
                                    let end = &cycle[(i + 1) % n];
                                    debug_assert_ne!(start, end);
                                    lines.push((start, end));
                                }
                                let dist_sq = |x: &GridCell, y: &GridCell| -> usize {
                                    let d = (x.0.abs_diff(y.0), x.1.abs_diff(y.1));
                                    d.0 * d.0 + d.1 * d.1
                                };
                                let max_dist_sq = lines
                                    .iter()
                                    .map(|(x, y)| {
                                        dist_sq(&point_to_grid_cell(x), &point_to_grid_cell(y))
                                    })
                                    .max()
                                    .unwrap();
                                let chosen_longest_line_idx = lines
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, (x, y))| {
                                        dist_sq(&point_to_grid_cell(x), &point_to_grid_cell(y))
                                            == max_dist_sq
                                    })
                                    .max_by_key(|(_, (x, _))| x)
                                    .map(|(i, _)| i)
                                    .unwrap();
                                lines.rotate_left(chosen_longest_line_idx + 1);
                                lines.pop().unwrap();

                                // Draw circles everywhere except the end
                                for (i, (start, _)) in lines.iter().enumerate() {
                                    shape = &shape
                                        | &Shape::regular_polygon(
                                            coordinates.cell_to_pos(point_to_grid_cell(start)),
                                            if i == 0 { large_radius } else { small_radius },
                                            12,
                                            0.0,
                                        );
                                }

                                // Draw the last line with the arrow head
                                let (start, end) = lines.pop().unwrap();
                                draw_line(
                                    &mut shape,
                                    &start,
                                    &end,
                                    line_width,
                                    LineCap::Round(0.1),
                                    arrowhead_cap(1.5),
                                );

                                // Draw all the other lines without arrow heads
                                for (start, end) in lines {
                                    draw_line(
                                        &mut shape,
                                        &start,
                                        &end,
                                        line_width,
                                        LineCap::Round(0.1),
                                        LineCap::Round(0.1),
                                    );
                                }
                            }

                            self.sources_and_shapes.push((
                                ShapeSource::Cycle(
                                    cycle.into_iter().map(|p| point_to_grid_cell(&p)).collect(),
                                ),
                                shape,
                            ));
                        }
                    }
                    State::Arrows(arrows) => {
                        for (start, end) in arrows {
                            let mut shape = Shape::empty();

                            if start == end {
                                shape = &shape
                                    | &Shape::regular_polygon(
                                        coordinates.cell_to_pos(point_to_grid_cell(start)),
                                        large_radius,
                                        12,
                                        0.0,
                                    );
                            } else {
                                draw_line(
                                    &mut shape,
                                    &start,
                                    &end,
                                    line_width,
                                    LineCap::Round(0.1),
                                    arrowhead_cap(1.5),
                                );
                            }

                            self.sources_and_shapes.push((
                                ShapeSource::Arrow(
                                    point_to_grid_cell(&start),
                                    point_to_grid_cell(&end),
                                ),
                                shape,
                            ));
                        }
                    }
                }
                self.state = Some((new_state, coordinates));
            } else {
                self.state = None;
            }
        }
    }

    pub fn set_arrows(&mut self, arrows: &[(Point, Point)], coordinates: GridShower) {
        self.set_state(
            Some(State::Arrows(arrows.iter().cloned().collect())),
            coordinates,
        );
    }

    pub fn set_permutation(
        &mut self,
        permutation: Option<ConstSizePermutation<24, Point>>,
        coordinates: GridShower,
    ) {
        self.set_state(
            permutation.map(|permutation| State::Permutation(permutation)),
            coordinates,
        );
    }

    pub fn sources_and_shapes(&self) -> &Vec<(ShapeSource, Shape)> {
        &self.sources_and_shapes
    }
}
