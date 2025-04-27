import re

INPUT_FILE_NAME = "input.nvs"
OUTPUT_FILE_NAME = "program_asm.nvb"

input_file = open(INPUT_FILE_NAME, "r")

with open(OUTPUT_FILE_NAME, "w"):
    pass # cleaning output file is already existing
output_file = open(OUTPUT_FILE_NAME, "ab")

instr_formats = {
    # Format:
    # Opcode, oplen, argslen
    "halt": [0x0, 1, 0, 0],
    "ncall": [0x1, 4, 2, 1],
    "uload": [0x10, 10, 1, 8],
    "uadd": [0x11, 3, 1, 1],
    "umul": [0x12, 3, 1, 1],
    "usub": [0x13, 3, 1, 1],
    "jmp": [0x40, 9, 8],
    "jz": [0x41, 9, 8]
}

lines = input_file.readlines()
for line in lines:
    lexems = line.strip().split()
    if not lexems:
        continue
    print(lexems)
    instr = instr_formats.get(lexems[0])
    if (instr == None): continue

    opcode = instr[0]
    instr_len = instr[1]

    print(instr)

    instr_b = bytearray(instr_len)
    instr_b[0] = instr[0]

    if (instr_len < 2):
        print("less")
        print(instr_b)
        continue

        #
    print("lexems1: : ", lexems[1:])
    for i, arg in enumerate(lexems[1:]):
        print("curarg: ", arg, " i = ", i)
        if ("r" in arg):
            starting = sum(instr[2:(i+2)]) + 1
            print("register num:", int(arg[1:]), " starting: ", starting, " sum: ", sum(instr[2::(i+3)]))
            print("instr to ", (instr[2:(i+2)]))
            instr_b[starting:(starting + 1 + 1)] = int(arg[1:]).to_bytes(1, "big")
        else:
            try:
                arglen = instr[2 + i]
                starting = sum(instr[2:(i+2)]) + 1
                print("starting: ", starting, "sum:", sum(instr[2::(i+3)]))
                instr_b[starting:(starting + arglen + 1)] = int(arg).to_bytes(arglen, "big")
                print(int(arg).to_bytes(arglen, "big"))
            #except ValueError:
                #print(lexems[1], " is not a number.")
            finally: pass

    print(instr_b)
    output_file.write(instr_b)
    print()
