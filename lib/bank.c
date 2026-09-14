#include "bank.h"

// Zero-initialized software state does not discover the cartridge startup mapping.
u8 kq_bank_current;

// Record the bank in the library shadow and invoke the mapper-aware compiler intrinsic.
void bank_switch(u8 bank)
{
    kq_bank_current = bank;
    __bankswitch(bank);
}

// Read this wrapper's bank shadow; arbitrary mapper writes do not update it.
u8 bank_get_current(void)
{
    return kq_bank_current;
}

// Read one byte from an explicit ROM bank and 16-bit CPU address.
u8 far_data_read8(u8 bank, const void* addr)
{
    return __farpeek8(bank, (u16)addr);
}

// Read a 16-bit value through the bank-aware compiler intrinsic.
u16 far_data_read16(u8 bank, const void* addr)
{
    return __farpeek16(bank, (u16)addr);
}

// Copy len bytes from banked ROM to caller-owned writable storage.
void far_data_read(u8 bank, const void* addr, void* dst, u16 len)
{
    __far_memcpy((u8*)dst, bank, (u16)addr, len);
}

// Store a bank/address pair without reading memory; null output is ignored.
void farptr_make(BankPtr* out, u8 bank, const u8* ptr)
{
    if (out == 0) return;
    out->bank = bank;
    out->ptr = ptr;
}

// Read one byte using both the bank and address of the supplied far pointer.
u8 farptr_read8(BankPtr ptr)
{
    return __farpeek8(ptr.bank, (u16)ptr.ptr);
}

// Read a 16-bit value using both components of the far pointer.
u16 farptr_read16(BankPtr ptr)
{
    return __farpeek16(ptr.bank, (u16)ptr.ptr);
}

// Copy len bytes from a far pointer; the caller provides a sufficiently large destination.
void farptr_read(BankPtr ptr, void* dst, u16 len)
{
    __far_memcpy((u8*)dst, ptr.bank, (u16)ptr.ptr, len);
}
