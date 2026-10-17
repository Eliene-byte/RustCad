//! Entidades CAD (desenhos geométricos) — equivalente a "objetos" do AutoCAD.
//! Cada entidade tem um ID único, pertence a uma camada e possui propriedades visuais.

use crate::math::{Arc as MathArc, Bounds2, EntityType, LineSegment, Point2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Propriedades visuais de uma entidade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityProperties {
    pub color: Color,
    pub linetype: Linetype,
    pub linewidth: LineWeight,
    pub visible: bool,
}

impl Default for EntityProperties {
    fn default() -> Self {
        Self {
            color: Color::ByLayer,
            linetype: Linetype::Continuous,
            linewidth: LineWeight::Default,
            visible: true,
        }
    }
}

/// Cor de entidade (pode referenciar a camada ou ser uma cor RGB direta).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    ByLayer,
    ByBlock,
    Rgb(u8, u8, u8),
    Index(u8), // índice de cor AutoCAD (1-255)
}

impl Default for Color {
    fn default() -> Self {
        Color::ByLayer
    }
}

/// Tipo de linha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Linetype {
    Continuous,
    Dashed,
    Dotted,
    DashDot,
    ByLayer,
}

impl Default for Linetype {
    fn default() -> Self {
        Linetype::Continuous
    }
}

/// Espessura de linha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineWeight {
    Default,
    W0_05,
    W0_09,
    W0_13,
    W0_15,
    W0_18,
    W0_20,
    W0_25,
    W0_30,
    W0_35,
    W0_40,
    W0_50,
    W0_53,
    W0_60,
    W0_70,
    W0_80,
    W0_90,
    W1_00,
    W1_06,
    W1_20,
    W1_40,
    W1_58,
    W2_00,
    W2_11,
}

impl Default for LineWeight {
    fn default() -> Self {
        LineWeight::Default
    }
}

/// Identificador de camada.
pub type LayerId = Uuid;

/// Identificador de entidade.
pub type EntityId = Uuid;

/// Base para todas as entidades CAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityBase {
    pub id: EntityId,
    pub layer_id: LayerId,
    pub properties: EntityProperties,
}

/// Trait que toda entidade CAD deve implementar.
pub trait Entity: std::fmt::Debug + Send + Sync {
    fn id(&self) -> EntityId;
    fn layer_id(&self) -> LayerId;
    fn properties(&self) -> &EntityProperties;
    fn properties_mut(&mut self) -> &mut EntityProperties;
    fn set_layer(&mut self, layer_id: LayerId);
    fn bounds(&self) -> Bounds2;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn entity_type(&self) -> EntityType;
    fn translate(&mut self, dx: f64, dy: f64);
    fn rotate(&mut self, center: Point2, angle: f64);
    fn scale(&mut self, center: Point2, factor: f64);
}

/// Linha reta — comando LINE do AutoCAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub base: EntityBase,
    pub start: Point2,
    pub end: Point2,
}

impl Line {
    pub fn new(layer_id: LayerId, start: Point2, end: Point2) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            start,
            end,
        }
    }
}

impl Entity for Line {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        Bounds2::from_points(&[self.start, self.end]).unwrap_or_default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Line
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.start.x += dx;
        self.start.y += dy;
        self.end.x += dx;
        self.end.y += dy;
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        for p in [&mut self.start, &mut self.end] {
            let rel_x = p.x - center.x;
            let rel_y = p.y - center.y;
            p.x = center.x + rel_x * cos - rel_y * sin;
            p.y = center.y + rel_x * sin + rel_y * cos;
        }
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.start = Point2::new(
            center.x + (self.start.x - center.x) * factor,
            center.y + (self.start.y - center.y) * factor,
        );
        self.end = Point2::new(
            center.x + (self.end.x - center.x) * factor,
            center.y + (self.end.y - center.y) * factor,
        );
    }
}

/// Círculo — comando CIRCLE do AutoCAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub base: EntityBase,
    pub center: Point2,
    pub radius: f64,
}

impl Circle {
    pub fn new(layer_id: LayerId, center: Point2, radius: f64) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            center,
            radius,
        }
    }
}

impl Entity for Circle {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        let r = self.radius;
        Bounds2 {
            min: Point2::new(self.center.x - r, self.center.y - r),
            max: Point2::new(self.center.x + r, self.center.y + r),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Circle
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.center.x += dx;
        self.center.y += dy;
    }

    fn rotate(&mut self, _center: Point2, _angle: f64) {
        // Círculo é invariante à rotação em torno do próprio centro
        // Se rotacionar em torno de outro ponto, apenas move o centro
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.center = Point2::new(
            center.x + (self.center.x - center.x) * factor,
            center.y + (self.center.y - center.y) * factor,
        );
        self.radius *= factor;
    }
}

/// Arco — comando ARC do AutoCAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub base: EntityBase,
    pub inner: MathArc,
}

impl Arc {
    pub fn new(
        layer_id: LayerId,
        center: Point2,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    ) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            inner: MathArc::new(center, radius, start_angle, end_angle, ccw),
        }
    }
}

impl Entity for Arc {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        let mut bounds = Bounds2::empty();
        bounds.expand(self.inner.start_point());
        bounds.expand(self.inner.end_point());
        // Adiciona pontos cardeais se o arco os cobrir
        let start = self.inner.start_angle;
        let end = self.inner.end_angle;
        let ccw = self.inner.ccw;
        let angles = [
            0.0,
            std::f64::consts::PI / 2.0,
            std::f64::consts::PI,
            3.0 * std::f64::consts::PI / 2.0,
        ];
        for a in angles {
            let contains = if ccw {
                if start <= end {
                    a >= start && a <= end
                } else {
                    a >= start || a <= end
                }
            } else {
                if end <= start {
                    a >= end && a <= start
                } else {
                    a >= end || a <= start
                }
            };
            if contains {
                let p = self.inner.center.polar(a, self.inner.radius);
                bounds.expand(p);
            }
        }
        if bounds.min.x == f64::INFINITY {
            // fallback: apenas os pontos finais
            bounds = Bounds2::from_points(&[self.inner.start_point(), self.inner.end_point()])
                .unwrap_or_default();
        }
        bounds
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Arc
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.inner.center.x += dx;
        self.inner.center.y += dy;
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let rel_x = self.inner.center.x - center.x;
        let rel_y = self.inner.center.y - center.y;
        self.inner.center.x = center.x + rel_x * cos - rel_y * sin;
        self.inner.center.y = center.y + rel_x * sin + rel_y * cos;
        self.inner.start_angle += angle;
        self.inner.end_angle += angle;
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.inner.center = Point2::new(
            center.x + (self.inner.center.x - center.x) * factor,
            center.y + (self.inner.center.y - center.y) * factor,
        );
        self.inner.radius *= factor;
    }
}

/// Ponto — comando POINT do AutoCAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointEntity {
    pub base: EntityBase,
    pub location: Point2,
}

impl PointEntity {
    pub fn new(layer_id: LayerId, location: Point2) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            location,
        }
    }
}

impl Entity for PointEntity {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        Bounds2::from_points(&[self.location]).unwrap_or_default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Point
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.location.x += dx;
        self.location.y += dy;
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let rel_x = self.location.x - center.x;
        let rel_y = self.location.y - center.y;
        self.location.x = center.x + rel_x * cos - rel_y * sin;
        self.location.y = center.y + rel_x * sin + rel_y * cos;
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.location = Point2::new(
            center.x + (self.location.x - center.x) * factor,
            center.y + (self.location.y - center.y) * factor,
        );
    }
}

/// Polilinha — sequência de segmentos conectados.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    pub base: EntityBase,
    pub vertices: Vec<Point2>,
    pub closed: bool,
}

impl Polyline {
    pub fn new(layer_id: LayerId, vertices: Vec<Point2>, closed: bool) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            vertices,
            closed,
        }
    }

    pub fn segments(&self) -> Vec<LineSegment> {
        let mut segs = Vec::new();
        for i in 0..self.vertices.len() {
            let j = if i + 1 < self.vertices.len() {
                i + 1
            } else if self.closed {
                0
            } else {
                break;
            };
            segs.push(LineSegment::new(self.vertices[i], self.vertices[j]));
        }
        segs
    }
}

impl Entity for Polyline {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        Bounds2::from_points(&self.vertices).unwrap_or_default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Polyline
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        for v in &mut self.vertices {
            v.x += dx;
            v.y += dy;
        }
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        for v in &mut self.vertices {
            let rel_x = v.x - center.x;
            let rel_y = v.y - center.y;
            v.x = center.x + rel_x * cos - rel_y * sin;
            v.y = center.y + rel_x * sin + rel_y * cos;
        }
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        for v in &mut self.vertices {
            *v = Point2::new(
                center.x + (v.x - center.x) * factor,
                center.y + (v.y - center.y) * factor,
            );
        }
    }
}

/// Texto — comando TEXT do AutoCAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub base: EntityBase,
    pub position: Point2,
    pub content: String,
    pub height: f64,
    pub rotation: f64,
}

impl Text {
    pub fn new(layer_id: LayerId, position: Point2, content: String, height: f64) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            position,
            content,
            height,
            rotation: 0.0,
        }
    }
}

impl Entity for Text {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        // Aproximação simples: retângulo ao redor do texto
        let w = self.content.len() as f64 * self.height * 0.6;
        let h = self.height;
        Bounds2 {
            min: Point2::new(self.position.x, self.position.y - h),
            max: Point2::new(self.position.x + w, self.position.y),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::Text
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.position.x += dx;
        self.position.y += dy;
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let rel_x = self.position.x - center.x;
        let rel_y = self.position.y - center.y;
        self.position.x = center.x + rel_x * cos - rel_y * sin;
        self.position.y = center.y + rel_x * sin + rel_y * cos;
        self.rotation += angle;
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.position = Point2::new(
            center.x + (self.position.x - center.x) * factor,
            center.y + (self.position.y - center.y) * factor,
        );
        self.height *= factor;
    }
}

/// Referência de bloco (INSERT do AutoCAD).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockReference {
    pub base: EntityBase,
    pub block_name: String,
    pub insertion_point: Point2,
    pub scale: f64,
    pub rotation: f64,
}

impl BlockReference {
    pub fn new(layer_id: LayerId, block_name: String, insertion_point: Point2) -> Self {
        Self {
            base: EntityBase {
                id: Uuid::new_v4(),
                layer_id,
                properties: EntityProperties::default(),
            },
            block_name,
            insertion_point,
            scale: 1.0,
            rotation: 0.0,
        }
    }
}

impl Entity for BlockReference {
    fn id(&self) -> EntityId {
        self.base.id
    }

    fn layer_id(&self) -> LayerId {
        self.base.layer_id
    }

    fn properties(&self) -> &EntityProperties {
        &self.base.properties
    }

    fn properties_mut(&mut self) -> &mut EntityProperties {
        &mut self.base.properties
    }

    fn set_layer(&mut self, layer_id: LayerId) {
        self.base.layer_id = layer_id;
    }

    fn bounds(&self) -> Bounds2 {
        Bounds2::from_points(&[self.insertion_point]).unwrap_or_default()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn entity_type(&self) -> EntityType {
        EntityType::BlockReference
    }

    fn translate(&mut self, dx: f64, dy: f64) {
        self.insertion_point.x += dx;
        self.insertion_point.y += dy;
    }

    fn rotate(&mut self, center: Point2, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let rel_x = self.insertion_point.x - center.x;
        let rel_y = self.insertion_point.y - center.y;
        self.insertion_point.x = center.x + rel_x * cos - rel_y * sin;
        self.insertion_point.y = center.y + rel_x * sin + rel_y * cos;
        self.rotation += angle;
    }

    fn scale(&mut self, center: Point2, factor: f64) {
        self.insertion_point = Point2::new(
            center.x + (self.insertion_point.x - center.x) * factor,
            center.y + (self.insertion_point.y - center.y) * factor,
        );
        self.scale *= factor;
    }
}

/// Enum que encapsula qualquer entidade CAD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CadEntity {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Point(PointEntity),
    Polyline(Polyline),
    Text(Text),
    BlockReference(BlockReference),
}

impl CadEntity {
    pub fn id(&self) -> EntityId {
        match self {
            CadEntity::Line(e) => e.id(),
            CadEntity::Circle(e) => e.id(),
            CadEntity::Arc(e) => e.id(),
            CadEntity::Point(e) => e.id(),
            CadEntity::Polyline(e) => e.id(),
            CadEntity::Text(e) => e.id(),
            CadEntity::BlockReference(e) => e.id(),
        }
    }

    pub fn layer_id(&self) -> LayerId {
        match self {
            CadEntity::Line(e) => e.layer_id(),
            CadEntity::Circle(e) => e.layer_id(),
            CadEntity::Arc(e) => e.layer_id(),
            CadEntity::Point(e) => e.layer_id(),
            CadEntity::Polyline(e) => e.layer_id(),
            CadEntity::Text(e) => e.layer_id(),
            CadEntity::BlockReference(e) => e.layer_id(),
        }
    }

    pub fn properties(&self) -> &EntityProperties {
        match self {
            CadEntity::Line(e) => e.properties(),
            CadEntity::Circle(e) => e.properties(),
            CadEntity::Arc(e) => e.properties(),
            CadEntity::Point(e) => e.properties(),
            CadEntity::Polyline(e) => e.properties(),
            CadEntity::Text(e) => e.properties(),
            CadEntity::BlockReference(e) => e.properties(),
        }
    }

    pub fn properties_mut(&mut self) -> &mut EntityProperties {
        match self {
            CadEntity::Line(e) => e.properties_mut(),
            CadEntity::Circle(e) => e.properties_mut(),
            CadEntity::Arc(e) => e.properties_mut(),
            CadEntity::Point(e) => e.properties_mut(),
            CadEntity::Polyline(e) => e.properties_mut(),
            CadEntity::Text(e) => e.properties_mut(),
            CadEntity::BlockReference(e) => e.properties_mut(),
        }
    }

    pub fn set_layer(&mut self, layer_id: LayerId) {
        match self {
            CadEntity::Line(e) => e.set_layer(layer_id),
            CadEntity::Circle(e) => e.set_layer(layer_id),
            CadEntity::Arc(e) => e.set_layer(layer_id),
            CadEntity::Point(e) => e.set_layer(layer_id),
            CadEntity::Polyline(e) => e.set_layer(layer_id),
            CadEntity::Text(e) => e.set_layer(layer_id),
            CadEntity::BlockReference(e) => e.set_layer(layer_id),
        }
    }

    pub fn bounds(&self) -> Bounds2 {
        match self {
            CadEntity::Line(e) => e.bounds(),
            CadEntity::Circle(e) => e.bounds(),
            CadEntity::Arc(e) => e.bounds(),
            CadEntity::Point(e) => e.bounds(),
            CadEntity::Polyline(e) => e.bounds(),
            CadEntity::Text(e) => e.bounds(),
            CadEntity::BlockReference(e) => e.bounds(),
        }
    }

    pub fn entity_type(&self) -> EntityType {
        match self {
            CadEntity::Line(_) => EntityType::Line,
            CadEntity::Circle(_) => EntityType::Circle,
            CadEntity::Arc(_) => EntityType::Arc,
            CadEntity::Point(_) => EntityType::Point,
            CadEntity::Polyline(_) => EntityType::Polyline,
            CadEntity::Text(_) => EntityType::Text,
            CadEntity::BlockReference(_) => EntityType::BlockReference,
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        match self {
            CadEntity::Line(e) => e.translate(dx, dy),
            CadEntity::Circle(e) => e.translate(dx, dy),
            CadEntity::Arc(e) => e.translate(dx, dy),
            CadEntity::Point(e) => e.translate(dx, dy),
            CadEntity::Polyline(e) => e.translate(dx, dy),
            CadEntity::Text(e) => e.translate(dx, dy),
            CadEntity::BlockReference(e) => e.translate(dx, dy),
        }
    }

    pub fn rotate(&mut self, center: Point2, angle: f64) {
        match self {
            CadEntity::Line(e) => e.rotate(center, angle),
            CadEntity::Circle(e) => e.rotate(center, angle),
            CadEntity::Arc(e) => e.rotate(center, angle),
            CadEntity::Point(e) => e.rotate(center, angle),
            CadEntity::Polyline(e) => e.rotate(center, angle),
            CadEntity::Text(e) => e.rotate(center, angle),
            CadEntity::BlockReference(e) => e.rotate(center, angle),
        }
    }

    pub fn scale(&mut self, center: Point2, factor: f64) {
        match self {
            CadEntity::Line(e) => e.scale(center, factor),
            CadEntity::Circle(e) => e.scale(center, factor),
            CadEntity::Arc(e) => e.scale(center, factor),
            CadEntity::Point(e) => e.scale(center, factor),
            CadEntity::Polyline(e) => e.scale(center, factor),
            CadEntity::Text(e) => e.scale(center, factor),
            CadEntity::BlockReference(e) => e.scale(center, factor),
        }
    }
}

impl Default for Bounds2 {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_bounds() {
        let line = Line::new(
            Uuid::new_v4(),
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 10.0),
        );
        let b = line.bounds();
        assert!((b.min.x - 0.0).abs() < 1e-9);
        assert!((b.min.y - 0.0).abs() < 1e-9);
        assert!((b.max.x - 10.0).abs() < 1e-9);
        assert!((b.max.y - 10.0).abs() < 1e-9);
    }

    #[test]
    fn circle_bounds() {
        let circle = Circle::new(Uuid::new_v4(), Point2::new(5.0, 5.0), 3.0);
        let b = circle.bounds();
        assert!((b.min.x - 2.0).abs() < 1e-9);
        assert!((b.max.x - 8.0).abs() < 1e-9);
    }

    #[test]
    fn translate_entity() {
        let mut line = Line::new(Uuid::new_v4(), Point2::new(0.0, 0.0), Point2::new(1.0, 1.0));
        line.translate(5.0, 5.0);
        assert!((line.start.x - 5.0).abs() < 1e-9);
        assert!((line.end.x - 6.0).abs() < 1e-9);
    }
}
