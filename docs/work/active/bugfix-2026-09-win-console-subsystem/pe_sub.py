import struct, sys
def subsystem(path):
    with open(path,'rb') as f:
        d = f.read()
    pe = struct.unpack_from('<I', d, 0x3C)[0]
    opt = pe + 24
    sub = struct.unpack_from('<H', d, opt+68)[0]
    return {2:'GUI', 3:'CONSOLE'}.get(sub, str(sub))
print(sys.argv[1], '->', subsystem(sys.argv[1]))
