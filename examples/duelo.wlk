// Dos instancias de la misma clase mandandose mensajes entre si.
// `new Personaje(...)` no acepta argumentos todavia (el compilador los
// descarta), asi que el nombre se pone con un setter despues de crear.
class Personaje {
  property nombre = "???"
  property vida = 100

  method nombre() = nombre
  method vida() = vida
  method estaVivo() = vida > 0

  method setNombre(unNombre) {
    nombre = unNombre
  }

  method recibirDanio(cantidad) {
    vida = vida - cantidad
  }

  method atacar(otro) {
    otro.recibirDanio(20)
    console.println(nombre, " ataca a ", otro.nombre(), "! le quedan ", otro.vida(), " de vida")
  }
}

const goku = new Personaje()
goku.setNombre("Goku")
const vegeta = new Personaje()
vegeta.setNombre("Vegeta")

goku.atacar(vegeta)
vegeta.atacar(goku)
goku.atacar(vegeta)
vegeta.atacar(goku)

console.println(vegeta.nombre(), " sigue en pie? ", vegeta.estaVivo())
