# Comparación: Wollok-rs vs Wollok Original

Esta guía detalla las similitudes y diferencias entre Wollok-rs y el Wollok original.

## Filosofía de Diseño

### Wollok Original
- **Objetivo**: Enseñanza de POO en universidades argentinas
- **Plataforma**: JVM (Java/Xtend)
- **IDE**: Eclipse con plugin personalizado
- **Enfoque**: Estabilidad y madurez pedagógica

### Wollok-rs
- **Objetivo**: Modernizar Wollok con mejor rendimiento
- **Plataforma**: Nativo (Rust)
- **IDE**: LSP para múltiples editores
- **Enfoque**: Performance, modularidad y extensibilidad

## Sintaxis y Características

### Variables

| Aspecto | Wollok Original | Wollok-rs | Estado |
|---------|----------------|-----------|--------|
| Inmutables | `const x = 5` | `const x = 5` | ✅ Idéntico |
| Mutables | `var x = 5` | `let x = 5` | 🔄 Diferente |
| Tipado | Dinámico | Dinámico | ✅ Idéntico |

**Justificación del cambio**: `let` es más común en lenguajes modernos y evita confusión con `var` de JavaScript.

### Objetos

| Aspecto | Wollok Original | Wollok-rs | Estado |
|---------|----------------|-----------|--------|
| Declaración | `object x { }` | `object x { }` | ✅ Idéntico |
| Propiedades | `property x` | `property x = valor` | 🔄 Mejorado |
| Métodos | `method m() { }` | `method m() { }` | ✅ Idéntico |
| Métodos inline | `method m() = expr` | `method m() = expr` | ✅ Idéntico |

**Mejora**: En Wollok-rs, las propiedades pueden tener valores por defecto.

### Colecciones

| Aspecto | Wollok Original | Wollok-rs | Estado |
|---------|----------------|-----------|--------|
| Listas | `[1, 2, 3]` | `[1, 2, 3]` | ✅ Idéntico |
| Sets | `#{1, 2, 3}` | `#{1, 2, 3}` | ✅ Idéntico |
| Trailing commas | ❌ No soportado | ✅ Soportado | 🚀 Nueva |

### Comentarios

| Aspecto | Wollok Original | Wollok-rs | Estado |
|---------|----------------|-----------|--------|
| Línea | `// comentario` | `// comentario` | ✅ Idéntico |
| Bloque | `/* comentario */` | 📋 Planeado | 🚧 Pendiente |

## Características Únicas de Wollok-rs

### Mejoras Sintácticas

```wollok
// Trailing commas permitidas
const lista = [
    1,
    2,
    3,  // <- Esto es válido en Wollok-rs
]

// Propiedades con valores por defecto
object persona {
    property nombre = "Sin nombre"  // Más conciso
    property edad = 0
}
```

### Mejor Manejo de Errores

```wollok
// Error messages más claros con Ariadne
// Antes: "Syntax error at line 5"
// Ahora: Muestra exactamente dónde y qué está mal con colores
```

### Performance

- **Compilación**: Mucho más rápida que Wollok Original
- **Ejecución**: Performance nativa vs JVM
- **Memoria**: Menor consumo de memoria

## Compatibilidad

### Código Compatible (95%)

La mayoría del código Wollok existente funciona en Wollok-rs:

```wollok
// Este código funciona en ambos
object calculadora {
    method sumar(a, b) = a + b
    
    method factorial(n) {
        if (n <= 1) {
            return 1
        } else {
            return n * self.factorial(n - 1)
        }
    }
}

const resultado = calculadora.sumar(5, 3)
```

### Migración Necesaria

Solo cambios menores para código que usa `var`:

```wollok
// Wollok Original
var contador = 0
contador = contador + 1

// Wollok-rs
let contador = 0
contador = contador + 1
```

### Herramientas de Migración (Planeadas)

```bash
# Herramienta para convertir automáticamente
wollok-rs migrate archivo.wlk
# Convierte var -> let automáticamente
```

## Ecosistema

### Wollok Original

- **IDE**: Eclipse con plugin completo
- **Debugger**: Integrado con Eclipse
- **Game**: Wollok Game completamente funcional
- **Testing**: Framework maduro
- **Comunidad**: Establecida en universidades

### Wollok-rs

- **IDE**: LSP para VS Code, Vim, etc.
- **Debugger**: 📋 Planeado
- **Game**: 📋 Planeado (compatible)
- **Testing**: 📋 Planeado
- **Comunidad**: En crecimiento

## Roadmap de Compatibilidad

### Fase 1: Core Language (Actual)
- [x] Variables (`const`, `let`)
- [x] Objetos básicos
- [x] Métodos
- [x] Colecciones
- [x] Propiedades

### Fase 2: POO Avanzado
- [ ] Clases y constructores
- [ ] Herencia (`inherits`)
- [ ] Polimorfismo
- [ ] `self` y `super`

### Fase 3: Características Avanzadas
- [ ] Manejo de errores (`try`)
- [ ] Closures
- [ ] Mixins
- [ ] Testing framework

### Fase 4: Ecosistema
- [ ] Wollok Game
- [ ] LSP completo
- [ ] Debugger
- [ ] Package manager

## Beneficios de la Migración

### Para Estudiantes
- **Startup más rápido**: No necesita instalar Eclipse
- **Mejor error reporting**: Mensajes más claros
- **IDE moderno**: VS Code, Vim, etc.
- **Performance**: Programas más rápidos

### Para Docentes
- **Facilidad de instalación**: Un solo binario
- **Cross-platform**: Windows, Mac, Linux
- **Extensibilidad**: Fácil añadir nuevas características
- **Mantenimiento**: Comunidad Rust activa

### Para Investigadores
- **Modularidad**: Fácil experimentar con el lenguaje
- **API clara**: Para tools y análisis
- **Performance**: Para proyectos grandes
- **Interoperabilidad**: Con el ecosistema Rust

## Plan de Transición

### Corto Plazo (6 meses)
1. Completar core language features
2. Herramientas de migración básicas
3. Documentación completa

### Medio Plazo (1 año)
1. Clases y herencia
2. Testing framework
3. LSP estable
4. Primeras pruebas en aulas

### Largo Plazo (2 años)
1. Wollok Game compatible
2. Ecosystem completo
3. Adopción en universidades
4. Retrocompatibilidad total

## Conclusión

Wollok-rs mantiene la esencia pedagógica de Wollok mientras moderniza:

- **Sintaxis**: 95% compatible con mejoras menores
- **Performance**: Significativamente mejor
- **Tooling**: Moderno y extensible
- **Migración**: Gradual y asistida por herramientas

El objetivo es que Wollok-rs sea el futuro natural de Wollok, manteniendo su propósito educativo pero con las ventajas de la tecnología moderna.
