use crate::bytecode::op::*;
use crate::common::ptr::*;
use crate::runtime::cell::*;
use crate::runtime::frame::*;
use crate::runtime::function::*;
use crate::runtime::process::*;
use crate::runtime::symbol::Symbol;
use crate::runtime::value::*;

pub const INLINE_CACHE_MAX: u16 = 10;

pub fn lda_by_id_impl(frame: &mut Frame, mut func: Ptr<Cell>, vpc: Ptr<u8>) {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    let mut should_cache = true;
    let mut misses = 0;
    if let FeedBack::Cache(_, _, m) = feedback {
        if *m >= INLINE_CACHE_MAX {
            should_cache = false;
        }
        misses = *m;
    }
    if !base.is_cell() || !should_cache {
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
        *vpc.offset(0) = Op::LdaSlowById as u8;
    } else {
        let mut cell = base.as_cell();
        if cell.lookup(Symbol::new_value(key), &mut slot) {
            if slot.base.raw == cell.raw {
                *vpc.offset(0) = Op::LdaOwnProperty as u8;
            } else {
                if let Some(proto) = cell.prototype {
                    if slot.base.raw == proto.raw {
                        *vpc.offset(0) = Op::LdaProtoProperty as u8;
                    } else {
                        *vpc.offset(0) = Op::LdaChainProperty as u8;
                    }
                } else {
                    // this case *must* be unreachable, if slot is found then value must have to exist somewhere.
                    unreachable!()
                }
            }
            frame.rax = slot.value();
            *feedback = FeedBack::Cache(slot.base.attributes, slot.offset, misses);
        } else {
            frame.rax = slot.value(); // this is undefined or value from array.
        }
    }
}

pub fn lda_by_val_impl(frame: &mut Frame, mut func: Ptr<Cell>, vpc: Ptr<u8>) {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let _feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    if !base.is_cell() {
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
    } else {
        let mut cell = base.as_cell();
        cell.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value();
    }
}

pub fn lda_by_idx_impl(frame: &mut Frame, mut func: Ptr<Cell>, vpc: Ptr<u8>) {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = Value::new_int(key as i32);
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    let mut should_cache = true;
    let mut misses = 0;
    if let FeedBack::Cache(_, _, m) = feedback {
        if *m >= INLINE_CACHE_MAX {
            should_cache = false;
        }
        misses = *m;
    }
    if !base.is_cell() || !should_cache {
        base.lookup(Symbol::new_value(key), &mut slot);
        frame.rax = slot.value(); // this is undefined or property.
        if !should_cache {
            *vpc.offset(0) = Op::LdaSlowByIdx as u8;
        }
    } else {
        let mut cell = base.as_cell();
        if cell.lookup(Symbol::new_value(key), &mut slot) {
            if slot.base.raw == cell.raw {
                *vpc.offset(0) = Op::LdaOwnProperty as u8;
            } else {
                if let Some(proto) = cell.prototype {
                    if slot.base.raw == proto.raw {
                        *vpc.offset(0) = Op::LdaProtoId as u8;
                    } else {
                        *vpc.offset(0) = Op::LdaChainId as u8;
                    }
                } else {
                    // this case *must* be unreachable, if slot is found then value must have to exist somewhere.
                    unreachable!()
                }
            }
            frame.rax = slot.value();
            *feedback = FeedBack::Cache(slot.base.attributes, slot.offset, misses);
        } else {
            frame.rax = slot.value(); // this is undefined or value from array.
        }
    }
}

pub fn sta_by_id_impl(frame: &mut Frame, mut func: Ptr<Cell>, vpc: Ptr<u8>) {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    let mut should_cache = true;
    let mut misses = 0;
    if let FeedBack::Cache(_, _, m) = feedback {
        if *m >= INLINE_CACHE_MAX {
            should_cache = false;
        }
        misses = *m;
    }
    if !base.is_cell() || !should_cache {
        if base.lookup(Symbol::new_value(key), &mut slot) {
            *slot.value = frame.rax;
        }
        if !should_cache {
            *vpc.offset(0) = Op::StaSlowById as u8;
        }
    //frame.rax = slot.value; // this is undefined or property.
    } else {
        let mut cell = base.as_cell();
        if cell.lookup(Symbol::new_value(key), &mut slot) {
            if slot.base.raw == cell.raw {
                *vpc.offset(0) = Op::StaOwnProperty as u8;
            } else {
                if let Some(proto) = cell.prototype {
                    if slot.base.raw == proto.raw {
                        *vpc.offset(0) = Op::StaProtoProperty as u8;
                    } else {
                        *vpc.offset(0) = Op::StaChainProperty as u8;
                    }
                } else {
                    // this case *must* be unreachable, if slot is found then value must have to exist somewhere.
                    unreachable!()
                }
            }
            *slot.value = frame.rax;
            *feedback = FeedBack::Cache(slot.base.attributes, slot.offset, misses);
        } else {
            cell.insert(Symbol::new_value(key), frame.rax, &mut slot);
            *vpc.offset(0) = Op::StaOwnProperty as u8;
            *feedback = FeedBack::Cache(cell.attributes, slot.offset, misses);
        }
    }
}

pub fn sta_by_idx_impl(frame: &mut Frame, mut func: Ptr<Cell>, vpc: Ptr<u8>) {
    let base_reg = *vpc.offset(1);
    let key = *vpc.offset(2).cast::<u32>();
    let key = unsafe {
        let key = *func
            .func_value_unchecked_mut()
            .constants
            .get_unchecked(key as usize);
        key
    };
    let cache = vpc.offset(6).cast::<u32>();
    let f = func.func_value_unchecked_mut();
    let feedback = unsafe { f.feedback_vector.get_unchecked_mut(*cache as usize) };
    let mut base = *frame.r(base_reg as usize);
    let mut slot = Slot::new();
    let mut should_cache = true;
    let mut misses = 0;
    if let FeedBack::Cache(_, _, m) = feedback {
        if *m >= INLINE_CACHE_MAX {
            should_cache = false;
        }
        misses = *m;
    }
    if !base.is_cell() || !should_cache {
        if base.lookup(Symbol::new_value(key), &mut slot) {
            *slot.value = frame.rax;
        }
        if !should_cache {
            *vpc.offset(0) = Op::StaSlowByIdx as u8;
        }
    //frame.rax = slot.value; // this is undefined or property.
    } else {
        let mut cell = base.as_cell();
        if cell.lookup(Symbol::new_value(key), &mut slot) {
            if slot.base.raw == cell.raw {
                *vpc.offset(0) = Op::StaOwnProperty as u8;
            } else {
                if let Some(proto) = cell.prototype {
                    if slot.base.raw == proto.raw {
                        *vpc.offset(0) = Op::StaProtoProperty as u8;
                    } else {
                        *vpc.offset(0) = Op::StaChainProperty as u8;
                    }
                } else {
                    // this case *must* be unreachable, if slot is found then value must have to exist somewhere.
                    unreachable!()
                }
            }
            *slot.value = frame.rax;
            *feedback = FeedBack::Cache(slot.base.attributes, slot.offset, misses);
        } else {
            cell.insert(Symbol::new_value(key), frame.rax, &mut slot);
            *vpc.offset(0) = Op::StaOwnId as u8;
            *feedback = FeedBack::Cache(cell.attributes, slot.offset, misses);
        }
    }
}
