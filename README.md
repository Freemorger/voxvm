# Simple Virtual machine using 64-bit registers, static typization, etc.
## Features
- Own and simple bytecode
- Powerful heap allocator with custom "Split/merge first-fit" strategy
- Bytecode assembly
- .vve (voxvm executable) file format
- It's (comparably) fast :D   [10x faster than python 3.11 in my tests, at least]
- full docs will be available ~once~ soon

## Usage:
```
voxvm --vve=filename.vve  runs a vve (voxvm executable) file
      \--vvr=filename.vvr  runs a vvr (voxvm raw) file, not recommended
      \--init-ram=num  specifies a starting value of RAM for main memory (in bytes)
      \--init-stack-size=num  specifies a starting size of VM stack (in bytes)
      \--init-heap-size=num specifies a starting size of VM heap (in bytes)
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

## Repository structure
1. nconfigs/ - FFI examples
2. src/ - source code files
  - assembly.rs - voxvm assembly tool
  - callstack.rs - the call stack implementation
  - exceptions.rs - voxvm exceptions enum
  - fileformats.rs - tooling for voxvm's fileformats .vvr, .vve
  - func_ops.rs - function Instructions handlers
  - gc.rs - the GC (garbage collector) implementation
  - heap.rs - the heap implementation && Instructions handlers
  - main.rs - entry point
  - native.rs - FFI implementation
  - stack.rs - data stack implementation && instr handlers
  - tables.rs - default tables
  - vm.rs - main VM implementation
3. tools/ - currently used for .vvs (voxvm assembly) examples, the name is legacy
4. docs/ - will be once...

## How to run
`tools/input.vvs` contains a .vvs (voxvm assembly) program for latest version tests. You can run it (as well as any other .vvs program) like this:
```bash
./voxvm --vas=tools/input.vvs --vas-out=tools/program.vve
./voxvm --vve=tools/program.vve 
```
