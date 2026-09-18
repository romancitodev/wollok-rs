// Un objeto singleton con estado propio y algo de lógica.
object pepita {
  property energia = 100

  method energia() = energia

  method volar() {
    energia = energia - 15
  }

  method comer() {
    energia = energia + 30
  }

  method estaCansada() = energia < 50
}

pepita.volar()
pepita.volar()
pepita.volar()
console.println("pepita tiene ", pepita.energia(), " de energia")
console.println("esta cansada? ", pepita.estaCansada())

pepita.comer()
console.println("despues de comer tiene ", pepita.energia())
console.println(pepita)
