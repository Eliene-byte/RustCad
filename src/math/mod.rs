//! Matemática e geometria para o motor CAD.
//! Fornece vetores 2D/3D, pontos, transformações e primitivas geométricas.

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Ponto 2D no espaço do desenho.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub const ORIGIN: Point2 = Point2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point2) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn midpoint(&self, other: &Point2) -> Point2 {
        Point2 {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }

    pub fn angle_to(&self, other: &Point2) -> f64 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    pub fn polar(&self, angle: f64, distance: f64) -> Point2 {
        Point2 {
            x: self.x + distance * angle.cos(),
            y: self.y + distance * angle.sin(),
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn from_vec2(v: Vec2) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
        }
    }
}

impl std::ops::Add for Point2 {
    type Output = Point2;
    fn add(self, rhs: Point2) -> Point2 {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Point2 {
    type Output = Point2;
    fn sub(self, rhs: Point2) -> Point2 {
        Point2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for Point2 {
    type Output = Point2;
    fn mul(self, s: f64) -> Point2 {
        Point2 {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

/// Vetor 2D.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let m = self.magnitude();
        if m == 0.0 {
            return Self { x: 0.0, y: 0.0 };
        }
        Self {
            x: self.x / m,
            y: self.y / m,
        }
    }

    pub fn dot(&self, other: &Vector2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross_z(&self, other: &Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn rotate(&self, angle: f64) -> Self {
        Self {
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.x * angle.sin() + self.y * angle.cos(),
        }
    }

    pub fn to_point(&self) -> Point2 {
        Point2::new(self.x, self.y)
    }
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, s: f64) -> Vector2 {
        Vector2 {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

/// Transformação afim 2D (translação, rotação, escala).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2 {
    pub translation: Point2,
    pub rotation: f64, // radianos
    pub scale: f64,
}

impl Transform2 {
    pub fn identity() -> Self {
        Self {
            translation: Point2::ORIGIN,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn translate(x: f64, y: f64) -> Self {
        Self {
            translation: Point2::new(x, y),
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn rotate(angle: f64) -> Self {
        Self {
            translation: Point2::ORIGIN,
            rotation: angle,
            scale: 1.0,
        }
    }

    pub fn scale(s: f64) -> Self {
        Self {
            translation: Point2::ORIGIN,
            rotation: 0.0,
            scale: s,
        }
    }

    pub fn apply(&self, point: Point2) -> Point2 {
        // Aplica: escala -> rotaciona em torno da origem -> transla
        let scaled = Point2::new(point.x * self.scale, point.y * self.scale);
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rx = scaled.x * cos - scaled.y * sin;
        let ry = scaled.x * sin + scaled.y * cos;
        Point2::new(self.translation.x + rx, self.translation.y + ry)
    }

    pub fn compose(&self, other: Transform2) -> Transform2 {
        // Aplica self depois de other
        let p = self.apply(Point2::ORIGIN);
        let rot = self.rotation + other.rotation;
        let scale = self.scale * other.scale;
        Transform2 {
            translation: p,
            rotation: rot,
            scale,
        }
    }
}

/// Limites de uma entidade (bounding box).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds2 {
    pub min: Point2,
    pub max: Point2,
}

impl Bounds2 {
    pub fn empty() -> Self {
        Self {
            min: Point2::new(f64::INFINITY, f64::INFINITY),
            max: Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn from_points(points: &[Point2]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min = points[0];
        let mut max = points[0];
        for p in points.iter().skip(1) {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some(Self { min, max })
    }

    pub fn contains(&self, p: Point2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    pub fn expand(&mut self, p: Point2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
}

/// Segmento de reta definido por dois pontos.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineSegment {
    pub start: Point2,
    pub end: Point2,
}

impl LineSegment {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    pub fn midpoint(&self) -> Point2 {
        self.start.midpoint(&self.end)
    }

    pub fn angle(&self) -> f64 {
        self.start.angle_to(&self.end)
    }

    /// Verifica se um ponto está próximo do segmento (dentro de uma tolerância).
    pub fn distance_to_point(&self, p: Point2) -> f64 {
        let d = self.end - self.start;
        let d2 = d.x * d.x + d.y * d.y;
        if d2 == 0.0 {
            return p.distance_to(&self.start);
        }
        let t = ((p.x - self.start.x) * d.x + (p.y - self.start.y) * d.y) / d2;
        let t = t.clamp(0.0, 1.0);
        let closest = Point2::new(self.start.x + t * d.x, self.start.y + t * d.y);
        p.distance_to(&closest)
    }

    /// Interseção entre dois segmentos (retorna o ponto se existirem).
    pub fn intersection(&self, other: &LineSegment) -> Option<Point2> {
        let p = self.start;
        let q = other.start;
        let r = Vector2::new(self.end.x - self.start.x, self.end.y - self.start.y);
        let s = Vector2::new(other.end.x - other.start.x, other.end.y - other.start.y);

        let r_cross_s = r.cross_z(&s);
        let q_minus_p = Vector2::new(q.x - p.x, q.y - p.y);
        let q_minus_p_cross_r = q_minus_p.cross_z(&r);

        if r_cross_s == 0.0 {
            return None; // paralelos ou colineares
        }

        let t = q_minus_p.cross_z(&s) / r_cross_s;
        let u = q_minus_p_cross_r / r_cross_s;

        if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
            Some(Point2::new(p.x + t * r.x, p.y + t * r.y))
        } else {
            None
        }
    }
}

/// Arco circular.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point2,
    pub radius: f64,
    pub start_angle: f64, // radianos
    pub end_angle: f64,   // radianos
    pub ccw: bool,        // true = anti-horário
}

impl Arc {
    pub fn new(center: Point2, radius: f64, start_angle: f64, end_angle: f64, ccw: bool) -> Self {
        Self {
            center,
            radius,
            start_angle,
            end_angle,
            ccw,
        }
    }

    pub fn start_point(&self) -> Point2 {
        self.center.polar(self.start_angle, self.radius)
    }

    pub fn end_point(&self) -> Point2 {
        self.center.polar(self.end_angle, self.radius)
    }

    pub fn length(&self) -> f64 {
        let mut sweep = self.end_angle - self.start_angle;
        if !self.ccw {
            sweep = -sweep;
        }
        while sweep < 0.0 {
            sweep += 2.0 * PI;
        }
        while sweep > 2.0 * PI {
            sweep -= 2.0 * PI;
        }
        self.radius * sweep
    }
}

/// Tipo de entidade CAD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Line,
    Circle,
    Arc,
    Point,
    Polyline,
    Text,
    BlockReference,
}

/// Modo de snap de objeto (OSNAP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsnapMode {
    None,
    Endpoint,
    Midpoint,
    Center,
    Intersection,
    Perpendicular,
    Nearest,
    Quadrant,
    Tangent,
}

/// Resultado de snap de objeto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OsnapResult {
    pub point: Point2,
    pub mode: OsnapMode,
}

/// Utilitários geométricos gerais.
pub mod geometry {
    use super::*;

    /// Verifica se três pontos são colineares.
    pub fn are_collinear(a: Point2, b: Point2, c: Point2) -> bool {
        let ab = Vector2::new(b.x - a.x, b.y - a.y);
        let ac = Vector2::new(c.x - a.x, c.y - a.y);
        ab.cross_z(&ac).abs() < 1e-9
    }

    /// Calcula a área de um polígono (fórmula do sapato).
    pub fn polygon_area(points: &[Point2]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            area += points[i].x * points[j].y - points[j].x * points[i].y;
        }
        area.abs() / 2.0
    }

    /// Verifica se um ponto está dentro de um polígono (ray casting).
    pub fn point_in_polygon(p: Point2, polygon: &[Point2]) -> bool {
        if polygon.len() < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = polygon.len() - 1;
        for i in 0..polygon.len() {
            if (polygon[i].y > p.y) != (polygon[j].y > p.y) {
                let x_intersect = (polygon[j].x - polygon[i].x) * (p.y - polygon[i].y)
                    / (polygon[j].y - polygon[i].y)
                    + polygon[i].x;
                if p.x < x_intersect {
                    inside = !inside;
                }
            }
            j = i;
        }
        inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn line_segment_intersection() {
        let l1 = LineSegment::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0));
        let l2 = LineSegment::new(Point2::new(0.0, 2.0), Point2::new(2.0, 0.0));
        let inter = l1.intersection(&l2);
        assert!(inter.is_some());
        let p = inter.unwrap();
        assert!((p.x - 1.0).abs() < 1e-9 && (p.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn transform_apply() {
        let t = Transform2::translate(10.0, 20.0);
        let p = t.apply(Point2::new(1.0, 2.0));
        assert!((p.x - 11.0).abs() < 1e-9 && (p.y - 22.0).abs() < 1e-9);
    }

    #[test]
    fn polygon_area() {
        let square = vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 10.0),
            Point2::new(10.0, 10.0),
            Point2::new(10.0, 0.0),
        ];
        assert!((geometry::polygon_area(&square) - 100.0).abs() < 1e-9);
    }
}
