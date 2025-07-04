use crate::vm::{VM, args_to_u64};

pub fn op_call(vm: &mut VM) {
    // 0x90, size: 9
    // call ind (index of function in func table)
    if (vm.call_stack.len() + 1 > vm.rec_depth_max) {
        panic!("Recursion depth exceed at IP = {}!", vm.ip);
    } else {
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
