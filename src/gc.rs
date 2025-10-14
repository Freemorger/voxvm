use std::collections::{HashMap, HashSet, VecDeque};

use crate::heap::HeapBlock;

#[derive(Debug)]
pub struct GC {
    // mark and sweep
    pub objects: Vec<GcObject>,
    pub main_refs: HashSet<u64>,
    pub t2_refs: HashMap<u64, HashSet<u64>>,
    unmarked: Vec<usize>, // indices
}

impl GC {
    pub fn new() -> GC {
        GC {
            objects: Vec::new(),
            unmarked: Vec::new(),
            main_refs: HashSet::new(),
            t2_refs: HashMap::new(),
        }
    }
    pub fn pin_object(&mut self, obj: GcObject) {
        self.objects.push(obj);
    }
    pub fn mark(&mut self, t1_refs: &HashSet<u64>, t2_refs: &HashMap<u64, HashSet<u64>>) {
        let refs: HashSet<u64> = self.main_refs.union(t1_refs).cloned().collect();
        self.t2_refs = t2_refs.clone();

        let mut reachable: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<u64> = VecDeque::new();

        for root in &refs {
            queue.push_back(*root);
        }

        while let Some(cur_ptr) = queue.pop_front() {
            if reachable.contains(&cur_ptr) {
                continue;
            }

            reachable.insert(cur_ptr);

            if let Some(referenced_ptrs) = t2_refs.get(&cur_ptr) {
                for ptr in referenced_ptrs {
                    if !reachable.contains(ptr) {
                        queue.push_back(*ptr);
                    }
                }
            }
        }

        self.unmarked.clear();
        for (idx, obj) in self.objects.iter_mut().enumerate() {
            if reachable.contains(&(obj.heap_ptr as u64)) {
                obj.marked = true;
            } else {
                obj.marked = false;
                self.unmarked.push(idx);
            }
        }
    }

    pub fn sweep(&mut self) -> Vec<u64> {
        // vec of ptr to heap object to remove
        let mut res: Vec<u64> = Vec::new();

        self.unmarked.sort_unstable_by(|a, b| b.cmp(a));
        self.unmarked.dedup();

        for &idx in self.unmarked.iter().rev() {
            if idx < self.objects.len() {
                let gc_obj = self.objects.remove(idx);
                res.push(gc_obj.heap_ptr);
            }
        }

        self.unmarked.clear();
        res
    }
}

#[derive(Debug)]
pub struct GcObject {
    heap_ptr: u64,
    marked: bool,
}

impl GcObject {
    pub fn new(ptr: u64) -> GcObject {
        GcObject {
            heap_ptr: ptr,
            marked: false,
        }
    }
}
