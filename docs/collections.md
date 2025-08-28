# Colecciones

Wollok-rs soporta dos tipos principales de colecciones: Arrays (listas ordenadas) y Sets (conjuntos únicos).

## Arrays

Los arrays son colecciones ordenadas que permiten elementos duplicados.

### Sintaxis

```wollok
const numeros = [1, 2, 3, 4, 5]
const nombres = ["Juan", "María", "Pedro"]
const mixto = [1, "texto", true, null]
```

### Arrays anidados

```wollok
const matriz = [[1, 2], [3, 4], [5, 6]]
const tabla = [
    ["A", "B", "C"],
    ["1", "2", "3"]
]
```

### Arrays vacíos

```wollok
const lista = []
const otroArray = [[]]  // Array con un array vacío
```

### Trailing commas

Wollok-rs permite comas finales para mejor legibilidad:

```wollok
const elementos = [
    "primero",
    "segundo",
    "tercero",  // <- Coma final permitida
]
```

## Sets

Los sets son colecciones no ordenadas de elementos únicos.

### Sintaxis

```wollok
const numeros = #{1, 2, 3, 4, 5}
const colores = #{"rojo", "verde", "azul"}
const mixto = #{1, "texto", true}
```

### Sets anidados

```wollok
const conjuntos = #{#{1, 2}, #{3, 4}}
```

### Sets vacíos

```wollok
const vacio = #{}
```

### Características únicas

- **No duplicados**: `#{1, 2, 2, 3}` resulta en `#{1, 2, 3}`
- **Sin orden garantizado**: El orden de iteración puede variar

## Operaciones con Colecciones (Planeadas)

### Métodos de Array

```wollok
const numeros = [1, 2, 3, 4, 5]

// Acceso por índice
const primer = numeros.get(0)

// Tamaño
const tamaño = numeros.size()

// Agregar elementos
numeros.add(6)

// Filtrar
const pares = numeros.filter({ n => n.even() })

// Mapear
const dobles = numeros.map({ n => n * 2 })

// Reducir
const suma = numeros.sum()
```

### Métodos de Set

```wollok
const conjunto = #{1, 2, 3}

// Agregar elemento
conjunto.add(4)

// Verificar pertenencia
const contiene = conjunto.contains(2)

// Unión
const union = conjunto.union(#{4, 5, 6})

// Intersección
const inter = conjunto.intersection(#{2, 3, 4})
```

## Iteración (Planeada)

### Con forEach

```wollok
const numeros = [1, 2, 3]
numeros.forEach({ n => console.println(n) })
```

### Con closures

```wollok
const pares = [1, 2, 3, 4, 5].filter({ n => n % 2 == 0 })
const suma = [1, 2, 3].fold(0, { acc, n => acc + n })
```

## Conversiones (Planeadas)

### Array a Set

```wollok
const array = [1, 2, 2, 3]
const set = array.asSet()  // #{1, 2, 3}
```

### Set a Array

```wollok
const set = #{3, 1, 2}
const array = set.asList()  // [1, 2, 3] (orden puede variar)
```

## Ejemplos Prácticos

### Lista de tareas

```wollok
object listaTareas {
    property tareas = []
    
    method agregarTarea(descripcion) {
        const tarea = object {
            property descripcion = descripcion
            property completada = false
            
            method completar() {
                completada = true
            }
        }
        tareas.add(tarea)
    }
    
    method tareasCompletadas() = 
        tareas.filter({ t => t.completada() })
    
    method tareasPendientes() = 
        tareas.filter({ t => !t.completada() })
}
```

### Conjunto de etiquetas

```wollok
object articulo {
    property titulo = ""
    property etiquetas = #{}
    
    method agregarEtiqueta(etiqueta) {
        etiquetas.add(etiqueta)
    }
    
    method tieneEtiqueta(etiqueta) = 
        etiquetas.contains(etiqueta)
    
    method etiquetasComunes(otroArticulo) = 
        etiquetas.intersection(otroArticulo.etiquetas())
}
```

## Comparación con Wollok Original

| Aspecto | Wollok Original | Wollok-rs |
|---------|----------------|-----------|
| Arrays | `[1, 2, 3]` | `[1, 2, 3]` ✅ |
| Sets | `#{1, 2, 3}` | `#{1, 2, 3}` ✅ |
| Trailing commas | No soportado | ✅ Soportado |
| Arrays vacíos | `[]` | `[]` ✅ |
| Sets vacíos | `#{}` | `#{}` ✅ |
| Anidamiento | Soportado | ✅ Soportado |

## Estado de Implementación

| Característica | Estado | Notas |
|----------------|--------|-------|
| Array literal `[]` | ✅ Implementado | Completamente funcional |
| Set literal `#{}` | ✅ Implementado | Completamente funcional |
| Arrays anidados | ✅ Implementado | `[[1, 2], [3, 4]]` |
| Sets anidados | ✅ Implementado | `#{#{1}, #{2}}` |
| Trailing commas | ✅ Implementado | En arrays y sets |
| Arrays vacíos | ✅ Implementado | `[]` |
| Sets vacíos | ✅ Implementado | `#{}` |
| Métodos de Array | 📋 Planeado | `.size()`, `.get()`, etc. |
| Métodos de Set | 📋 Planeado | `.add()`, `.contains()`, etc. |
| Iteración con closures | 📋 Planeado | `.forEach()`, `.map()`, etc. |
| Conversiones | 📋 Planeado | `.asSet()`, `.asList()` |
