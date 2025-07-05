#[derive(Debug, PartialEq)]
pub enum Exception {
    ZeroDivision,
    HeapAllocationFault,
    HeapFreeFault,
    HeapWriteFault,
}
