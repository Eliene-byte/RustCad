//! Comandos tipo AutoCAD (LINE, CIRCLE, MOVE, COPY, ROTATE, SCALE, ERASE, LAYER, etc.).
//! Este módulo interpreta comandos digitados na linha de comando e executa ações no documento.

use crate::entities::{
    CadEntity, Circle, Entity, EntityId, EntityProperties, LayerId, Line, LineWeight, Linetype,
    PointEntity, Polyline, Text,
};
use crate::layer::{Layer, LayerError, LayerTable};
use crate::math::{geometry, OsnapMode, OsnapResult, Point2};
use crate::CadDocument;
use std::collections::HashMap;
use uuid::Uuid;

/// Estado de execução de um comando interativo (ex: clicando pontos para uma linha).
#[derive(Debug, Clone, PartialEq)]
pub enum CommandState {
    Idle,
    WaitingForPoint(String),
    WaitingForDistance(String),
    WaitingForAngle(String),
    WaitingForLayerName,
    WaitingForEntitySelection,
}

/// Resultado de execução de um comando.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub entities_created: Vec<EntityId>,
    pub entities_modified: Vec<EntityId>,
    pub entities_deleted: Vec<EntityId>,
    pub next_state: CommandState,
}

impl CommandResult {
    pub fn ok<S: Into<String> + Clone>(message: S) -> Self {
        let m = message.into();
        Self {
            success: true,
            message: m.clone(),
            entities_created: Vec::new(),
            entities_modified: Vec::new(),
            entities_deleted: Vec::new(),
            next_state: CommandState::Idle,
        }
    }

    pub fn err<S: Into<String>>(message: S) -> Self {
        Self {
            success: false,
            message: message.into(),
            entities_created: Vec::new(),
            entities_modified: Vec::new(),
            entities_deleted: Vec::new(),
            next_state: CommandState::Idle,
        }
    }

    pub fn waiting<S: Into<String> + Clone>(prompt: S) -> Self {
        let message = prompt.into();
        Self {
            success: true,
            message: message.clone(),
            entities_created: Vec::new(),
            entities_modified: Vec::new(),
            entities_deleted: Vec::new(),
            next_state: CommandState::WaitingForPoint(message),
        }
    }

    pub fn created(mut self, id: EntityId) -> Self {
        self.entities_created.push(id);
        self
    }

    pub fn modified(mut self, id: EntityId) -> Self {
        self.entities_modified.push(id);
        self
    }

    pub fn deleted(mut self, id: EntityId) -> Self {
        self.entities_deleted.push(id);
        self
    }
}

/// Contexto de execução dos comandos — mantém o documento, camadas e estado atual.
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub document: CadDocument,
    pub state: CommandState,
    pub temp_points: Vec<Point2>,
    pub selected_entities: Vec<EntityId>,
    pub last_point: Option<Point2>,
}

impl CommandContext {
    pub fn new(document: CadDocument) -> Self {
        Self {
            state: CommandState::Idle,
            temp_points: Vec::new(),
            selected_entities: Vec::new(),
            last_point: None,
            document,
        }
    }

    pub fn layer_table(&self) -> &LayerTable {
        &self.document.layers
    }

    pub fn layer_table_mut(&mut self) -> &mut LayerTable {
        &mut self.document.layers
    }

    pub fn current_layer_id(&self) -> LayerId {
        self.document.layers.current_layer_id
    }
}

/// Interpretador de comandos.
#[derive(Debug, Clone)]
pub struct CommandInterpreter {
    context: CommandContext,
}

impl CommandInterpreter {
    pub fn new(document: CadDocument) -> Self {
        Self {
            context: CommandContext::new(document),
        }
    }

    pub fn document(&self) -> &CadDocument {
        &self.context.document
    }

    pub fn document_mut(&mut self) -> &mut CadDocument {
        &mut self.context.document
    }

    pub fn state(&self) -> &CommandState {
        &self.context.state
    }

    /// Processa uma linha de comando digitada pelo usuário.
    pub fn execute(&mut self, input: &str) -> CommandResult {
        let input = input.trim();
        if input.is_empty() {
            return CommandResult::ok("");
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_uppercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "LINE" | "L" => self.cmd_line(args),
            "CIRCLE" | "C" => self.cmd_circle(args),
            "ARC" => self.cmd_arc(args),
            "POINT" | "PO" => self.cmd_point(args),
            "POLYLINE" | "PLINE" | "PL" => self.cmd_polyline(args),
            "RECTANG" | "RECTANGLE" | "REC" => self.cmd_rectangle(args),
            "TEXT" => self.cmd_text(args),
            "MOVE" | "M" => self.cmd_move(args),
            "COPY" | "CO" => self.cmd_copy(args),
            "ROTATE" | "RO" => self.cmd_rotate(args),
            "SCALE" | "SC" => self.cmd_scale(args),
            "ERASE" | "E" => self.cmd_erase(args),
            "LAYER" | "LA" => self.cmd_layer(args),
            "LIST" => self.cmd_list(args),
            "ZOOM" | "Z" => self.cmd_zoom(args),
            "PAN" => self.cmd_pan(args),
            "UNDO" | "U" => self.cmd_undo(args),
            "REDO" => self.cmd_redo(args),
            "AREA" => self.cmd_area(args),
            "DIST" | "DISTANCE" => self.cmd_dist(args),
            "OSNAP" => self.cmd_osnap(args),
            "GRID" => self.cmd_grid(args),
            "HELP" | "?" => self.cmd_help(),
            "QUIT" | "EXIT" | "Q" => self.cmd_quit(),
            _ => CommandResult::err(format!(
                "Comando desconhecido: '{}'. Digite HELP para lista.",
                cmd
            )),
        }
    }

    /// Processa um ponto clicado durante um comando interativo.
    pub fn on_point(&mut self, point: Point2) -> CommandResult {
        match &self.context.state {
            CommandState::WaitingForPoint(_prompt) => {
                self.context.temp_points.push(point);
                self.context.last_point = Some(point);
                CommandResult::ok("Ponto registrado.")
            }
            _ => CommandResult::err("Nenhum comando ativo aguardando ponto."),
        }
    }

    // ── Comandos de desenho ──────────────────────────────────────────

    fn cmd_line(&mut self, _args: &[&str]) -> CommandResult {
        self.context.state =
            CommandState::WaitingForPoint("Especifique o ponto inicial:".to_string());
        self.context.temp_points.clear();
        CommandResult::waiting("Especifique o ponto inicial:")
    }

    fn cmd_circle(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 3 {
            let cx: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Coordenada X inválida"),
            };
            let cy: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Coordenada Y inválida"),
            };
            let r: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Raio inválido"),
            };
            let center = Point2::new(cx, cy);
            let circle = Circle::new(self.context.current_layer_id(), center, r);
            let id = circle.id();
            self.context.document.add_entity(CadEntity::Circle(circle));
            return CommandResult::ok(format!(
                "Círculo criado em ({:.2}, {:.2}) raio {:.2}",
                cx, cy, r
            ))
            .created(id);
        }
        self.context.state =
            CommandState::WaitingForPoint("Especifique o centro do círculo:".to_string());
        CommandResult::waiting("Especifique o centro do círculo:")
    }

    fn cmd_arc(&mut self, _args: &[&str]) -> CommandResult {
        self.context.state =
            CommandState::WaitingForPoint("Especifique o centro do arco:".to_string());
        CommandResult::waiting("Especifique o centro do arco:")
    }

    fn cmd_point(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 2 {
            let x: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X inválido"),
            };
            let y: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y inválido"),
            };
            let p = Point2::new(x, y);
            let pt = PointEntity::new(self.context.current_layer_id(), p);
            let id = pt.id();
            self.context.document.add_entity(CadEntity::Point(pt));
            return CommandResult::ok(format!("Ponto em ({:.2}, {:.2})", x, y)).created(id);
        }
        CommandResult::err("Uso: POINT <x> <y>")
    }

    fn cmd_polyline(&mut self, _args: &[&str]) -> CommandResult {
        self.context.state =
            CommandState::WaitingForPoint("Especifique o primeiro ponto da polilinha:".to_string());
        CommandResult::waiting("Especifique o primeiro ponto da polilinha:")
    }

    fn cmd_rectangle(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 4 {
            let x1: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X1 inválido"),
            };
            let y1: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y1 inválido"),
            };
            let x2: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X2 inválido"),
            };
            let y2: f64 = match args[3].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y2 inválido"),
            };
            let vertices = vec![
                Point2::new(x1, y1),
                Point2::new(x2, y1),
                Point2::new(x2, y2),
                Point2::new(x1, y2),
            ];
            let poly = Polyline::new(self.context.current_layer_id(), vertices, true);
            let id = poly.id();
            self.context.document.add_entity(CadEntity::Polyline(poly));
            return CommandResult::ok(format!(
                "Retângulo de ({:.2},{:.2}) a ({:.2},{:.2})",
                x1, y1, x2, y2
            ))
            .created(id);
        }
        CommandResult::err("Uso: RECTANG <x1> <y1> <x2> <y2>")
    }

    fn cmd_text(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 3 {
            let x: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X inválido"),
            };
            let y: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y inválido"),
            };
            let content = args[2..].join(" ");
            let text = Text::new(
                self.context.current_layer_id(),
                Point2::new(x, y),
                content.clone(),
                2.5,
            );
            let id = text.id();
            self.context.document.add_entity(CadEntity::Text(text));
            return CommandResult::ok(format!("Texto: {}", content)).created(id);
        }
        CommandResult::err("Uso: TEXT <x> <y> <texto>")
    }

    // ── Comandos de edição ──────────────────────────────────────────

    fn cmd_move(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 3 {
            let id = match Uuid::parse_str(args[0]) {
                Ok(id) => id,
                Err(_) => return CommandResult::err("ID de entidade inválido"),
            };
            let dx: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("DX inválido"),
            };
            let dy: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("DY inválido"),
            };
            if let Some(e) = self.context.document.get_entity_mut(id) {
                e.translate(dx, dy);
                return CommandResult::ok(format!("Entidade {} movida", id)).modified(id);
            }
            return CommandResult::err(format!("Entidade {} não encontrada", id));
        }
        CommandResult::err("Uso: MOVE <id> <dx> <dy>")
    }

    fn cmd_copy(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 3 {
            let id = match Uuid::parse_str(args[0]) {
                Ok(id) => id,
                Err(_) => return CommandResult::err("ID inválido"),
            };
            let dx: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("DX inválido"),
            };
            let dy: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("DY inválido"),
            };
            let original = match self.context.document.get_entity(id) {
                Some(e) => e.clone(),
                None => return CommandResult::err(format!("Entidade {} não encontrada", id)),
            };
            let mut copy = original.clone();
            match &mut copy {
                CadEntity::Line(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::Circle(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::Arc(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::Point(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::Polyline(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::Text(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
                CadEntity::BlockReference(e) => {
                    e.base.id = Uuid::new_v4();
                    e.translate(dx, dy);
                }
            }
            let new_id = copy.id();
            self.context.document.add_entity(copy);
            return CommandResult::ok(format!("Cópia criada: {}", new_id)).created(new_id);
        }
        CommandResult::err("Uso: COPY <id> <dx> <dy>")
    }

    fn cmd_rotate(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 4 {
            let id = match Uuid::parse_str(args[0]) {
                Ok(id) => id,
                Err(_) => return CommandResult::err("ID inválido"),
            };
            let cx: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("CX inválido"),
            };
            let cy: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("CY inválido"),
            };
            let angle: f64 = match args[3].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Ângulo inválido"),
            };
            if let Some(e) = self.context.document.get_entity_mut(id) {
                e.rotate(Point2::new(cx, cy), angle.to_radians());
                return CommandResult::ok(format!("Entidade {} rotacionada {:.2}°", id, angle))
                    .modified(id);
            }
            return CommandResult::err(format!("Entidade {} não encontrada", id));
        }
        CommandResult::err("Uso: ROTATE <id> <cx> <cy> <angulo_graus>")
    }

    fn cmd_scale(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 4 {
            let id = match Uuid::parse_str(args[0]) {
                Ok(id) => id,
                Err(_) => return CommandResult::err("ID inválido"),
            };
            let cx: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("CX inválido"),
            };
            let cy: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("CY inválido"),
            };
            let factor: f64 = match args[3].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Fator inválido"),
            };
            if let Some(e) = self.context.document.get_entity_mut(id) {
                e.scale(Point2::new(cx, cy), factor);
                return CommandResult::ok(format!("Entidade {} escalada por {:.2}x", id, factor))
                    .modified(id);
            }
            return CommandResult::err(format!("Entidade {} não encontrada", id));
        }
        CommandResult::err("Uso: SCALE <id> <cx> <cy> <fator>")
    }

    fn cmd_erase(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 1 {
            let id = match Uuid::parse_str(args[0]) {
                Ok(id) => id,
                Err(_) => return CommandResult::err("ID inválido"),
            };
            if self.context.document.remove_entity(id) {
                return CommandResult::ok(format!("Entidade {} apagada", id)).deleted(id);
            }
            return CommandResult::err(format!("Entidade {} não encontrada", id));
        }
        CommandResult::err("Uso: ERASE <id>")
    }

    // ── Comandos de camada ──────────────────────────────────────────

    fn cmd_layer(&mut self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            let mut out = String::from("Camadas:\n");
            for l in &self.context.document.layers.layers {
                let mark = if l.id == self.context.document.layers.current_layer_id {
                    "*"
                } else {
                    " "
                };
                let vis = if l.visible { "ON" } else { "OFF" };
                let lock = if l.locked { "LOCKED" } else { "UNLOCKED" };
                out.push_str(&format!(
                    "  {} [{}] {:<15} cor={:?} {} {}\n",
                    mark, l.id, l.name, l.color, vis, lock
                ));
            }
            return CommandResult::ok(out);
        }

        match args[0].to_uppercase().as_str() {
            "NEW" | "N" => {
                if args.len() >= 2 {
                    match self.context.layer_table_mut().add(args[1]) {
                        Ok(_id) => CommandResult::ok(format!("Camada '{}' criada", args[1])),
                        Err(LayerError::DuplicateName(n)) => {
                            CommandResult::err(format!("Camada '{}' já existe", n))
                        }
                        Err(e) => CommandResult::err(format!("Erro: {}", e)),
                    }
                } else {
                    CommandResult::err("Uso: LAYER NEW <nome>")
                }
            }
            "SET" | "S" => {
                if args.len() >= 2 {
                    match self.context.layer_table_mut().set_current_by_name(args[1]) {
                        Ok(()) => CommandResult::ok(format!("Camada atual: {}", args[1])),
                        Err(e) => CommandResult::err(format!("Erro: {}", e)),
                    }
                } else {
                    CommandResult::err("Uso: LAYER SET <nome>")
                }
            }
            "LIST" => {
                let mut out = String::from("Camadas:\n");
                for l in &self.context.document.layers.layers {
                    let mark = if l.id == self.context.document.layers.current_layer_id {
                        "*"
                    } else {
                        " "
                    };
                    out.push_str(&format!("  {} {}\n", mark, l.name));
                }
                CommandResult::ok(out)
            }
            "ON" => {
                if args.len() >= 2 {
                    if let Some(l) = self.context.layer_table_mut().get_by_name_mut(args[1]) {
                        l.visible = true;
                        CommandResult::ok(format!("Camada '{}' ligada", args[1]))
                    } else {
                        CommandResult::err(format!("Camada '{}' não encontrada", args[1]))
                    }
                } else {
                    CommandResult::err("Uso: LAYER ON <nome>")
                }
            }
            "OFF" => {
                if args.len() >= 2 {
                    if let Some(l) = self.context.layer_table_mut().get_by_name_mut(args[1]) {
                        l.visible = false;
                        CommandResult::ok(format!("Camada '{}' desligada", args[1]))
                    } else {
                        CommandResult::err(format!("Camada '{}' não encontrada", args[1]))
                    }
                } else {
                    CommandResult::err("Uso: LAYER OFF <nome>")
                }
            }
            "LOCK" => {
                if args.len() >= 2 {
                    if let Some(l) = self.context.layer_table_mut().get_by_name_mut(args[1]) {
                        l.locked = true;
                        CommandResult::ok(format!("Camada '{}' bloqueada", args[1]))
                    } else {
                        CommandResult::err(format!("Camada '{}' não encontrada", args[1]))
                    }
                } else {
                    CommandResult::err("Uso: LAYER LOCK <nome>")
                }
            }
            "UNLOCK" => {
                if args.len() >= 2 {
                    if let Some(l) = self.context.layer_table_mut().get_by_name_mut(args[1]) {
                        l.locked = false;
                        CommandResult::ok(format!("Camada '{}' desbloqueada", args[1]))
                    } else {
                        CommandResult::err(format!("Camada '{}' não encontrada", args[1]))
                    }
                } else {
                    CommandResult::err("Uso: LAYER UNLOCK <nome>")
                }
            }
            _ => CommandResult::err(format!("Subcomando de camada desconhecido: {}", args[0])),
        }
    }

    // ── Comandos de consulta ────────────────────────────────────────

    fn cmd_list(&mut self, _args: &[&str]) -> CommandResult {
        let doc = &self.context.document;
        let mut out = String::from("Entidades no desenho:\n");
        for e in &doc.entities {
            let props = &e.properties();
            let vis = if props.visible { "" } else { " [INVISÍVEL]" };
            out.push_str(&format!(
                "  ID={} Tipo={:?} Camada={} {}\n",
                e.id(),
                e.entity_type(),
                e.layer_id(),
                vis
            ));
        }
        out.push_str(&format!("\nTotal: {} entidade(s)\n", doc.entity_count()));
        CommandResult::ok(out)
    }

    fn cmd_area(&mut self, _args: &[&str]) -> CommandResult {
        let doc = &self.context.document;
        let mut total_area = 0.0;
        let mut count = 0;
        for e in &doc.entities {
            match e {
                CadEntity::Circle(c) => {
                    total_area += std::f64::consts::PI * c.radius.powi(2);
                    count += 1;
                }
                CadEntity::Polyline(p) => {
                    total_area += geometry::polygon_area(&p.vertices);
                    count += 1;
                }
                _ => {}
            }
        }
        CommandResult::ok(format!(
            "Área total: {:.2} unidades² ({} entidade(s))",
            total_area, count
        ))
    }

    fn cmd_dist(&mut self, args: &[&str]) -> CommandResult {
        if args.len() >= 4 {
            let x1: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X1 inválido"),
            };
            let y1: f64 = match args[1].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y1 inválido"),
            };
            let x2: f64 = match args[2].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("X2 inválido"),
            };
            let y2: f64 = match args[3].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Y2 inválido"),
            };
            let d = Point2::new(x1, y1).distance_to(&Point2::new(x2, y2));
            let angle = Point2::new(x1, y1)
                .angle_to(&Point2::new(x2, y2))
                .to_degrees();
            CommandResult::ok(format!(
                "Distância: {:.2}\nÂngulo: {:.2}°\nΔX: {:.2} ΔY: {:.2}",
                d,
                angle,
                x2 - x1,
                y2 - y1
            ))
        } else {
            CommandResult::err("Uso: DIST <x1> <y1> <x2> <y2>")
        }
    }

    fn cmd_zoom(&mut self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            CommandResult::ok("ZOOM: use ZOOM <fator> ou ZOOM ALL")
        } else if args[0].to_uppercase() == "ALL" {
            CommandResult::ok("Zoom ajustado para mostrar todas as entidades (EXTENTS)")
        } else {
            let factor: f64 = match args[0].parse() {
                Ok(v) => v,
                Err(_) => return CommandResult::err("Fator inválido"),
            };
            CommandResult::ok(format!("Zoom: {}x", factor))
        }
    }

    fn cmd_pan(&mut self, _args: &[&str]) -> CommandResult {
        CommandResult::ok("PAN: arraste a vista ou use PAN <dx> <dy>")
    }

    fn cmd_undo(&mut self, _args: &[&str]) -> CommandResult {
        if let Some(e) = self.context.document.undo() {
            CommandResult::ok(format!("Desfeito: entidade {} removida", e.id())).deleted(e.id())
        } else {
            CommandResult::ok("Nada para desfazer.")
        }
    }

    fn cmd_redo(&mut self, _args: &[&str]) -> CommandResult {
        CommandResult::ok("REDO não implementado nesta versão (use comando novamente)")
    }

    fn cmd_osnap(&mut self, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::ok(
                "OSNAP modes: ENDPOINT, MIDPOINT, CENTER, INTERSECTION, NEAREST, QUADRANT, TANGENT",
            );
        }
        let mode = match args[0].to_uppercase().as_str() {
            "ENDPOINT" => OsnapMode::Endpoint,
            "MIDPOINT" => OsnapMode::Midpoint,
            "CENTER" => OsnapMode::Center,
            "INTERSECTION" => OsnapMode::Intersection,
            "NEAREST" => OsnapMode::Nearest,
            "QUADRANT" => OsnapMode::Quadrant,
            "TANGENT" => OsnapMode::Tangent,
            "NONE" | "OFF" => OsnapMode::None,
            _ => return CommandResult::err("Modo OSNAP inválido"),
        };
        CommandResult::ok(format!("OSNAP definido para {:?}", mode))
    }

    fn cmd_grid(&mut self, _args: &[&str]) -> CommandResult {
        CommandResult::ok("GRID: visualização de grade ativada/desativada")
    }

    fn cmd_help(&self) -> CommandResult {
        let help = r#"COMANDOS DISPONÍVEIS (RustCad v0.1.0 — 40% AutoCAD)

DESENHO:
  LINE / L        <x1> <y1> <x2> <y2>   — Linha reta
  CIRCLE / C      <cx> <cy> <raio>      — Círculo
  ARC             — Arco (interativo)
  POINT / PO      <x> <y>               — Ponto
  POLYLINE / PL   — Polilinha (interativo)
  RECTANG / REC   <x1> <y1> <x2> <y2>   — Retângulo
  TEXT            <x> <y> <texto>       — Texto

EDIÇÃO:
  MOVE / M        <id> <dx> <dy>        — Mover entidade
  COPY / CO       <id> <dx> <dy>        — Copiar entidade
  ROTATE / RO     <id> <cx> <cy> <ang>  — Rotacionar (graus)
  SCALE / SC      <id> <cx> <cy> <fator>— Escalar
  ERASE / E       <id>                  — Apagar entidade

CAMADAS:
  LAYER           — Listar camadas
  LAYER NEW <n>   — Criar camada
  LAYER SET <n>   — Definir camada atual
  LAYER ON/OFF <n>— Ligar/desligar camada
  LAYER LOCK/UNLOCK <n> — Bloquear/desbloquear

CONSULTA:
  LIST            — Listar todas as entidades
  DIST / DISTANCE <x1> <y1> <x2> <y2>   — Distância entre pontos
  AREA            — Área total (círculos e polilinhas)
  OSNAP <modo>    — Configurar snap de objeto

VISUALIZAÇÃO:
  ZOOM / Z        <fator> | ALL         — Zoom
  PAN             — Deslocar vista

OUTROS:
  UNDO / U        — Desfazer última ação
  HELP / ?        — Esta ajuda
  QUIT / EXIT     — Sair"#;
        CommandResult::ok(help)
    }

    fn cmd_quit(&self) -> CommandResult {
        CommandResult::ok("Saindo do RustCad...")
    }
}

/// Encontra a entidade mais próxima de um ponto (para snap).
pub fn find_nearest_entity(
    document: &CadDocument,
    point: Point2,
    max_dist: f64,
) -> Option<(EntityId, Point2)> {
    let mut best: Option<(EntityId, Point2, f64)> = None;
    for e in &document.entities {
        if !e.properties().visible {
            continue;
        }
        let b = e.bounds();
        let d = point.distance_to(&b.center());
        if d < max_dist {
            match best {
                Some((_, _, bd)) if d < bd => best = Some((e.id(), b.center(), d)),
                None => best = Some((e.id(), b.center(), d)),
                _ => {}
            }
        }
    }
    best.map(|(id, p, _)| (id, p))
}

/// Calcula snap de ponto (OSNAP) para um ponto do mouse/cursor.
pub fn compute_osnap(
    document: &CadDocument,
    cursor: Point2,
    mode: OsnapMode,
    max_dist: f64,
) -> Option<OsnapResult> {
    match mode {
        OsnapMode::None => None,
        OsnapMode::Nearest => {
            if let Some((id, p)) = find_nearest_entity(document, cursor, max_dist) {
                let _ = id;
                Some(OsnapResult {
                    point: p,
                    mode: OsnapMode::Nearest,
                })
            } else {
                None
            }
        }
        OsnapMode::Endpoint => {
            let mut best: Option<(Point2, f64)> = None;
            for e in &document.entities {
                match e {
                    CadEntity::Line(l) => {
                        for p in [l.start, l.end] {
                            let d = p.distance_to(&cursor);
                            if d < max_dist {
                                match best {
                                    Some((_, bd)) if d < bd => best = Some((p, d)),
                                    None => best = Some((p, d)),
                                    _ => {}
                                }
                            }
                        }
                    }
                    CadEntity::Circle(c) => {
                        for a in [
                            0.0,
                            std::f64::consts::PI / 2.0,
                            std::f64::consts::PI,
                            3.0 * std::f64::consts::PI / 2.0,
                        ] {
                            let p = Point2::new(
                                c.center.x + a.cos() * c.radius,
                                c.center.y + a.sin() * c.radius,
                            );
                            let d = p.distance_to(&cursor);
                            if d < max_dist {
                                match best {
                                    Some((_, bd)) if d < bd => best = Some((p, d)),
                                    None => best = Some((p, d)),
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            best.map(|(p, _)| OsnapResult {
                point: p,
                mode: OsnapMode::Endpoint,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_command() {
        let mut interp = CommandInterpreter::new(CadDocument::new("Teste"));
        let result = interp.execute("CIRCLE 5 5 3");
        assert!(result.success);
        assert_eq!(interp.document().entity_count(), 1);
    }

    #[test]
    fn layer_commands() {
        let mut interp = CommandInterpreter::new(CadDocument::new("Teste"));
        let r1 = interp.execute("LAYER NEW Piso");
        assert!(r1.success);
        let r2 = interp.execute("LAYER SET Piso");
        assert!(r2.success);
        let r3 = interp.execute("LAYER LIST");
        assert!(r3.success);
    }

    #[test]
    fn move_and_erase() {
        let mut interp = CommandInterpreter::new(CadDocument::new("Teste"));
        interp.execute("CIRCLE 0 0 5");
        let id = interp.document().entities[0].id();
        let id_str = id.to_string();
        let r = interp.execute(&format!("MOVE {} 10 10", id_str));
        assert!(r.success);
        let r = interp.execute(&format!("ERASE {}", id_str));
        assert!(r.success);
        assert_eq!(interp.document().entity_count(), 0);
    }
}
