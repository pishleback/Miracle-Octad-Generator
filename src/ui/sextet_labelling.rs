use std::{collections::HashSet, hash::Hash};

use crate::{
    AppState,
    logic::{
        finite_field_4::Point as F4Point,
        miracle_octad_generator::{Point, Vector},
    },
    ui::mog::{LabelledMOGPoints, draw_f4, f4_selection, sextet_idx_to_colour},
};
use eframe::egui::{Button, CentralPanel, Rect, SidePanel};

// #[derive(Debug)]
// struct FoursomeLabel {
//     foursome_idx: usize,
//     point_idx: usize,
//     label: F4Point,
// }

// #[derive(Debug)]
// enum PartialLabellingState {
//     Zero,
//     One {
//         label: FoursomeLabel,
//     },
//     TwoDifferent {
//         label_1: FoursomeLabel,
//         label_2: FoursomeLabel,
//         // Are the two foursomes in one of the three adjacent pairs?
//         adjacent: bool,
//     },
//     TwoSame {
//         // Two labels
//         label_1: FoursomeLabel,
//         label_2: FoursomeLabel,
//     },
//     ThreeDifferent {
//         // These two foursomes are one of the three adjacent pairs
//         label_1: FoursomeLabel,
//         label_2: FoursomeLabel,
//         // This is one of the other 4 foursomes
//         label_3: FoursomeLabel,
//     },
//     ThreeSame {
//         // These two labels are on the same foursome
//         label_1: FoursomeLabel,
//         label_2: FoursomeLabel,
//         // This is one of the other 5 foursomes
//         label_3: FoursomeLabel,
//         // Are the two foursomes in one of the three adjacent pairs?
//         adjacent: bool,
//     },
//     Four {
//         // This foursome has exactly 1 label set
//         label_1: FoursomeLabel,
//         // These labels are on the same foursome and are adjacent to the first foursome
//         label_2: FoursomeLabel,
//         label_3: FoursomeLabel,
//         // Any of the other 4 foursomes
//         label_4: FoursomeLabel,
//     },
//     // The labels are over constrained
//     Invalid,
// }

// impl PartialLabellingState {
//     fn add_label(self, new_label: FoursomeLabel, ordering: &Vec<usize>) -> Self {
//         let are_adjacent = |foursome_idx_1: usize, foursome_idx_2: usize| {
//             debug_assert_ne!(foursome_idx_1, foursome_idx_2);
//             ordering.iter().position(|i| *i == foursome_idx_1).unwrap() / 2
//                 == ordering.iter().position(|i| *i == foursome_idx_2).unwrap() / 2
//         };

//         match self {
//             PartialLabellingState::Zero => Self::One { label: new_label },
//             PartialLabellingState::One { label } => {
//                 debug_assert_ne!(label.point_idx, new_label.point_idx);
//                 if label.foursome_idx == new_label.foursome_idx {
//                     Self::TwoSame {
//                         label_1: label,
//                         label_2: new_label,
//                     }
//                 } else {
//                     let adjacent = are_adjacent(label.foursome_idx, new_label.foursome_idx);
//                     Self::TwoDifferent {
//                         label_1: label,
//                         label_2: new_label,
//                         adjacent,
//                     }
//                 }
//             }
//             PartialLabellingState::TwoDifferent {
//                 label_1,
//                 label_2,
//                 adjacent,
//             } => {
//                 debug_assert_ne!(label_1.foursome_idx, label_2.foursome_idx);
//                 debug_assert_ne!(label_1.point_idx, new_label.point_idx);
//                 debug_assert_ne!(label_2.point_idx, new_label.point_idx);
//                 if label_1.foursome_idx == new_label.foursome_idx {
//                     #[allow(clippy::redundant_field_names)]
//                     Self::ThreeSame {
//                         label_1: label_1,
//                         label_2: new_label,
//                         label_3: label_2,
//                         adjacent,
//                     }
//                 } else if label_2.foursome_idx == new_label.foursome_idx {
//                     Self::ThreeSame {
//                         label_1: label_2,
//                         label_2: new_label,
//                         label_3: label_1,
//                         adjacent,
//                     }
//                 } else if adjacent {
//                     Self::ThreeDifferent {
//                         label_1,
//                         label_2,
//                         label_3: new_label,
//                     }
//                 } else if are_adjacent(label_1.foursome_idx, new_label.foursome_idx) {
//                     #[allow(clippy::redundant_field_names)]
//                     Self::ThreeDifferent {
//                         label_1: label_1,
//                         label_2: new_label,
//                         label_3: label_2,
//                     }
//                 } else if are_adjacent(label_2.foursome_idx, new_label.foursome_idx) {
//                     Self::ThreeDifferent {
//                         label_1: label_2,
//                         label_2: new_label,
//                         label_3: label_1,
//                     }
//                 } else {
//                     // The 3 labels all belong to different ones of the 3 pairs
//                     Self::Invalid
//                 }
//             }
//             PartialLabellingState::TwoSame { label_1, label_2 } => {
//                 let foursome_idx = label_1.foursome_idx;
//                 debug_assert_eq!(foursome_idx, label_2.foursome_idx);
//                 if new_label.foursome_idx == foursome_idx {
//                     Self::Invalid
//                 } else {
//                     let adjacent = are_adjacent(new_label.foursome_idx, foursome_idx);
//                     Self::ThreeSame {
//                         label_1,
//                         label_2,
//                         label_3: new_label,
//                         adjacent,
//                     }
//                 }
//             }
//             PartialLabellingState::ThreeDifferent {
//                 label_1,
//                 label_2,
//                 label_3,
//             } => {
//                 if new_label.foursome_idx == label_2.foursome_idx {
//                     #[allow(clippy::redundant_field_names)]
//                     Self::Four {
//                         label_1: label_1,
//                         label_2: label_2,
//                         label_3: new_label,
//                         label_4: label_3,
//                     }
//                 } else if new_label.foursome_idx == label_1.foursome_idx {
//                     Self::Four {
//                         label_1: label_2,
//                         label_2: label_1,
//                         label_3: new_label,
//                         label_4: label_3,
//                     }
//                 } else {
//                     Self::Invalid
//                 }
//             }
//             PartialLabellingState::ThreeSame {
//                 label_1,
//                 label_2,
//                 label_3,
//                 adjacent,
//             } => {
//                 let foursome_idx_12 = label_1.foursome_idx;
//                 debug_assert_eq!(foursome_idx_12, label_2.foursome_idx);

//                 if new_label.foursome_idx == foursome_idx_12
//                     || new_label.foursome_idx == label_3.foursome_idx
//                 {
//                     Self::Invalid
//                 } else if adjacent {
//                     Self::Four {
//                         label_1,
//                         label_2,
//                         label_3,
//                         label_4: new_label,
//                     }
//                 } else if are_adjacent(foursome_idx_12, new_label.foursome_idx) {
//                     Self::Four {
//                         label_1,
//                         label_2,
//                         label_3: new_label,
//                         label_4: label_3,
//                     }
//                 } else {
//                     Self::Invalid
//                 }
//             }
//             PartialLabellingState::Four {
//                 label_1,
//                 label_2,
//                 label_3,
//                 label_4,
//             } => Self::Invalid,
//             PartialLabellingState::Invalid => PartialLabellingState::Invalid,
//         }
//     }
// }

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
        // First foursome
        //  - Has one point labelled x
        foursome1: usize,
        // Second foursome
        //  - Must form one of the 3 pairs with the first foursome
        //  - Has one point labelled x and another labelled y
        foursome23: usize,
        // Third foursome
        //  - Must be different from the first 2
        //  - Has one point labelled z
        foursome4: usize,
    },
    Overset,
}

#[derive(Clone)]
pub struct State<PrevState: AppState + Clone + 'static> {
    prev_state: PrevState,
    sextet: [Vector; 6],
    ordering: Vec<usize>, // A permutation of 0..6
    labelling: LabelledMOGPoints<Option<F4Point>>,
}

impl<PrevState: AppState + Clone> State<PrevState> {
    pub fn from_foursome(prev_state: PrevState, vector: Vector) -> Self {
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
            sextet: std::array::from_fn(|i| sextet[i].clone()),
            ordering: (0..6).collect(),
            labelling: LabelledMOGPoints::default(),
        }
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
        let are_adjacent = |foursome_idx_1: usize, foursome_idx_2: usize| {
            debug_assert_ne!(foursome_idx_1, foursome_idx_2);
            self.ordering
                .iter()
                .position(|i| *i == foursome_idx_1)
                .unwrap()
                / 2
                == self
                    .ordering
                    .iter()
                    .position(|i| *i == foursome_idx_2)
                    .unwrap()
                    / 2
        };

        let mut used_labels: [HashSet<F4Point>; 6] = Default::default();
        for foursome_idx in 0..6 {
            for point_idx in self.sextet[foursome_idx]
                .points()
                .map(Vector::point_to_usize)
            {
                if let Some(label) = *self.labelling.get(point_idx)
                    && !used_labels[foursome_idx].insert(label)
                {
                    // No duplicate labels per foursome
                    return PartialLabellingState::Overset;
                }
            }
        }

        if used_labels.iter().any(|labels| labels.len() >= 3) {
            // No foursomes with >= 3 labels
            return PartialLabellingState::Overset;
        }

        let with_label = used_labels
            .iter()
            .enumerate()
            .filter_map(|(foursome_idx, labels)| {
                if !labels.is_empty() {
                    Some(foursome_idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if let Some(two_labels_foursome_idx) =
            used_labels.iter().position(|labels| labels.len() == 2)
        {
            for foursome_idx in 0..6 {
                if foursome_idx != two_labels_foursome_idx && used_labels[foursome_idx].len() >= 2 {
                    // At most one foursome with >= 2 labels
                    return PartialLabellingState::Overset;
                }
            }

            if with_label.len() == 2 {
                // There is exactly 1 foursome with 2 labels
                // The rest have 1 or 0 labels
                // So, in this case, there is exactly 1 other label somewhere

                let one_label_foursome_idx = used_labels
                    .iter()
                    .position(|labels| labels.len() == 1)
                    .unwrap();

                if are_adjacent(one_label_foursome_idx, two_labels_foursome_idx) {
                    let one_label = used_labels[one_label_foursome_idx].iter().next().unwrap();
                    if !used_labels[two_labels_foursome_idx]
                        .iter()
                        .any(|two_label| one_label == two_label)
                    {
                        // The label in the foursomes with one label must match one of the labels in the foursome with 2 labels
                        return PartialLabellingState::Overset;
                    }
                }
                // Need a fourth label given in one of the other 4 foursomes
            } else if with_label.len() == 3 {
                if let Some(one_label_adjacent_foursome_idx) = used_labels
                    .iter()
                    .enumerate()
                    .position(|(foursome_idx, labels)| {
                        labels.len() == 1 && are_adjacent(foursome_idx, two_labels_foursome_idx)
                    })
                {
                    let one_label_nonadjacent_foursome_idx = used_labels
                        .iter()
                        .enumerate()
                        .position(|(foursome_idx, labels)| {
                            labels.len() == 1 && foursome_idx != one_label_adjacent_foursome_idx
                        })
                        .unwrap();

                    let one_label_adjacent = used_labels[one_label_adjacent_foursome_idx]
                        .iter()
                        .next()
                        .unwrap();
                    if !used_labels[two_labels_foursome_idx]
                        .iter()
                        .any(|two_label| one_label_adjacent == two_label)
                    {
                        // The label in the foursomes with one label must match one of the labels in the foursome with 2 labels
                        return PartialLabellingState::Overset;
                    }

                    let one_label_nonadjacent = used_labels[one_label_nonadjacent_foursome_idx]
                        .iter()
                        .next()
                        .unwrap();

                    let x = *one_label_adjacent;
                    let y = *used_labels[two_labels_foursome_idx]
                        .iter()
                        .find(|two_label| **two_label != x)
                        .unwrap();
                    let z = *one_label_nonadjacent;
                    return PartialLabellingState::Perfect {
                        x,
                        y,
                        z,
                        foursome1: one_label_adjacent_foursome_idx,
                        foursome23: two_labels_foursome_idx,
                        foursome4: one_label_nonadjacent_foursome_idx,
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
                if !are_adjacent(with_label[0], with_label[1])
                    && !are_adjacent(with_label[0], with_label[2])
                    && !are_adjacent(with_label[1], with_label[2])
                {
                    return PartialLabellingState::Overset;
                }
            }

            PartialLabellingState::Underset
        }
    }

    // Given the labels currently set in self.labelling, return a list of allowable labels for each point
    fn allowed_labels(&self) -> LabelledMOGPoints<HashSet<F4Point>> {
        let mut result = LabelledMOGPoints::<HashSet<_>>::default();
        for i in 0..24 {
            for label in [F4Point::Zero, F4Point::One, F4Point::Alpha, F4Point::Beta] {
                let mut modified_self = self.clone();
                *modified_self.labelling.get_mut(i) = Some(label);
                match modified_self.partial_labelling_state() {
                    PartialLabellingState::Underset | PartialLabellingState::Perfect { .. } => {
                        result.get_mut(i).insert(label);
                    }
                    PartialLabellingState::Overset => {}
                }
            }
        }
        result
    }

    // Given the labels currently set in self.labelling return the completion to a labelling of the MOG if unique
    fn complete_labelling(&self) -> Option<LabelledMOGPoints<F4Point>> {
        match self.partial_labelling_state() {
            PartialLabellingState::Underset | PartialLabellingState::Overset => None,
            PartialLabellingState::Perfect {
                x,
                y,
                z,
                foursome1,
                foursome23,
                foursome4,
            } => {
                todo!()
            }
        }
    }
}

impl<PrevState: AppState + Clone> AppState for State<PrevState> {
    fn update(
        &mut self,
        ctx: &eframe::egui::Context,
        frame: &mut eframe::Frame,
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

        struct State<'a> {
            labelling: &'a mut LabelledMOGPoints<Option<F4Point>>,
            allowed_labels: &'a LabelledMOGPoints<HashSet<F4Point>>,
        }

        let mut mog_visuals = super::grid::GridVisuals::<State>::default();

        // The 6x4 MOG grid
        for (foursome_idx, foursome) in self.sextet.iter().enumerate() {
            for p in foursome.points() {
                let i = Vector::point_to_usize(p);
                mog_visuals.draw(
                    (i as isize % 6, i as isize / 6),
                    Box::new(move |ui, response, painter, rect, state| {
                        let colour = sextet_idx_to_colour(foursome_idx);

                        // Draw the coloured box for the point of the MOG
                        painter.rect_filled(
                            rect,
                            10.0,
                            colour.linear_multiply(0.3)
                                + ui.visuals().faint_bg_color.linear_multiply(0.7),
                        );

                        // Draw a border when dragging to indicate a label can be set here
                        if response.is_pointer_button_down_on()
                            && !state.allowed_labels.get(i).is_empty()
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
                                state.allowed_labels.get(i).clone(),
                                state.labelling.get(i).is_some(),
                            );
                            if response.drag_stopped() || response.clicked() {
                                match result {
                                    crate::ui::mog::F4SelectionResult::None => {}
                                    crate::ui::mog::F4SelectionResult::Point(label) => {
                                        *state.labelling.get_mut(i) = Some(label);
                                    }
                                    crate::ui::mog::F4SelectionResult::Cross => {
                                        *state.labelling.get_mut(i) = None;
                                    }
                                }
                            }
                        } else if let Some(label) = *state.labelling.get(i) {
                            // Draw labels
                            draw_f4(ui, painter, rect, ui.visuals().strong_text_color(), label);
                        }
                    }),
                );
            }
        }

        CentralPanel::default().show(ctx, |ui| {
            mog_visuals.show(
                ui,
                State {
                    labelling: &mut self.labelling,
                    allowed_labels: &allowed_labels,
                },
            );
        });

        None
    }
}
