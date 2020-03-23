/// Creates a label that can be jumped to. Parameter $name must be a string and a unique label
/// name.
macro_rules! label {
    ($name:expr) => (
        unsafe { asm!(concat!($name, ":") : : : : "volatile"); }
    )
}

/// Loads the address of a label and returns it. Parameter $name must be a string and an existing
/// label name.
#[cfg(target_arch = "x86_64")]
macro_rules! label_addr {
    ($name:expr) => (
        {
            let addr: usize;
            unsafe { asm!(concat!("leaq ", $name, "(%rip), $0")
                          : "=&r"(addr)
                          :
                          :
                          : "volatile" );
            }
            addr
        }
    )
}

/// Reads the address of the next instruction from the jump table and jumps there.
#[cfg(target_arch = "x86_64")]
macro_rules! dispatch {
    ($vm:expr, $pc:expr, $opcode:expr, $jumptable:expr, $counter:expr) => {
        $counter += 1;
        let addr = $jumptable[operator($opcode) as usize];

        unsafe {
            // the inputs of this asm block force these locals to be in the specified
            // registers after $action is exited, so that on entry to the consecutive
            // $action, the previous asm block will be set up with the right register
            // to locals mapping
            asm!("jmpq *$0"
                 :
                 : "r"(addr), "{r8d}"($counter), "{ecx}"($opcode), "{rdx}"($pc)
                 :
                 : "volatile"
            );
        }
    }
}

/// Encapsulates a VM instruction between register constraints and dispatches to the
/// next instruction.
///  * $name must be a label name as a string
///  * $pc must be a function-local usize
///  * $opcode must be a function-local u32
///  * $counter must be a function-local integer
///  * $action must be a block containing the VM instruction code
#[cfg(target_arch = "x86_64")]
macro_rules! do_and_dispatch {
    ($vm:expr, $jumptable:expr, $name:expr, $pc:expr, $opcode:expr, $counter:expr, $action:expr) => {

        // the outputs of this asm block essentially force these locals to
        // be in the specified registers when $action is entered
        unsafe {
            asm!(concat!($name, ":")
                 : "={r8d}"($counter), "={ecx}"($opcode), "={rdx}"($pc)
                 :
                 :
                 : "volatile");
        }

        {
            $action
        }

        dispatch!($vm, $pc, $opcode, $jumptable, $counter);
    }
}

#[cfg(target_arch = "x86")]
macro_rules! label_addr {
    ($name:expr) => (
        {
            let addr: usize;
            unsafe { asm!(concat!("lea ", $name, ", $0")
                          : "=&r"(addr)
                          :
                          :
                          : "volatile" );
            }
            addr
        }
    )
}

#[cfg(target_arch = "x86")]
macro_rules! dispatch {
    ($vm:expr, $pc:expr, $opcode:expr, $jumptable:expr, $counter:expr) => {
        $counter += 1;
        let addr = $jumptable[operator($opcode) as usize];

        unsafe {
            asm!("jmpl *$0"
                 :
                 : "r"(addr), "{edi}"($counter), "{ecx}"($opcode), "{edx}"($pc)
                 :
                 : "volatile"
            );
        }
    }
}

#[cfg(target_arch = "x86")]
macro_rules! do_and_dispatch {
    ($vm:expr, $jumptable:expr, $name:expr, $pc:expr, $opcode:expr, $counter:expr, $action:expr) => {

        unsafe {
            asm!(concat!($name, ":")
                 : "={edi}"($counter), "={ecx}"($opcode), "={edx}"($pc)
                 :
                 :
                 : "volatile");
        }

        {
            $action
        }

        dispatch!($vm, $pc, $opcode, $jumptable, $counter);
    }
}

#[cfg(target_arch = "aarch64")]
macro_rules! label_addr {
    ($name:expr) => (
        {
            let addr: usize;
            unsafe { asm!(concat!("adrp $0, ", $name, "\n",
                                  "add $0, $0, :lo12:", $name)
                          : "=&r"(addr)
                          :
                          :
                          : "volatile" );
            }
            addr
        }
    )
}

#[cfg(target_arch = "aarch64")]
macro_rules! dispatch {
    ($vm:expr, $pc:expr, $opcode:expr, $jumptable:expr, $counter:expr) => {
        $counter += 1;
        let addr = $jumptable[operator($opcode) as usize];

        // Pinning $vm to x12 doesn't seem actually use x12 in a useful way, but it does
        // somehow accidentally make llvm allocate registers consistently.
        // I think there may be a better solution, though.
        unsafe {
            asm!("br $0"
                 :
                 : "r"(addr), "{x11}"($counter), "{w9}"($opcode), "{x10}"($pc), "{x12}"($vm)
                 :
                 : "volatile"
            );
        }
    }
}

#[cfg(target_arch = "aarch64")]
macro_rules! do_and_dispatch {
    ($vm:expr, $jumptable:expr, $name:expr, $pc:expr, $opcode:expr, $counter:expr, $action:expr) => {

        unsafe {
            asm!(concat!($name, ":")
                 : "={x11}"($counter), "={w9}"($opcode), "={x10}"($pc), "={x12}"($vm)
                 :
                 :
                 : "volatile");
        }

        {
            $action
        }

        dispatch!($vm, $pc, $opcode, $jumptable, $counter);
    }
}

#[cfg(target_arch = "arm")]
macro_rules! label_addr {
    ($name:expr) => (
        {
            // https://llvm.org/bugs/show_bug.cgi?id=24350

            let addr: usize;
            unsafe { asm!(//concat!("adr $0, 1f
                          //         add $0, $0, #(", $name, "-1f)
                          //         1:")
                          concat!("add $0, pc, #(", $name, " - .) & 0xFF00
                                   add $0, $0, #((", $name, " - .) - ((", $name, " - .) & 0xFF00)) - 4")
                          : "=&r"(addr)
                          :
                          :
                          : "volatile" );
            }
            addr
        }
    )
}

#[cfg(target_arch = "arm")]
macro_rules! dispatch {
    ($vm:expr, $pc:expr, $opcode:expr, $jumptable:expr, $counter:expr) => {
        $counter += 1;
        let addr = $jumptable[operator($opcode) as usize];

        unsafe {
            asm!("bx $0"
                 :
                 : "r"(addr), "{r11}"($counter), "{r9}"($opcode), "{r10}"($pc), "{r8}"($vm)
                 :
                 : "volatile"
            );
        }
    }
}

#[cfg(target_arch = "arm")]
macro_rules! do_and_dispatch {
    ($vm:expr, $jumptable:expr, $name:expr, $pc:expr, $opcode:expr, $counter:expr, $action:expr) => {

        unsafe {
            asm!(concat!($name, ":")
                 : "={r11}"($counter), "={r9}"($opcode), "={r10}"($pc), "={r8}"($vm)
                 :
                 :
                 : "volatile");
        }

        {
            $action
        }

        dispatch!($vm, $pc, $opcode, $jumptable, $counter);
    }
}
