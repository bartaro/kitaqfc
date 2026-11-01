#ifndef BANK_H
#define BANK_H

#include "core.h"
#include "intrinsics.h"

typedef __packed struct BankPtr {
    u8 bank;
    const u8* ptr;
} BankPtr;

extern u8 kq_bank_current;

void bank_switch(u8 bank);
u8 bank_get_current(void);
u8 far_data_read8(u8 bank, const void* addr);
u16 far_data_read16(u8 bank, const void* addr);
void far_data_read(u8 bank, const void* addr, void* dst, u16 len);
void farptr_make(BankPtr* out, u8 bank, const u8* ptr);
u8 farptr_read8(BankPtr ptr);
u16 farptr_read16(BankPtr ptr);
void farptr_read(BankPtr ptr, void* dst, u16 len);

#define far_call(bank, func) __farcall((bank), func)

#endif
