# Diseño de la VM de wollok-rs

Este documento es la propuesta de arquitectura para la VM, escrita **antes** de
implementar nada, para validar las decisiones de fondo primero. No es
documentación de una feature terminada — es el borrador a discutir.

## Objetivo y no-objetivo

Objetivo: una VM rápida para un lenguaje de objetos con dispatch por mensajes
(Wollok es, en espíritu, Smalltalk con sintaxis moderna). El cuello de botella
real en este tipo de lenguajes no es "interpretar", es **resolver a qué método
te referís** en cada envío de mensaje — eso pasa en cada operación, hasta en
`1 + 1`. El diseño tiene que atacar eso específicamente.

No-objetivo (por ahora): JIT, compilación a nativo, multi-threading. Se puede
agregar después si hace falta; no son prerequisito para que esto sea rápido
comparado con un tree-walker ingenuo.

## Pipeline

```
AST (wollok-ast, ya existe)
  → Resolver          (nombres de variables/campos → índices de slot)
  → Compilador         (AST resuelto → bytecode plano)
  → VM                 (ejecuta el bytecode)
```

El Resolver es la etapa de "precompilado" que se pidió: en vez de que la VM
busque `valor` en un `HashMap<String, Value>` cada vez que se lee una
variable, el Resolver decide en compile-time "`valor` es el slot 2 del frame
actual" o "es la property 0 de la clase actual", y el bytecode ya referencia
directamente ese índice. Esto solo, sin tocar el dispatch de métodos, ya saca
la mayoría del overhead de un tree-walker naive.

## Bytecode: stack-based, no register-based

Dos familias posibles: stack-based (push/pop, como CPython/JVM) o
register-based (como Lua 5.x). Register-based genera menos instrucciones y
menos shuffling, pero es sustancialmente más difícil de compilar bien de
entrada.

**Decisión: arrancar stack-based.** Es mucho más simple de emitir y tener
correcto desde el día uno. Si el profiling después muestra que el
push/pop de la pila es el cuello de botella real (no lo va a ser — el
dispatch de métodos lo va a ser primero), se migra a registros en un
segundo paso. No optimizar lo que no se midió.

### Set de instrucciones (propuesta inicial)

```
// Constantes / literales
PushConst(ConstIdx)      // empuja un literal de la tabla de constantes
PushNull
PushTrue
PushFalse

// Variables locales (slots resueltos por el Resolver)
LoadLocal(SlotIdx)
StoreLocal(SlotIdx)

// Campos del objeto actual (self)
LoadField(FieldIdx)
StoreField(FieldIdx)

// Envío de mensajes — la instrucción más importante de todas
Send(MethodNameIdx, ArgCount)   // pop receiver + args, resuelve y llama
SendSuper(MethodNameIdx, ArgCount)

// Control de flujo
Jump(Offset)
JumpIfFalse(Offset)
Return

// Construcción de objetos/colecciones
NewInstance(ClassIdx, ArgCount)
NewArray(Count)
NewSet(Count)
NewClosure(ClosureIdx)         // captura upvalues, empuja un valor closure

// Cierre de bloque try
PushTryHandler(CatchOffset)
PopTryHandler
Throw
```

`Send` es la instrucción que hay que optimizar en serio (ver Inline Caching
más abajo) — todo lo demás es relativamente barato de por sí.

## Representación de `Value`

```rust
enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Object(ObjRef),   // índice al heap, no puntero
}
```

Un `enum` tageado normal, no NaN-boxing. NaN-boxing (empaquetar todo en un
`u64`, como hacen los motores de JS) es más rápido pero necesita `unsafe`
real y es bastante más difícil de tener bien. Es la optimización de
"exprimir el último 20%" — se deja para **después**, cuando el profiler
confirme que el tamaño/branching de `Value` (no el dispatch de métodos) es
el cuello de botella. Empezar simple y correcto.

### Principio: los primitivos NUNCA son objetos de heap con header

Esto es una decisión de fondo, no un detalle de implementación: aunque el
lenguaje es OOP de punta a punta (todo entiende mensajes, todo tiene una
clase, `3.plus(4)` es conceptualmente un `Send`), eso es una ficción a nivel
semántico/dispatch — **no implica que un `Int`/`Float`/`Bool`/`String` viva
en el heap como una instancia con header de clase + campos**, al estilo de
`PyLongObject` en CPython (un `int` de un dígito ya ocupa ~28 bytes: refcount
+ puntero a tipo + tamaño + el dígito). Esa es exactamente la vergüenza que
este VM no se puede permitir.

Regla concreta: los tipos primitivos se representan nativos/unboxed —
variantes de `Value` que cargan el dato en línea (`Int(i64)`, `Float(f64)`,
`Bool(bool)`, y lo que se decida para `String`/`Char`/etc.), nunca un
`ObjRef` a un `HeapObject` con `class: ClassId` + `Vec<Value>` de fields.
El "es una clase" de `Int` es una ficción a nivel de `ClassTable`/dispatch
(un `ClassId` reservado que el resolver de `Send` conoce especialmente, ver
ítem 2 de `backlog.md`), no una representación real en memoria. Esto
descarta explícitamente la opción "string como objeto de heap de verdad"
que se había anotado como alternativa en `backlog.md` ítem 2b — queda
tachada por este principio, no por costo de implementación.

Consecuencia práctica para cualquier primitivo nuevo que se agregue: la
pregunta no es "¿le doy un `ObjRef` como a todo lo demás?" (no), es "¿qué
representación unboxed le entra a `Value` sin romper que sea `Copy`, y cómo
hace `Send` para reconocer su clase sin pasar por el heap?".

## Heap: arena + GC simple, no `Rc<RefCell<T>>` por todos lados

`Rc<RefCell<T>>` en cada objeto es la salida fácil pero tiene dos problemas
reales, no hipotéticos:

1. Incrementar/decrementar el refcount en cada acceso/clon es overhead
   constante en el hot path.
2. Wollok va a tener ciclos de referencias con total naturalidad (un objeto
   que se referencia a través de otro, listas circulares, observers) — y
   `Rc` sin colector de ciclos **pierde esa memoria silenciosamente**. No es
   un edge case, es estructural al lenguaje.

**Decisión: heap tipo arena.** Los objetos viven en un `Vec<ObjectSlot>`
indexado por `ObjRef(u32)`, no detrás de punteros sueltos. Ventajas:
alocación es push al Vec (barato, cache-friendly), y permite un GC
mark-sweep simple recorriendo el arena en vez de perseguir punteros por
toda la heap del proceso. Se puede evolucionar a generacional después si
hace falta, pero mark-sweep sobre un arena ya es un salto grande respecto a
leakear con `Rc`.

## Dispatch de métodos: Inline Caching por call-site

Esta es la optimización de mayor impacto, más que cualquier decisión de
bytecode o de `Value`. Sin esto, cualquier VM de un lenguaje de mensajes es
lenta sin importar qué tan bien esté el resto.

El problema: cada `Send` en teoría tiene que buscar el método recorriendo la
linearización de clases (`inherits`/`with` ya calculan esto en el AST, pero
en runtime hay que resolverlo por instancia). Hacer esa búsqueda en *cada*
envío es caro.

La solución clásica (V8, Smalltalk-80, SELF): cada call-site en el bytecode
cachea "la última clase que vi acá → el método ya resuelto para esa clase".

```rust
struct CallSite {
    method_name: MethodNameIdx,
    cache: Option<(ClassId, MethodRef)>,  // caso monomórfico
}
```

En el caso común (monomórfico — el mismo call-site casi siempre ve la misma
clase de receptor, que es lo típico en código real), resolver un mensaje es
una comparación de `ClassId` + salto directo al método cacheado, no una
búsqueda. Si la clase cambia (polimórfico), se cae a la búsqueda completa y
se actualiza el cache — con una cache pequeña de 2-4 entradas (polymorphic
inline cache) si hace falta más adelante.

La linearización de clases (orden `inherits ... with ...`) se calcula **una
sola vez** cuando se carga la clase, no en cada búsqueda — se guarda como
una lista plana de "para este selector, qué método correspondería" (una
especie de vtable por clase), para que incluso el camino sin cache sea un
lookup en una tabla, no un recorrido de la jerarquía.

## Loop de ejecución

Un loop `match` plano sobre el opcode. Rust no tiene `computed goto`, pero
LLVM suele compilar un `match` denso y exhaustivo sobre un enum tipo-C a una
jump table de todos modos — no hace falta simular threading directo a mano
con trucos `unsafe`, eso es ganancia marginal por mucha complejidad.

## REPL (para después, no ahora)

Nota para cuando llegue el momento: un REPL con `ratatui` necesita que la VM
exponga ejecución incremental (evaluar una expresión sueltasobre un estado
de sesión persistente — objetos/clases ya cargados), no re-ejecutar todo el
programa cada vez. Vale la pena tenerlo en mente al diseñar el "loader" de
clases/objetos (que probablemente va a ser incremental de por sí, para
soportar múltiples archivos/imports), pero no bloquea nada del diseño de
arriba.

## Compromisos baratos ahora, carísimos después

Segunda pasada sobre el documento buscando específicamente: "¿qué decisión de
acá es barata de tomar bien ahora, pero cara de agregar después porque
implicaría recompilar todo o re-auditar el codebase entero?" — a diferencia
de cosas como NaN-boxing o un VM de registros, que son trabajo difícil se
hagan cuando se hagan (no hay atajo, así que esperar a tener datos reales de
profiling es lo correcto, no pereza).

Tres cosas sí caen en la categoría "hacerlo ahora":

1. **El slot de inline cache tiene que reservarse en el formato del
   bytecode desde el compilador, no agregarse después.** Si `Send` no nace
   con un hueco para el cache (aunque la lógica arranque monomórfica y
   simple), agregarlo más adelante significa recompilar/reformatear todo el
   bytecode ya emitido. El campo existe desde el día uno; la sofisticación
   de qué guarda ahí puede crecer después.

   ```rust
   struct Send {
       method_name: MethodNameIdx,
       arg_count: u8,
       cache_slot: CacheSlotIdx,   // reservado desde el compilador, siempre
   }
   ```

2. **Write barriers en cada mutación de heap, desde el día uno**
   (`StoreField`, mutación de arrays/sets), aunque sean no-op hasta que
   exista un GC generacional. Agregarlo ahora es una llamada a función en
   cada sitio de mutación; agregarlo después significa re-auditar cada
   mutación del codebase para no romper un GC generacional que ya está
   corriendo en producción.

   ```rust
   trait Heap {
       fn write_field(&mut self, obj: ObjRef, field: FieldIdx, value: Value) {
           // hoy: solo escribe. cuando exista GC generacional: además
           // marca la carta/remembered-set si corresponde.
           self.store(obj, field, value);
       }
   }
   ```

3. **`Value` se accede solo por métodos (`as_int()`, `is_truthy()`,
   `as_object()`...), nunca por `match` directo desparramado en los
   opcodes.** No cambia la representación interna ahora, pero si más
   adelante SÍ conviene empaquetarla distinto (con datos reales de
   profiling en mano, no antes), el cambio queda contenido al módulo
   `value.rs` en vez de tocar cada handler de opcode del VM.

## Qué falta decidir (para la próxima conversación, no bloqueante)

- Modelo exacto de excepciones (`throw`/`catch` de Wollok real vs. el `try`
  estilo Result de este dialecto — corren por caminos de VM distintos).
- Cómo se resuelven las stdlib (`console`, `assert`, colecciones) — ¿nativas
  en Rust con un trait `NativeMethod`, o Wollok puro compilado a bytecode
  al arrancar la VM?
- Estrategia de módulos/imports en runtime (un `import` en el AST hoy es
  solo sintaxis; falta el loader real).

Lista concreta y priorizada de qué implementar primero (constructores,
dispatch nativo sobre `Int`/`Float`/`Bool`/`Str`, `console.println`, resto):
ver `backlog.md`.
