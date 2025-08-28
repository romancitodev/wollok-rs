# Expresiones

Las expresiones en Wollok-rs representan valores que pueden ser evaluados y combinados.

## Tipos de Expresiones

### Literales

Las expresiones más básicas son los valores literales:

```wollok
42          // Número entero
3.14        // Número decimal
"texto"     // String
true        // Booleano
null        // Valor nulo
```

### Identificadores

Referencias a variables, propiedades o métodos:

```wollok
const nombre = "Juan"
let edad = 25

// 'nombre' y 'edad' son expresiones identificador
```

### Expresiones de Asignación

Asignan valores a variables mutables:

```wollok
let contador = 0
contador = contador + 1  // Expresión de asignación
```

### Expresiones de Campo

Acceso a propiedades de objetos:

```wollok
object persona {
    property nombre = "Juan"
}

// persona.nombre() es una expresión de campo
const nombrePersona = persona.nombre()
```

## Expresiones Compuestas

### Llamadas a Métodos (Planeadas)

```wollok
object calculadora {
    method sumar(a, b) = a + b
}

const resultado = calculadora.sumar(5, 3)  // Expresión de llamada
```

### Expresiones Aritméticas (Planeadas)

```wollok
const suma = 5 + 3
const producto = 4 * 7
const division = 20 / 4
const potencia = 2 ** 8
```

### Expresiones de Comparación (Planeadas)

```wollok
const esIgual = 5 == 5
const esMayor = 10 > 5
const esMenorOIgual = 3 <= 8
```

### Expresiones Lógicas (Planeadas)

```wollok
const ambosVerdaderos = true && false
const algunoVerdadero = true || false
const negacion = !true
```

## Expresiones Complejas

### Condicionales (Planeadas)

```wollok
const mensaje = if (edad >= 18) "Adulto" else "Menor"
```

### Try Expressions (Planeadas)

Para manejo de errores:

```wollok
const resultado = try operacionRiesgosa()
```

### Closures (Planeadas)

Bloques de código como valores:

```wollok
const duplicar = n => n * 2
const numeros = [1, 2, 3].map(duplicar)
```

## Precedencia de Operadores (Planeada)

```wollok
// Paréntesis tienen mayor precedencia
const resultado1 = (2 + 3) * 4  // 20

// Sin paréntesis, multiplicación primero
const resultado2 = 2 + 3 * 4    // 14

// Orden de precedencia (mayor a menor):
// 1. () - Paréntesis
// 2. ** - Potencia
// 3. *, /, % - Multiplicación, división, módulo
// 4. +, - - Suma, resta
// 5. <, <=, >, >= - Comparación
// 6. ==, != - Igualdad
// 7. && - AND lógico
// 8. || - OR lógico
```

## Expresiones en Contextos

### En declaraciones de variables

```wollok
const resultado = 5 + 3 * 2  // La expresión se evalúa primero
let acumulador = 0
```

### En métodos

```wollok
object calculadora {
    method calcular(a, b) = a * b + 1  // Expresión como cuerpo
    
    method procesar(datos) {
        const procesado = datos.map({ x => x * 2 })  // Expresión en assignment
        return procesado.sum()  // Expresión en return
    }
}
```

### En condicionales (Planeadas)

```wollok
object evaluador {
    method evaluar(numero) {
        if (numero > 0) {
            return "positivo"
        } else if (numero < 0) {
            return "negativo"
        } else {
            return "cero"
        }
    }
}
```

## Expresiones Avanzadas (Planeadas)

### Operadores de Error Handling

```wollok
// Propagación de error (?)
fallible method procesamientoComplejo(datos) {
    const validados = validar(datos)?        // Si falla, propaga error
    const procesados = procesar(validados)?  // Si falla, propaga error
    return guardar(procesados)?              // Si falla, propaga error
}

// Assertion/Panic (!)
method operacionSegura(config) {
    const configuracion = cargarConfig(config)!  // Explota si falla
    return configuracion.valor
}
```

### Try Expressions

```wollok
const resultado = try operacionRiesgosa()
```

### Closures (Sintaxis Simplificada)

```wollok
const duplicar = n => n * 2
const sumar = (a, b) => a + b
const filtrar = lista => lista.filter(x => x > 0)
```

## Ejemplos Prácticos

### Calculadora de expresiones

```wollok
object evaluadorExpresiones {
    method evaluar(expresion) = expresion()
    
    method sumar(a, b) = { => a + b }
    method multiplicar(a, b) = { => a * b }
    
    method ejemplo() {
        const suma = self.evaluar(self.sumar(5, 3))
        const producto = self.evaluar(self.multiplicar(4, 7))
        return suma + producto
    }
}
```

### Procesador de datos

```wollok
object procesador {
    method procesar(datos) {
        // Cadena de expresiones
        return datos
            .filter(x => x > 0)
            .map(x => x * 2)
            .sum()
    }
    
    method transformar(valor, operaciones) {
        // Aplicar múltiples expresiones
        let resultado = valor
        operaciones.forEach(op => {
            resultado = op.apply(resultado)
        })
        return resultado
    }
}
```

## Estado de Implementación

| Característica | Estado | Notas |
|----------------|--------|-------|
| Literales | ✅ Implementado | Números, strings, booleanos, null |
| Identificadores | ✅ Implementado | Acceso a variables |
| Asignación (`=`) | ✅ Implementado | Para variables mutables |
| Campos de objeto | ✅ Implementado | Acceso a propiedades |
| Paréntesis | ✅ Implementado | Agrupación de expresiones |
| Arrays `[]` | ✅ Implementado | Como expresiones |
| Sets `#{}` | ✅ Implementado | Como expresiones |
| Operadores aritméticos | 🚧 En desarrollo | +, -, *, /, ** |
| Operadores de comparación | 📋 Planeado | ==, !=, <, >, <=, >= |
| Operadores lógicos | 📋 Planeado | &&, \|\|, ! |
| Llamadas a métodos | 📋 Planeado | obj.metodo(args) |
| Condicionales | 📋 Planeado | if-else expressions |
| Operador `?` (propagación) | 📋 Planeado | Para error handling |
| Operador `!` (assertion) | 📋 Planeado | Panic en runtime |
| Try expressions | 📋 Planeado | try expr |
| Closures | 📋 Planeado | () => expr (sin llaves) |
