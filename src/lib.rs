//! RustCad — Motor CAD compatível com AutoCAD escrito em Rust.
//!
//! Fase 1 (meta: 40% das funcionalidades do AutoCAD):
//! - Entidades 2D: linha, círculo, arco, ponto, polilinha, retângulo, texto
//! - Sistema de camadas (layers) com cor, tipo de linha, visibilidade e bloqueio
//! - Comandos tipo AutoCAD via linha de comando interativa
//! - Snap de objetos (OSNAP), grid, medição (distância, área)
//! - Salvamento/carregamento DXF
//! - Undo/Redo
//!
//! Use `cargo run` para iniciar a interface de linha de comando.

pub mod command;
pub mod entities;
pub mod file;
pub mod layer;
pub mod math;

pub use command::{CommandInterpreter, CommandResult, CommandState};
pub use entities::{CadEntity, EntityId, LayerId};
pub use layer::{CadDocument, Layer, LayerError, LayerTable, Units};
pub use math::{
    Arc as MathArc, Bounds2, LineSegment, OsnapMode, OsnapResult, Point2, Transform2, Vector2,
};
