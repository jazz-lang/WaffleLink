//! All builtin objects,methods and functions is declared here in runtime.rs or under runtime/ directory.

pub mod cell;

/// Cell types definition.
pub mod cell_type;

/// Object representation.
pub mod object;

pub mod callframe;

/// Array implementation
pub mod array;

pub mod class;

pub mod map;

pub mod string;

pub mod module;
