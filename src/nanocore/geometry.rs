//! Operações geométricas otimizadas do NanoCore.
//! Zero alocação durante operações comuns — ideal para notebooks fracos.

use crate::math::{LineSegment, Point2};
use crate::nanocore::kernel::NanoEntity;

/// Resultado de interseção otimizado.
#[derive(Debug, Clone, Copy)]
pub struct IntersectionResult {
    pub point: Point2,
    pub entity_a_id: u32,
    pub entity_b_id: u32,
}

/// Verifica interseção entre duas entidades Nano.
pub fn intersect(a: &NanoEntity, b: &NanoEntity) -> Option<Point2> {
    match (a.entity_type, b.entity_type) {
        (crate::nanocore::kernel::NanoEntityType::Line, crate::nanocore::kernel::NanoEntityType::Line) => {
            let seg_a = LineSegment::new(a.start_point(), a.end_point());
            let seg_b = LineSegment::new(b.start_point(), b.end_point());
            seg_a.intersection(&seg_b)
        }
        (crate::nanocore::kernel::NanoEntityType::Circle, crate::nanocore::kernel::NanoEntityType::Line) => {
            circle_line_intersection(b, a)
        }
        (crate::nanocore::kernel::NanoEntityType::Line, crate::nanocore::kernel::NanoEntityType::Circle) => {
            circle_line_intersection(a, b)
        }
        (crate::nanocore::kernel::NanoEntityType::Circle, crate::nanocore::kernel::NanoEntityType::Circle) => {
            circle_circle_intersection(a, b)
        }
        _ => None,
    }
}

/// Distância entre duas entidades (mínima).
pub fn distance_between(a: &NanoEntity, b: &NanoEntity) -> f64 {
    // Bounding box check rápido
    let ba = a.bounds();
    let bb = b.bounds();
    if ba.max.x < bb.min.x || bb.max.x < ba.min.x || ba.max.y < bb.min.y || bb.max.y < ba.min.y {
        return f64::INFINITY;
    }
    match (a.entity_type, b.entity_type) {
        (crate::nanocore::kernel::NanoEntityType::Point, crate::nanocore::kernel::NanoEntityType::Point) => {
            a.start_point().distance_to(&b.start_point())
        }
        _ => {
            // Aproximação: distância entre centros dos bounding boxes
            ba.center().distance_to(&bb.center())
        }
    }
}

/// Calcule a distância de um ponto até uma entidade.
pub fn point_to_entity_distance(p: Point2, e: &NanoEntity) -> f64 {
    if !e.bounds().contains(p) && p.distance_to(&e.bounds().center()) > 1000.0 {
        return f64::INFINITY;
    }
    match e.entity_type {
        crate::nanocore::kernel::NanoEntityType::Point => p.distance_to(&e.start_point()),
        crate::nanocore::kernel::NanoEntityType::Line => {
            let seg = LineSegment::new(e.start_point(), e.end_point());
            seg.distance_to_point(p)
        }
        crate::nanocore::kernel::NanoEntityType::Circle => {
            (p.distance_to(&e.start_point()) - e.data[2]).abs()
        }
        _ => p.distance_to(&e.bounds().center()),
    }
}

/// Interseção círculo-linha.
fn circle_line_intersection(circle: &NanoEntity, line: &NanoEntity) -> Option<Point2> {
    let center = circle.start_point();
    let r = circle.data[2];
    let seg = LineSegment::new(line.start_point(), line.end_point());
    // Simplificação: verifica se o ponto mais próximo do segmento ao centro está dentro do raio
    let closest = closest_point_on_segment(center, &seg);
    if closest.distance_to(&center) <= r {
        Some(closest)
    } else {
        None
    }
}

/// Interseção círculo-círculo.
fn circle_circle_intersection(a: &NanoEntity, b: &NanoEntity) -> Option<Point2> {
    let c1 = a.start_point();
    let c2 = b.start_point();
    let r1 = a.data[2];
    let r2 = b.data[2];
    let d = c1.distance_to(&c2);
    if d > r1 + r2 || d < (r1 - r2).abs() {
        return None;
    }
    // Retorna o ponto médio entre as interseções (aproximação rápida)
    Some(Point2::new((c1.x + c2.x) / 2.0, (c1.y + c2.y) / 2.0))
}

/// Ponto mais próximo em um segmento a um ponto dado.
fn closest_point_on_segment(p: Point2, seg: &LineSegment) -> Point2 {
    let d = seg.end - seg.start;
    let d2 = d.x * d.x + d.y * d.y;
    if d2 == 0.0 {
        return seg.start;
    }
    let t = ((p.x - seg.start.x) * d.x + (p.y - seg.start.y) * d.y) / d2;
    let t = t.clamp(0.0, 1.0);
    Point2::new(seg.start.x + t * d.x, seg.start.y + t * d.y)
}

/// Calcule o comprimento total de todas as linhas.
pub fn total_line_length(entities: &[NanoEntity]) -> f64 {
    entities
        .iter()
        .filter(|e| e.visible && matches!(e.entity_type, crate::nanocore::kernel::NanoEntityType::Line))
        .map(|e| LineSegment::new(e.start_point(), e.end_point()).length())
        .sum()
}

/// Encontre a entidade mais próxima de um ponto.
pub fn nearest_entity<'a>(point: Point2, entities: &'a [NanoEntity]) -> Option<(&'a NanoEntity, f64)> {
    let mut best: Option<(&'a NanoEntity, f64)> = None;
    for e in entities {
        if !e.visible {
            continue;
        }
        let d = point_to_entity_distance(point, e);
        if d < f64::INFINITY {
            match best {
                Some((_, bd)) if d < bd => best = Some((e, d)),
                None => best = Some((e, d)),
                _ => {}
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanocore::kernel::NanoEntity;

    #[test]
    fn line_line_intersection() {
        let a = NanoEntity::new_line(1, 0, 0.0, 0.0, 10.0, 10.0);
        let b = NanoEntity::new_line(2, 0, 0.0, 10.0, 10.0, 0.0);
        let inter = intersect(&a, &b);
        assert!(inter.is_some());
        let p = inter.unwrap();
        assert!((p.x - 5.0).abs() < 1e-9 && (p.y - 5.0).abs() < 1e-9);
    }

    #[test]
    fn circle_line_intersection() {
        let circle = NanoEntity::new_circle(1, 0, 5.0, 5.0, 3.0);
        let line = NanoEntity::new_line(2, 0, 5.0, 0.0, 5.0, 10.0);
        let inter = intersect(&line, &circle);
        assert!(inter.is_some());
    }

    #[test]
    fn no_intersection() {
        let a = NanoEntity::new_line(1, 0, 0.0, 0.0, 1.0, 1.0);
        let b = NanoEntity::new_line(2, 0, 100.0, 100.0, 101.0, 101.0);
        assert!(intersect(&a, &b).is_none());
    }

    #[test]
    fn test_nearest_entity() {
        let entities = vec![
            NanoEntity::new_point(1, 0, 0.0, 0.0),
            NanoEntity::new_point(2, 0, 100.0, 100.0),
        ];
        let nearest = nearest_entity(Point2::new(1.0, 1.0), &entities);
        assert!(nearest.is_some());
        assert_eq!(nearest.unwrap().0.id, 1);
    }

    #[test]
    fn test_total_line_length() {
        let entities = vec![
            NanoEntity::new_line(1, 0, 0.0, 0.0, 3.0, 4.0), // length 5
        ];
        let len = total_line_length(&entities);
        assert!((len - 5.0).abs() < 1e-9);
    }
}
