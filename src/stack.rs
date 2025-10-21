use crate::{
    registers::Register,
    vm::{RegTypes, VM},
};

#[derive(Debug)]
pub struct VMStack {
    pub stack: Vec<StackFrame>,
}

impl VMStack {
    pub fn new(cap: usize) -> VMStack {
        VMStack {
            stack: (Vec::with_capacity(cap)),
        }
    }

    pub fn push(&mut self, dat: u64, dattype: RegTypes) {
        self.stack.push(StackFrame::new(dat, dattype));
    }

    pub fn pop(&mut self) -> (Option<u64>, Option<RegTypes>) {
        match self.stack.pop() {
            Some(v) => {
                return (Some(v.val), Some(v.ftype));
            }
            None => {
                return (None, None);
            }
        }
    }

    pub fn update_val(&mut self, ind: usize, newval: u64, newtype: RegTypes) -> Result<(), ()> {
        if (ind >= self.stack.len()) {
            return Err(());
        }

        self.stack[ind].val = newval;
        self.stack[ind].ftype = newtype;
        Ok(())
    }

    pub fn get_val(&mut self, ind: usize) -> Option<&StackFrame> {
        self.stack.get(ind).clone()
    }
}

#[derive(Debug)]
pub struct StackFrame {
    pub val: u64,
    pub ftype: RegTypes,
}

impl StackFrame {
    pub fn new(val: u64, ftype: RegTypes) -> StackFrame {
        StackFrame {
            val: (val),
            ftype: (ftype),
        }
    }
}

pub fn op_push(vm: &mut VM) {
    // 0x80, size: 2
    // push Rsrc
    // Does not zero the Rsrc by default
    let r_src_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let val: u64 = vm.registers[r_src_ind].as_u64_bitwise();
    let r_type: RegTypes = vm.reg_types[r_src_ind];

    // Pushes value, then type.
    vm.stack.push(val, r_type);

    vm.ip += 2;
    return;
}

pub fn op_pop(vm: &mut VM) {
    // 0x81, size: 2
    // pop Rdest
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;

    let (val_opt, r_type_opt) = vm.stack.pop();

    let r_type: RegTypes = match r_type_opt {
        Some(val) => val,
        None => {
            panic!(
                "CRITICAL: Attempting to pop metadata in empty stack!\n\tAt IP = {}",
                vm.ip
            );
        }
    };

    let val = match val_opt {
        Some(val) => val,
        None => {
            panic!(
                "CRITICAL: Attempting to pop value in empty stack!\n\tAt IP = {}",
                vm.ip
            );
        }
    };

    vm.registers[r_dest_ind] = Register::from_u64_bits(val, r_type);
    vm.reg_types[r_dest_ind] = r_type;

    vm.ip += 2;
    return;
}

pub fn op_pushall(vm: &mut VM) {
    // 0x82, size: 1
    // pushall - pushes all register values into the stack. (with metadata - types)
    for i in 0..vm.registers.len().saturating_sub(1) {
        vm.stack
            .push(vm.registers[i].as_u64_bitwise(), vm.reg_types[i]);
    }

    vm.ip += 1;
    return;
}

pub fn op_popall(vm: &mut VM) {
    // 0x83, size: 1
    // popall - pops last 32 values from stack in registers. Normally used after pushall
    for i in vm.registers.len().saturating_sub(1)..0 {
        let (dat_opt, rtype_opt) = vm.stack.pop();

        let r_type: RegTypes = match rtype_opt {
            Some(v) => v,
            None => {
                panic!(
                    "CRITICAL: Attempting to pop metadata in the empty stack\n\tAt IP = {} (popall)",
                    vm.ip
                );
            }
        };
        let val = match dat_opt {
            Some(v) => v,
            None => {
                panic!(
                    "CRITICAL: Attempting to pop value in the empty stack\n\tAt IP = {} (popall)",
                    vm.ip
                );
            }
        };

        vm.registers[i] = Register::from_u64_bits(val, r_type);
        vm.reg_types[i] = r_type;
    }

    vm.ip += 1;
    return;
}

pub fn op_gsf(vm: &mut VM) {
    // 0x84, size: 3
    // Gets stack frame [Rsrc] and loads its value into
    // Rdst. Changes types
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let r_src_ind: usize = vm.memory[(vm.ip + 2)] as usize;

    let ind: usize = vm.registers[r_src_ind].as_u64() as usize;

    match vm.stack.get_val(ind) {
        Some(v) => {
            vm.registers[r_dest_ind] = Register::from_u64_bits(v.val, v.ftype);
            vm.reg_types[r_dest_ind] = v.ftype;
        }
        None => {}
    };

    vm.ip += 3;
    return;
}

pub fn op_usf(vm: &mut VM) {
    // 0x85, size: 3
    // Updates stack frame [Rdst] and changes its value into
    // Rsrc value. Changes types
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let r_src_ind: usize = vm.memory[(vm.ip + 2)] as usize;

    let ind: usize = vm.registers[r_dest_ind].as_u64() as usize;
    let newval: u64 = vm.registers[r_src_ind].as_u64_bitwise();
    let newtype: RegTypes = vm.reg_types[r_src_ind];

    match vm.stack.update_val(ind, newval, newtype) {
        Ok(()) => {}
        Err(()) => {
            eprintln!("Stack frame {} was not updated.", ind);
        }
    }

    vm.ip += 3;
    return;
}
