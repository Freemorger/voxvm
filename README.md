# Simple Virutal machine using 64-bit registers, static typization, etc.

## Usage:
voxvm --vve=filename.vve  runs a vve (voxvm executable) file
      --vvr=filename.vvr  runs a vvr (voxvm raw) file, not recommended
      --init-ram=num  specifies a starting value of RAM for main memory (in bytes)
      --stack-size=num  specifies a starting size of VM stack (in bytes)
      --heap-size=num specifies a starting size of VM heap (in bytes)
      --vas=filename  runs voxvm assembly with filename as input file
      --vas-out=filename  specifies voxvm assembly output filename

## Last implementations + todos:
| Thing                 | Is implemented |
|-----------------------|----------------|
| uint64 + ops          | ✅              |
| int64 + ops           | ✅              |
| float64 + ops         | ✅              |
| jump ops, print, halt | ✅              |
| type convs            | ✅              |
| voxasm rework         | ✅              |
| data segment          | 🔴              |
| soon more..           | 🔴              |

## Instruction actual info table: https://docs.google.com/spreadsheets/d/1bpkqAGjcDWKBDQTO2B2RmPHxcuN5EIkYm-DKRgUFVq8
