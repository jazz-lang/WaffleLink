# Modules
## PIC/PIE modules

These modules have `imports` section which specify module names that should be imported to module scope.
When compiling to PIC/PIE bytecode compiler should use `load_mstatic` opcode to resolve imported module name.

## Static executable
These modules cannot have `imports` section and cannot be imported by another module.
When emitting static executable file compile cannot use `load_mstatic` opcode and should resolve all imports at compile time or at link time.
When linking static file with PIC/PIE module all `load_mstatic` opcodes is replaced with `accglobal` opcodes with proper index to global.