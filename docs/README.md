# Documentación de Wollok-rs

Bienvenido a la documentación completa de Wollok-rs. Esta documentación está organizada por temas para facilitar el aprendizaje y la referencia.

## 📖 Guías por Nivel

### Principiante
1. **[Características Básicas](basics.md)** - Comienza aquí
   - Variables (`const`, `let`)
   - Literales (números, strings, booleanos)
   - Comentarios
   - Operadores básicos

2. **[Objetos](objects.md)** - El corazón de Wollok
   - Declaración de objetos
   - Propiedades y métodos
   - Ejemplos prácticos

### Intermedio
3. **[Colecciones](collections.md)** - Trabajando con datos
   - Arrays `[1, 2, 3]`
   - Sets `#{1, 2, 3}`
   - Operaciones (planeadas)

4. **[Expresiones](expressions.md)** - Combinando valores
   - Tipos de expresiones
   - Precedencia de operadores
   - Expresiones complejas

### Avanzado
5. **[Características Avanzadas](advanced.md)** - El futuro de Wollok-rs
   - Clases y herencia
   - Manejo de errores
   - Closures y programación funcional
   - Mixins
   - Wollok Game

## 🔄 Migración

6. **[Comparación con Wollok Original](comparison.md)** - Para usuarios existentes
   - Diferencias sintácticas
   - Plan de migración
   - Herramientas de transición

## 🎯 Referencia Rápida

### Sintaxis Esencial

```wollok
// Variables
const inmutable = 42
let mutable = "puede cambiar"

// Objetos
object miObjeto {
    property valor = 100
    
    method hacer() = "algo útil"
    
    method procesar(dato) {
        // lógica aquí
        return dato * 2
    }
}

// Colecciones
const lista = [1, 2, 3]
const conjunto = #{1, 2, 3}
```

### Estado de Implementación

| 🟢 Implementado | 🟡 En Desarrollo | 🔴 Planeado |
|-----------------|------------------|-------------|
| Variables | Operadores | Clases |
| Objetos | Llamadas a métodos | Herencia |
| Propiedades | | Closures |
| Métodos | | Testing |
| Colecciones | | Wollok Game |
| Comentarios | | Imports |

## 📋 Cómo Leer Esta Documentación

1. **Si eres nuevo en Wollok**: Comienza con [Características Básicas](basics.md)
2. **Si vienes de Wollok original**: Lee [Comparación](comparison.md) primero
3. **Si quieres ver ejemplos**: Cada sección tiene ejemplos prácticos
4. **Si buscas algo específico**: Usa el índice de cada documento

## 🤝 Contribuir a la Documentación

Esta documentación está en el repositorio y acepta contribuciones:

- Corrige errores de sintaxis o gramática
- Añade ejemplos más claros
- Sugiere mejores explicaciones
- Reporta secciones confusas

## 📚 Recursos Adicionales

- [Wollok Original](https://www.wollok.org/) - Sitio oficial
- [Repositorio GitHub](https://github.com/romancitodev/wollok-rs)
- [Issues y Sugerencias](https://github.com/romancitodev/wollok-rs/issues)

---

¿Encontraste algo confuso o incorrecto? ¡Abre un issue en GitHub!
