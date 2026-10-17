//! Importação e exportação de arquivos DXF (Drawing Exchange Format).
//! Implementação simplificada para a fase 1 — suporte a LINE, CIRCLE, ARC, POINT, POLYLINE.
//! Formato DXF R12/R2010 (texto ASCII).

use crate::entities::{CadEntity, Color, EntityProperties, LayerId, LineWeight, Linetype};
use crate::layer::{CadDocument, Layer, LayerTable};
use crate::math::{Bounds2, Point2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use thiserror::Error;

/// Erros de leitura/escrita DXF.
#[derive(Debug, Error)]
pub enum DxfError {
    #[error("Arquivo não encontrado: {0}")]
    NotFound(String),
    #[error("Erro de leitura DXF na linha {0}: {1}")]
    ParseError(usize, String),
    #[error("Entidade DXF não suportada: {0}")]
    UnsupportedEntity(String),
    #[error("Erro de IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Versão DXF não suportada: {0}")]
    UnsupportedVersion(String),
}

/// Código de grupo DXF (código + valor).
#[derive(Debug, Clone)]
struct GroupCode {
    code: i32,
    value: String,
}

/// Lê pares de código de grupo de um arquivo DXF.
fn read_groups<P: AsRef<Path>>(path: P) -> Result<Vec<GroupCode>, DxfError> {
    let file = fs::File::open(path).map_err(DxfError::Io)?;
    let reader = BufReader::new(file);
    let mut groups = Vec::new();
    let mut lines = reader.lines().peekable();
    let mut line_num = 0usize;

    while let Some(Ok(line)) = lines.next() {
        line_num += 1;
        let code_line = line.trim();
        if code_line.is_empty() {
            continue;
        }
        let code: i32 = match code_line.parse() {
            Ok(c) => c,
            Err(_) => {
                return Err(DxfError::ParseError(
                    line_num,
                    format!("código inválido: {}", code_line),
                ))
            }
        };
        let value_line = lines
            .next()
            .ok_or_else(|| DxfError::ParseError(line_num, "valor ausente".into()))?;
        line_num += 1;
        let value = value_line?.trim().to_string();
        groups.push(GroupCode { code, value });
    }

    Ok(groups)
}

/// Converte um grupo de códigos DXF em um documento CAD.
pub fn parse_dxf<P: AsRef<Path>>(path: P) -> Result<CadDocument, DxfError> {
    let groups = read_groups(path)?;
    let mut doc = CadDocument::new("Importado do DXF");
    let mut layers: HashMap<String, LayerId> = HashMap::new();
    let mut current_layer: Option<LayerId> = None;

    // Primeira passagem: ler camadas da tabela
    let mut i = 0;
    while i < groups.len() {
        let g = &groups[i];
        if g.code == 0 && g.value == "TABLE" {
            // Avança até encontrar a tabela de camadas
            i += 1;
            while i < groups.len() {
                let g2 = &groups[i];
                if g2.code == 2 && g2.value == "LAYER_TABLE" {
                    // Lê entradas da tabela de camadas
                    i += 1;
                    while i < groups.len() {
                        let g3 = &groups[i];
                        if g3.code == 0 && g3.value == "ENDTAB" {
                            break;
                        }
                        if g3.code == 0 && g3.value == "LAYER" {
                            // Lê uma entrada de camada
                            let mut layer_name = String::new();
                            let mut layer_color = Color::ByLayer;
                            let mut layer_on = true;
                            let mut layer_locked = false;
                            i += 1;
                            while i < groups.len() {
                                let g4 = &groups[i];
                                if g4.code == 0 {
                                    break;
                                }
                                match g4.code {
                                    2 => layer_name = g4.value.clone(),
                                    62 => {
                                        // código de cor
                                        if let Ok(idx) = g4.value.parse::<i32>() {
                                            layer_color = Color::Index(idx as u8);
                                        }
                                    }
                                    70 => {
                                        // flags (16 = bloqueado, 0 = normal)
                                        if let Ok(flags) = g4.value.parse::<i32>() {
                                            layer_locked = flags & 16 != 0;
                                        }
                                    }
                                    _ => {}
                                }
                                i += 1;
                            }
                            let mut layer = Layer::new(&layer_name);
                            layer.color = layer_color;
                            if layer_locked {
                                layer = layer.lock();
                            }
                            if !layer_on {
                                layer = layer.freeze();
                            }
                            let id = layer.id;
                            layers.insert(layer_name.clone(), id);
                            doc.layers.layers.push(layer);
                        }
                        i += 1;
                    }
                    break;
                }
                i += 1;
            }
            break;
        }
        i += 1;
    }

    // Define camada "0" se não existir
    if !layers.contains_key("0") {
        let layer0 = Layer::new("0");
        let id = layer0.id;
        layers.insert("0".to_string(), id);
        doc.layers.layers.push(layer0);
    }
    doc.layers.current_layer_id = *layers.get("0").unwrap_or(&Uuid::nil());

    // Segunda passagem: ler entidades
    let mut i = 0;
    while i < groups.len() {
        let g = &groups[i];
        if g.code == 0 {
            match g.value.as_str() {
                "LINE" => {
                    let mut start = Point2::ORIGIN;
                    let mut end = Point2::ORIGIN;
                    let mut layer_name = "0".to_string();
                    i += 1;
                    while i < groups.len() {
                        let g2 = &groups[i];
                        if g2.code == 0 {
                            break;
                        }
                        match g2.code {
                            8 => layer_name = g2.value.clone(),
                            10 => start.x = g2.value.parse().unwrap_or(0.0),
                            20 => start.y = g2.value.parse().unwrap_or(0.0),
                            11 => end.x = g2.value.parse().unwrap_or(0.0),
                            21 => end.y = g2.value.parse().unwrap_or(0.0),
                            _ => {}
                        }
                        i += 1;
                    }
                    let layer_id = *layers
                        .get(&layer_name)
                        .unwrap_or(&doc.layers.current_layer_id);
                    let line = crate::entities::Line::new(layer_id, start, end);
                    doc.add_entity(CadEntity::Line(line));
                }
                "CIRCLE" => {
                    let mut center = Point2::ORIGIN;
                    let mut radius: f64 = 0.0;
                    let mut layer_name = "0".to_string();
                    i += 1;
                    while i < groups.len() {
                        let g2 = &groups[i];
                        if g2.code == 0 {
                            break;
                        }
                        match g2.code {
                            8 => layer_name = g2.value.clone(),
                            10 => center.x = g2.value.parse().unwrap_or(0.0),
                            20 => center.y = g2.value.parse().unwrap_or(0.0),
                            40 => radius = g2.value.parse().unwrap_or(0.0),
                            _ => {}
                        }
                        i += 1;
                    }
                    let layer_id = *layers
                        .get(&layer_name)
                        .unwrap_or(&doc.layers.current_layer_id);
                    let circle = crate::entities::Circle::new(layer_id, center, radius);
                    doc.add_entity(CadEntity::Circle(circle));
                }
                "ARC" => {
                    let mut center = Point2::ORIGIN;
                    let mut radius: f64 = 0.0;
                    let mut start_angle: f64 = 0.0;
                    let mut end_angle: f64 = 0.0;
                    let mut layer_name = "0".to_string();
                    i += 1;
                    while i < groups.len() {
                        let g2 = &groups[i];
                        if g2.code == 0 {
                            break;
                        }
                        match g2.code {
                            8 => layer_name = g2.value.clone(),
                            10 => center.x = g2.value.parse().unwrap_or(0.0),
                            20 => center.y = g2.value.parse().unwrap_or(0.0),
                            40 => radius = g2.value.parse().unwrap_or(0.0),
                            50 => start_angle = g2.value.parse().unwrap_or(0.0),
                            51 => end_angle = g2.value.parse().unwrap_or(0.0),
                            _ => {}
                        }
                        i += 1;
                    }
                    let layer_id = *layers
                        .get(&layer_name)
                        .unwrap_or(&doc.layers.current_layer_id);
                    let arc = crate::entities::Arc::new(
                        layer_id,
                        center,
                        radius,
                        start_angle.to_radians(),
                        end_angle.to_radians(),
                        true,
                    );
                    doc.add_entity(CadEntity::Arc(arc));
                }
                "POINT" => {
                    let mut location = Point2::ORIGIN;
                    let mut layer_name = "0".to_string();
                    i += 1;
                    while i < groups.len() {
                        let g2 = &groups[i];
                        if g2.code == 0 {
                            break;
                        }
                        match g2.code {
                            8 => layer_name = g2.value.clone(),
                            10 => location.x = g2.value.parse().unwrap_or(0.0),
                            20 => location.y = g2.value.parse().unwrap_or(0.0),
                            _ => {}
                        }
                        i += 1;
                    }
                    let layer_id = *layers
                        .get(&layer_name)
                        .unwrap_or(&doc.layers.current_layer_id);
                    let pt = crate::entities::PointEntity::new(layer_id, location);
                    doc.add_entity(CadEntity::Point(pt));
                }
                "LWPOLYLINE" | "POLYLINE" => {
                    let mut vertices = Vec::new();
                    let mut layer_name = "0".to_string();
                    i += 1;
                    while i < groups.len() {
                        let g2 = &groups[i];
                        if g2.code == 0 {
                            break;
                        }
                        match g2.code {
                            8 => layer_name = g2.value.clone(),
                            10 => {
                                let x = g2.value.parse().unwrap_or(0.0);
                                // próximo grupo 20 é Y
                                i += 1;
                                if i < groups.len() {
                                    let g3 = &groups[i];
                                    if g3.code == 20 {
                                        let y = g3.value.parse().unwrap_or(0.0);
                                        vertices.push(Point2::new(x, y));
                                    }
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    let layer_id = *layers
                        .get(&layer_name)
                        .unwrap_or(&doc.layers.current_layer_id);
                    let poly = crate::entities::Polyline::new(layer_id, vertices, false);
                    doc.add_entity(CadEntity::Polyline(poly));
                }
                _ => {
                    // entidade não suportada, pula
                    i += 1;
                    while i < groups.len() {
                        if groups[i].code == 0 {
                            break;
                        }
                        i += 1;
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(doc)
}

/// Escreve um documento CAD como arquivo DXF.
pub fn write_dxf<P: AsRef<Path>>(doc: &CadDocument, path: P) -> Result<(), DxfError> {
    let file = fs::File::create(path)?;
    write_dxf_to_writer(doc, file)
}

/// Escreve um documento CAD em DXF para qualquer destino que implemente `Write`.
pub fn write_dxf_to_writer<W: Write>(doc: &CadDocument, mut writer: W) -> Result<(), DxfError> {
    // Cabeçalho DXF
    writeln!(writer, "  0")?;
    writeln!(writer, "SECTION")?;
    writeln!(writer, "  2")?;
    writeln!(writer, "HEADER")?;
    writeln!(writer, "  9")?;
    writeln!(writer, "$ACADVER")?;
    writeln!(writer, "  1")?;
    writeln!(writer, "AC1015")?;
    writeln!(writer, "  0")?;
    writeln!(writer, "ENDSEC")?;

    // TABELA DE CAMADAS
    writeln!(writer, "  0")?;
    writeln!(writer, "TABLE")?;
    writeln!(writer, "  2")?;
    writeln!(writer, "LAYER_TABLE")?;
    writeln!(writer, "  70")?;
    writeln!(writer, "{}", doc.layers.layers.len())?;

    for layer in &doc.layers.layers {
        writeln!(writer, "  0")?;
        writeln!(writer, "LAYER")?;
        writeln!(writer, "  2")?;
        writeln!(writer, "{}", layer.name)?;
        writeln!(writer, "  70")?;
        let flags = if layer.locked { 16 } else { 0 };
        writeln!(writer, "{}", flags)?;
        writeln!(writer, "  62")?;
        match layer.color {
            Color::Index(idx) => writeln!(writer, "{}", idx)?,
            _ => writeln!(writer, "7")?,
        }
    }

    writeln!(writer, "  0")?;
    writeln!(writer, "ENDTAB")?;

    // BLOCO DE ENTIDADES
    writeln!(writer, "  0")?;
    writeln!(writer, "SECTION")?;
    writeln!(writer, "  2")?;
    writeln!(writer, "ENTITIES")?;

    for entity in &doc.entities {
        write_entity_dxf(&mut writer, entity)?;
    }

    writeln!(writer, "  0")?;
    writeln!(writer, "ENDSEC")?;
    writeln!(writer, "  0")?;
    writeln!(writer, "EOF")?;

    Ok(())
}

/// Escreve uma entidade individual no formato DXF.
fn write_entity_dxf<W: Write>(writer: &mut W, entity: &CadEntity) -> Result<(), std::io::Error> {
    match entity {
        CadEntity::Line(l) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "LINE")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(l.base.layer_id))?;
            writeln!(writer, " 10")?;
            writeln!(writer, "{:.6}", l.start.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{:.6}", l.start.y)?;
            writeln!(writer, " 11")?;
            writeln!(writer, "{:.6}", l.end.x)?;
            writeln!(writer, " 21")?;
            writeln!(writer, "{:.6}", l.end.y)?;
        }
        CadEntity::Circle(c) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "CIRCLE")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(c.base.layer_id))?;
            writeln!(writer, " 10")?;
            writeln!(writer, "{:.6}", c.center.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{:.6}", c.center.y)?;
            writeln!(writer, " 40")?;
            writeln!(writer, "{:.6}", c.radius)?;
        }
        CadEntity::Arc(a) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "ARC")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(a.base.layer_id))?;
            writeln!(writer, " 10")?;
            writeln!(writer, "{:.6}", a.inner.center.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{:.6}", a.inner.center.y)?;
            writeln!(writer, " 40")?;
            writeln!(writer, "{:.6}", a.inner.radius)?;
            writeln!(writer, " 50")?;
            writeln!(writer, "{:.6}", a.inner.start_angle.to_degrees())?;
            writeln!(writer, " 51")?;
            writeln!(writer, "{:.6}", a.inner.end_angle.to_degrees())?;
        }
        CadEntity::Point(p) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "POINT")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(p.base.layer_id))?;
            writeln!(writer, " 10")?;
            writeln!(writer, "{:.6}", p.location.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{:.6}", p.location.y)?;
        }
        CadEntity::Polyline(p) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "LWPOLYLINE")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(p.base.layer_id))?;
            writeln!(writer, " 90")?;
            writeln!(writer, "{}", p.vertices.len())?;
            for v in &p.vertices {
                writeln!(writer, " 10")?;
                writeln!(writer, "{:.6}", v.x)?;
                writeln!(writer, " 20")?;
                writeln!(writer, "{:.6}", v.y)?;
            }
        }
        CadEntity::Text(t) => {
            writeln!(writer, "  0")?;
            writeln!(writer, "TEXT")?;
            writeln!(writer, "  8")?;
            writeln!(writer, "{}", layer_name_placeholder(t.base.layer_id))?;
            writeln!(writer, " 10")?;
            writeln!(writer, "{:.6}", t.position.x)?;
            writeln!(writer, " 20")?;
            writeln!(writer, "{:.6}", t.position.y)?;
            writeln!(writer, " 40")?;
            writeln!(writer, "{:.6}", t.height)?;
            writeln!(writer, "  1")?;
            writeln!(writer, "{}", t.content)?;
        }
        CadEntity::BlockReference(_) => {
            // Simplificado: não exporta blocos na v1
        }
    }
    Ok(())
}

/// Placeholder para nome da camada (precisa de uma tabela de lookup reverso).
fn layer_name_placeholder(_id: LayerId) -> String {
    "0".to_string() // simplificado para v1
}

/// Carrega um documento DXF de um arquivo.
pub fn load_dxf<P: AsRef<Path>>(path: P) -> Result<CadDocument, DxfError> {
    parse_dxf(path)
}

/// Salva um documento CAD como DXF.
pub fn save_dxf<P: AsRef<Path>>(doc: &CadDocument, path: P) -> Result<(), DxfError> {
    write_dxf(doc, path)
}

/// Bounds gerais do documento (para zoom extents).
pub fn document_bounds(doc: &CadDocument) -> Option<Bounds2> {
    let mut bounds: Option<Bounds2> = None;
    for e in &doc.entities {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entity;
    use std::io::Cursor;

    #[test]
    fn roundtrip_circle() {
        let mut doc = CadDocument::new("Teste");
        let circle =
            crate::entities::Circle::new(doc.layers.current_layer_id, Point2::new(10.0, 20.0), 5.0);
        let id = circle.id();
        doc.add_entity(CadEntity::Circle(circle));

        let mut buf = Vec::new();
        write_dxf_to_writer(&doc, &mut buf).unwrap();
        let content = String::from_utf8(buf).unwrap();
        assert!(content.contains("CIRCLE"));
        assert!(content.contains("10.000000"));
        assert!(content.contains("20.000000"));
        assert!(content.contains("5.000000"));

        let _ = id;
    }

    #[test]
    fn roundtrip_line() {
        let mut doc = CadDocument::new("Teste");
        let line = crate::entities::Line::new(
            doc.layers.current_layer_id,
            Point2::new(0.0, 0.0),
            Point2::new(100.0, 50.0),
        );
        doc.add_entity(CadEntity::Line(line));

        let mut buf = Vec::new();
        write_dxf_to_writer(&doc, &mut buf).unwrap();
        let content = String::from_utf8(buf).unwrap();
        assert!(content.contains("LINE"));
    }
}

use uuid::Uuid;
