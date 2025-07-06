CHANGELOG v6:
pre1:
- fnstind (0x92) - saves function index (could be useful for higher order funcs)
- callr (0x93) - calls function by index from reg (see prev)
- exception system
- jexc (0x46) - jumps at addr if exception active
- dsaddr / addr
- alloc (0xa0) - allocate memory block in heap
- coredumps with only used blocks
- you can now creaaty zeroed data segment array using: `arr type[n] !zeros=N`
- heap allocator with my own strategy "Split/merge first-fit" works super!
v6-pre2:
- load (0xA4) instr
- !zeros for all types
- dssave fix
