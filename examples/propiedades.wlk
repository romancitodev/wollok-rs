// `property` genera getter y setter solo con declararla.
// `const property` genera getter nomas (solo lectura, sin setter).
// Un `method` escrito a mano pisa el getter/setter automatico.
class Persona {
  property nombre = "sin nombre"
  const property dni = 0
  property edad = 0

  method nombre() = "Sr./Sra. " + nombre
  method esMayorDeEdad() = edad >= 18
}

const roman = new Persona()
roman.nombre = "Roman"
roman.edad = 30

console.println(roman.nombre())
console.println("edad: ", roman.edad(), ", es mayor? ", roman.esMayorDeEdad())
console.println("dni (solo lectura, nunca se toco): ", roman.dni())

// roman.dni = 999 -> "Persona does not understand #dni=", el setter no existe.
