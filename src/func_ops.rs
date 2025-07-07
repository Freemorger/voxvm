use crate::vm::{RegTypes, VM, args_to_u64};

pub fn op_call(vm: &mut VM) {
    // 0x90, size: 9
    // call ind (index of function in func table)
    if (vm.call_stack.len() + 1 > vm.rec_depth_max) {
        panic!("Recursion depth exceed at IP = {}!", vm.ip);
    }

    let ind: u64 = args_to_u64(&vm.memory[(vm.ip + 1)..(vm.ip + 9)]);
    let tojmp: u64 = match vm.func_table.get(ind as usize) {
        Some(v) => *v,
        None => {
            panic!(
                "Function with index {} can't be found in function table",
                ind
            );
        }
    };

    vm.call_stack.push((vm.ip + 9) as u64);
    vm.ip = tojmp as usize;
}

pub fn op_ret(vm: &mut VM) {
    // 0x91, size: 1
    // ret (returns to return address from call stack
    let ret_addr: u64 = match vm.call_stack.pop() {
        Some(addr) => addr,
        None => {
            panic!(
                "Attempting to return but call stack is empty!\n\tAt IP = {}",
                vm.ip
            );
        }
    };

    vm.ip = ret_addr as usize;
}

pub fn op_fnstind(vm: &mut VM) {
    // 0x92, size: 10
    // fnstind Rdest ind - saves function index as u64.
    // Potentially a thing for higher order functions.
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let ind: u64 = args_to_u64(&vm.memory[(vm.ip + 2)..(vm.ip + 10)]);

    vm.registers[r_dest_ind] = ind;
    vm.reg_types[r_dest_ind] = RegTypes::uint64;

    vm.ip += 10;
    return;
}

pub fn op_callr(vm: &mut VM) {
    // 0x93, size: 2
    // callr Rsrc - calls instr by its function table register.
    let r_src_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let ind: usize = vm.registers[r_src_ind] as usize;

    let addr = match vm.func_table.get(ind) {
        Some(v) => v,
        None => {
            panic!(
                "Can't get function with index {}!\n\tAt IP = {}",
                ind, vm.ip
            );
        }
    };

    vm.call_stack.push((vm.ip + 2) as u64);
    vm.ip = *addr as usize;
}
