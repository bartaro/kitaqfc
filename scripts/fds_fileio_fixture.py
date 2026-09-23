"""Original MIT fixture for LoadFiles/WriteFile ABI, not real disk emulation.

No BIOS bytes are incorporated. The fixture models a single PRG save slot in
RAM at $9000, verifies inline argument delivery, and returns injected errors.
It does not model disk timing, CRC, power loss, or physical write protection.
"""
# Copyright (c) 2026 DAISUKE OBA

def fileio_firmware(file, load_error=0, save_error=0, loaded_count=1):
    image=bytearray(8192);code=bytearray();labels={};fixups=[]
    def emit(*v):code.extend(v)
    def absolute(op,a):emit(op,a&255,a>>8)
    def mark(s):labels[s]=0xE400+len(code)
    def jump(s,op=0x4c):emit(op,0,0);fixups.append((len(code)-2,s))
    def branch(op,s):emit({0xf0:0xd0,0xd0:0xf0,0x90:0xb0,0xb0:0x90}[op],3);jump(s)
    def pointer_args():
        emit(0xba);absolute(0xbd,0x101);emit(0x85,0);absolute(0xbd,0x102);emit(0x85,1)
        # Decode both inline pointer words, skipping them on return.
        emit(0xa0,1,0xb1,0,0x85,2,0xc8,0xb1,0,0x85,3,0xc8,0xb1,0,0x85,4,0xc8,0xb1,0,0x85,5)
        emit(0x18,0xa5,0,0x69,4);absolute(0x9d,0x101);emit(0xa5,1,0x69,0);absolute(0x9d,0x102)
        # Record the disk ID passed by the wrapper for independent assertions.
        emit(0xa0,0);mark('disk_id_'+str(len(code)));loop=len(code)
        emit(0xb1,2);absolute(0x99,0x710);emit(0xc8,0xc0,10,0xd0,(loop-len(code)-5)&255)
    def result(a,y):
        emit(0xa9,0xd6)
        for i in range(0xc0,0xd0):emit(0x85,i)
        for i in range(16):emit(0x85,i)
        emit(0xa2,0x59,0xa0,y,0xa9,a,0x60)
    def copy_fixed(src,dst,size):
        for start in range(0,size,256):
            n=min(256,size-start);emit(0xa2,0);loop=len(code)
            absolute(0xbd,src+start);absolute(0x9d,dst+start);emit(0xe8)
            if n<256:emit(0xe0,n)
            emit(0xd0,(loop-len(code)-2)&255)
    # These are ABI entry addresses, not bytes extracted from any BIOS.
    image[0x1f8:0x1fb]=bytes([0x4c,0,0xe4]);mark('load')
    absolute(0xee,0x700);pointer_args()
    emit(0xa0,0,0xb1,4);absolute(0x8d,0x702)
    emit(0xc9,file['id']);branch(0xd0,'missing')
    emit(0xc8,0xb1,4,0xc9,255);branch(0xd0,'missing')
    if load_error:result(load_error,0)
    else:
        absolute(0xad,0x703);branch(0xd0,'saved')
        copy_fixed(0xf000,file['address'],len(file['data']));jump('loaded')
        mark('saved');copy_fixed(0x9000,file['address'],len(file['data']));mark('loaded');result(0,loaded_count)
    mark('missing');result(0x40,0)
    mark('save');absolute(0x8d,0x701);absolute(0xee,0x704);pointer_args()
    emit(0xa0,0);loop=len(code);emit(0xb1,4);absolute(0x99,0x720);emit(0xc8,0xc0,17,0xd0,(loop-len(code)-5)&255)
    if save_error:result(save_error,0)
    else:
        # Copy from the descriptor's CPU source; independent expected bytes are checked outside this fixture.
        emit(0xa0,14,0xb1,4,0x85,6,0xc8,0xb1,4,0x85,7)
        for start in range(0,len(file['data']),256):
            n=min(256,len(file['data'])-start);emit(0xa0,0);loop=len(code);emit(0xb1,6);absolute(0x99,0x9000+start);emit(0xc8)
            if n<256:emit(0xc0,n)
            emit(0xd0,(loop-len(code)-2)&255,0xe6,7)
        emit(0xa9,1);absolute(0x8d,0x703);result(0,0)
    for at,s in fixups:code[at:at+2]=labels[s].to_bytes(2,'little')
    image[0x239:0x23c]=bytes([0x4c,labels['save']&255,labels['save']>>8])
    assert len(code)<0xc00 and len(file['data'])<=0xff0
    image[0x400:0x400+len(code)]=code;image[0x1000:0x1000+len(file['data'])]=file['data']
    return bytes(image)
