#include <stdint.h>
#include <stdio.h>

typedef struct VMValue {
    uint32_t typeind;
    uint64_t data;
} VMValue;

VMValue unsigned_add(VMValue* args, uint32_t argc) {
    if (argc < 2) {
        return (VMValue){.typeind=1, .data=0}; // 1 - uint
    }
    uint64_t a = args[0].data;
    uint64_t b = args[1].data;

    uint64_t res = a + b;
    return (VMValue){.typeind=1, .data=res};
}
