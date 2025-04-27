import re

INPUT_FILE_NAME = "input.nvs"
OUTPUT_FILE_NAME = "program_asm.nvb"

input_file = open(INPUT_FILE_NAME, "r")

with open(OUTPUT_FILE_NAME, "w"):
    pass # cleaning output file is already existing
output_file = open(OUTPUT_FILE_NAME, "ab")

addr_ctr = 0x0
instr_formats = {
    # Format:
    # Opcode, oplen, argslen
    "halt": [0xFF, 1, 0, 0],
    "ncall": [0x1, 4, 2, 1],
    "uload": [0x10, 10, 1, 8],
    "uadd": [0x11, 3, 1, 1],
    "umul": [0x12, 3, 1, 1],
    "usub": [0x13, 3, 1, 1],
    "udiv": [0x14, 4, 1, 1, 1],
    "urem": [0x15, 4, 1, 1, 1],
    "ucmp": [0x16, 3, 1, 1],
    "jmp": [0x40, 9, 8],
    "jz": [0x41, 9, 8],
    "jn": [0x42, 9, 8],
    "jg": [0x43, 9, 8],
}

labels = {"": 0}
def save_label(line) -> None:
    line = line.strip().split()
    labelname = line[1]
    print("saving label:", labelname)
    labels[labelname] = addr_ctr

def goto(line):
    line = line.strip().split()
    labelname = line[1]
    print(labelname)
    target_addr = labels.get(labelname)
    jmp_instr = instr_formats.get("jmp")
    if (jmp_instr is not None):
        jmp_op = jmp_instr[0]
        if (target_addr is not None) and (jmp_op is not None):
            target_addr = int(target_addr).to_bytes(8, "big")
            print("target addr:", target_addr, " jmp op: ", jmp_op)
            res = bytearray([jmp_op])
            res.extend(target_addr)
            return res

        else:
            print("ERROR: Label name ", labelname, " is unknown")
            return bytearray()

    else:
        print("problenms")



special_instr = {
    "label": save_label,
    "goto": goto
}



lines = input_file.readlines()
for line in lines:
    lexems = line.strip().split()
    if not lexems:
        continue
    print(lexems)

    spec = special_instr.get(lexems[0])
    if (spec):
        spec_res = spec(line)
        if (spec_res is not None):
            print(spec_res)
            output_file.write(spec_res)
            addr_ctr += len(spec_res)
        continue

    instr = instr_formats.get(lexems[0])
    if (instr == None): continue

    opcode = instr[0]

    instr_len = instr[1]

    #print(instr)

    instr_b = bytearray(instr_len)
    instr_b[0] = instr[0]

    if (instr_len < 2):
        output_file.write(instr_b)
        continue
    for i, arg in enumerate(lexems[1:]):
        if ("r" in arg):
            starting = sum(instr[2:(i+2)]) + 1
            instr_b[starting:(starting + 1 + 1)] = int(arg[1:]).to_bytes(1, "big")
        else:
            arglen = instr[2 + i]
            starting = sum(instr[2:(i+2)]) + 1
            try:
                instr_b[starting:(starting + arglen + 1)] = int(arg).to_bytes(arglen, "big")
            except ValueError:
                instr_b[starting:(starting + arglen + 1)] = int(arg, 16).to_bytes(arglen, "big")


    print(instr_b)
    output_file.write(instr_b)
    addr_ctr += len(instr_b)
    print()

print(addr_ctr)
