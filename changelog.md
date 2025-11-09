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
todo: todo.md
