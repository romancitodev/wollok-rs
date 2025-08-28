# Objetos

Los objetos son la base de la programación orientada a objetos en Wollok.

## Declaración de Objetos

### Sintaxis básica

```wollok
object nombreDelObjeto {
    // Contenido del objeto
}
```

### Ejemplo completo

```wollok
object calculadora {
    property memoria = 0
    
    method sumar(a, b) = a + b
    
    method guardarEnMemoria(valor) {
        memoria = valor
    }
}
```

## Propiedades

Las propiedades son variables que pertenecen al objeto.

### Con `property`

Genera automáticamente getter y setter:

```wollok
object persona {
    property nombre = "Sin nombre"
    property edad = 0
}

// Uso implícito de getters/setters
persona.nombre() // Getter
persona.nombre("Juan") // Setter
```

### Con `const` y `let`

Variables internas del objeto:

```wollok
object contador {
    let valor = 0  // Variable mutable interna
    const limite = 100  // Constante interna
    
    method incrementar() {
        if (valor < limite) {
            valor = valor + 1
        }
    }
    
    method obtenerValor() = valor
}
```

## Métodos

### Métodos con cuerpo de bloque

```wollok
object procesador {
    method procesarDatos(datos) {
        const inicio = obtenerTiempo()
        let resultado = datos.transformar()
        resultado = resultado.validar()
        return resultado
    }
}
```

### Métodos inline (con `=`)

Para métodos simples de una expresión:

```wollok
object matematicas {
    method cuadrado(n) = n * n
    method doble(n) = n * 2
    method esPositivo(n) = n > 0
}
```

### Métodos con parámetros

```wollok
object utilidades {
    method saludar(nombre) = "Hola, " + nombre
    
    method calcular(a, b, operacion) {
        if (operacion == "suma") {
            return a + b
        } else if (operacion == "resta") {
            return a - b
        }
    }
}
```

### Métodos sin parámetros

```wollok
object configuracion {
    const version = "1.0.0"
    
    method obtenerVersion() = version
    method reiniciar() {
        // Lógica de reinicio
    }
}
```

## Características Especiales de Wollok-rs

### Diferencias con Wollok original

| Aspecto | Wollok Original | Wollok-rs |
|---------|----------------|-----------|
| Keywords mutables | `var` | `let` |
| Sintaxis de comentarios | `//` y `/* */` | Solo `//` (por ahora) |
| Self referencia | `self` | `self` (planeado) |

### Características únicas

- **Trailing commas**: Permitidas en listas de parámetros
- **Mejor error reporting**: Usando Ariadne para mensajes claros
- **Sintaxis más limpia**: Menos keywords obligatorios

## Ejemplo Completo

```wollok
object biblioteca {
    property libros = []
    const capacidadMaxima = 1000
    
    method agregarLibro(titulo, autor) {
        if (libros.size() < capacidadMaxima) {
            const libro = object {
                property titulo = titulo
                property autor = autor
                property prestado = false
                
                method prestar() {
                    prestado = true
                }
                
                method devolver() {
                    prestado = false
                }
            }
            libros.add(libro)
        }
    }
    
    method buscarPorAutor(autor) = 
        libros.filter({ libro => libro.autor() == autor })
    
    method librosDisponibles() = 
        libros.filter({ libro => !libro.prestado() })
}
```

## Estado de Implementación

| Característica | Estado | Notas |
|----------------|--------|-------|
| Declaración `object` | ✅ Implementado | Completamente funcional |
| Propiedades con `property` | ✅ Implementado | Con valores por defecto |
| Variables internas (`const`/`let`) | ✅ Implementado | Dentro de objetos |
| Métodos con bloque | ✅ Implementado | Con `{}` |
| Métodos inline | ✅ Implementado | Con `=` |
| Parámetros de métodos | ✅ Implementado | Con trailing commas |
| Self referencia | 📋 Planeado | `self` keyword |
| Herencia | 📋 Planeado | `inherits` |
| Super calls | 📋 Planeado | `super.metodo()` |
| Métodos abstractos | 📋 Planeado | En clases abstractas |
