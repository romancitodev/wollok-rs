// Una clase de verdad (no singleton), con guardas via `if` y
// concatenación de strings.
class CuentaBancaria {
  property saldo = 0

  method saldo() = saldo

  method depositar(monto) {
    saldo = saldo + monto
  }

  method extraer(monto) {
    if (monto > saldo) {
      console.println("fondos insuficientes, saldo actual: ", saldo)
    } else {
      saldo = saldo - monto
      console.println("extrajiste ", monto, ", saldo actual: ", saldo)
    }
  }
}

const cuenta = new CuentaBancaria()
cuenta.depositar(500)
cuenta.depositar(250)
cuenta.extraer(100)
cuenta.extraer(10000)

console.println("saldo final: " + cuenta.saldo().toString())
