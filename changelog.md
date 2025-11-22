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
pre11:
- FS ncalls finished!
    - ncall 0x12 (fwrite)
    - ncall 0x13 (fread)
    - ncall 0x14 (fdel)
    - ncall 0x15 (fseekget)
    - ncall 0x16 (fseekset)
pre12:
- Networking std ncalls is here:
    - ncall 0x20 (nc_open)
    - ncall 0x21 (nc_close)
    - ncall 0x22 (nc_accept)
    - ncall 0x23 (nc_write)
    - ncall 0x24 (nc_read)
    - ncall 0x25 (nc_getaddr)
- `ubd` instruction for easy UTF-8 => UTF-16 convertions inside the heap
- seems like v8 finished!
todo: todo.md
