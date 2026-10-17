# RustCad

**Clone de código aberto do AutoCAD escrito em Rust.**

[![CI](https://github.com/Eliene-byte/RustCad/actions/workflows/ci.yml/badge.svg)](https://github.com/Eliene-byte/RustCad/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.99+-orange.svg)](https://rust-lang.org)

> Meta da versão 1.0: atingir **40% das funcionalidades principais do AutoCAD** — desenho 2D, edição, camadas, medição e arquivos DXF.

## O que é o RustCad

O RustCad é um motor CAD compatível com fluxos de trabalho do AutoCAD, escrito inteiramente em Rust. Ele roda no terminal (CLI interativa) e pode ser expandido com interface gráfica (GUI) nas próximas fases. A análise e renderização são locais, sem dependência de nuvem.

## Funcionalidades da v1 (40% do AutoCAD)

| Categoria | Funcionalidade | Status |
|-----------|---------------|--------|
| **Desenho 2D** | Linha (`LINE`) | ✅ |
| | Círculo (`CIRCLE`) | ✅ |
| | Arco (`ARC`) | ✅ |
| | Ponto (`POINT`) | ✅ |
| | Polilinha (`POLYLINE`) | ✅ |
| | Retângulo (`RECTANG`) | ✅ |
| | Texto (`TEXT`) | ✅ |
| **Edição** | Mover (`MOVE`) | ✅ |
| | Copiar (`COPY`) | ✅ |
| | Rotacionar (`ROTATE`) | ✅ |
| | Escalar (`SCALE`) | ✅ |
| | Apagar (`ERASE`) | ✅ |
| **Camadas** | Criar, listar, definir atual | ✅ |
| | Ligar/desligar (ON/OFF) | ✅ |
| | Bloquear/desbloquear (LOCK/UNLOCK) | ✅ |
| **Consulta** | Listar entidades (`LIST`) | ✅ |
| | Distância entre pontos (`DIST`) | ✅ |
| | Área (`AREA`) | ✅ |
| | Snap de objetos (`OSNAP`) | ✅ (parcial) |
| **Arquivos** | Exportar DXF | ✅ |
| | Importar DXF | ✅ |
| **Sessão** | Desfazer (`UNDO`) | ✅ |
| | Histórico de comandos | ✅ |

## Instalação

### Pré-requisitos

- Rust 1.99+ (ou superior)
- Cargo

```bash
# Instalar Rust (se ainda não tiver)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar o repositório
git clone https://github.com/Eliene-byte/RustCad.git
cd RustCad

# Compilar
cargo build --release

# Executar
cargo run
# ou diretamente:
./target/release/rustcad
```

### Via GitHub Actions (binários pré-compilados)

Os binários de release são gerados automaticamente em cada tag (`v*`) na aba **Releases** do repositório.

## Uso rápido

Inicie o RustCad:

```bash
cargo run
```

Exemplo de sessão:

```
[0] Comando: LAYER NEW Piso
Camada 'Piso' criada
[Piso] Comando: LAYER SET Piso
Camada atual: Piso
[Piso] Comando: CIRCLE 100 100 50
Círculo criado em (100.00, 100.00) raio 50.00
[Piso] Comando: LINE 0 0 200 200
Linha criada
[Piso] Comando: LIST
Entidades no desenho:
  ID=... Tipo=Circle Camada=...
  ID=... Tipo=Line Camada=...
Total: 2 entidade(s)
[Piso] Comando: DIST 0 0 100 100
Distância: 141.42
Ângulo: 45.00°
ΔX: 100.00 ΔY: 100.00
[Piso] Comando: SAIR
```

### Arquitetura do projeto

```
src/
  main.rs          # Ponto de entrada (CLI interativa com rustyline)
  lib.rs           # Módulo raiz e exportações
  math/            # Vetores 2D, pontos, transformações, bounding boxes, interseções
  entities/        # Line, Circle, Arc, Point, Polyline, Text, BlockReference
  layer/           # Camadas (LayerTable), cores, tipos de linha, bloqueio
  command/         # Interpretador de comandos tipo AutoCAD
  file/            # Importação/exportação DXF
.github/
  workflows/
    ci.yml         # Build, testes, lint e release multiplataforma
```

## Contribuindo

1. Faça um fork do repositório
2. Crie uma branch: `git checkout -b feature/nome-da-feature`
3. Faça suas alterações e adicione testes
4. Verifique: `cargo fmt --check`, `cargo clippy`, `cargo test`
5. Abra um Pull Request

## Roadmap

- **v0.1 (atual):** CLI, entidades 2D básicas, camadas, DXF, 40% do AutoCAD
- **v0.2:** Interface gráfica (egui/iced), snap de objetos completo, blocos (BLOCK/INSERT)
- **v0.3:** Suporte a 3D básico, coordenadas polares, reference externo (XREF)
- **v1.0:** 100% das funcionalidades essenciais do AutoCAD LT

## Licença

MIT License — veja o arquivo [LICENSE](LICENSE).

## Autoria

Projeto idealizado por [Eliene-byte](https://github.com/Eliene-byte). Construído com Rust e a comunidade open source.
