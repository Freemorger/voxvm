#[derive(Debug)]
pub struct CallStack {
    pub stack: Vec<CSFrame>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            stack: (Vec::new()),
        }
    }

    pub fn push(&mut self, retaddr: u64) {
        self.stack.push(CSFrame::new(retaddr));
    }

    pub fn pop(&mut self) -> Option<u64> {
        match self.stack.pop() {
            Some(val) => {
                //println!("Poping addr {}", val.retaddr);
                Some(val.retaddr)
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct CSFrame {
    retaddr: u64,
    locals: Vec<u64>,
    checked: bool,
}

impl CSFrame {
    pub fn new(addr: u64) -> CSFrame {
        CSFrame {
            retaddr: (addr),
            locals: (Vec::new()),
            checked: (false),
        }
    }
}
