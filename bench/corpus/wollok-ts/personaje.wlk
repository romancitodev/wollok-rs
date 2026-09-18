class Personaje {
  var property vida = 1000
  method recibirDanio(cantidad) { vida = vida - cantidad }
  method atacar(otro) { otro.recibirDanio(1) }
}
