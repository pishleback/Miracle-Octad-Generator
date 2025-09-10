pub mod mog_permutation_shapes;
pub mod permutation_selection;
pub mod point_toggle;
pub mod sextet_labelling;
pub mod shape;

mod mog {
    use crate::logic::finite_field_4::Point as F4Point;
    use crate::logic::miracle_octad_generator::BinaryGolayCode;
    use eframe::egui::{Color32, Rect};
    use std::collections::HashSet;
    use std::sync::OnceLock;

    static MOG: OnceLock<BinaryGolayCode> = OnceLock::new();

    pub fn mog() -> &'static BinaryGolayCode {
        MOG.get_or_init(BinaryGolayCode::default)
    }

    // Draw an F4 element
    pub fn draw_f4(
        _ui: &mut eframe::egui::Ui,
        painter: &eframe::egui::Painter,
        rect: eframe::egui::Rect,
        colour: Color32,
        x: F4Point,
    ) {
        let label_size = 0.7 * rect.height();
        if x == F4Point::Beta {
            painter.text(
                rect.center()
                    + eframe::egui::Vec2 {
                        x: 0.0,
                        y: -0.8 * label_size,
                    },
                eframe::egui::Align2::CENTER_CENTER,
                "_",
                eframe::egui::FontId::proportional(label_size),
                colour,
            );
        }
        painter.text(
            rect.center(),
            eframe::egui::Align2::CENTER_CENTER,
            match x {
                F4Point::Zero => "0",
                F4Point::One => "1",
                F4Point::Alpha | F4Point::Beta => "ω",
            },
            eframe::egui::FontId::proportional(label_size),
            colour,
        );
    }

    pub fn row_to_f4(r: usize) -> F4Point {
        match r {
            0 => F4Point::Zero,
            1 => F4Point::One,
            2 => F4Point::Alpha,
            3 => F4Point::Beta,
            _ => unreachable!(),
        }
    }

    pub fn sextet_idx_to_colour(i: usize) -> Color32 {
        match i {
            0 => Color32::RED,
            1 => Color32::BLUE,
            2 => Color32::GREEN,
            3 => Color32::BROWN,
            4 => Color32::MAGENTA,
            5 => Color32::ORANGE,
            _ => unreachable!(),
        }
    }

    #[derive(Debug)]
    pub enum F4SelectionResult {
        None,
        Point(F4Point),
        Cross,
    }

    pub fn f4_selection(
        ui: &mut eframe::egui::Ui,
        painter: &eframe::egui::Painter,
        response: &eframe::egui::Response,
        rect: eframe::egui::Rect,
        include: impl Into<HashSet<F4Point>>,
        include_cross: bool,
    ) -> F4SelectionResult {
        let include = include.into();

        let top_left = Rect::from_min_max(rect.left_top(), rect.center());
        let top_right = Rect::from_min_max(rect.center_top(), rect.right_center());
        let bottom_left = Rect::from_min_max(rect.left_center(), rect.center_bottom());
        let bottom_right = Rect::from_min_max(rect.center(), rect.right_bottom());
        let middle = Rect::from_center_size(rect.center(), rect.size() / 3.0);

        let point_rects = [
            (F4Point::Zero, top_left),
            (F4Point::One, top_right),
            (F4Point::Alpha, bottom_left),
            (F4Point::Beta, bottom_right),
        ]
        .into_iter()
        .filter(|(point, _)| include.contains(point))
        .collect::<Vec<_>>();

        let mut result = F4SelectionResult::None;
        for (point, point_rect) in &point_rects {
            if (response.is_pointer_button_down_on()
                || response.drag_stopped()
                || response.clicked())
                && point_rect.contains(response.interact_pointer_pos().unwrap())
            {
                result = F4SelectionResult::Point(*point);
            }
        }
        if include_cross
            && (response.is_pointer_button_down_on()
                || response.drag_stopped()
                || response.clicked())
            && middle.contains(response.interact_pointer_pos().unwrap())
        {
            result = F4SelectionResult::Cross;
        }

        for (point, point_rect) in point_rects {
            let colour = if let F4SelectionResult::Point(selected_point) = result {
                if point == selected_point {
                    ui.visuals().strong_text_color()
                } else {
                    ui.visuals().weak_text_color()
                }
            } else {
                ui.visuals().weak_text_color()
            };
            draw_f4(ui, painter, point_rect, colour, point);
        }

        if include_cross {
            painter.text(
                middle.center(),
                eframe::egui::Align2::CENTER_CENTER,
                "🗙",
                eframe::egui::FontId::proportional(0.4 * middle.height()),
                match result {
                    F4SelectionResult::Cross => ui.visuals().strong_text_color(),
                    _ => ui.visuals().weak_text_color(),
                },
            );
        }

        result
    }
}

mod grid {
    use eframe::egui::{Painter, Pos2, Rect, Response, Sense, Vec2};
    use std::collections::HashSet;

    pub type GridCell = (isize, isize);

    pub struct GridBuilder {
        pad: f32, // The gap between squares
        elements: HashSet<GridCell>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct GridShower {
        rect: Rect,
        unit: f32,
        pad: f32,
        min_cell: GridCell,
    }

    impl GridShower {
        pub fn cell_to_pos(&self, cell: GridCell) -> Pos2 {
            Pos2 {
                x: self.rect.left() + ((cell.0 - self.min_cell.0) as f32 + 0.5) * self.unit,
                y: self.rect.top() + ((cell.1 - self.min_cell.1) as f32 + 0.5) * self.unit,
            }
        }

        pub fn cell_to_rect(&self, cell: GridCell) -> Rect {
            Rect::from_center_size(
                self.cell_to_pos(cell),
                Vec2 {
                    x: self.unit - self.pad,
                    y: self.unit - self.pad,
                },
            )
        }

        pub fn cell_scalar_to_pos_scalar(&self, lambda: f32) -> f32 {
            lambda * self.unit
        }
    }

    impl Default for GridBuilder {
        fn default() -> Self {
            Self {
                pad: 10.0,
                elements: HashSet::new(),
            }
        }
    }

    impl GridBuilder {
        pub fn include_cell(&mut self, cell: GridCell) {
            self.elements.insert(cell);
        }

        pub fn show(self, ui: &mut eframe::egui::Ui) -> (Response, Painter, GridShower) {
            let min_i = self.elements.iter().map(|(i, _)| *i).min().unwrap_or(0);
            let max_i = self.elements.iter().map(|(i, _)| *i).max().unwrap_or(0);
            let min_j = self.elements.iter().map(|(_, j)| *j).min().unwrap_or(0);
            let max_j = self.elements.iter().map(|(_, j)| *j).max().unwrap_or(0);
            let size_i = max_i - min_i + 1;
            let size_j = max_j - min_j + 1;

            let (response, painter) = ui.allocate_painter(
                {
                    let available = ui.available_size();
                    let mut size = Vec2 {
                        x: available.x,
                        y: (size_j as f32 / size_i as f32) * available.x,
                    };
                    if size.y > available.y {
                        size = size * available.y / size.y;
                    }
                    size
                },
                Sense::click_and_drag(),
            );

            let coordinates = GridShower {
                rect: response.rect,
                unit: response.rect.width() / (size_i as f32),
                pad: self.pad,
                min_cell: (min_i, min_j),
            };

            (response, painter, coordinates)
        }
    }
}
