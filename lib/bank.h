#ifndef BANK_H
#define BANK_H

#include "core.h"
#include "intrinsics.h"

// Pair a bank selector with its CPU-window address; a near pointer alone cannot select the bank.
typedef __packed struct BankPtr {
    u8 bank;
    const u8* ptr;
} BankPtr;

extern u8 kq_bank_current;

// Record the bank in the library shadow and invoke the mapper-aware compiler intrinsic.
void bank_switch(u8 bank);
// Read this wrapper's bank shadow; arbitrary mapper writes do not update it.
u8 bank_get_current(void);
// Read one byte from an explicit ROM bank and 16-bit CPU address.
u8 far_data_read8(u8 bank, const void* addr);
// Read a 16-bit value through the bank-aware compiler intrinsic.
u16 far_data_read16(u8 bank, const void* addr);
// Copy len bytes from banked ROM to caller-owned writable storage.
void far_data_read(u8 bank, const void* addr, void* dst, u16 len);
// Store a bank/address pair without reading memory; null output is ignored.
void farptr_make(BankPtr* out, u8 bank, const u8* ptr);
// Read one byte using both the bank and address of the supplied far pointer.
u8 farptr_read8(BankPtr ptr);
// Read a 16-bit value using both components of the far pointer.
u16 farptr_read16(BankPtr ptr);
// Copy len bytes from a far pointer; the caller provides a sufficiently large destination.
void farptr_read(BankPtr ptr, void* dst, u16 len);

// Route a declared zero-argument function through the far-call intrinsic; this is not a general
// callback-pointer dispatcher. Mapper support and fixed-bank call-site rules still apply.
#define far_call(bank, func) __farcall((bank), func)

#endif
