use rand::Rng;

// Allocator works at custom strategy "Split/merge first-fit":
// On allocation: find the first free block with at least n bytes of size,
// take only n bytes.
// On free: free the block, merge freed block with other free blocks nearby
use crate::vm::{RegTypes, VM, args_to_u64};

#[derive(Debug)]
pub struct Heap {
    pub heap: Vec<u8>,
    pub free_list: Vec<HeapBlock>,
    pub allocated: Vec<HeapBlock>,
}

impl Heap {
    pub fn new(heap_size: usize) -> Heap {
        let heap: Vec<u8> = Vec::with_capacity(heap_size);
        let freelist: Vec<HeapBlock> = vec![HeapBlock::new(0, heap_size.saturating_sub(1))];
        let alloced_list: Vec<HeapBlock> = Vec::new();
        Heap {
            heap: heap,
            free_list: freelist,
            allocated: alloced_list,
        }
    }
    pub fn alloc(&mut self, count_bytes: usize) -> Option<u64> {
        // Strategy: find first free block with at least `count_bytes` size;
        // Take only the needed part.
        for (ind, free_block) in self.free_list.iter_mut().enumerate() {
            if free_block.size >= count_bytes {
                let start_ptr = free_block.start_byte;
                let end_ptr = start_ptr + count_bytes;

                let new_alloc = HeapBlock::new(start_ptr, end_ptr);
                self.allocated.push(new_alloc);

                if (free_block.last_byte.saturating_sub(end_ptr) == 0) {
                    let _ = self.free_list.remove(ind);
                } else {
                    free_block.realloc(end_ptr + 1, free_block.last_byte);
                }

                return Some(start_ptr as u64);
            }
        }
        return None;
    }

    pub fn free(&mut self, ptr: u64) -> Result<(), ()> {
        // Strategy: free the block, merge with near free blocks.
        let mut freed_end: Option<usize> = None;
        let mut to_free: Option<usize> = None;
        for (ind, alloced_block) in self.allocated.iter().enumerate() {
            if alloced_block.start_byte == ptr as usize {
                freed_end = Some(alloced_block.last_byte);
                to_free = Some(ind);
                break;
            }
        }
        if (freed_end == None || to_free == None) {
            return Err(());
        }
        self.allocated.remove(to_free.unwrap());

        //Merging free blocks for less fragmentation
        let mut merges_count = 0;
        // let mut inds_to_rm: Vec<usize> = Vec::new();
        // for (ind, free_block) in self.free_list.iter_mut().enumerate() {
        //     if free_block.last_byte == ptr.saturating_sub(1) as usize {
        //         free_block.realloc(free_block.start_byte, freed_end.unwrap());
        //         merges_count += 1;
        //     }
        //     if free_block.start_byte == (freed_end.unwrap() + 1) {
        //         free_block.realloc(ptr as usize, free_block.last_byte);
        //         merges_count += 1;
        //     }
        // }
        if merges_count == 0 {
            let new_free_block: HeapBlock = HeapBlock::new(ptr as usize, freed_end.unwrap());
            self.free_list.push(new_free_block);
        }
        self.free_list
            .sort_by(|a, b| a.start_byte.cmp(&b.start_byte));
        self.merge_free_blocks();

        return Ok(());
    }

    fn merge_free_blocks(&mut self) {
        let mut cur_ind: usize = 0;
        while cur_ind < self.free_list.len() {
            let mut next_block_start: usize;
            let mut next_block_end: usize;
            {
                let next_block = match self.free_list.get(cur_ind + 1) {
                    Some(v) => v,
                    None => {
                        cur_ind += 1;
                        continue;
                    }
                };
                next_block_start = next_block.start_byte;
                next_block_end = next_block.last_byte;
            }
            let cur_block = match self.free_list.get_mut(cur_ind) {
                Some(v) => v,
                None => {
                    panic!("Can't get cur block while merging!");
                }
            };
            if cur_block.last_byte == next_block_start.saturating_sub(1) {
                cur_block.realloc(cur_block.start_byte, next_block_end);
                self.free_list.remove(cur_ind + 1);
                continue;
            }

            cur_ind += 1;
        }
    }

    pub fn free_all(&mut self) {
        let mut ptrs: Vec<u64> = Vec::new();
        for alloced_block in &self.allocated {
            ptrs.push(alloced_block.start_byte as u64);
        }
        for ptr in &ptrs {
            self.free(*ptr);
        }
    }

    pub fn write(&mut self, ptr: u64, data: Vec<u8>) -> Result<(), ()> {
        for alloced_block in &self.allocated {
            let last_towrite = ptr + (data.len()) as u64;
            // bounds check
            if (ptr >= alloced_block.start_byte as u64)
                && (ptr <= alloced_block.last_byte as u64)
                && (last_towrite <= alloced_block.last_byte as u64)
            {
                while (self.heap.len() < ptr as usize + 1)
                    && (self.heap.len() <= self.heap.capacity())
                {
                    self.heap.push(0);
                }
                for (ind, byte_towrite) in data.iter().enumerate() {
                    if ((ptr as usize) + ind + 1 > self.heap.len()) {
                        self.heap.push(*byte_towrite);
                        continue;
                    }
                    self.heap[(ptr as usize) + ind] = *byte_towrite;
                }
                return Ok(());
            }
        }
        Err(())
    }

    // for tests
    pub fn stress_heap(&mut self) {
        for _ in 0..1000 {
            let size_alloc = self.random_8_to_256() as u64;
            if rand::random::<f32>() < 0.5 {
                self.alloc(size_alloc as usize);
            }
        }
    }

    // for tests
    pub fn free_half(&mut self) {
        let mut inds: Vec<u64> = Vec::new();
        for block in &self.allocated {
            if rand::random::<bool>() {
                inds.push(block.start_byte as u64);
            }
        }
        for ind in &inds {
            self.free(*ind);
        }
    }

    fn random_8_to_256(&mut self) -> u32 {
        let mut rng = rand::rng();
        rng.random_range(8..=256)
    }
}

#[derive(Debug)]
pub struct HeapBlock {
    start_byte: usize,
    last_byte: usize,
    size: usize, // last - start
}

impl HeapBlock {
    pub fn new(start: usize, end: usize) -> HeapBlock {
        if (end < start) {
            panic!("While creating new heap block: end byte < start byte!")
        }
        HeapBlock {
            start_byte: start,
            last_byte: end,
            size: end - start,
        }
    }
    pub fn realloc(&mut self, start: usize, end: usize) {
        if (end < start) {
            panic!("While creating new heap block: end byte < start byte!")
        }
        self.start_byte = start;
        self.last_byte = end;
        self.size = end - start;
    }
}

pub fn op_alloc(vm: &mut VM) {
    // 0xA0, size: 10
    // alloc Rdest Size_bytes
    // Attempts to allocate size bytes of memory in heap;
    // Saves ptr to allocated block if allocation was successfull
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let size_bytes: u64 = args_to_u64(&vm.memory[(vm.ip + 2)..(vm.ip + 10)]);

    let res = match vm.heap.alloc(size_bytes as usize) {
        Some(addr) => addr,
        None => {
            vm.exceptions_active
                .push(crate::exceptions::Exception::HeapAllocationFault);
            0
        }
    };

    vm.registers[r_dest_ind] = res;
    vm.reg_types[r_dest_ind] = RegTypes::address;

    vm.ip += 10;
    return;
}

pub fn op_allocr(vm: &mut VM) {
    // 0xA3, size: 3
    // alloc Rdest Rsize
    // Attempts to allocate Rsize bytes of memory in heap;
    // Saves ptr to allocated block if allocation was successfull
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;
    let r_size_ind: usize = vm.memory[(vm.ip + 2)] as usize;
    let size_bytes: u64 = vm.registers[r_size_ind];

    let res = match vm.heap.alloc(size_bytes as usize) {
        Some(addr) => addr,
        None => {
            vm.exceptions_active
                .push(crate::exceptions::Exception::HeapAllocationFault);
            0
        }
    };

    vm.registers[r_dest_ind] = res;
    vm.reg_types[r_dest_ind] = RegTypes::address;

    vm.ip += 3;
    return;
}

pub fn op_free(vm: &mut VM) {
    // 0xA1, size: 2
    // free Rsrc
    // frees the heap memory block from ptr on Rsrc
    let r_src_ind: usize = vm.memory[(vm.ip + 1)] as usize;

    let r_src_val = vm.registers[r_src_ind];
    match vm.heap.free(r_src_val) {
        Ok(()) => {}
        Err(()) => {
            vm.exceptions_active
                .push(crate::exceptions::Exception::HeapFreeFault);
        }
    }

    vm.ip += 2;
    return;
}

pub fn op_store(vm: &mut VM) {
    // 0xA2, size: 3
    // store Rdest Rsrc
    // stores Rsrc val in heap addr.
    // No metadata, so Type safety on dev!
    let r_src_ind: usize = vm.memory[(vm.ip + 2)] as usize;
    let r_dest_ind: usize = vm.memory[(vm.ip + 1)] as usize;

    let val: u64 = vm.registers[r_src_ind];
    let ptr: u64 = vm.registers[r_dest_ind];
    match vm.heap.write(ptr, val.to_be_bytes().to_vec()) {
        Ok(()) => {}
        Err(()) => {
            vm.exceptions_active
                .push(crate::exceptions::Exception::HeapWriteFault);
        }
    }

    vm.ip += 3;
    return;
}
