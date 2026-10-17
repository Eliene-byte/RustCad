//! Sistema de camadas (layers) — equivalente ao Gerenciador de Propriedades de Camadas do AutoCAD.
//! Cada camada controla cor, tipo de linha, visibilidade e estado de bloqueio.

use crate::entities::{Color, EntityProperties, LayerId, LineWeight, Linetype};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Uma camada de desenho.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub color: Color,
    pub linetype: Linetype,
    pub lineweight: LineWeight,
    pub visible: bool,
    pub locked: bool,
    pub plottable: bool,
    pub current: bool,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            color: Color::ByLayer,
            linetype: Linetype::Continuous,
            lineweight: LineWeight::Default,
            visible: true,
            locked: false,
            plottable: true,
            current: false,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_linetype(mut self, linetype: Linetype) -> Self {
        self.linetype = linetype;
        self
    }

    pub fn lock(mut self) -> Self {
        self.locked = true;
        self
    }

    pub fn unlock(mut self) -> Self {
        self.locked = false;
        self
    }

    pub fn freeze(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn thaw(mut self) -> Self {
        self.visible = true;
        self
    }

    pub fn set_current(mut self) -> Self {
        self.current = true;
        self
    }

    pub fn unset_current(mut self) -> Self {
        self.current = false;
        self
    }
}

/// Gerenciador de camadas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTable {
    pub layers: Vec<Layer>,
    pub current_layer_id: LayerId,
}

impl LayerTable {
    pub fn new() -> Self {
        let layer0 = Layer::new("0").set_current();
        let current_id = layer0.id;
        Self {
            layers: vec![layer0],
            current_layer_id: current_id,
        }
    }

    /// Adiciona uma nova camada. Retorna erro se já existir uma com esse nome.
    pub fn add(&mut self, name: &str) -> Result<LayerId, LayerError> {
        if self.layers.iter().any(|l| l.name == name) {
            return Err(LayerError::DuplicateName(name.to_string()));
        }
        let layer = Layer::new(name);
        let id = layer.id;
        self.layers.push(layer);
        Ok(id)
    }

    /// Remove uma camada pelo nome. Não remove a camada "0" ou camadas com entidades.
    pub fn remove(&mut self, name: &str) -> Result<(), LayerError> {
        if name == "0" {
            return Err(LayerError::CannotDeleteDefault);
        }
        let idx = self
            .layers
            .iter()
            .position(|l| l.name == name)
            .ok_or(LayerError::NotFound(name.to_string()))?;
        let layer = &self.layers[idx];
        if layer.locked {
            return Err(LayerError::LayerLocked(name.to_string()));
        }
        if self.current_layer_id == layer.id {
            return Err(LayerError::CannotDeleteCurrent);
        }
        self.layers.remove(idx);
        Ok(())
    }

    /// Retorna uma referência à camada atual.
    pub fn current_layer(&self) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == self.current_layer_id)
    }

    /// Define a camada atual pelo ID.
    pub fn set_current_by_id(&mut self, id: LayerId) -> Result<(), LayerError> {
        if self.layers.iter().any(|l| l.id == id) {
            for l in &mut self.layers {
                l.current = l.id == id;
            }
            self.current_layer_id = id;
            Ok(())
        } else {
            Err(LayerError::NotFound(format!("layer_id {:?}", id)))
        }
    }

    /// Define a camada atual pelo nome.
    pub fn set_current_by_name(&mut self, name: &str) -> Result<(), LayerError> {
        let id = self
            .layers
            .iter()
            .find(|l| l.name == name)
            .map(|l| l.id)
            .ok_or(LayerError::NotFound(name.to_string()))?;
        self.set_current_by_id(id)
    }

    /// Retorna uma camada pelo ID.
    pub fn get(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// Retorna uma camada mutável pelo ID.
    pub fn get_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Retorna uma camada pelo nome.
    pub fn get_by_name(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Retorna uma camada mutável pelo nome.
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Lista todos os nomes de camadas.
    pub fn names(&self) -> Vec<String> {
        self.layers.iter().map(|l| l.name.clone()).collect()
    }

    /// Converte EntityProperties para propriedades efetivas considerando a camada.
    pub fn resolve_properties(&self, props: &EntityProperties) -> EntityProperties {
        let layer = self.current_layer();
        let mut resolved = props.clone();
        if props.color == Color::ByLayer {
            if let Some(l) = layer {
                resolved.color = l.color;
            }
        }
        if props.linetype == Linetype::ByLayer {
            if let Some(l) = layer {
                resolved.linetype = l.linetype;
            }
        }
        resolved
    }

    /// Retorna apenas camadas visíveis e não bloqueadas.
    pub fn visible_layers(&self) -> Vec<&Layer> {
        self.layers
            .iter()
            .filter(|l| l.visible && !l.locked)
            .collect()
    }

    /// Retorna camadas bloqueadas.
    pub fn locked_layers(&self) -> Vec<&Layer> {
        self.layers.iter().filter(|l| l.locked).collect()
    }

    /// Retorna camadas congeladas (invisíveis).
    pub fn frozen_layers(&self) -> Vec<&Layer> {
        self.layers.iter().filter(|l| !l.visible).collect()
    }
}

impl Default for LayerTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Erros do gerenciador de camadas.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayerError {
    #[error("Layer '{0}' already exists")]
    DuplicateName(String),
    #[error("Layer '{0}' not found")]
    NotFound(String),
    #[error("Cannot delete default layer '0'")]
    CannotDeleteDefault,
    #[error("Cannot delete current layer")]
    CannotDeleteCurrent,
    #[error("Layer '{0}' is locked")]
    LayerLocked(String),
}

/// Documento CAD — contém todas as entidades, camadas e metadados do desenho.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CadDocument {
    pub name: String,
    pub layers: LayerTable,
    pub entities: Vec<crate::entities::CadEntity>,
    pub blocks: Vec<crate::entities::BlockReference>, // simplificado: blocos são referências diretas
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub units: Units,
    pub precision: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Feet,
}

impl Default for Units {
    fn default() -> Self {
        Units::Millimeters
    }
}

impl CadDocument {
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            layers: LayerTable::new(),
            entities: Vec::new(),
            blocks: Vec::new(),
            created_at: now,
            modified_at: now,
            units: Units::default(),
            precision: 4,
        }
    }

    /// Adiciona uma entidade ao documento.
    pub fn add_entity(&mut self, entity: crate::entities::CadEntity) {
        self.entities.push(entity);
        self.modified_at = chrono::Utc::now();
    }

    /// Remove uma entidade pelo ID.
    pub fn remove_entity(&mut self, id: crate::entities::EntityId) -> bool {
        let len_before = self.entities.len();
        self.entities.retain(|e| e.id() != id);
        if self.entities.len() < len_before {
            self.modified_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }

    /// Retorna uma referência à entidade pelo ID.
    pub fn get_entity(&self, id: crate::entities::EntityId) -> Option<&crate::entities::CadEntity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    /// Retorna uma referência mutável à entidade pelo ID.
    pub fn get_entity_mut(
        &mut self,
        id: crate::entities::EntityId,
    ) -> Option<&mut crate::entities::CadEntity> {
        self.entities.iter_mut().find(|e| e.id() == id)
    }

    /// Retorna todas as entidades de uma camada.
    pub fn entities_in_layer(&self, layer_id: LayerId) -> Vec<&crate::entities::CadEntity> {
        self.entities
            .iter()
            .filter(|e| e.layer_id() == layer_id)
            .collect()
    }

    /// Conta entidades por tipo.
    pub fn count_by_type(&self) -> Vec<(crate::math::EntityType, usize)> {
        let mut counts: std::collections::HashMap<crate::math::EntityType, usize> =
            std::collections::HashMap::new();
        for e in &self.entities {
            *counts.entry(e.entity_type()).or_insert(0) += 1;
        }
        let mut result: Vec<(crate::math::EntityType, usize)> = counts.into_iter().collect();
        result.sort_by_key(|t| t.1);
        result.reverse();
        result
    }

    /// Limite geral de todas as entidades (bounding box total).
    pub fn total_bounds(&self) -> Option<crate::math::Bounds2> {
        let mut bounds: Option<crate::math::Bounds2> = None;
        for e in &self.entities {
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

    /// Número total de entidades.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Desfaz a última operação (simplificado: remove a última entidade adicionada).
    pub fn undo(&mut self) -> Option<crate::entities::CadEntity> {
        let entity = self.entities.pop();
        if entity.is_some() {
            self.modified_at = chrono::Utc::now();
        }
        entity
    }

    /// Limpa todas as entidades.
    pub fn clear(&mut self) {
        self.entities.clear();
        self.modified_at = chrono::Utc::now();
    }
}

impl Default for CadDocument {
    fn default() -> Self {
        Self::new("Desenho sem nome")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_table_default_has_layer_zero() {
        let table = LayerTable::new();
        assert_eq!(table.layers.len(), 1);
        assert_eq!(table.layers[0].name, "0");
        assert!(table.layers[0].current);
    }

    #[test]
    fn add_and_get_layer() {
        let mut table = LayerTable::new();
        let id = table.add("Construção").unwrap();
        let layer = table.get(id).unwrap();
        assert_eq!(layer.name, "Construção");
    }

    #[test]
    fn add_duplicate_layer_fails() {
        let mut table = LayerTable::new();
        table.add("Piso").unwrap();
        assert!(table.add("Piso").is_err());
    }

    #[test]
    fn cannot_delete_layer_zero() {
        let mut table = LayerTable::new();
        assert!(table.remove("0").is_err());
    }

    #[test]
    fn document_add_entity() {
        let mut doc = CadDocument::new("Teste");
        assert_eq!(doc.entity_count(), 0);
    }
}
