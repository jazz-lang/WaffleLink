# Waffle
Simple and small language that I created to learn about creating backends and register allocation. Currently this compiler got only one backend: Cranelift.
I want to create my own small IR and assember for learning purposes and use that IR in this compiler.

# TODO
- Name manglings
  Example:
  ```
  func (v *Point) getX() int {return v.x}
  ```
  That method name should look like this after mangling: `@Point_getX()`
- Methods
- Interfaces
