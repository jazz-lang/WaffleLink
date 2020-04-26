//! All JIT tiers implementations.
//!
//! Currently JIT is performed on entire function rather than compiling some
//! hot parts of function but this might be solved by implementing OSR.
//!
//! ## OSR
//! What OSR does is allows switching between interpreter context and JIT code.
//! OSR might help in JIT compiling hot loops in functions.
//! 
//! 
//! ## Simple JIT
//! 
//! Simple JIT is unoptimized JIT compiler that translates bytecode straight into
//! Cranelift IR and then emits machine code. Simple JIT can collect type information
//! during execution and change feedback vector.
//! 
//! Inline cachign is impossible right now in Simple JIT due to [Issue 1074](https://github.com/bytecodealliance/wasmtime/issues/1074)
//! and if interpreter succesfully cached some instruction generated machine code will contain
//! fast path and slow path: fast path is direct cached load and slow path is lookup in object properties.
//! 
//! ## Easy JIT
//! Essentially a less optimizing version of the Full JIT. This compiler does not use Cranelift IR instead
//! it uses MIR and LIR to perform specific optimizations and register allocation on LIR. Easy JIT and Full JIT
//! does not allocate stack frames on heap, instead they try to use native machine stack as much as possible. 
//! 
//! Inline caching is possible in Easy JIT and Full JIT since we use our own code generator and IR representation 
//! and we can support LLVM patchpoint like functions.
//! 
//! Optimizations included in Easy JIT:
//! 
//! 
//! - Constant folding
//! - CSE
//! - Small Loop Unrolling
//! 
//! 
//! ## Full JIT
//! 
//! Fully optimizing JIT compiler. Code produced by Full JIT is fastest but emitting optimized code costs some time.
//! 
//! 
//! Optimizations included in Full JIT:
//! - All of Easy JIT optimizations
//! - Inlining
//! - SRoA
//! - Escape analysis
//! - Loop unrolling
//! - Vectorization
//! 
//! 
//! If Full JIT code deoptimizes then it deoptimizes right back into interpreter. Why? To collect more type information
//! and make latency better since Easy JIT and Full JIT uses quite a lot of memory and time to generate  optimized code.

pub mod simple;
