//! NanoCore — Núcleo de geometria vetorial ultra-leve do RustCad.
//!
//! Tecnologia exclusiva que torna o RustCad extremamente leve (~140 MB) com
//! extrema performance em notebooks fracos.
//!
//! Princípios de design:
//! - Structs densas com layout de memória previsível (`#[repr(C)]`)
//! - Armazenamento contíguo de entidades (cache-friendly)
//! - Zero alocação por frame durante operações de consulta
//! - Índice espacial simples (grid uniforme) para buscas O(1) aproximado
//! - Sem dependência de GUI pesada — puro terminal/CLI

pub mod geometry;
pub mod kernel;
pub mod store;

pub use kernel::{NanoCore, NanoEntity, NanoLayer, NanoTransform};
pub use store::NanoStore;

/// Versão do NanoCore.
pub const NANOCORE_VERSION: &str = "1.0.0";

/// Alvo de tamanho do binário release (em MB).
pub const TARGET_BINARY_SIZE_MB: u32 = 140;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nanocore_version_is_set() {
        assert!(!NANOCORE_VERSION.is_empty());
    }

    #[test]
    fn target_size_is_140_mb() {
        assert_eq!(TARGET_BINARY_SIZE_MB, 140);
    }
}
