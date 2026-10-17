//! Kernel do NanoCore — estruturas de dados compactas e armazenamento
//! cache-friendly para entidades CAD.

use crate::math::{Bounds2, Point2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipo de entidade no NanoCore (representação compacta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NanoEntityType {
    Line = 0,
    Circle = 1,
    Arc = 2,
    Point = 3,
    Polyline = 4,
    Text = 5,
}

/// Camada simplificada do NanoCore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NanoLayer {
    pub id: u32,
    pub name: [u8; 32],      // nome fixo de 32 bytes (sem alocação String)
    pub color: u32,
    pub visible: bool,
    pub locked: bool,
}

impl NanoLayer {
    pub fn new(id: u32, name: &str) -> Self {
        let mut buf = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(31);
        buf[..len].copy_from_slice(&bytes[..len]);
        Self {
            id,
            name: buf,
            color: 0xFFFFFF,
            visible: true,
            locked: false,
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        std::str::from_utf8(&self.name[..len]).unwrap_or("")
    }
}

/// Entidade Nano — struct densa com layout fixo para cache-localidade.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct NanoEntity {
    pub id: u32,
    pub layer_id: u32,
    pub entity_type: NanoEntityType,
    pub visible: bool,
    // Geometria compacta: 8 f64 = 64 bytes (suficiente para linha, círculo, ponto)
    pub data: [f64; 8],
    // Bounding box pré-calculado para culling rápido
    pub bounds_min_x: f64,
    pub bounds_min_y: f64,
    pub bounds_max_x: f64,
    pub bounds_max_y: f64,
}

impl NanoEntity {
    pub const SIZE_BYTES: usize = std::mem::size_of::<Self>();

    pub fn new_line(id: u32, layer_id: u32, x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        Self {
            id,
            layer_id,
            entity_type: NanoEntityType::Line,
            visible: true,
            data: [x1, y1, x2, y2, 0.0, 0.0, 0.0, 0.0],
            bounds_min_x: min_x,
            bounds_min_y: min_y,
            bounds_max_x: max_x,
            bounds_max_y: max_y,
        }
    }

    pub fn new_circle(id: u32, layer_id: u32, cx: f64, cy: f64, r: f64) -> Self {
        Self {
            id,
            layer_id,
            entity_type: NanoEntityType::Circle,
            visible: true,
            data: [cx, cy, r, 0.0, 0.0, 0.0, 0.0, 0.0],
            bounds_min_x: cx - r,
            bounds_min_y: cy - r,
            bounds_max_x: cx + r,
            bounds_max_y: cy + r,
        }
    }

    pub fn new_point(id: u32, layer_id: u32, x: f64, y: f64) -> Self {
        Self {
            id,
            layer_id,
            entity_type: NanoEntityType::Point,
            visible: true,
            data: [x, y, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            bounds_min_x: x,
            bounds_min_y: y,
            bounds_max_x: x,
            bounds_max_y: y,
        }
    }

    pub fn start_point(&self) -> Point2 {
        match self.entity_type {
            NanoEntityType::Line => Point2::new(self.data[0], self.data[1]),
            NanoEntityType::Circle => Point2::new(self.data[0], self.data[1]),
            NanoEntityType::Point => Point2::new(self.data[0], self.data[1]),
            _ => Point2::ORIGIN,
        }
    }

    pub fn end_point(&self) -> Point2 {
        match self.entity_type {
            NanoEntityType::Line => Point2::new(self.data[2], self.data[3]),
            _ => self.start_point(),
        }
    }

    pub fn bounds(&self) -> Bounds2 {
        Bounds2 {
            min: Point2::new(self.bounds_min_x, self.bounds_min_y),
            max: Point2::new(self.bounds_max_x, self.bounds_max_y),
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self.entity_type {
            NanoEntityType::Line => {
                self.data[0] += dx;
                self.data[1] += dy;
                self.data[2] += dx;
                self.data[3] += dy;
            }
            NanoEntityType::Circle | NanoEntityType::Point => {
                self.data[0] += dx;
                self.data[1] += dy;
            }
            _ => {}
        }
        self.bounds_min_x += dx;
        self.bounds_min_y += dy;
        self.bounds_max_x += dx;
        self.bounds_max_y += dy;
    }

    pub fn scale(&mut self, cx: f64, cy: f64, factor: f64) {
        match self.entity_type {
            NanoEntityType::Line => {
                self.data[0] = cx + (self.data[0] - cx) * factor;
                self.data[1] = cy + (self.data[1] - cy) * factor;
                self.data[2] = cx + (self.data[2] - cx) * factor;
                self.data[3] = cy + (self.data[3] - cy) * factor;
            }
            NanoEntityType::Circle => {
                self.data[0] = cx + (self.data[0] - cx) * factor;
                self.data[1] = cy + (self.data[1] - cy) * factor;
                self.data[2] *= factor;
            }
            NanoEntityType::Point => {
                self.data[0] = cx + (self.data[0] - cx) * factor;
                self.data[1] = cy + (self.data[1] - cy) * factor;
            }
            _ => {}
        }
        let b = self.bounds();
        self.bounds_min_x = b.min.x;
        self.bounds_min_y = b.min.y;
        self.bounds_max_x = b.max.x;
        self.bounds_max_y = b.max.y;
    }

    pub fn rotate(&mut self, cx: f64, cy: f64, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        match self.entity_type {
            NanoEntityType::Line => {
                for i in [0usize, 2] {
                    let px = self.data[i];
                    let py = self.data[i + 1];
                    let rel_x = px - cx;
                    let rel_y = py - cy;
                    self.data[i] = cx + rel_x * cos - rel_y * sin;
                    self.data[i + 1] = cy + rel_x * sin + rel_y * cos;
                }
            }
            NanoEntityType::Circle | NanoEntityType::Point => {
                let px = self.data[0];
                let py = self.data[1];
                let rel_x = px - cx;
                let rel_y = py - cy;
                self.data[0] = cx + rel_x * cos - rel_y * sin;
                self.data[1] = cy + rel_x * sin + rel_y * cos;
            }
            _ => {}
        }
        let b = self.bounds();
        self.bounds_min_x = b.min.x;
        self.bounds_min_y = b.min.y;
        self.bounds_max_x = b.max.x;
        self.bounds_max_y = b.max.y;
    }

    pub fn contains_point(&self, p: Point2, tolerance: f64) -> bool {
        if !self.bounds().contains(p) {
            return false;
        }
        match self.entity_type {
            NanoEntityType::Point => {
                p.distance_to(&self.start_point()) < tolerance
            }
            NanoEntityType::Line => {
                let seg = crate::math::LineSegment::new(self.start_point(), self.end_point());
                seg.distance_to_point(p) < tolerance
            }
            NanoEntityType::Circle => {
                let d = p.distance_to(&self.start_point());
                (d - self.data[2]).abs() < tolerance
            }
            _ => false,
        }
    }

    pub fn area(&self) -> f64 {
        match self.entity_type {
            NanoEntityType::Circle => std::f64::consts::PI * self.data[2].powi(2),
            _ => 0.0,
        }
    }
}

impl Default for NanoEntity {
    fn default() -> Self {
        Self {
            id: 0,
            layer_id: 0,
            entity_type: NanoEntityType::Line,
            visible: true,
            data: [0.0; 8],
            bounds_min_x: 0.0,
            bounds_min_y: 0.0,
            bounds_max_x: 0.0,
            bounds_max_y: 0.0,
        }
    }
}

/// Transformação compacta do NanoCore.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct NanoTransform {
    pub tx: f64,
    pub ty: f64,
    pub rotation: f64,
    pub scale: f64,
}

impl NanoTransform {
    pub fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    pub fn apply(&self, p: Point2) -> Point2 {
        let scaled = Point2::new(p.x * self.scale, p.y * self.scale);
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        Point2::new(
            self.tx + scaled.x * cos - scaled.y * sin,
            self.ty + scaled.x * sin + scaled.y * cos,
        )
    }
}

/// Núcleo principal do NanoCore — gerencia todas as entidades e camadas com armazenamento contíguo e índice espacial simples.
pub struct NanoCore {
    pub layers: Vec<NanoLayer>,
    pub entities: Vec<NanoEntity>,
    pub next_entity_id: u32,
    pub next_layer_id: u32,
    pub current_layer_id: u32,
    // Índice espacial: grid simples para culling rápido
    spatial_grid: Option<SpatialGrid>,
}

impl NanoCore {
    pub fn new() -> Self {
        let mut layers = Vec::new();
        layers.push(NanoLayer::new(0, "0"));
        Self {
            layers,
            entities: Vec::new(),
            next_entity_id: 1,
            next_layer_id: 1,
            current_layer_id: 0,
            spatial_grid: None,
        }
    }

    pub fn add_layer(&mut self, name: &str) -> u32 {
        let id = self.next_layer_id;
        self.next_layer_id += 1;
        self.layers.push(NanoLayer::new(id, name));
        id
    }

    pub fn set_current_layer(&mut self, id: u32) -> bool {
        if self.layers.iter().any(|l| l.id == id) {
            self.current_layer_id = id;
            true
        } else {
            false
        }
    }

    pub fn current_layer(&self) -> Option<&NanoLayer> {
        self.layers.iter().find(|l| l.id == self.current_layer_id)
    }

    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(NanoEntity::new_line(id, self.current_layer_id, x1, y1, x2, y2));
        self.spatial_grid = None; // invalida índice
        id
    }

    pub fn add_circle(&mut self, cx: f64, cy: f64, r: f64) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(NanoEntity::new_circle(id, self.current_layer_id, cx, cy, r));
        self.spatial_grid = None;
        id
    }

    pub fn add_point(&mut self, x: f64, y: f64) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(NanoEntity::new_point(id, self.current_layer_id, x, y));
        self.spatial_grid = None;
        id
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn visible_entity_count(&self) -> usize {
        self.entities.iter().filter(|e| e.visible).count()
    }

    pub fn get_entity(&self, id: u32) -> Option<&NanoEntity> {
        self.entities.iter().find(|e| e.id == id && e.visible)
    }

    pub fn translate_entity(&mut self, id: u32, dx: f64, dy: f64) -> bool {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == id) {
            e.translate(dx, dy);
            self.spatial_grid = None;
            true
        } else {
            false
        }
    }

    pub fn erase_entity(&mut self, id: u32) -> bool {
        let len_before = self.entities.len();
        self.entities.retain(|e| e.id != id);
        if self.entities.len() < len_before {
            self.spatial_grid = None;
            true
        } else {
            false
        }
    }

    /// Constrói o índice espacial (grid) para consultas rápidas.
    fn ensure_spatial_grid(&mut self) {
        if self.spatial_grid.is_some() {
            return;
        }
        if self.entities.is_empty() {
            return;
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for e in &self.entities {
            if !e.visible {
                continue;
            }
            min_x = min_x.min(e.bounds_min_x);
            min_y = min_y.min(e.bounds_min_y);
            max_x = max_x.max(e.bounds_max_x);
            max_y = max_y.max(e.bounds_max_y);
        }
        let mut grid = SpatialGrid::new(
            min_x, min_y, max_x, max_y,
            16, 16, // grid 16x16
        );
        // Popula as células com as entidades visíveis
        for (i, e) in self.entities.iter().enumerate() {
            if e.visible {
                grid.add_entity(i, e);
            }
        }
        self.spatial_grid = Some(grid);
    }

    /// Encontra entidades próximas a um ponto (snap de objeto otimizado).
    pub fn find_nearby(&mut self, point: Point2, radius: f64) -> Vec<u32> {
        self.ensure_spatial_grid();
        let mut result = Vec::new();
        if let Some(grid) = &self.spatial_grid {
            let cell_ids = grid.cells_around(point, radius);
            for &cid in &cell_ids {
                if let Some(indices) = grid.cells.get(&cid) {
                    for &ei in indices {
                        let e = &self.entities[ei];
                        if e.visible && point.distance_to(&e.bounds().center()) <= radius {
                            result.push(e.id);
                        }
                    }
                }
            }
        } else {
            // fallback: busca linear
            for e in &self.entities {
                if e.visible && point.distance_to(&e.bounds().center()) <= radius {
                    result.push(e.id);
                }
            }
        }
        result
    }

    /// Área total de todas as entidades visíveis.
    pub fn total_area(&self) -> f64 {
        self.entities.iter().filter(|e| e.visible).map(|e| e.area()).sum()
    }

    /// Limite geral de todas as entidades.
    pub fn total_bounds(&self) -> Option<Bounds2> {
        let mut bounds: Option<Bounds2> = None;
        for e in &self.entities {
            if !e.visible {
                continue;
            }
            let b = e.bounds();
            match &mut bounds {
                Some(cur) => {
                    cur.expand(b.min);
                    cur.expand(b.max);
                }
                None => bounds = Some(b),
            }
        }
        bounds
    }
}

impl Default for NanoCore {
    fn default() -> Self {
        Self::new()
    }
}

/// Grid espacial simples para culling rápido.
#[derive(Debug, Clone)]
struct SpatialGrid {
    min_x: f64,
    min_y: f64,
    cell_w: f64,
    cell_h: f64,
    cols: usize,
    rows: usize,
    cells: HashMap<u64, Vec<usize>>,
}

impl SpatialGrid {
    fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64, cols: usize, rows: usize) -> Self {
        let range_x = (max_x - min_x).max(1.0);
        let range_y = (max_y - min_y).max(1.0);
        Self {
            min_x,
            min_y,
            cell_w: range_x / cols as f64,
            cell_h: range_y / rows as f64,
            cols,
            rows,
            cells: HashMap::new(),
        }
    }

    fn cell_key(col: i32, row: i32) -> u64 {
        ((col as u32) as u64) << 32 | (row as u32) as u64
    }

    fn cell_for_point(&self, p: Point2) -> (i32, i32) {
        let col = ((p.x - self.min_x) / self.cell_w).floor() as i32;
        let row = ((p.y - self.min_y) / self.cell_h).floor() as i32;
        (col, row)
    }

    fn cells_around(&self, p: Point2, radius: f64) -> Vec<u64> {
        let (c0, r0) = self.cell_for_point(p);
        let (c1, r1) = self.cell_for_point(Point2::new(p.x + radius, p.y + radius));
        let (c2, r2) = self.cell_for_point(Point2::new(p.x - radius, p.y - radius));
        let min_col = c0.min(c1).min(c2);
        let max_col = c0.max(c1).max(c2);
        let min_row = r0.min(r1).min(r2);
        let max_row = r0.max(r1).max(r2);
        let mut keys = Vec::new();
        for c in min_col..=max_col {
            for r in min_row..=max_row {
                keys.push(Self::cell_key(c, r));
            }
        }
        keys
    }

    fn add_entity(&mut self, entity_index: usize, entity: &NanoEntity) {
        if !entity.visible {
            return;
        }
        let center = entity.bounds().center();
        let (col, row) = self.cell_for_point(center);
        let key = Self::cell_key(col, row);
        self.cells.entry(key).or_default().push(entity_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_find_line() {
        let mut core = NanoCore::new();
        let id = core.add_line(0.0, 0.0, 10.0, 10.0);
        assert_eq!(core.entity_count(), 1);
        let e = core.get_entity(id).unwrap();
        assert_eq!(e.start_point(), Point2::new(0.0, 0.0));
        assert_eq!(e.end_point(), Point2::new(10.0, 10.0));
    }

    #[test]
    fn translate_entity() {
        let mut core = NanoCore::new();
        let id = core.add_line(0.0, 0.0, 5.0, 5.0);
        core.translate_entity(id, 10.0, 10.0);
        let e = core.get_entity(id).unwrap();
        assert!((e.start_point().x - 10.0).abs() < 1e-9);
        assert!((e.end_point().x - 15.0).abs() < 1e-9);
    }

    #[test]
    fn erase_entity() {
        let mut core = NanoCore::new();
        let id = core.add_circle(0.0, 0.0, 5.0);
        assert!(core.erase_entity(id));
        assert!(core.get_entity(id).is_none());
        assert_eq!(core.entity_count(), 0);
    }

    #[test]
    fn find_nearby() {
        let mut core = NanoCore::new();
        core.add_line(0.0, 0.0, 10.0, 0.0);
        core.add_line(100.0, 100.0, 110.0, 100.0);
        let nearby = core.find_nearby(Point2::new(5.0, 0.0), 1.0);
        assert!(!nearby.is_empty());
    }

    #[test]
    fn layer_operations() {
        let mut core = NanoCore::new();
        let lid = core.add_layer("Piso");
        assert!(core.set_current_layer(lid));
        assert_eq!(core.current_layer().unwrap().name_str(), "Piso");
    }

    #[test]
    fn entity_size_is_compact() {
        // NanoEntity deve ser compacto (64 bytes de dados + cabeçalho)
        assert!(std::mem::size_of::<NanoEntity>() <= 128);
    }

    #[test]
    fn total_area() {
        let mut core = NanoCore::new();
        core.add_circle(0.0, 0.0, 2.0);
        let area = core.total_area();
        assert!((area - std::f64::consts::PI * 4.0).abs() < 1e-9);
    }
}
