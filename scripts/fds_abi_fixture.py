"""Original MIT CPU fixture for the documented LoadFiles calling convention.

This does not emulate BIOS boot, the disk controller, CRC, motor timing or saves.
It copies original test payloads packaged by KITAQFC and logs requested IDs.
Never present a passing fixture run as physical-disk or real-BIOS verification.
"""
# Copyright (c) 2026 DAISUKE OBA
from pathlib import Path

def disk_files(path):
    raw=Path(path).read_bytes();assert raw[:4]==b'FDS\x1a'
    files=[]
    for side_number in range(raw[4]):
        side=raw[16+65500*side_number:16+65500*(side_number+1)]
        assert side[0]==1 and side[56]==2;offset=58
        for ordinal in range(side[57]):
            assert side[offset]==3;h=side[offset:offset+16];size=int.from_bytes(h[13:15],'little')
            assert side[offset+16]==4 and offset+17+size<=len(side)
            files.append(dict(ordinal=ordinal,number=h[1],name=bytes(h[3:11]),id=h[2],address=int.from_bytes(h[11:13],'little'),type=h[15],boot=h[2]<=side[25],side=side_number,data=side[offset+17:offset+17+size]))
            offset+=17+size
    return files

def loadfiles_firmware(files,failures=None,clobber_arguments=True,failure_after_copy=False):
    """Generate independent 6502 code at E1F8; failures maps 1-based call to A.

    Game tests reserve 0700..07FF. 0700 counts calls, 0701 flags a malformed list,
    0702 stores the last ID, and 0780 onward records IDs. Maximum 64 loads.
    The fixture restores every original payload byte, including trailing FF,
    while encoding repeated FF bytes compactly to fit its own 8 KiB image.
    """
    image=bytearray(8192);code=bytearray();data_at=0xEC00;failures=failures or {}
    emit=lambda *values:code.extend(values)
    def absolute(op,address):emit(op,address&255,address>>8)
    def here():return 0xE400+len(code)
    def skip_if_not_equal():
        emit(0xF0,3,0x4C,0,0);return len(code)-2
    def patch_jump(at):code[at:at+2]=here().to_bytes(2,'little')
    def return_status(status):
        if clobber_arguments:
            emit(0xA9,0xD6)
            for address in range(0xC0,0xD0):emit(0x85,address)
        emit(0xA2,0x59,0xA0,0xA3,0xA9,status,0x60)
    def injected_failure():
        for count,error in sorted(failures.items()):
            absolute(0xAD,0x0700);emit(0xC9,count);jump=skip_if_not_equal()
            return_status(error);patch_jump(jump)
    def fill_ff(address,length):
        emit(0xA9,255)
        for start in range(0,length,256):
            count=min(256,length-start);emit(0xA2,0);loop=len(code)
            absolute(0x9D,address+start);emit(0xE8)
            if count<256:emit(0xE0,count)
            emit(0xD0,(loop-len(code)-2)&255)
    image[0x1F8:0x1FB]=bytes([0x4C,0x00,0xE4])
    # The two return-stack bytes point just before the inline pointer words.
    emit(0xBA);absolute(0xBD,0x0101);emit(0x85,0xF0);absolute(0xBD,0x0102);emit(0x85,0xF1)
    emit(0xA0,3,0xB1,0xF0,0x85,0xF2,0xC8,0xB1,0xF0,0x85,0xF3)
    emit(0xA0,0,0xB1,0xF2);absolute(0x8D,0x0702)
    absolute(0xAC,0x0700);absolute(0x99,0x0780);absolute(0xEE,0x0700)
    emit(0xA0,1,0xB1,0xF2,0xC9,255,0xF0,3);absolute(0xEE,0x0701)
    emit(0x18,0xA5,0xF0,0x69,4);absolute(0x9D,0x0101)
    emit(0xA5,0xF1,0x69,0);absolute(0x9D,0x0102)
    if not failure_after_copy:injected_failure()
    for f in files:
        if f['type']!=0 or f['address']!=0x6000:continue
        raw=f['data'];data=raw.rstrip(b'\xff');assert data_at+len(data)<0xFFF0
        image[data_at-0xE000:data_at-0xE000+len(data)]=data
        absolute(0xAD,0x0702);emit(0xC9,f['id']);jump=skip_if_not_equal()
        fill_ff(f['address'],len(raw))
        for start in range(0,len(data),256):
            count=min(256,len(data)-start);emit(0xA2,0);loop=len(code)
            absolute(0xBD,data_at+start);absolute(0x9D,f['address']+start);emit(0xE8)
            if count<256:emit(0xE0,count)
            emit(0xD0,(loop-len(code)-2)&255)
        if failure_after_copy:injected_failure()
        return_status(0);patch_jump(jump);data_at+=len(data)
    return_status(0x40)
    assert len(code)<=0x800,'Fixture code exceeds reserved space'
    image[0x400:0x400+len(code)]=code
    return bytes(image)
