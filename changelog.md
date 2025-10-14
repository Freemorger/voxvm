CHANGELOG v7:
pre1:
- formated sizes in args, init messages (e.g. `--init-ram=100MB`)
- inc/dec instructions
- data segment const (as a flag of type)
- gota do gc next!
pre2:
- GC
- now allocates memory based on available memory by default
- stack, callstack separate structs

todo: getting refs for gc from ds
