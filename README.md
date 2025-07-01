# Simple Virutal machine using 64-bit registers, static typization, etc.

## Usage:
<<<<<<< HEAD
voxvm --vve=filename.vve  runs a vve (voxvm executable) file
      \--vvr=filename.vvr  runs a vvr (voxvm raw) file, not recommended
      \--init-ram=num  specifies a starting value of RAM for main memory (in bytes)
      \--stack-size=num  specifies a starting size of VM stack (in bytes)
      \--heap-size=num specifies a starting size of VM heap (in bytes)
      \--vas=filename  runs voxvm assembly with filename as input file
      \--vas-out=filename  specifies voxvm assembly output filename
      \--coredump_exit  coredumps after halt, saves it into `voxvm.dump` file
=======
voxvm --vve=filename.vve  runs a vve (voxvm executable) file\
      --vvr=filename.vvr  runs a vvr (voxvm raw) file, not recommended\
      --init-ram=num  specifies a starting value of RAM for main memory (in bytes)\
      --stack-size=num  specifies a starting size of VM stack (in bytes)\
      --heap-size=num specifies a starting size of VM heap (in bytes)\
      --vas=filename  runs voxvm assembly with filename as input file\
      --vas-out=filename  specifies voxvm assembly output filename\
>>>>>>> 1256e3186d7ec24c95c0a397c1bee5d1fa33025e

## Last implementations + todos:
| Thing                 | Is implemented |
|-----------------------|----------------|
<<<<<<< HEAD
| u64, i64, f64         | [X]            |
| ..their ops           | [X]            |
| jump ops, print       | [X]            |
| type convertions      | [X]            |
| voxasm rework         | [X]            |
| data segment          | [X]            |
| stack                 | []            |
| soon more..           | []             |
=======
| uint64 + ops          | ✅              |
| int64 + ops           | ✅              |
| float64 + ops         | ✅              |
| jump ops, print, halt | ✅              |
| type convs            | ✅              |
| voxasm rework         | ✅              |
| data segment          | 🕒              |
| stack                 | 🔴              |
| functions      `      | 🔴              |
| heap                  | 🔴              |
| gc                    | 🔴              |
| soon more..           | 🔴              |
>>>>>>> 1256e3186d7ec24c95c0a397c1bee5d1fa33025e

## Instruction (not really) actual info table: https://docs.google.com/spreadsheets/d/1bpkqAGjcDWKBDQTO2B2RmPHxcuN5EIkYm-DKRgUFVq8
