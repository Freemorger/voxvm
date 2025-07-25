use crate::vm::{RegTypes, VM};

pub fn op_push(vm: &mut VM) {
    // 0x80, size: 2
    // push Rsrc
    // Does not zero the Rsrc by default
    let r_src_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let val: u64 = vm.registers[r_src_ind];
    let r_type: RegTypes = vm.reg_types[r_src_ind];

    // Pushes value, then type.
    vm.stack.push(val);
    vm.stack.push((r_type as u64) + 1);

    vm.ip += 2;
    return;
}

pub fn op_pop(vm: &mut VM) {
    // 0x81, size: 2
    // pop Rdest
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let n_type: u64 = match vm.stack.pop() {
        Some(val) => val,
        None => {
            panic!(
                "CRITICAL: Attempting to pop metadata in empty stack!\n\tAt IP = {}",
                vm.ip
            );
        }
    };
    let r_type: RegTypes = match n_type {
        0x1 => RegTypes::uint64,
        0x2 => RegTypes::int64,
        0x3 => RegTypes::float64,
        0x4 => RegTypes::StrAddr,
        0x5 => RegTypes::address,
        other => {
            panic!(
                "CRITICAL: Stack member metadata corrupt!\n\tUnknown data type: {}\n\tat IP = {}",
                other, vm.ip
            );
        }
    };

    let val = match vm.stack.pop() {
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
        vm.stack.push(vm.registers[i]);
        vm.stack.push((vm.reg_types[i] as u64) + 1); // pushing metadata next to value
    }

    vm.ip += 1;
    return;
}

pub fn op_popall(vm: &mut VM) {
    // 0x83, size: 1
    // popall - pops last 32 values from stack in registers. Normally used after pushall
    for i in vm.registers.len().saturating_sub(1)..0 {
        let n_type = match vm.stack.pop() {
            Some(v) => v,
            None => {
                panic!(
                    "CRITICAL: Attempting to pop metadata in the empty stack\n\tAt IP = {} (popall)",
                    vm.ip
                );
            }
        };
        let val = match vm.stack.pop() {
            Some(v) => v,
            None => {
                panic!(
                    "CRITICAL: Attempting to pop value in the empty stack\n\tAt IP = {} (popall)",
                    vm.ip
                );
            }
        };

        vm.registers[i] = val;
        vm.reg_types[i] = match n_type {
            0x1 => RegTypes::uint64,
            0x2 => RegTypes::int64,
            0x3 => RegTypes::float64,
            0x4 => RegTypes::StrAddr,
            0x5 => RegTypes::address,
            other => {
                panic!(
                    "CRITICAL: Stack member metadata corrupt!\n\tUnknown data type: {}\n\tat IP = {}",
                    other, vm.ip
                );
            }
        }
    }

    vm.ip += 1;
    return;
}
