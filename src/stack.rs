use crate::vm::{RegTypes, VM};

#[derive(Debug)]
pub struct VMStack {
    pub stack: Vec<u64>,
    pub types: Vec<RegTypes>,
}

impl VMStack {
    pub fn new(cap: usize) -> VMStack {
        VMStack {
            stack: (Vec::with_capacity(cap / 2)),
            types: (Vec::with_capacity(cap / 2)),
        }
    }

    pub fn push(&mut self, dat: u64, dattype: RegTypes) {
        self.stack.push(dat);
        self.types.push(dattype);
    }

    pub fn pop(&mut self) -> (Option<u64>, Option<RegTypes>) {
        (self.stack.pop(), self.types.pop())
    }
}

pub fn op_push(vm: &mut VM) {
    // 0x80, size: 2
    // push Rsrc
    // Does not zero the Rsrc by default
    let r_src_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let val: u64 = vm.registers[r_src_ind];
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

    vm.registers[r_dest_ind] = val;
    vm.reg_types[r_dest_ind] = r_type;

    vm.ip += 2;
    return;
}

pub fn op_pushall(vm: &mut VM) {
    // 0x82, size: 1
    // pushall - pushes all register values into the stack. (with metadata - types)
    for i in 0..vm.registers.len().saturating_sub(1) {
        vm.stack.push(vm.registers[i], vm.reg_types[i]);
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

        vm.registers[i] = val;
        vm.reg_types[i] = r_type;
    }

    vm.ip += 1;
    return;
}
