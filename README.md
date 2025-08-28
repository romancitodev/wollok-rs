# Wollok-rs 🦀

[![GitHub Release](https://img.shields.io/github/v/release/romancitodev/wollok-rs)](https://github.com/romancitodev/wollok-rs/releases)
[![License](https://img.shields.io/github/license/romancitodev/wollok-rs)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)

Una implementación moderna de [Wollok](https://www.wollok.org/) en Rust, enfocada en la enseñanza de programación orientada a objetos.

## 📋 Tabla de Contenidos

- [🎯 Objetivo](#-objetivo)
- [✨ Características](#-características)
- [🚀 Instalación](#-instalación)
- [📚 Documentación del Lenguaje](#-documentación-del-lenguaje)
- [🏗️ Arquitectura](#️-arquitectura)
- [🤝 Contribuir](#-contribuir)
- [📜 Licencia](#-licencia)

## 🎯 Objetivo

Wollok-rs es una reimplementación de Wollok en Rust que busca:

- **Mejor rendimiento**: Aprovechando la velocidad y seguridad de Rust
- **Modularidad**: Arquitectura basada en crates independientes
- **Extensibilidad**: Diseño que facilita nuevas características
- **Compatibilidad**: Mantener la esencia pedagógica de Wollok original

## ✨ Características

### ✅ Implementadas

- [x] **Literales**: Números, strings, booleanos, null
- [x] **Colecciones**: Arrays `[1, 2, 3]` y Sets `#{1, 2, 3}`
- [x] **Objetos**: Declaración con `object` y cuerpo con propiedades/métodos
- [x] **Variables**: `const` (inmutables) y `let` (mutables)
- [x] **Propiedades**: Con `property` para getters/setters automáticos
- [x] **Métodos**: Con parámetros y cuerpos de bloque o inline
- [x] **Comentarios**: Soporte para `//` comentarios de línea
- [x] **Asignaciones**: Expresiones de asignación con `=`

### 🚧 En Desarrollo

- [ ] **Clases**: Declaración, constructores, herencia
- [ ] **Manejo de Errores**: `fallible` methods con `?` (propagación) y `!` (assertion)
- [ ] **Closures**: Sintaxis simple `() => expr` (sin llaves)
- [ ] **Imports**: Sistema de módulos e importaciones
- [ ] **Tests**: Framework de testing integrado
- [ ] **Wollok Game**: Librería para juegos

### 📋 Planificadas

- [ ] **Mixins**: Composición de comportamiento
- [ ] **Herencia**: `inherits` keyword y `super` calls
- [ ] **Polimorfismo**: Dynamic dispatch
- [ ] **REPL**: Consola interactiva
- [ ] **LSP**: Language Server Protocol
- [ ] **Debugging**: Soporte para debuggers

## 🚀 Instalación

### Prerrequisitos

- Rust 1.80+ ([Instalar Rust](https://rustup.rs/))

### Desde el código fuente

```bash
git clone https://github.com/romancitodev/wollok-rs.git
cd wollok-rs
cargo build --release
```

### Ejecución

```bash
# Ejecutar el ejemplo incluido
cargo run

# Con logs de debug
RUST_LOG=debug cargo run
```

## 📚 Documentación del Lenguaje

La documentación completa del lenguaje está organizada en módulos:

- **[Características Básicas](docs/basics.md)**: Variables, literales, comentarios
- **[Objetos](docs/objects.md)**: Declaración, propiedades, métodos
- **[Colecciones](docs/collections.md)**: Arrays, sets, operaciones
- **[Expresiones](docs/expressions.md)**: Asignaciones, llamadas a métodos
- **[Características Avanzadas](docs/advanced.md)**: Features planeadas para el futuro
- **[Comparación con Wollok Original](docs/comparison.md)**: Diferencias y similitudes

### Ejemplo Rápido

```wollok
object calculadora {
    property memoria = 0
    
    method sumar(a, b) = a + b
    
    method guardarEnMemoria(valor) {
        memoria = valor
    }
    
    method recuperarMemoria() = memoria
}

const resultado = calculadora.sumar(5, 3)
calculadora.guardarEnMemoria(resultado)
```

## 🏗️ Arquitectura

El proyecto está organizado en varios crates:

```
wollok-rs/
├── wollok-lexer/     # Tokenización y análisis léxico
├── wollok-ast/       # Parser y AST
├── wollok-common/    # Tipos y utilidades compartidas
├── wollok-cli/       # Interfaz de línea de comandos
└── src/              # Ejecutable principal
```

### Flujo de Procesamiento

1. **Lexer** (`wollok-lexer`): Convierte texto a tokens
2. **Parser** (`wollok-ast`): Genera AST desde tokens
3. **Análisis** (planeado): Validación semántica
4. **Evaluación** (planeado): Interpretación del código

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Por favor:

1. Fork el proyecto
2. Crea tu feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push al branch (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

### Desarrollo

```bash
# Ejecutar tests
cargo test

# Formatear código
cargo fmt

# Linter
cargo clippy

# Con just (opcional)
just test
just fmt
just clippy
```

## 📜 Licencia

Este proyecto está licenciado bajo [LICENSE](LICENSE) - ver el archivo para detalles.

---

**Nota**: Este proyecto está en desarrollo activo. Muchas características están planeadas pero no implementadas. Ver [issues](https://github.com/romancitodev/wollok-rs/issues) para el estado actual.