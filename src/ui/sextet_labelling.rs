use crate::logic::permutation::Permutation;
use crate::logic::traits::{Enumerated, Labelled};
use crate::logic::{hexacode, miracle_octad_generator::*};
use crate::ui::grid::GridCell;
use crate::ui::shape::{Polygon, arrowhead_cap};
use crate::{
    AppState,
    logic::finite_field_4::Point as F4Point,
    ui::mog::{draw_f4, f4_selection, sextet_idx_to_colour},
};
use eframe::egui::{Button, CentralPanel, Color32, SidePanel, Vec2};
use i_overlay::mesh::style::LineCap;
use std::collections::HashSet;

#[derive(Debug)]
enum PartialLabellingState {
    Underset,
    Perfect {
        // 3 labels
        // x != y
        // z can be anything
        x: F4Point,
        y: F4Point,
        z: F4Point,
        // This pair is the pair of foursomes with labels x and x,y
        pair: hexacode::Pair,
        // This is the foursome in the pair with just the label x
        // The other is the foursomes in the pair with the labels x and y
        side: hexacode::Side,
        // This is the other foursome with label z
        third: hexacode::Point,
    },
    Overset,
}

#[derive(Clone)]
pub struct State<PrevState: AppState + Clone + 'static> {
    prev_state: PrevState,
    sextet: Vec<Vector>,
    ordering: Vec<usize>, // A permutation of 0..6
    labelling: Labelled<Point, Option<F4Point>>,
}

impl<PrevState: AppState + Clone> State<PrevState> {
    pub fn from_foursome(prev_state: PrevState, vector: &Vector) -> Self {
        let mog = super::mog::mog();
        let mut sextet = mog
            .complete_sextet(vector)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();
        sextet.sort_unstable();
        sextet.reverse();
        Self {
            prev_state,
            sextet,
            ordering: (0..6).collect(),
            labelling: Labelled::new_constant(None),
        }
    }

    fn get_foursome(&self, foursome: hexacode::Point) -> &Vector {
        &self.sextet[self
            .ordering
            .iter()
            .position(|i| *i == foursome.point_to_usize())
            .unwrap()]
    }

    pub fn ordered_sextet(&self) -> OrderedSextet {
        OrderedSextet::from_foursomes(Labelled::from_fn(|h| self.get_foursome(h).clone()))
    }

    /*
    A partial labelling of the following form can be extended uniquely to a labelling of the MOG

    x x  z -  - -
    - y  - -  - -
    - -  - -  - -
    - -  - -  - -

    Where
     - `x` in the first column can be located anywhere in that column
     - `x` and `y` in the second column can be located anywhere in that column
     - The `x` in the first and second column have the same label
     - `x` and `y` have different labels
     - `z` is any label and can be located anywhere in the last 4 columns

    That's because after applying some automorphisms such a labelling can be made to look like

    0 0  z -  - -
    - 1  - -  - -
    - -  - -  - -
    - -  - -  - -

    which, by standard theory, extends to a unique labelling.
     */
    fn partial_labelling_state(&self) -> PartialLabellingState {
        // let are_adjacent = |foursome_idx_1: usize, foursome_idx_2: usize| {
        //     debug_assert_ne!(foursome_idx_1, foursome_idx_2);
        //     self.ordering
        //         .iter()
        //         .position(|i| *i == foursome_idx_1)
        //         .unwrap()
        //         / 2
        //         == self
        //             .ordering
        //             .iter()
        //             .position(|i| *i == foursome_idx_2)
        //             .unwrap()
        //             / 2
        // };

        let sextet: Labelled<hexacode::Point, Vector> =
            Labelled::from_fn(|h: hexacode::Point| self.get_foursome(h).clone());

        let mut used_labels: Labelled<hexacode::Point, HashSet<F4Point>> =
            Labelled::new_constant(HashSet::new());

        for foursome in hexacode::Point::points() {
            for p in sextet.get(foursome).points() {
                if let Some(label) = *self.labelling.get(p)
                    && !used_labels.get_mut(foursome).insert(label)
                {
                    // No duplicate labels per foursome
                    return PartialLabellingState::Overset;
                }
            }
        }

        if used_labels.iter().any(|(_, labels)| labels.len() >= 3) {
            // No foursomes with >= 3 labels
            return PartialLabellingState::Overset;
        }

        // Which foursomes have labels
        let with_label = used_labels
            .iter()
            .filter_map(|(foursome, labels)| {
                if !labels.is_empty() {
                    Some(foursome)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if let Some(two_labels_foursome) = used_labels
            .iter()
            .find_map(|(h, labels)| if labels.len() == 2 { Some(h) } else { None })
        {
            for foursome in hexacode::Point::points() {
                if foursome != two_labels_foursome && used_labels.get(foursome).len() >= 2 {
                    // At most one foursome with >= 2 labels
                    return PartialLabellingState::Overset;
                }
            }

            if with_label.len() == 2 {
                // There is exactly 1 foursome with 2 labels
                // The rest have 1 or 0 labels
                // So, in this case, there is exactly 1 other label somewhere

                let one_label_foursome = used_labels
                    .iter()
                    .find_map(|(h, labels)| if labels.len() == 1 { Some(h) } else { None })
                    .unwrap();

                if one_label_foursome.pair == two_labels_foursome.pair {
                    let one_label = used_labels.get(one_label_foursome).iter().next().unwrap();
                    if !used_labels
                        .get(two_labels_foursome)
                        .iter()
                        .any(|two_label| one_label == two_label)
                    {
                        // The label in the foursomes with one label must match one of the labels in the foursome with 2 labels
                        return PartialLabellingState::Overset;
                    }
                }
                // Need a fourth label given in one of the other 4 foursomes
            } else if with_label.len() == 3 {
                if let Some(one_label_adjacent_foursome) =
                    used_labels.iter().find_map(|(foursome, labels)| {
                        if labels.len() == 1 && foursome.pair == two_labels_foursome.pair {
                            Some(foursome)
                        } else {
                            None
                        }
                    })
                {
                    let pair = two_labels_foursome.pair;
                    debug_assert_eq!(pair, one_label_adjacent_foursome.pair);

                    let one_label_nonadjacent_foursome = used_labels
                        .iter()
                        .find_map(|(foursome, labels)| {
                            if labels.len() == 1 && foursome != one_label_adjacent_foursome {
                                Some(foursome)
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    let one_label_adjacent = used_labels
                        .get(one_label_adjacent_foursome)
                        .iter()
                        .next()
                        .unwrap();
                    if !used_labels
                        .get(two_labels_foursome)
                        .iter()
                        .any(|two_label| one_label_adjacent == two_label)
                    {
                        // The label in the foursomes with one label must match one of the labels in the foursome with 2 labels
                        return PartialLabellingState::Overset;
                    }

                    let one_label_nonadjacent = used_labels
                        .get(one_label_nonadjacent_foursome)
                        .iter()
                        .next()
                        .unwrap();

                    let x = *one_label_adjacent;
                    let y = *used_labels
                        .get(two_labels_foursome)
                        .iter()
                        .find(|two_label| **two_label != x)
                        .unwrap();
                    let z = *one_label_nonadjacent;
                    return PartialLabellingState::Perfect {
                        x,
                        y,
                        z,
                        pair,
                        side: one_label_adjacent_foursome.side,
                        third: one_label_nonadjacent_foursome,
                    };
                } else {
                    // One of the foursomes with 1 label must be adjacent to the foursome with 2 labels
                    return PartialLabellingState::Overset;
                }
            } else if with_label.len() >= 4 {
                // Too many labels
                return PartialLabellingState::Overset;
            }

            PartialLabellingState::Underset
        } else {
            if with_label.len() >= 4 {
                // At most 3 foursomes with 1 label
                return PartialLabellingState::Overset;
            }

            if with_label.len() == 3 {
                // Two must make up one of the three pairs
                if with_label[0].pair != with_label[1].pair
                    && with_label[0].pair != with_label[2].pair
                    && with_label[1].pair != with_label[2].pair
                {
                    return PartialLabellingState::Overset;
                }
            }

            PartialLabellingState::Underset
        }
    }

    // Given the labels currently set in self.labelling, return a list of allowable labels for each point
    fn allowed_labels(&self) -> Labelled<Point, HashSet<F4Point>> {
        let mut result = Labelled::new_constant(HashSet::new());
        for p in Point::points() {
            for label in [F4Point::Zero, F4Point::One, F4Point::Alpha, F4Point::Beta] {
                let mut modified_self = self.clone();
                modified_self.labelling.set(p, Some(label));
                match modified_self.partial_labelling_state() {
                    PartialLabellingState::Underset | PartialLabellingState::Perfect { .. } => {
                        result.get_mut(p).insert(label);
                    }
                    PartialLabellingState::Overset => {}
                }
            }
        }
        result
    }

    // Given the labels currently set in self.labelling return the completion to a labelling of the MOG if unique
    fn complete_labelling(&self) -> Option<OrderedSextetLabelling> {
        match self.partial_labelling_state() {
            PartialLabellingState::Underset | PartialLabellingState::Overset => None,
            PartialLabellingState::Perfect {
                x,
                y,
                z,
                pair,
                side,
                third,
            } => {
                let empty_pair = hexacode::Pair::points()
                    .find(|p| *p != pair && *p != third.pair)
                    .unwrap();

                let mut ordered_sextet = self.ordered_sextet();

                let h1 = hexacode::Point { side, pair };
                let h23 = hexacode::Point {
                    side: side.flip(),
                    pair,
                };
                let h4 = third;

                let foursome1 = self.get_foursome(h1);
                let foursome23 = self.get_foursome(h23);
                let foursome4 = self.get_foursome(h4);
                let point1 = foursome1
                    .points()
                    .find(|p| *self.labelling.get(*p) == Some(x))
                    .unwrap();
                let point2 = foursome23
                    .points()
                    .find(|p| *self.labelling.get(*p) == Some(x))
                    .unwrap();
                let point3 = foursome23
                    .points()
                    .find(|p| *self.labelling.get(*p) == Some(y))
                    .unwrap();
                let point4 = foursome4
                    .points()
                    .find(|p| *self.labelling.get(*p) == Some(z))
                    .unwrap();

                let mut foursome_perms = vec![];

                // Apply an automorphism such that foursome1 is left and foursome23 is right in their pair
                if side == hexacode::Side::Right {
                    for p in [pair, empty_pair] {
                        foursome_perms.push(Permutation::new_swap(
                            &hexacode::Point {
                                side: hexacode::Side::Left,
                                pair: p,
                            },
                            &hexacode::Point {
                                side: hexacode::Side::Right,
                                pair: p,
                            },
                        ));
                    }
                }

                // Apply an automorphism such that foursome4 is the lefthand foursome in its pair
                if h4.side == hexacode::Side::Right {
                    for p in [h4.pair, empty_pair] {
                        foursome_perms.push(Permutation::new_swap(
                            &hexacode::Point {
                                side: hexacode::Side::Left,
                                pair: p,
                            },
                            &hexacode::Point {
                                side: hexacode::Side::Right,
                                pair: p,
                            },
                        ));
                    }
                }

                // Apply an automorphism such that foursome1 is the first foursome, foursome23 is the second foursome, and foursome4 is the third foursome
                foursome_perms.push(
                    Permutation::from_fn(|h: hexacode::Point| match h.pair {
                        hexacode::Pair::Left => hexacode::Point { side: h.side, pair },
                        hexacode::Pair::Middle => hexacode::Point {
                            side: h.side,
                            pair: h4.pair,
                        },
                        hexacode::Pair::Right => hexacode::Point {
                            side: h.side,
                            pair: empty_pair,
                        },
                    })
                    .inverse(),
                );

                for perm in &foursome_perms {
                    ordered_sextet = ordered_sextet.permute(perm);
                }

                debug_assert_eq!(
                    ordered_sextet.foursome(hexacode::Point {
                        side: hexacode::Side::Left,
                        pair: hexacode::Pair::Left
                    }),
                    foursome1
                );
                debug_assert_eq!(
                    ordered_sextet.foursome(hexacode::Point {
                        side: hexacode::Side::Right,
                        pair: hexacode::Pair::Left
                    }),
                    foursome23
                );
                debug_assert_eq!(
                    ordered_sextet.foursome(hexacode::Point {
                        side: hexacode::Side::Left,
                        pair: hexacode::Pair::Middle
                    }),
                    foursome4
                );

                let mog = crate::ui::mog::mog();
                // This labelling gives point1 and point2 a label of 0, point3 a label of 1, and point4 a label of z/(x+y)
                let mut labelling = mog.complete_labelling(
                    ordered_sextet,
                    point1,
                    point2,
                    point3,
                    point4,
                    z * (x + y).inverse().unwrap(),
                );
                // Apply some more automorphism so that point1 and point2 are labelled x, point3 is labelled y, and point4 is labelled z

                // Multiply by x+y
                labelling = labelling.scalar_mul((x + y).inverse().unwrap()); // .inverse() here because we want to apply the scalar mul to the labels not to the points

                // Add the hexacodeword xx00xx
                labelling =
                    labelling.add_vector(hexacode::Vector::from_fn(|h: hexacode::Point| match h {
                        hexacode::Point {
                            pair: hexacode::Pair::Left | hexacode::Pair::Right,
                            ..
                        } => x,
                        hexacode::Point {
                            pair: hexacode::Pair::Middle,
                            ..
                        } => F4Point::Zero,
                    }));

                debug_assert_eq!(*labelling.labels().get(point1), x);
                debug_assert_eq!(*labelling.labels().get(point2), x);
                debug_assert_eq!(*labelling.labels().get(point3), y);
                debug_assert_eq!(*labelling.labels().get(point4), z);

                // Undo the permutation of the foursomes
                for perm in foursome_perms.into_iter().rev() {
                    labelling = labelling.permute_foursomes(&perm.inverse());
                }

                Some(labelling)
            }
        }
    }
}

impl<PrevState: AppState + Clone> AppState for State<PrevState> {
    fn update(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>> {
        if let Some(new_state) = SidePanel::left("left_panel")
            .exact_width(200.0)
            .show(ctx, |ui| {
                // Reset
                if ui.button("Back").clicked() {
                    return Some(self.prev_state.clone());
                }

                // Reorder the sextets
                ui.heading("Reorder Foursomes");
                egui_dnd::dnd(ui, "foursome_ordering").show_vec(
                    &mut self.ordering,
                    |ui, item, handle, state| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, |ui| {
                                ui.add_enabled(
                                    true,
                                    Button::new(format!("Foursome {:?}", state.index + 1)).fill(
                                        sextet_idx_to_colour(*item).linear_multiply(0.3)
                                            + ui.visuals().panel_fill.linear_multiply(0.7),
                                    ),
                                );
                            });
                        });
                    },
                );

                None
            })
            .inner
        {
            return Some(Box::new(new_state));
        }

        let allowed_labels = self.allowed_labels();
        let completed_labels = self.complete_labelling();
        let mut hovered_point = None;

        struct State<'a> {
            labelling: &'a mut Labelled<Point, Option<F4Point>>,
            allowed_labels: &'a Labelled<Point, HashSet<F4Point>>,
            completed_labels: &'a Option<OrderedSextetLabelling>,
            hovered_point: &'a mut Option<Point>,
        }

        let point_to_cell = |p: Point| -> GridCell {
            let i = p.point_to_usize();
            (i as isize % 6, i as isize / 6)
        };

        let mut grid = super::grid::GridVisuals::<State>::default();

        // The 6x4 MOG grid
        for (foursome_idx, foursome) in self.sextet.iter().enumerate() {
            for p in foursome.points() {
                grid.draw_cell(
                    point_to_cell(p),
                    Box::new(move |ui, response, painter, rect, state| {
                        let colour = sextet_idx_to_colour(foursome_idx);

                        // Draw the coloured box for the point of the MOG
                        painter.rect_filled(
                            rect,
                            10.0,
                            colour.linear_multiply(0.3)
                                + ui.visuals().faint_bg_color.linear_multiply(0.7),
                        );

                        // Check if the mouse is over this point
                        if let Some(pos) = response.hover_pos()
                            && rect.contains(pos)
                        {
                            *state.hovered_point = Some(p);
                        }

                        // Draw a border when dragging to indicate a label can be set here
                        if response.is_pointer_button_down_on()
                            && (!state.allowed_labels.get(p).is_empty()
                                || state.labelling.get(p).is_some())
                        {
                            painter.rect_stroke(
                                rect,
                                10.0,
                                ui.visuals().widgets.hovered.fg_stroke,
                                eframe::egui::StrokeKind::Middle,
                            );
                        }

                        if (response.is_pointer_button_down_on()
                            || response.drag_stopped()
                            || response.clicked())
                            && rect.contains(response.interact_pointer_pos().unwrap())
                        {
                            // Label selection
                            let result = f4_selection(
                                ui,
                                painter,
                                response,
                                rect,
                                state.allowed_labels.get(p).clone(),
                                state.labelling.get(p).is_some(),
                            );
                            if response.drag_stopped() || response.clicked() {
                                match result {
                                    crate::ui::mog::F4SelectionResult::None => {}
                                    crate::ui::mog::F4SelectionResult::Point(label) => {
                                        state.labelling.set(p, Some(label));
                                    }
                                    crate::ui::mog::F4SelectionResult::Cross => {
                                        state.labelling.set(p, None);
                                    }
                                }
                            }
                        } else if let Some(label) = *state.labelling.get(p) {
                            // Draw labels
                            draw_f4(ui, painter, rect, ui.visuals().strong_text_color(), label);
                        } else if let Some(completed_labels) = state.completed_labels {
                            draw_f4(
                                ui,
                                painter,
                                rect,
                                ui.visuals().weak_text_color(),
                                *completed_labels.labels().get(p),
                            );
                        }
                    }),
                );
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, state, coordinates) = grid.show(
                ui,
                State {
                    labelling: &mut self.labelling,
                    allowed_labels: &allowed_labels,
                    completed_labels: &completed_labels,
                    hovered_point: &mut hovered_point,
                },
            );

            if let Some(completed_labels) = &completed_labels {
                let permutation: Permutation<Point> = Permutation::from_fn(|p| Point {
                    col: *completed_labels.foursomes().get(p),
                    row: *completed_labels.labels().get(p),
                });

                let line_width = coordinates.cell_scalar_to_pos_scalar(0.1) as f64;

                let draw_line = |shape: &mut Polygon,
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
                                | &Polygon::bezier(
                                    vec![start_pos, start_pos + 0.5 * vec + 0.17 * perp, end_pos],
                                    width,
                                    20,
                                    start_cap,
                                    end_cap,
                                );
                        }
                        _ => {
                            *shape = &*shape
                                | &Polygon::line(start_pos, end_pos, width, start_cap, end_cap);
                        }
                    }
                };

                for cycle in permutation.disjoint_cycles() {
                    let mut shape = Polygon::empty();
                    let n = cycle.len();
                    debug_assert!(n >= 2);

                    if n == 2 {
                        // Only draw one line for 2-cycles
                        let p = cycle[0];
                        let q = cycle[1];
                        let mut start = point_to_cell(*p);
                        let mut end = point_to_cell(*q);
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
                            | &Polygon::regular_polygon(
                                coordinates.cell_to_pos(start),
                                line_width,
                                36,
                                0.0,
                            );
                        shape = &shape
                            | &Polygon::regular_polygon(
                                coordinates.cell_to_pos(end),
                                line_width,
                                36,
                                0.0,
                            );
                    } else {
                        // Draw n-cycles for n >= 3 as o--o--o->o
                        // Omit the longest line
                        // If there are multiple equally longest lines, pick one to omit in a systematic way
                        let mut lines = vec![];
                        for i in 0..n {
                            let p = cycle[i];
                            let q = cycle[(i + 1) % n];
                            debug_assert_ne!(p, q);
                            let start_cell = point_to_cell(*p);
                            let end_cell = point_to_cell(*q);
                            lines.push((start_cell, end_cell));
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
                                | &Polygon::regular_polygon(
                                    coordinates.cell_to_pos(*start),
                                    if i == 0 {
                                        1.0 * line_width
                                    } else {
                                        0.8 * line_width
                                    },
                                    36,
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

                    let colour = if let Some(p) = state.hovered_point
                        && cycle.contains(&&*p)
                    {
                        Color32::BLACK
                    } else {
                        Color32::BLACK * Color32::from_white_alpha(96)
                    };
                    painter.add(shape.to_egui_mesh(colour));
                }
            }
        });

        None
    }
}
