# Características Avanzadas

Este documento describe las características avanzadas planeadas para Wollok-rs.

## Clases y Herencia

### Declaración de Clases

```wollok
class Persona {
    property nombre
    property edad
    
    constructor(_nombre, _edad) {
        nombre = _nombre
        edad = _edad
    }
    
    method saludar() = "Hola, soy " + nombre
}
```

### Herencia

```wollok
class Estudiante inherits Persona {
    property universidad
    
    constructor(_nombre, _edad, _universidad) {
        super(_nombre, _edad)
        universidad = _universidad
    }
    
    override method saludar() = 
        super.saludar() + " y estudio en " + universidad
}
```

### Clases Abstractas

```wollok
abstract class Figura {
    abstract method area()
    abstract method perimetro()
    
    method mostrarInfo() {
        console.println("Área: " + area())
        console.println("Perímetro: " + perimetro())
    }
}

class Rectangulo inherits Figura {
    property ancho
    property alto
    
    constructor(_ancho, _alto) {
        ancho = _ancho
        alto = _alto
    }
    
    override method area() = ancho * alto
    override method perimetro() = 2 * (ancho + alto)
}
```

## Manejo de Errores

Wollok-rs implementa un sistema de manejo de errores estricto y seguro, inspirado en Rust.

### Regla Fundamental: Solo Fallibles en Fallibles

**Los métodos fallibles SOLO pueden ser llamados desde otros métodos fallibles:**

```wollok
// ❌ ERROR: No puedes llamar método fallible desde método normal
method calcularNormal(x, y) {
    const resultado = dividir(x, y)  // ERROR: dividir es fallible
    return resultado
}

// ✅ CORRECTO: Método fallible puede llamar otros fallibles
fallible method calcularFallible(x, y) {
    const resultado = dividir(x, y)  // OK: ambos son fallibles
    return resultado
}
```

### Operadores de Propagación y Assertion

#### Operador `?` - Propagación de Error

Propaga el error hacia arriba en la cadena de llamadas:

```wollok
fallible method dividir(a, b) {
    if (b == 0) {
        throw new DivisionPorCeroException()
    }
    return a / b
}

fallible method calcularComplejo(x, y, z) {
    const paso1 = dividir(x, y)?     // Si falla, propaga error
    const paso2 = dividir(paso1, z)? // Si falla, propaga error
    return paso2
}

// Equivalente a:
fallible method calcularComplejoExplicito(x, y, z) {
    const paso1 = try dividir(x, y)
    if (paso1.isError()) {
        throw paso1.error()
    }
    
    const paso2 = try dividir(paso1.value(), z)
    if (paso2.isError()) {
        throw paso2.error()
    }
    
    return paso2.value()
}
```

#### Operador `!` - Assertion (Panic en Runtime)

Fuerza la ejecución y explota si falla (como `.unwrap()` en Rust):

```wollok
// ⚠️ PELIGROSO: Solo usar cuando estás 100% seguro
method calcularConAssert(x, y) {
    // "Sé que y nunca será 0 en este contexto"
    const resultado = dividir(x, y)!  // Explota si y == 0
    return resultado
}

// Otro ejemplo
method obtenerConfiguracion() {
    const config = cargarConfiguracion()!  // Debe existir o explota
    return config
}
```

### Try Expressions

Para uso explícito del manejo de errores:

```wollok
fallible method procesarDatos(datos) {
    const validados = try validar(datos)
    if (validados.isError()) {
        return datos  // Usar datos sin validar como fallback
    }
    return validados.value()
}

method usarTryEnMetodoNormal(datos) {
    const resultado = try validar(datos)
    return resultado.orElse(datos)  // Fallback seguro
}
```

### Excepciones Personalizadas

```wollok
class ValidationError inherits Exception {
    property campo
    property valorInvalido
    
    constructor(_campo, _valor, _mensaje) {
        super(_mensaje)
        campo = _campo
        valorInvalido = _valor
    }
}

fallible method validarEdad(edad) {
    if (edad < 0) {
        throw new ValidationError("edad", edad, "La edad no puede ser negativa")
    }
    if (edad > 150) {
        throw new ValidationError("edad", edad, "La edad parece irreal")
    }
    return edad
}
```

### Comparación de Enfoques

```wollok
// 1. Método fallible que propaga errores
fallible method enfoqueFallible(datos) {
    const validados = validar(datos)?        // Propaga si falla
    const procesados = procesar(validados)?  // Propaga si falla
    return guardar(procesados)?              // Propaga si falla
}

// 2. Método normal que usa try para manejar errores
method enfoqueConTry(datos) {
    const validados = try validar(datos)
    if (validados.isError()) {
        console.println("Error en validación")
        return datos
    }
    
    const procesados = try procesar(validados.value())
    if (procesados.isError()) {
        console.println("Error en procesamiento")
        return validados.value()
    }
    
    const guardados = try guardar(procesados.value())
    return guardados.orElse(procesados.value())
}

// 3. Método normal que usa assertion (peligroso)
method enfoqueConAssert(datos) {
    const validados = validar(datos)!    // Explota si falla
    const procesados = procesar(validados)!  // Explota si falla
    return guardar(procesados)!              // Explota si falla
}
```

## Closures y Programación Funcional

Wollok-rs implementa closures simples y elegantes, similares a las arrow functions de JavaScript.

### Sintaxis Simplificada

**NO hay llaves `{}` - solo expresiones directas:**

```wollok
// ✅ CORRECTO: Sintaxis simple
const duplicar = n => n * 2
const saludar = nombre => "Hola " + nombre
const sumar = (a, b) => a + b
const obtenerFecha = () => new Date()

// ❌ INCORRECTO: No se soportan llaves
const duplicarMal = n => { n * 2 }  // ERROR
const sumarMal = (a, b) => { 
    return a + b  // ERROR
}
```

### Closures sin Parámetros

```wollok
const obtenerConstante = () => 42
const saludoGenerico = () => "Hola mundo"
const horaActual = () => new Date().toString()
```

### Closures con Un Parámetro

```wollok
const cuadrado = n => n * n
const esPositivo = x => x > 0
const longitud = texto => texto.length()
const primerCaracter = str => str.charAt(0)
```

### Closures con Múltiples Parámetros

```wollok
const sumar = (a, b) => a + b
const concatenar = (str1, str2) => str1 + str2
const mayor = (x, y) => if (x > y) x else y
const distancia = (x1, y1, x2, y2) => math.sqrt((x2-x1)**2 + (y2-y1)**2)
```

### Closures con Capture de Contexto

```wollok
object contador {
    let valor = 0
    
    method incrementador() = () => {
        valor = valor + 1
        return valor
    }
    
    method decrementador() = () => {
        valor = valor - 1
        return valor
    }
}

const inc = contador.incrementador()
inc()  // 1
inc()  // 2
```

### Uso con Colecciones

```wollok
const numeros = [1, 2, 3, 4, 5]

// Map
const dobles = numeros.map(n => n * 2)
const cuadrados = numeros.map(x => x * x)

// Filter
const pares = numeros.filter(n => n % 2 == 0)
const positivos = numeros.filter(x => x > 0)

// Reduce/Fold
const suma = numeros.fold(0, (acc, n) => acc + n)
const producto = numeros.fold(1, (acc, n) => acc * n)

// Combinaciones
const resultado = numeros
    .filter(n => n > 2)
    .map(n => n * 3)
    .fold(0, (sum, n) => sum + n)
```

### Funciones de Alto Orden

```wollok
object FunctionalUtils {
    method aplicarTres(funcion, valor) = 
        funcion(funcion(funcion(valor)))
    
    method componer(f, g) = x => f(g(x))
    
    method curry(funcion2) = a => b => funcion2(a, b)
    
    method flip(funcion2) = (a, b) => funcion2(b, a)
}

// Uso
const duplicar = n => n * 2
const resultado1 = FunctionalUtils.aplicarTres(duplicar, 5)  // 40

const sumar = (a, b) => a + b
const sumar5 = FunctionalUtils.curry(sumar)(5)
const resultado2 = sumar5(3)  // 8

const dividir = (a, b) => a / b
const dividirPor = FunctionalUtils.flip(dividir)
const mitad = dividirPor(2)
const resultado3 = mitad(10)  // 5
```

### Closures como Valores de Primera Clase

```wollok
object OperacionesMatematicas {
    const operaciones = #{
        "suma" -> ((a, b) => a + b),
        "resta" -> ((a, b) => a - b),
        "multiplicacion" -> ((a, b) => a * b),
        "division" -> ((a, b) => a / b)
    }
    
    method calcular(operacion, a, b) {
        const funcion = operaciones.get(operacion)
        return funcion(a, b)
    }
    
    method agregarOperacion(nombre, funcion) {
        operaciones.put(nombre, funcion)
    }
}

// Uso
OperacionesMatematicas.agregarOperacion("potencia", (a, b) => a ** b)
const resultado = OperacionesMatematicas.calcular("potencia", 2, 3)  // 8
```

### Limitaciones Intencionales

```wollok
// ❌ NO soportado: Múltiples statements
const complejo = n => {
    const temp = n * 2
    return temp + 1
}

// ✅ Alternativa: Usar métodos auxiliares
object Helpers {
    method procesoComplejo(n) {
        const temp = n * 2
        return temp + 1
    }
}
const complejo = n => Helpers.procesoComplejo(n)

// ❌ NO soportado: Control de flujo complejo
const condicionalComplejo = n => {
    if (n > 10) {
        return "grande"
    } else {
        return "pequeño"
    }
}

// ✅ Alternativa: Expresión ternaria o método auxiliar
const condicionalSimple = n => if (n > 10) "grande" else "pequeño"
```

## Mixins

### Definición de Mixins

```wollok
mixin Comparable {
    method <(otro) = self.compareTo(otro) < 0
    method <=(otro) = self.compareTo(otro) <= 0
    method >(otro) = self.compareTo(otro) > 0
    method >=(otro) = self.compareTo(otro) >= 0
    method ==(otro) = self.compareTo(otro) == 0
    
    // Método abstracto que debe implementar quien use el mixin
    abstract method compareTo(otro)
}

mixin Mostrable {
    method mostrar() {
        console.println(self.toString())
    }
    
    method mostrarDetallado() {
        console.println("=== " + self.toString() + " ===")
        console.println(self.descripcionDetallada())
    }
    
    abstract method descripcionDetallada()
}
```

### Uso de Mixins

```wollok
class Persona with Comparable, Mostrable {
    property nombre
    property edad
    
    constructor(_nombre, _edad) {
        nombre = _nombre
        edad = _edad
    }
    
    override method compareTo(otra) = 
        self.edad() - otra.edad()
    
    override method toString() = 
        nombre + " (" + edad + " años)"
    
    override method descripcionDetallada() = 
        "Nombre: " + nombre + "\nEdad: " + edad
}
```

## Sistema de Módulos

### Imports

```wollok
import math.*
import colecciones.{Lista, Set}
import io.Console as Consola

object calculadora {
    method raizCuadrada(n) = math.sqrt(n)
    method mostrar(mensaje) = Consola.println(mensaje)
}
```

### Exports

```wollok
// archivo: matematicas.wlk
export object constantes {
    const PI = 3.14159
    const E = 2.71828
}

export class Vector {
    property x
    property y
    
    constructor(_x, _y) {
        x = _x
        y = _y
    }
    
    method magnitud() = math.sqrt(x * x + y * y)
}

export { suma, resta } from operaciones
```

## Testing Avanzado

### Describe y Test

```wollok
describe "Calculadora" {
    const calc = new Calculadora()
    
    test "suma dos números positivos" {
        assert.equals(5, calc.sumar(2, 3))
    }
    
    test "división por cero lanza excepción" {
        assert.throwsException({ calc.dividir(5, 0) })
    }
    
    describe "operaciones con negativos" {
        test "suma con negativo" {
            assert.equals(-1, calc.sumar(2, -3))
        }
        
        test "multiplicación con negativo" {
            assert.equals(-6, calc.multiplicar(2, -3))
        }
    }
}
```

### Fixtures y Setup

```wollok
describe "Lista de tareas" {
    let lista
    
    setup {
        lista = new ListaTareas()
        lista.agregar("Tarea 1")
        lista.agregar("Tarea 2")
    }
    
    before {
        lista.limpiar()
    }
    
    test "agregar tarea aumenta el tamaño" {
        const tamañoInicial = lista.tamaño()
        lista.agregar("Nueva tarea")
        assert.equals(tamañoInicial + 1, lista.tamaño())
    }
}
```

## Wollok Game

### Configuración del Juego

```wollok
import wollokGame.*

program juego {
    game.title("Mi Juego")
    game.width(10)
    game.height(10)
    game.cellSize(50)
    
    // Agregar visuales
    game.addVisual(jugador)
    game.addVisual(enemigo)
    
    // Configurar controles
    keyboard.left().onPressDo({ jugador.moverIzquierda() })
    keyboard.right().onPressDo({ jugador.moverDerecha() })
    
    // Iniciar juego
    game.start()
}
```

### Objetos de Juego

```wollok
object jugador {
    property position = game.at(5, 5)
    property vida = 100
    
    method image() = "jugador.png"
    
    method moverIzquierda() {
        position = position.left(1)
    }
    
    method moverDerecha() {
        position = position.right(1)
    }
    
    method recibirDaño(cantidad) {
        vida = vida - cantidad
        if (vida <= 0) {
            game.removeVisual(self)
            game.say(self, "Game Over!")
        }
    }
}
```

## Estado de Implementación

| Característica | Estado | Prioridad | Notas |
|----------------|--------|-----------|-------|
| **Clases** |
| Declaración básica | 📋 Planeado | Alta | |
| Constructores | 📋 Planeado | Alta | |
| Herencia (`inherits`) | 📋 Planeado | Media | |
| Clases abstractas | 📋 Planeado | Media | |
| **Manejo de Errores** |
| `fallible` methods | 📋 Planeado | Alta | Solo fallibles pueden llamar fallibles |
| Operador `?` (propagación) | 📋 Planeado | Alta | Propaga errores hacia arriba |
| Operador `!` (assertion) | 📋 Planeado | Alta | Panic en runtime como `.unwrap()` |
| Try expressions | 📋 Planeado | Media | Para manejo explícito |
| Excepciones personalizadas | 📋 Planeado | Baja | |
| **Closures** |
| Sintaxis simple `() => expr` | 📋 Planeado | Alta | Sin llaves, solo expresiones |
| Capture de contexto | 📋 Planeado | Media | |
| Funciones de alto orden | 📋 Planeado | Media | |
| **Mixins** |
| Definición | 📋 Planeado | Baja | |
| Composición | 📋 Planeado | Baja | |
| **Módulos** |
| Import/Export | 📋 Planeado | Media | |
| Namespaces | 📋 Planeado | Baja | |
| **Testing** |
| Framework básico | 📋 Planeado | Media | |
| Describe/Test | 📋 Planeado | Media | |
| **Wollok Game** |
| API básica | 📋 Planeado | Baja | |
| Visuales | 📋 Planeado | Baja | |
