# Backlog de la VM

Lo que falta para que un `.wlk` real (no solo los ejemplos de test) haga algo
útil. Ver `vm-design.md` para el diseño ya decidido; esto es la lista de "lo
próximo a construir", en orden de qué desbloquea qué.

## 1. ~~Constructores (inicializar campos al `new`)~~ — hecho

`NewInstance` ahora corre un constructor sintetizado por clase/objeto (un
método de aridad 0 que evalúa cada expresión de campo y hace `StoreField`,
en orden de declaración) sobre la instancia recién alocada, antes de
devolverla — `ClassTable::ctor`/`compile_ctor` en `wollok-compiler`.
`pepita.saludar()` ya devuelve `"pepita"`, no `Null`.

Sigue faltando: argumentos en `new X(args)` (siguen descartados — no hay
forma de pasarle valores a un campo desde el `new`, solo desde el literal
de su declaración).

## 2. ~~Dispatch nativo sobre valores no-`Object`~~ — mecanismo hecho, stdlib pendiente

`Send` ya no asume `Value::Object`: cuando el receptor es un primitivo,
`vm.rs` lo manda a `wollok_vm::native::dispatch` en vez de al
vtable/inline-cache (que siguen siendo solo para objetos de heap de
verdad). `wollok-vm` define únicamente el *mecanismo* — un
`native::NativeTable` que mapea `(PrimitiveKind, selector, aridad) →
fn(&mut Vm, Value, &[Value]) -> Value` — sin una sola implementación de
método adentro. Es la misma idea que usan los runtimes de verdad (slots de
CPython, builtins registrados de V8, `rb_define_method` de Ruby): una tabla
de despacho llenada por registro, no un switch central.

Las implementaciones viven en la crate nueva **`wollok-std`**, una función
suelta por método, agrupadas por tipo (`numbers/`, `booleans/`, `strings/`),
registradas todas via `wollok_std::install(&mut vm)` — `main.rs` la llama
una sola vez, justo después de `Vm::new()`. Migrado ahí, tal cual estaba:
aritmética/comparación de `Int` (`+ - * / % < <= > >= == !=`) y
`Bool.negate()`. `==`/`!=` son intencionalmente especiales: comparar contra
un tipo distinto da `false`/`true`, no panic (`5 == "hola"` no explota) —
el resto de los operadores sí panickean con un tipo inválido, porque ahí sí
es un error real. **El resto queda deliberadamente sin implementar — es la
stdlib, y esa parte la escribe Roman:**

- **Int/Float**: `**`, `toString`, y todo lo de `Float` (arranca en cero).
- **Bool**: `toString` (`&&`/`||` ya compilan a jumps, no a `Send`).
- **Str**: todo — `strings/mod.rs` en `wollok-std` está vacío a propósito.
  `+` (concatenación), `==`, `toString`/identidad.

### 2b. Representación de strings — resuelto: tabla aparte + interning real

`Value::Str(StrIdx)` ahora indexa `Vm::strings` (`wollok_vm::strings::StringTable`),
no `Program::strings` — se movió de `Program` (fijo una vez compilado) a
`Vm` (lo que cambia en runtime) porque un método nativo que devuelve un
string nuevo (`toString`, concatenación) necesita un lugar mutable donde
ponerlo, y `Program` ya no lo es en runtime. La tabla interna por
contenido (dos literales `"pepita"`, o un literal y un `toString` con el
mismo texto, comparten `StrIdx`) — quedó la opción "statu quo mejorado" que
ya estaba anotada acá, no la small-string-optimization (más trabajo, no
hacía falta todavía). Sigue siendo unboxed: ningún `HeapObject` ni `Rc<str>`
de por medio, tal como pide el principio de primitivos en `vm-design.md`.

## 3. `console.println` (o como se llame) para poder ver algo

No hay ninguna forma de sacar un valor del programa salvo el resultado final
de `main`. Con dispatch nativo (ítem 2) ya resuelto, esto es agregar un
selector nativo más (`println`/`log`, 1 arg) que imprima usando `toString`.
No bloquea nada más, pero es lo que hace que probar código deje de requerir
mirar el valor de retorno por consola de `wollok_rs`.

## 4. Resto (no bloqueante, más lejos)

- Arrays/Sets (`NewArray`/`NewSet` son `todo!()` en el VM).
- Closures (`NewClosure` es `todo!()`).
- `try`/`catch`/`throw` (instrucciones ya reservadas, sin implementar).
- `super` (`SendSuper` es `todo!()`; la linearización de clases con
  `inherits`/`with` tampoco está — hoy una clase hija no ve los métodos de
  la madre salvo que los reimplemente, como en `Dog`/`Animal` de
  `simple.wlk`).
- Imports/módulos reales.

**Ya resuelto, aparte, sin que sea lo de arriba:** cuando `Send` (bytecode o
`Vm::send`) no encuentra el selector en la vtable de la clase del objeto,
cae a un bucket nativo (`native::PrimitiveKind::Object`, implementado en
`wollok-std/src/objects`) antes de panickear — hoy solo trae `toString`
(`"a NombreDeClase"`). Es la pieza mínima para que "cualquier objeto se
pueda representar como string" sin tener que resolver herencia real
primero. **No es** la linearización de `inherits`/`with` de arriba — una
clase que herede de otra sigue sin ver los métodos de la madre; esto es
solo el fallback final, compartido por todo objeto, independiente del
árbol de herencia (que todavía no existe).

De paso quedó `Vm::send(program, receiver, selector, args)` — un `Send`
dinámico (sin inline cache, resuelve por nombre) para que un método nativo
llame hacia atrás a un método de usuario (por ejemplo, para que
`Str.+`/`concat` pueda stringificar un objeto arbitrario con
`vm.send(program, valor, "toString", vec![])`, aunque `Str::concat` hoy
solo maneja `Str + Str`, no cualquier `Value`).
