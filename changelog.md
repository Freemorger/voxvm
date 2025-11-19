CHANGELOG v8:
pre1:
- operations gsf (), usf () for getting/updating stack frames respectively
- native interface, see src/native.rs
pre2:
- major refactor of registers
pre3:
- much faster init now - just changed `system::new_all` -> `system::new`!
pre4:
- better ffi config
- some stuff i forgot
pre5:
- default ncalls wip
- print works
pre6:
- updated `store` and `load` instructions
- new instructions: `shl`, `shr`
- heap copy func
pre7: 
- `memcpy` instruction 
- tried to copy refs wit memcpy, not sure 
pre8: 
- fixes
- `storedat` instruction 
- `dlbc` instruction (dynamically load bytecode)
- `jmpr` instruction
- finally a pathway to v8 release. i hope so.
pre9:
- refactor 
- `nsysos::other`
- rand std ncalls
- runcmd ncall 
pre10:
- runcmd output fix
- `FileController`
- ncall 0x10 (fopen)
- ncall 0x11 (fclose)
todo: todo.md
