#include "bank.h"

u8 kq_bank_current;

void bank_switch(u8 bank)
{
    kq_bank_current = bank;
    __bankswitch(bank);
}

u8 bank_get_current(void)
{
    return kq_bank_current;
}

u8 far_data_read8(u8 bank, const void* addr)
{
    return __farpeek8(bank, (u16)addr);
}

u16 far_data_read16(u8 bank, const void* addr)
{
    return __farpeek16(bank, (u16)addr);
}

void far_data_read(u8 bank, const void* addr, void* dst, u16 len)
{
    __far_memcpy((u8*)dst, bank, (u16)addr, len);
}

void farptr_make(BankPtr* out, u8 bank, const u8* ptr)
{
    if (out == 0) return;
    out->bank = bank;
    out->ptr = ptr;
}

u8 farptr_read8(BankPtr ptr)
{
    return __farpeek8(ptr.bank, (u16)ptr.ptr);
}

u16 farptr_read16(BankPtr ptr)
{
    return __farpeek16(ptr.bank, (u16)ptr.ptr);
}

void farptr_read(BankPtr ptr, void* dst, u16 len)
{
    __far_memcpy((u8*)dst, ptr.bank, (u16)ptr.ptr, len);
}
