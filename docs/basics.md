# Características Básicas

Este documento describe las características fundamentales de Wollok-rs.

## Variables

### Constantes (`const`)

Las constantes son inmutables una vez asignadas:

```wollok
const nombre = "Juan"
const edad = 25
const activo = true
```

### Variables mutables (`let`)

Las variables declaradas con `let` pueden cambiar su valor:

```wollok
let contador = 0
contador = contador + 1
```

## Literales

### Números

```wollok
const entero = 42
const decimal = 3.14
const negativo = -100
```

### Strings

Soporta comillas simples y dobles:

```wollok
const saludo = "Hola mundo"
const mensaje = 'Bienvenido'
```

### Booleanos

```wollok
const verdadero = true
const falso = false
```

### Null

```wollok
const vacio = null
```

## Comentarios

### Comentarios de línea

```wollok
// Este es un comentario de línea
const variable = 42 // Comentario al final
```

> **Nota**: Los comentarios de bloque (`/* */`) están planeados para futuras versiones.

## Operadores

### Asignación

```wollok
let variable = 10
variable = 20  // Reasignación
```

### Aritméticos (Planeados)

```wollok
const suma = 5 + 3
const resta = 10 - 4
const multiplicacion = 6 * 7
const division = 20 / 4
```

### Comparación (Planeados)

```wollok
const igual = 5 == 5
const diferente = 3 != 4
const mayor = 10 > 5
const menor = 3 < 8
```

## Estado de Implementación

| Característica | Estado | Notas |
|----------------|--------|-------|
| `const` | ✅ Implementado | Funcionando completamente |
| `let` | ✅ Implementado | Funcionando completamente |
| Números | ✅ Implementado | Enteros y decimales |
| Strings | ✅ Implementado | Comillas simples y dobles |
| Booleanos | ✅ Implementado | `true` y `false` |
| `null` | ✅ Implementado | Valor null |
| Comentarios `//` | ✅ Implementado | Comentarios de línea |
| Asignación `=` | ✅ Implementado | Para variables mutables |
| Operadores aritméticos | 🚧 En desarrollo | +, -, *, / |
| Operadores de comparación | 📋 Planeado | ==, !=, <, > |
| Comentarios de bloque | 📋 Planeado | `/* */` |
