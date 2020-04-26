//! SimpleJIT implementation.
//!
//! The first execution of any function always starts in the interpreter tier. As soon as any statement
//! in the function executes more than 100 times, or the function is called more than 10 times (whichever comes first),
//! execution is diverted into code compiled by the Simple JIT
//!
//!
//! What this Simple JIT does is removes interpreter loop dispatch overhead and compiles bytecode into single stream of
//! machine instructions without interpreter dispatch, this improves performance by helping CPU predicting branches.
