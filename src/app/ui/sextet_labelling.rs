use crate::app::ui::mog_arrow_shapes::{MogPermutationShapeCache, ShapeSource, point_to_grid_cell};
use crate::app::{
    AppState,
    ui::mog::{draw_f4, f4_selection, sextet_idx_to_colour},
};
use algebraeon::combinatorics::golay_codes::extended_binary_golay_code::{
    EbgcPointPermutation, LabelledPoints, OrderedSextet, OrderedSextetLabelling, Point, Vector,
    complete_sextet, complete_sextet_labelling,
};
use algebraeon::combinatorics::golay_codes::hexacode::{self, HexacodeVector};
use algebraeon::combinatorics::golay_codes::ordered_syntheme::{
    OrderedSynthemePair, OrderedSynthemePoint, OrderedSynthemeSide,
};
use algebraeon::rings::finite_fields::quaternary_field::QuaternaryField as F4;
use algebraeon::rings::structure::MetaTryReciprocalSignature;
use algebraeon::sets::sets::{ConstSizePermutation, Function};
use algebraeon::structures::MetaPermutationsSignature;
use algebraeon::structures::{MetaCompositionSignature, MetaGroupSignature};
use algebraeon::structures::{MetaCountableSetSignature, MetaFiniteSetPermutationsSignature};
use algebraeon::structures::{MetaIdentitySignature, MetaOrderedFiniteSetSignature};
use eframe::egui::{Button, CentralPanel, Color32, SidePanel};
use std::collections::HashSet;

#[derive(Debug)]
enum PartialLabellingState {
    Underset,
    Perfect {
        // 3 labels
        // x != y
        // z can be anything
        x: F4,
        y: F4,
        z: F4,
        // This pair is the pair of foursomes with labels x and x,y
        pair: OrderedSynthemePair,
        // This is the foursome in the pair with just the label x
        // The other is the foursomes in the pair with the labels x and y
        side: OrderedSynthemeSide,
        // This is the other foursome with label z
        third: OrderedSynthemePoint,
    },
    Overset,
}

mod foursome_index {
    use std::sync::atomic::{AtomicUsize, Ordering};

    // We cant sort a Vec<usize> for sorting foursomes when there are multiple foursome orderings on the page, because egui_dnd needs every item to be unique, even over different lists
    // So, wrap the foursome idx in this struct so it can be make unique
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FoursomeIndex {
        index: usize,
        unique_id: usize,
    }

    impl FoursomeIndex {
        pub fn new(index: usize) -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let unique_id = COUNTER.fetch_add(1, Ordering::Relaxed);
            Self { index, unique_id }
        }

        pub fn index(&self) -> usize {
            self.index
        }
    }
}
use foursome_index::*;

#[derive(Clone, PartialEq, Eq)]
struct SextetStabilizer {
    foursome_permutation: Vec<FoursomeIndex>,
    inner_permutations: Vec<ConstSizePermutation<4, F4>>,
}

impl Default for SextetStabilizer {
    fn default() -> Self {
        Self {
            foursome_permutation: (0..6).map(FoursomeIndex::new).collect(),
            inner_permutations: (0..6).map(|_| ConstSizePermutation::identity()).collect(),
        }
    }
}

impl SextetStabilizer {
    pub fn standard_ordered_sextet_permutation(&self) -> ConstSizePermutation<24, Point> {
        ConstSizePermutation::new_fn(|point: &Point| {
            let col_idx: usize = point.col.element_to_enumeration().try_into().unwrap();
            Point {
                col: OrderedSynthemePoint::enumeration_to_element(
                    &self.foursome_permutation[col_idx].index().into(),
                )
                .unwrap(),
                row: self.inner_permutations[col_idx].image(&point.row),
            }
        })
        .unwrap()
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
enum PermutationType {
    #[default]
    None,
    StandardToLabellingAut,
    LabellingToStandardAut,
    SextetStabilizer,
}

#[derive(Clone)]
pub struct State<PrevState: AppState + Clone + 'static> {
    prev_state: PrevState,
    sextet: [Vector; 6],
    ordering: Vec<FoursomeIndex>, // A permutation of 0..6
    labelling: LabelledPoints<Option<F4>>,
    permutation_shapes: MogPermutationShapeCache,
    selected_permutation_type: PermutationType,
    sextet_stabilizer_permutation: SextetStabilizer,
}

impl<PrevState: AppState + Clone> State<PrevState> {
    pub fn from_foursome(prev_state: PrevState, vector: &Vector) -> Self {
        let mut sextet = complete_sextet(vector.clone()).foursomes();
        sextet.sort_unstable();
        Self {
            prev_state,
            sextet,
            ordering: (0..6).map(FoursomeIndex::new).collect(),
            labelling: LabelledPoints::new_constant(&None),
            permutation_shapes: MogPermutationShapeCache::default(),
            selected_permutation_type: PermutationType::default(),
            sextet_stabilizer_permutation: SextetStabilizer::default(),
        }
    }

    fn get_foursome(&self, foursome: OrderedSynthemePoint) -> &Vector {
        let i: usize = foursome.element_to_enumeration().try_into().unwrap();
        &self.sextet[self.ordering[i].index()]
    }

    pub fn ordered_sextet(&self) -> OrderedSextet {
        OrderedSextet::from_foursomes(Function::new(|h| self.get_foursome(h).clone()))
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
        let sextet: hexacode::LabelledPoints<Vector> =
            hexacode::LabelledPoints::new(|h: OrderedSynthemePoint| self.get_foursome(h).clone());

        let mut used_labels: hexacode::LabelledPoints<HashSet<F4>> =
            hexacode::LabelledPoints::new_constant(&HashSet::new());

        for foursome in OrderedSynthemePoint::generate_all_elements() {
            for p in sextet.get(&foursome).points() {
                if let Some(label) = *self.labelling.get(&p)
                    && !used_labels.get_mut(&foursome).insert(label)
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
            for foursome in OrderedSynthemePoint::generate_all_elements() {
                if foursome != two_labels_foursome && used_labels.get(&foursome).len() >= 2 {
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
                    let one_label = used_labels.get(&one_label_foursome).iter().next().unwrap();
                    if !used_labels
                        .get(&two_labels_foursome)
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
                        .get(&one_label_adjacent_foursome)
                        .iter()
                        .next()
                        .unwrap();
                    if !used_labels
                        .get(&two_labels_foursome)
                        .iter()
                        .any(|two_label| one_label_adjacent == two_label)
                    {
                        // The label in the foursomes with one label must match one of the labels in the foursome with 2 labels
                        return PartialLabellingState::Overset;
                    }

                    let one_label_nonadjacent = used_labels
                        .get(&one_label_nonadjacent_foursome)
                        .iter()
                        .next()
                        .unwrap();

                    let x = *one_label_adjacent;
                    let y = *used_labels
                        .get(&two_labels_foursome)
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
    fn allowed_labels(&self) -> LabelledPoints<HashSet<F4>> {
        let mut result = LabelledPoints::new_constant(&HashSet::new());
        for p in Point::generate_all_elements() {
            for label in [F4::Zero, F4::One, F4::Alpha, F4::Beta] {
                let mut modified_self = self.clone();
                modified_self.labelling.set(&p, Some(label));
                match modified_self.partial_labelling_state() {
                    PartialLabellingState::Underset | PartialLabellingState::Perfect { .. } => {
                        result.get_mut(&p).insert(label);
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
                let empty_pair = OrderedSynthemePair::generate_all_elements()
                    .find(|p| *p != pair && *p != third.pair)
                    .unwrap();

                let mut ordered_sextet = self.ordered_sextet();

                let h1 = OrderedSynthemePoint { side, pair };
                let h23 = OrderedSynthemePoint {
                    side: side.flip(),
                    pair,
                };
                let h4 = third;

                let foursome1 = self.get_foursome(h1);
                let foursome23 = self.get_foursome(h23);
                let foursome4 = self.get_foursome(h4);
                let point1 = foursome1
                    .points()
                    .find(|p| *self.labelling.get(p) == Some(x))
                    .unwrap();
                let point2 = foursome23
                    .points()
                    .find(|p| *self.labelling.get(p) == Some(x))
                    .unwrap();
                let point3 = foursome23
                    .points()
                    .find(|p| *self.labelling.get(p) == Some(y))
                    .unwrap();
                let point4 = foursome4
                    .points()
                    .find(|p| *self.labelling.get(p) == Some(z))
                    .unwrap();

                let mut foursome_perms = vec![];

                // Apply an automorphism such that foursome1 is left and foursome23 is right in their pair
                if side == OrderedSynthemeSide::Right {
                    for p in [pair, empty_pair] {
                        foursome_perms.push(
                            ConstSizePermutation::new_swap(
                                OrderedSynthemePoint {
                                    side: OrderedSynthemeSide::Left,
                                    pair: p,
                                },
                                OrderedSynthemePoint {
                                    side: OrderedSynthemeSide::Right,
                                    pair: p,
                                },
                            )
                            .unwrap(),
                        );
                    }
                }

                // Apply an automorphism such that foursome4 is the lefthand foursome in its pair
                if h4.side == OrderedSynthemeSide::Right {
                    for p in [h4.pair, empty_pair] {
                        foursome_perms.push(
                            ConstSizePermutation::new_swap(
                                OrderedSynthemePoint {
                                    side: OrderedSynthemeSide::Left,
                                    pair: p,
                                },
                                OrderedSynthemePoint {
                                    side: OrderedSynthemeSide::Right,
                                    pair: p,
                                },
                            )
                            .unwrap(),
                        );
                    }
                }

                // Apply an automorphism such that foursome1 is the first foursome, foursome23 is the second foursome, and foursome4 is the third foursome
                foursome_perms.push(
                    ConstSizePermutation::new_fn(|h: &OrderedSynthemePoint| match h.pair {
                        OrderedSynthemePair::Left => OrderedSynthemePoint { side: h.side, pair },
                        OrderedSynthemePair::Middle => OrderedSynthemePoint {
                            side: h.side,
                            pair: h4.pair,
                        },
                        OrderedSynthemePair::Right => OrderedSynthemePoint {
                            side: h.side,
                            pair: empty_pair,
                        },
                    })
                    .unwrap()
                    .inverse(),
                );

                for perm in &foursome_perms {
                    ordered_sextet = ordered_sextet.permute(perm);
                }

                debug_assert_eq!(
                    ordered_sextet.foursomes().get(&OrderedSynthemePoint {
                        side: OrderedSynthemeSide::Left,
                        pair: OrderedSynthemePair::Left
                    }),
                    foursome1
                );
                debug_assert_eq!(
                    ordered_sextet.foursomes().get(&OrderedSynthemePoint {
                        side: OrderedSynthemeSide::Right,
                        pair: OrderedSynthemePair::Left
                    }),
                    foursome23
                );
                debug_assert_eq!(
                    ordered_sextet.foursomes().get(&OrderedSynthemePoint {
                        side: OrderedSynthemeSide::Left,
                        pair: OrderedSynthemePair::Middle
                    }),
                    foursome4
                );

                // This labelling gives point1 and point2 a label of 0, point3 a label of 1, and point4 a label of z/(x+y)
                let mut labelling = complete_sextet_labelling(
                    &ordered_sextet,
                    &point1,
                    &point2,
                    &point3,
                    &point4,
                    z * (x + y).try_reciprocal().unwrap(),
                );
                // Apply some more automorphism so that point1 and point2 are labelled x, point3 is labelled y, and point4 is labelled z

                // Multiply by x+y
                labelling = labelling.scalar_mul((x + y).try_reciprocal().unwrap()); // .inverse() here because we want to apply the scalar mul to the labels not to the points

                // Add the hexacodeword xx00xx
                labelling =
                    labelling.add_vector(&HexacodeVector::new(|h: OrderedSynthemePoint| match h {
                        OrderedSynthemePoint {
                            pair: OrderedSynthemePair::Left | OrderedSynthemePair::Right,
                            ..
                        } => x,
                        OrderedSynthemePoint {
                            pair: OrderedSynthemePair::Middle,
                            ..
                        } => F4::Zero,
                    }));

                debug_assert_eq!(*labelling.labels().get(&point1), x);
                debug_assert_eq!(*labelling.labels().get(&point2), x);
                debug_assert_eq!(*labelling.labels().get(&point3), y);
                debug_assert_eq!(*labelling.labels().get(&point4), z);

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
        let allowed_labels = self.allowed_labels();
        let completed_labels = self.complete_labelling();
        let mut hovered_point = None;

        let permutation = if let Some(completed_labels) = &completed_labels {
            let standard_labelling_to_completed_labelling =
                ConstSizePermutation::new_fn(|p| Point {
                    col: *completed_labels.point_foursomes().get(p),
                    row: *completed_labels.labels().get(p),
                })
                .unwrap();

            match self.selected_permutation_type {
                PermutationType::None => None,
                PermutationType::StandardToLabellingAut => {
                    Some(standard_labelling_to_completed_labelling)
                }
                PermutationType::LabellingToStandardAut => {
                    Some(standard_labelling_to_completed_labelling.inverse())
                }
                PermutationType::SextetStabilizer => Some(
                    (&standard_labelling_to_completed_labelling.compose(
                        &self
                            .sextet_stabilizer_permutation
                            .standard_ordered_sextet_permutation(),
                    ))
                        .compose(&standard_labelling_to_completed_labelling.inverse()),
                ),
            }
        } else {
            None
        };

        if let Some(new_state) = SidePanel::left("left_panel")
            .min_width(200.0)
            .show(ctx, |ui| {
                // Back
                if ui.button("Back").clicked() {
                    return Some(Box::<dyn AppState>::from(Box::new(self.prev_state.clone())));
                }

                ui.heading("Labelling Editor");

                // Reorder the sextets
                ui.label("Reorder Foursomes");
                egui_dnd::dnd(ui, "foursome_ordering").show_vec(
                    &mut self.ordering,
                    |ui, item: &mut FoursomeIndex, handle, state| {
                        handle.ui(ui, |ui| {
                            ui.add_enabled(
                                true,
                                Button::new(format!("Foursome {}", state.index + 1)).fill(
                                    sextet_idx_to_colour(item.index())
                                        .lerp_to_gamma(ui.visuals().panel_fill, 0.6),
                                ),
                            );
                        });
                        if state.index == 1 || state.index == 3 {
                            ui.add_space(4.0);
                        }
                    },
                );

                if completed_labels.is_none() {
                    ui.label(
                        "Select labels until there is a unique completion to a full labelling.",
                    );
                }

                // Permutations
                if completed_labels.is_some() {
                    ui.heading("Permutation");
                    ui.label(
                        "\
Construct permutations induced by this labelling",
                    );
                    ui.radio_value(
                        &mut self.selected_permutation_type,
                        PermutationType::None,
                        "None",
                    )
                    .on_hover_text("Don't show a permutation");
                    ui.radio_value(
                        &mut self.selected_permutation_type,
                        PermutationType::StandardToLabellingAut,
                        "To Standard Labelling",
                    )
                    .on_hover_text(
                        "\
The permutation taking the ordered foursomes to the ordered columns \
and the labelling labels to the row labels",
                    );
                    ui.radio_value(
                        &mut self.selected_permutation_type,
                        PermutationType::LabellingToStandardAut,
                        "From Standard Labelling",
                    )
                    .on_hover_text(
                        "\
The permutation taking ordered columns to the ordered foursomes \
and the row labels to the labelling labels",
                    );
                    ui.radio_value(
                        &mut self.selected_permutation_type,
                        PermutationType::SextetStabilizer,
                        "Sextet Stabilizer",
                    )
                    .on_hover_text(
                        "\
Configure permutations which preserve the unordered sextet",
                    );

                    if self.selected_permutation_type == PermutationType::SextetStabilizer {
                        ui.heading("Sextet Stabilizer Configuration");

                        ui.horizontal(|ui| {
                            if ui.button("Reset").clicked() {
                                self.sextet_stabilizer_permutation = SextetStabilizer::default();
                            }

                            if ui.button("×ω").clicked() {
                                for foursome_perm in
                                    &mut self.sextet_stabilizer_permutation.inner_permutations
                                {
                                    *foursome_perm = foursome_perm.compose(
                                        &ConstSizePermutation::new_cycle(vec![
                                            F4::One,
                                            F4::Alpha,
                                            F4::Beta,
                                        ])
                                        .unwrap(),
                                    );
                                }
                            }
                            if ui.button("Conjugate").clicked() {
                                for foursome_perm in
                                    &mut self.sextet_stabilizer_permutation.inner_permutations
                                {
                                    *foursome_perm = foursome_perm.compose(
                                        &ConstSizePermutation::new_swap(F4::Alpha, F4::Beta)
                                            .unwrap(),
                                    );
                                }
                            }
                        });

                        egui_dnd::dnd(ui, "foursome_permutation").show_vec(
                            &mut self.sextet_stabilizer_permutation.foursome_permutation,
                            |ui, item, handle, state| {
                                ui.horizontal(|ui| {
                                    handle.ui(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.add_enabled(
                                                true,
                                                Button::new(format!(
                                                    "Foursome {}",
                                                    self.ordering
                                                        .iter()
                                                        .find(|index| {
                                                            item.index() == index.index()
                                                        })
                                                        .unwrap()
                                                        .index()
                                                        + 1
                                                ))
                                                .fill(
                                                    sextet_idx_to_colour(
                                                        self.ordering[item.index()].index(),
                                                    )
                                                    .lerp_to_gamma(ui.visuals().panel_fill, 0.6),
                                                ),
                                            );
                                        });
                                    });

                                    let foursome_perm = &mut self
                                        .sextet_stabilizer_permutation
                                        .inner_permutations[item.index()];

                                    if ui.button("+1").clicked() {
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_swap(F4::Zero, F4::One)
                                                .unwrap(),
                                        );
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_swap(F4::Alpha, F4::Beta)
                                                .unwrap(),
                                        );
                                    }
                                    if ui.button("+ω").clicked() {
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_swap(F4::Zero, F4::Alpha)
                                                .unwrap(),
                                        );
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_swap(F4::One, F4::Beta)
                                                .unwrap(),
                                        );
                                    }
                                    if ui.button("×ω").clicked() {
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_cycle(vec![
                                                F4::One,
                                                F4::Alpha,
                                                F4::Beta,
                                            ])
                                            .unwrap(),
                                        );
                                    }
                                    if ui.button("Conjugate").clicked() {
                                        *foursome_perm = foursome_perm.compose(
                                            &ConstSizePermutation::new_swap(F4::Alpha, F4::Beta)
                                                .unwrap(),
                                        );
                                    }
                                });
                                if state.index == 1 || state.index == 3 {
                                    ui.add_space(4.0);
                                }
                            },
                        );
                    }

                    if let Some(permutation) = permutation.as_ref()
                        && let Some(new_state) = ui
                            .horizontal(|ui| {
                                let mut is_aut = permutation.is_ebgc_automorphism();
                                let text = if is_aut {
                                    "This permutation is an automorphism"
                                } else {
                                    "This permutation is not an automorphism"
                                };
                                ui.checkbox(&mut is_aut, "Automorphism").on_hover_text(text);

                                if ui.button("Select").clicked() {
                                    return Some(Box::<dyn AppState>::from(Box::new(
                                        crate::app::ui::point_toggle::State::new(
                                            Vector::zero(),
                                            permutation.clone(),
                                        ),
                                    )));
                                }
                                None
                            })
                            .inner
                    {
                        return Some(new_state);
                    };
                }

                None
            })
            .inner
        {
            return Some(new_state);
        }

        let mut grid_builder = super::grid::GridBuilder::default();

        // The 6x4 MOG grid
        for foursome in &self.sextet {
            for p in foursome.points() {
                grid_builder.include_cell(point_to_grid_cell(&p));
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            let (response, painter, grid) = grid_builder.show(ui);

            // The 6x4 MOG grid
            for (foursome_idx, foursome) in self.sextet.iter().enumerate() {
                for p in foursome.points() {
                    let rect = grid.cell_to_rect(point_to_grid_cell(&p));

                    let colour = sextet_idx_to_colour(foursome_idx);

                    // Draw the coloured box for the point of the MOG
                    painter.rect_filled(
                        rect,
                        grid.cell_scalar_to_pos_scalar(0.05),
                        colour.lerp_to_gamma(ui.visuals().faint_bg_color, 0.6),
                    );

                    // Check if the mouse is over this point
                    if let Some(pos) = response.hover_pos()
                        && rect.contains(pos)
                    {
                        hovered_point = Some(p.clone());
                    }

                    // Draw a border when dragging to indicate a label can be set here
                    if response.is_pointer_button_down_on()
                        && (!allowed_labels.get(&p).is_empty() || self.labelling.get(&p).is_some())
                    {
                        painter.rect_stroke(
                            rect,
                            grid.cell_scalar_to_pos_scalar(0.05),
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
                            &painter,
                            &response,
                            rect,
                            allowed_labels.get(&p).clone(),
                            self.labelling.get(&p).is_some(),
                        );
                        if response.drag_stopped() || response.clicked() {
                            match result {
                                crate::app::ui::mog::F4SelectionResult::None => {}
                                crate::app::ui::mog::F4SelectionResult::Point(label) => {
                                    self.labelling.set(&p, Some(label));
                                }
                                crate::app::ui::mog::F4SelectionResult::Cross => {
                                    self.labelling.set(&p, None);
                                }
                            }
                        }
                    } else if let Some(label) = *self.labelling.get(&p) {
                        // Draw labels
                        draw_f4(ui, &painter, rect, ui.visuals().strong_text_color(), label);
                    } else if let Some(completed_labels) = completed_labels.clone() {
                        draw_f4(
                            ui,
                            &painter,
                            rect,
                            ui.visuals().text_color(),
                            *completed_labels.labels().get(&p),
                        );
                    }
                }
            }

            // Draw the selected permutation
            self.permutation_shapes.set_permutation(permutation, grid);

            let colour = ui.visuals().strong_text_color();

            for (source, shape) in self.permutation_shapes.sources_and_shapes() {
                let colour = if let Some(p) = &hovered_point
                    && let ShapeSource::Cycle(cycle) = source
                    && cycle.contains(&point_to_grid_cell(p))
                {
                    colour
                } else {
                    colour * Color32::from_white_alpha(96)
                };

                painter.add(shape.to_egui_mesh(colour));
            }
        });

        None
    }
}
