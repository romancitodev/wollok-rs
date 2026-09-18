# Wollok-rs 🦀

[![GitHub Release](https://img.shields.io/github/v/release/romancitodev/wollok-rs)](https://github.com/romancitodev/wollok-rs/releases)
[![License](https://img.shields.io/github/license/romancitodev/wollok-rs)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)

Una implementación moderna de [Wollok](https://www.wollok.org/) en Rust, enfocada en la enseñanza de programación orientada a objetos.

## 📋 Tabla de Contenidos

- [🎯 Objetivo](#-objetivo)
- [🔥 Progreso reciente](#-progreso-reciente)
  - [⚠️ Comparación contra Wollok real](#️-comparación-contra-wollok-real-injusta-a-propósito)
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

## 🔥 Progreso reciente

Esto dejó de ser "solo parser": hay un compilador (`wollok-compiler`) que baja el AST a bytecode y una VM (`wollok-vm`) que lo corre de punta a punta. `cargo run -- archivo.wlk` ejecuta Wollok de verdad, no solo lo tokeniza.

- **Dispatch por inline cache polimórfica** (4 slots) para los sends a objetos, con el mismo criterio que usan los intérpretes adaptativos de verdad.
- **Los primitivos (`Int`/`Float`/`Bool`/`Str`) nunca son objetos de heap**: despachan contra una tabla nativa en Rust (`wollok_vm::native`), sin el costo de boxear cada entero.
- **La stdlib (`wollok-std`) es una crate aparte**, no está hardcodeada en la VM — cada método nativo es una función suelta, registrada por selector, para que crecer la librería no implique tocar el intérprete.
- **`console` es Wollok real**, compilado como cualquier `object`, con `println` declarado `native` y aridad variable.
- **`property`/`const property` generan getter y setter solos**, como en el Wollok original.

Ejemplos que corren de verdad (no pseudocódigo) en [`examples/`](examples/). Lo que falta (arrays/sets, closures, `inherits`/`super`, try/catch) está en [`docs/backlog.md`](docs/backlog.md); el porqué de cada decisión de diseño de la VM, en [`docs/vm-design.md`](docs/vm-design.md).

### ⚠️ Comparación contra Wollok real (injusta, a propósito)

Empezamos a medir [`wollok-rs`](.) contra `wollok run` (wollok-ts) sobre un corpus chico y aislado en [`bench/`](bench/) — mismos dos programas, escritos una vez en cada dialecto. Con `hyperfine` (proceso completo, arranque incluido):

| Programa | wollok-rs | wollok-ts | 
|---|---|---|
| `aritmetica` (30 sends) | ~14 ms | ~1.2 s |
| `mensajes` (30 sends cruzados) | ~14 ms | ~1.2 s |

Y por dentro, con `criterion` ([`benches/pipeline.rs`](benches/pipeline.rs), sin el print de la comparación anterior para no medir I/O):

| Fase | `aritmetica` | `mensajes` |
|---|---|---|
| parseo | ~0.68 ms | ~0.85 ms |
| + compilación | ~0.72 ms | ~0.97 ms |
| + ejecución | ~0.72 ms | ~0.98 ms |

**Por qué esto no es una comparación justa todavía:** wollok-ts arranca una VM de Node completa por cada corrida (~1.2 s son mayormente eso, no "correr el programa"); wollok-rs es un binario nativo que arranca casi gratis. Y sobre todo — **wollok-rs tiene casi nada de stdlib real** (unos pocos métodos de `Int`/`Bool`/`Str`, sin `Float`, sin colecciones, sin closures, sin herencia) contra una implementación completa y madura. Comparar tiempos hoy es más "cuánto tarda arrancar cada runtime" que "qué tan rápido corre Wollok" — va a volverse una comparación real a medida que la stdlib y el lenguaje se acerquen en cobertura.

De paso, armar este benchmark encontró un bug real: `wollok-lexer` capturaba un stack backtrace completo (`Backtrace::force_capture()`) en cada intento de match fallido durante el backtracking normal del parser — hasta ~23 veces por token. Cambiarlo a `Backtrace::capture()` (que respeta `RUST_BACKTRACE`, casi gratis si no está seteado) hizo el lexer **~36x más rápido**. Sin este tipo de benchmarks, ese bug seguía ahí.

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
├── crates/
│   ├── wollok-lexer/     # Tokenización y análisis léxico
│   ├── wollok-ast/       # Parser y AST
│   ├── wollok-common/    # Tipos y utilidades compartidas
│   ├── wollok-compiler/  # AST -> bytecode
│   ├── wollok-vm/        # La VM: heap, dispatch, el intérprete
│   ├── wollok-std/       # Stdlib nativa (Int/Float/Bool/Str, console)
│   └── wollok-cli/       # Interfaz de línea de comandos
└── src/                  # Ejecutable principal
```

### Flujo de Procesamiento

1. **Lexer** (`wollok-lexer`): Convierte texto a tokens
2. **Parser** (`wollok-ast`): Genera AST desde tokens
3. **Compilación** (`wollok-compiler`): AST a bytecode
4. **Ejecución** (`wollok-vm` + `wollok-std`): la VM corre el bytecode, despachando a la stdlib nativa cuando hace falta

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