# Simple Virtual machine using 64-bit registers, static typization, etc.
## Features
- Own and simple bytecode
- Powerful heap allocator with custom "Split/merge first-fit" strategy
- Bytecode assembly
- .vve (voxvm executable) file format
- It's (comparably) fast :D   [10x faster than python in my tests, at least]
- full docs will be available ~once~ soon

## Usage:
```
voxvm --vve=filename.vve  runs a vve (voxvm executable) file
      \--vvr=filename.vvr  runs a vvr (voxvm raw) file, not recommended
      \--init-ram=num  specifies a starting value of RAM for main memory (in bytes)
      \--stack-size=num  specifies a starting size of VM stack (in bytes)
      \--heap-size=num specifies a starting size of VM heap (in bytes)
      \--vas=filename  runs voxvm assembly with filename as input file
      \--vas-out=filename  specifies voxvm assembly output filename
      \--coredump_exit  coredumps after halt, saves it into `voxvm.dump` file
      \--max-recursion sets maximal recursion limit
      \--native-configs specifies directory with native libraries configs
```

## Last implementations + todos:
| Thing                 | Is implemented |
|-----------------------|----------------|
| u64, i64, f64         | [X]            |
| ..their ops           | [X]            |
| jump ops, print       | [X]            |
| type convertions      | [X]            |
| voxasm rework         | [X]            |
| data segment          | [X]            |
| stack                 | [X]            |
| functions, call stack | [X]            |
| heap                  | [X]            |
| GC                    | [X]            |
| better ffi            | [~]            |
| soon more..           | []             |

## Instruction (not really) actual info table: https://docs.google.com/spreadsheets/d/1bpkqAGjcDWKBDQTO2B2RmPHxcuN5EIkYm-DKRgUFVq8
