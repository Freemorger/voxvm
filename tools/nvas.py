# vvas.py will be deprecated soon. 
# Not all features may be available in vvas.py.
# Consider using vvas2 in voxvm --vvas=...

import struct

def ishex(s):
    try:
        int(s, 16)
        return True
    except ValueError:
        return False


INPUT_FILE_NAME = "input.vvs"
OUTPUT_FILE_NAME = "program_asm.vvr"

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
    "nop": [0x2, 1, 0],
    "uload": [0x10, 10, 1, 8],
    "uadd": [0x11, 3, 1, 1],
    "umul": [0x12, 3, 1, 1],
    "usub": [0x13, 3, 1, 1],
    "udiv": [0x14, 4, 1, 1, 1],
    "urem": [0x15, 4, 1, 1, 1],
    "ucmp": [0x16, 3, 1, 1],
    "iload": [0x20, 10, 1, 8],
    "iadd": [0x21, 3, 1, 1],
    "imul": [0x22, 3, 1, 1],
    "isub": [0x23, 3, 1, 1],
    "idiv": [0x24, 4, 1, 1, 1],
    "irem": [0x25, 4, 1, 1, 1],
    "icmp": [0x26, 3, 1, 1],
    "fload": [0x30, 10, 1, 8],
    "fadd": [0x31, 3, 1, 1],
    "fmul": [0x32, 3, 1, 1],
    "fsub": [0x33, 3, 1, 1],
    "fdiv": [0x34, 4, 1, 1, 1],
    "frem": [0x35, 4, 1, 1, 1],
    "fcmp": [0x36, 3, 1, 1],
    "fcmp_eps": [0x37, 3, 1, 1],
    "jmp": [0x40, 9, 8],
    "jz": [0x41, 9, 8],
    "jl": [0x42, 9, 8],
    "jg": [0x43, 9, 8],
    "jge": [0x44, 9, 8],
    "jle": [0x45, 9, 8],
    "utoi": [0x50, 3, 1, 1],
    "itou": [0x51, 3, 1, 1],
    "utof": [0x52, 3, 1, 1],
    "itof": [0x53, 3, 1, 1],
    "ftou": [0x54, 3, 1, 1],
    "ftoi": [0x55, 3, 1, 1],
}

labels = {"": 0}
halt_ptr = 0x0
def save_label(line) -> None:
    line = line.strip().split()
    labelname = line[1]
    if (labelname.isdigit()): #or (ishex(labelname)):
        print("Labelname should not be only numbers!")
    else:
        print("saving label:", labelname)
        labels[labelname] = addr_ctr
        return



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
        print("problems")



special_instr = {
    "label": save_label,
    "goto": goto
}


#first stage
input_file.seek(0)
lines1 = input_file.readlines()
for line in lines1:
    lexems = line.strip().split()
    if not lexems:
        continue
    print("FIRST: ", lexems)
    if (lexems[0] == "label"):
        save_label(line)
        continue
    if (lexems[0] == "goto"):
        addr_ctr += 9
        continue
    if (lexems[0] == "halt"):
        halt_ptr = addr_ctr
        break

    instr = instr_formats.get(lexems[0])
    if (instr == None):
        print("unknown instr: ", lexems[0])
        continue
    addr_ctr += instr[1]



#assemble
addr_ctr = 0
input_file.seek(0)
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

    if (instr[0] >= 0x40) and (instr[0] < 0x50):
        tgt_addr = 0x0
        tgt_addr = labels.get(lexems[1])
        if (tgt_addr == None):
            try:
                tgt_addr = int(lexems[1], 16)
            except ValueError:
                print("Can't parse ", lexems[1], " into address. Perhaps you inputed wrong label name?")
                exit(1)

        print("DBG tgt_addr: ", tgt_addr)
        instr_b[1:10] = tgt_addr.to_bytes(8, "big")
        print(instr_b)
        output_file.write(instr_b)
        addr_ctr += len(instr_b)
        print()
        continue

    if (instr_len < 2):
        output_file.write(instr_b)
        continue
    for i, arg in enumerate(lexems[1:]):
        if ("#" in arg) or (";" == arg):
            break

        if ("r" in arg):
            starting = sum(instr[2:(i+2)]) + 1
            instr_b[starting:(starting + 1 + 1)] = int(arg[1:]).to_bytes(1, "big")
        elif ("." in arg):
            starting = sum(instr[2:(i+2)]) + 1
            instr_b[starting:(starting + 8 + 1)] = struct.pack(">d", float(arg))
        else:
            isSigned: bool = False
            if (instr[0] >= 0x20) and (instr[0] < 0x30):
                isSigned = True
            print("DBG: isSigned = ", isSigned)
            arglen = instr[2 + i]
            starting = sum(instr[2:(i+2)]) + 1
            try:
                instr_b[starting:(starting + arglen + 1)] = int(arg).to_bytes(arglen, "big", signed=isSigned)
            except ValueError:
                instr_b[starting:(starting + arglen + 1)] = int(arg, 16).to_bytes(arglen, "big", signed=isSigned)


    print(instr_b)
    output_file.write(instr_b)
    addr_ctr += len(instr_b)
    print()

print(addr_ctr)
