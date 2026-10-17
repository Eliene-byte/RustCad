//! Armazenamento compacto do NanoCore — serialização binária eficiente
//! para carregamento/salvamento rápido de arquivos CAD.

use crate::nanocore::kernel::{NanoCore, NanoEntity, NanoLayer};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Cabeçalho do arquivo NanoStore (magic + versão + contagem).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoStoreHeader {
    pub magic: [u8; 4],        // "RCAD"
    pub version: u32,
    pub entity_count: u32,
    pub layer_count: u32,
    pub bounds_min_x: f64,
    pub bounds_min_y: f64,
    pub bounds_max_x: f64,
    pub bounds_max_y: f64,
}

impl NanoStoreHeader {
    pub const MAGIC: [u8; 4] = *b"RCAD";

    pub fn new(entity_count: u32, layer_count: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            version: 1,
            entity_count,
            layer_count,
            bounds_min_x: 0.0,
            bounds_min_y: 0.0,
            bounds_max_x: 0.0,
            bounds_max_y: 0.0,
        }
    }
}

/// Formato de armazenamento Nano — binário compacto para leitura/gravação rápida.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoStore {
    pub header: NanoStoreHeader,
    pub layers: Vec<NanoLayer>,
    pub entities: Vec<NanoEntity>,
}

impl NanoStore {
    pub const MAX_ENTITIES: usize = 1_000_000; // limite de segurança para notebooks fracos

    pub fn new() -> Self {
        Self {
            header: NanoStoreHeader::new(0, 0),
            layers: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn from_core(core: &NanoCore) -> Self {
        let bounds = core.total_bounds().unwrap_or_default();
        Self {
            header: NanoStoreHeader {
                magic: NanoStoreHeader::MAGIC,
                version: 1,
                entity_count: core.visible_entity_count() as u32,
                layer_count: core.layers.len() as u32,
                bounds_min_x: bounds.min.x,
                bounds_min_y: bounds.min.y,
                bounds_max_x: bounds.max.x,
                bounds_max_y: bounds.max.y,
            },
            layers: core.layers.clone(),
            entities: core.entities.iter().filter(|e| e.visible).copied().collect(),
        }
    }

    pub fn to_core(&self) -> NanoCore {
        let mut core = NanoCore::new();
        core.layers = self.layers.clone();
        if let Some(l) = self.layers.first() {
            core.current_layer_id = l.id;
        }
        for e in &self.entities {
            match e.entity_type {
                crate::nanocore::kernel::NanoEntityType::Line => {
                    core.add_line(e.data[0], e.data[1], e.data[2], e.data[3]);
                }
                crate::nanocore::kernel::NanoEntityType::Circle => {
                    core.add_circle(e.data[0], e.data[1], e.data[2]);
                }
                crate::nanocore::kernel::NanoEntityType::Point => {
                    core.add_point(e.data[0], e.data[1]);
                }
                _ => {}
            }
        }
        core
    }

    /// Serializa para bytes binários compactos (formato NanoStore).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Cabeçalho simples: magic + contagens
        buf.extend_from_slice(&NanoStoreHeader::MAGIC);
        buf.extend_from_slice(&1u32.to_le_bytes()); // versão
        buf.extend_from_slice(&(self.layers.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(self.entities.len() as u32).to_le_bytes());
        // Camadas
        for layer in &self.layers {
            buf.extend_from_slice(&layer.id.to_le_bytes());
            buf.extend_from_slice(&layer.name);
            buf.extend_from_slice(&layer.color.to_le_bytes());
            buf.push(if layer.visible { 1 } else { 0 });
            buf.push(if layer.locked { 1 } else { 0 });
        }
        // Entidades (estrutura binária densa)
        for entity in &self.entities {
            buf.extend_from_slice(&entity.id.to_le_bytes());
            buf.extend_from_slice(&entity.layer_id.to_le_bytes());
            buf.push(entity.entity_type as u8);
            buf.push(if entity.visible { 1 } else { 0 });
            for &d in &entity.data {
                buf.extend_from_slice(&d.to_le_bytes());
            }
            buf.extend_from_slice(&entity.bounds_min_x.to_le_bytes());
            buf.extend_from_slice(&entity.bounds_min_y.to_le_bytes());
            buf.extend_from_slice(&entity.bounds_max_x.to_le_bytes());
            buf.extend_from_slice(&entity.bounds_max_y.to_le_bytes());
        }
        buf
    }

    /// Deserializa de bytes binários.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 12 {
            return Err("Arquivo muito curto para ser um NanoStore válido".into());
        }
        if &data[0..4] != &NanoStoreHeader::MAGIC {
            return Err("Magic inválido — não é um arquivo NanoStore".into());
        }
        let _version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let layer_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let entity_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;

        let mut offset = 16;
        let mut layers = Vec::with_capacity(layer_count);
        for _ in 0..layer_count {
            if offset + 4 + 32 + 4 + 1 + 1 > data.len() {
                return Err("Dados de camada truncados".into());
            }
            let id = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            offset += 4;
            let name = data[offset..offset+32].try_into().unwrap_or([0u8; 32]);
            offset += 32;
            let color = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            offset += 4;
            let visible = data[offset] != 0;
            offset += 1;
            let locked = data[offset] != 0;
            offset += 1;
            layers.push(NanoLayer { id, name, color, visible, locked });
        }

        let mut entities = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            if offset + 4 + 4 + 1 + 1 + 64 + 32 > data.len() {
                return Err("Dados de entidade truncados".into());
            }
            let id = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            offset += 4;
            let layer_id = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
            offset += 4;
            let entity_type_byte = data[offset];
            offset += 1;
            let visible = data[offset] != 0;
            offset += 1;
            let mut data_arr = [0.0f64; 8];
            for i in 0..8 {
                let start = offset + i * 8;
                data_arr[i] = f64::from_le_bytes([data[start], data[start+1], data[start+2], data[start+3], data[start+4], data[start+5], data[start+6], data[start+7]]);
            }
            offset += 64;
            let bmin_x = f64::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3], data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
            offset += 8;
            let bmin_y = f64::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3], data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
            offset += 8;
            let bmax_x = f64::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3], data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
            offset += 8;
            let bmax_y = f64::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3], data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
            offset += 8;

            let entity_type = match entity_type_byte {
                0 => crate::nanocore::kernel::NanoEntityType::Line,
                1 => crate::nanocore::kernel::NanoEntityType::Circle,
                2 => crate::nanocore::kernel::NanoEntityType::Arc,
                3 => crate::nanocore::kernel::NanoEntityType::Point,
                4 => crate::nanocore::kernel::NanoEntityType::Polyline,
                5 => crate::nanocore::kernel::NanoEntityType::Text,
                _ => crate::nanocore::kernel::NanoEntityType::Line,
            };

            entities.push(NanoEntity {
                id,
                layer_id,
                entity_type,
                visible,
                data: data_arr,
                bounds_min_x: bmin_x,
                bounds_min_y: bmin_y,
                bounds_max_x: bmax_x,
                bounds_max_y: bmax_y,
            });
        }

        Ok(Self {
            header: NanoStoreHeader::new(entity_count as u32, layer_count as u32),
            layers,
            entities,
        })
    }

    /// Salva em um arquivo binário.
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let bytes = self.to_bytes();
        let mut file = std::fs::File::create(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    /// Carrega de um arquivo binário.
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        Self::from_bytes(&data)
    }

    /// Tamanho estimado em bytes do armazenamento.
    pub fn estimated_size_bytes(&self) -> usize {
        16 + self.layers.len() * 42 + self.entities.len() * 106
    }
}

impl Default for NanoStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanocore::kernel::NanoCore;

    #[test]
    fn roundtrip_store() {
        let mut core = NanoCore::new();
        core.add_line(0.0, 0.0, 10.0, 10.0);
        core.add_circle(5.0, 5.0, 3.0);
        let store = NanoStore::from_core(&core);
        let bytes = store.to_bytes();
        let restored = NanoStore::from_bytes(&bytes).unwrap();
        assert_eq!(restored.entities.len(), 2);
        assert_eq!(restored.layers.len(), 1);
        let core2 = restored.to_core();
        assert_eq!(core2.entity_count(), 2);
    }

    #[test]
    fn header_magic_check() {
        let store = NanoStore::new();
        let bytes = store.to_bytes();
        assert_eq!(&bytes[0..4], &NanoStoreHeader::MAGIC);
    }

    #[test]
    fn compact_size() {
        let store = NanoStore::new();
        // NanoStore vazio deve ser pequeno
        assert!(store.estimated_size_bytes() < 100);
    }
}
