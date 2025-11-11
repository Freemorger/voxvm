#[derive(Debug, PartialEq)]
pub enum Exception {
    ZeroDivision,
    HeapAllocationFault,
    HeapFreeFault,
    HeapWriteFault,
    HeapReadFault,
    NegativeSqrt,
    InvalidDataType,
    NativeFault,
    IncorrectRegType,
    HeapSegmFault,
    MainSegmFault,
}
