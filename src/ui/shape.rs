use eframe::egui::{Color32, Mesh, Pos2, TextureId};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::single::SingleFloatOverlay,
    mesh::{
        stroke::offset::StrokeOffset,
        style::{LineCap, LineJoin, StrokeStyle},
    },
};
use i_triangle::float::triangulatable::Triangulatable;
use std::ops::{BitAnd, BitOr, BitXor};

pub fn arrowhead_cap(size: f64) -> LineCap<[f64; 2], f64> {
    LineCap::Custom(vec![
        [-2.0 * size, -1.0],
        [-4.0 * size, -3.0 * size],
        [2.0, 0.0],
        [-4.0 * size, 3.0 * size],
        [-2.0 * size, 1.0],
    ])
}

#[derive(Debug, Clone)]
pub struct Polygon {
    // list of (anti-clockwise outer loop in the first entry followed by a list of clockwise holes) for a disjoint union of polygons
    shapes: Vec<Vec<Vec<[f64; 2]>>>,
}

impl Polygon {
    pub fn empty() -> Self {
        Self { shapes: vec![] }
    }

    // regular polygon with a vertex directly right of the center, rotated anticlockwise by the given angle
    pub fn regular_polygon(center: Pos2, radius: f64, n: usize, angle: f64) -> Self {
        assert!(n >= 3);
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let t = (i as f64) / (n as f64) * std::f64::consts::TAU + angle;
            pts.push([
                (center.x as f64) + radius * t.cos(),
                (center.y as f64) + radius * t.sin(),
            ]);
        }
        Self {
            shapes: vec![vec![pts]],
        }
    }

    pub fn line(
        start_point: Pos2,
        end_point: Pos2,
        width: f64,
        start_cap: LineCap<[f64; 2], f64>,
        end_cap: LineCap<[f64; 2], f64>,
    ) -> Self {
        Self::lines(vec![start_point, end_point], width, start_cap, end_cap)
    }

    pub fn lines(
        points: Vec<Pos2>,
        width: f64,
        start_cap: LineCap<[f64; 2], f64>,
        end_cap: LineCap<[f64; 2], f64>,
    ) -> Self {
        let style = StrokeStyle::new(width)
            .line_join(LineJoin::Round(0.1))
            .start_cap(start_cap)
            .end_cap(end_cap);
        let shapes = points
            .into_iter()
            .map(|p| [p.x as f64, p.y as f64])
            .collect::<Vec<_>>()
            .stroke(style, false);
        Self { shapes }
    }

    pub fn bezier(
        points: Vec<Pos2>,
        width: f64,
        segments: usize,
        start_cap: LineCap<[f64; 2], f64>,
        end_cap: LineCap<[f64; 2], f64>,
    ) -> Self {
        assert!(points.len() >= 2);
        fn compute_bezier(points: &Vec<Pos2>, t: f32) -> Pos2 {
            debug_assert!(!points.is_empty());
            if points.len() == 1 {
                return points[0];
            }
            let mut new_points = Vec::with_capacity(points.len() - 1);
            for i in 0..points.len() - 1 {
                let p = points[i] + t * (points[i + 1] - points[i]);
                new_points.push(p);
            }
            compute_bezier(&new_points, t)
        }
        assert!(segments >= 1);
        let mut interpolate_points = vec![];
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            interpolate_points.push(compute_bezier(&points, t));
        }
        Self::lines(interpolate_points, width, start_cap, end_cap)
    }
}

impl BitOr<&Polygon> for &Polygon {
    type Output = Polygon;

    fn bitor(self, other: &Polygon) -> Self::Output {
        Polygon {
            shapes: self
                .shapes
                .overlay(&other.shapes, OverlayRule::Union, FillRule::EvenOdd),
        }
    }
}

impl BitAnd<&Polygon> for &Polygon {
    type Output = Polygon;

    fn bitand(self, other: &Polygon) -> Self::Output {
        Polygon {
            shapes: self
                .shapes
                .overlay(&other.shapes, OverlayRule::Intersect, FillRule::EvenOdd),
        }
    }
}

impl BitXor<&Polygon> for &Polygon {
    type Output = Polygon;

    fn bitxor(self, other: &Polygon) -> Self::Output {
        Polygon {
            shapes: self
                .shapes
                .overlay(&other.shapes, OverlayRule::Xor, FillRule::EvenOdd),
        }
    }
}

impl Polygon {
    pub fn to_egui_mesh(&self, colour: Color32) -> Mesh {
        let triangulation = self.shapes.triangulate().to_triangulation();
        Mesh {
            indices: triangulation.indices,
            vertices: triangulation
                .points
                .iter()
                .map(|p| eframe::epaint::Vertex {
                    pos: Pos2 {
                        x: p[0] as f32,
                        y: p[1] as f32,
                    },
                    uv: Pos2 { x: 0.0, y: 0.0 },
                    color: colour,
                })
                .collect(),
            texture_id: TextureId::default(),
        }
    }
}
