object pepita {
  const name = "pepita"

  method saludar() {
    return name
  }
}

class Animal {
  property name = "Animal"

  method speak() {
    return "I am an animal"
  }

  fallible method eat() = "Eating food"
}

class Dog inherits Animal {
  /* override method speak() {
    return "Woof!"
  } */

  override fallible method eat() = "Eating dog food"
}

const animal = new Dog()

animal.speak()

pepita.saludar()
