#ifndef ASSET_H
#define ASSET_H

#include "core.h"
#include "bank.h"

#define ASSET_TYPE_RAW   ((u8)0)
#define ASSET_TYPE_TILES ((u8)1)
#define ASSET_TYPE_MAP   ((u8)2)
#define ASSET_TYPE_SONG  ((u8)3)

// The descriptor table bank and each payload bank are independent.
// ptr is a CPU-visible address and len counts bytes; type is a tag, not a decoder selection.
typedef __packed struct AssetDesc {
    u8 type;
    u8 bank;
    const u8* ptr;
    u16 len;
} AssetDesc;

// Retain a bank-qualified descriptor table; the caller keeps its storage valid.
void asset_set_table(u8 bank, const AssetDesc* table, u8 count);
// Copy a descriptor from its ROM bank into caller storage. Null output or an
// out-of-range ID returns zero without reading the table.
u8 asset_get(u8 asset_id, AssetDesc* out_desc);
// Copy the smaller of the asset length and max_len using a bank-aware transfer.
// Return one for a valid ID even when the payload is truncated.
u8 asset_load_raw(u8 asset_id, void* dst, u16 max_len);
// Switch to the asset bank, upload tile/raw data in chunks of at most 128 bytes,
// then restore the bank recorded by this library. The caller supplies safe PPU timing.
u8 asset_load_tiles(u8 asset_id, u16 vram_dst);
// Return the asset bank or zero on lookup failure; bank zero may also be valid.
u8 asset_get_bank(u8 asset_id);
// Return the stored address without switching banks. Pair it with the descriptor
// bank before reading banked ROM; a failed lookup returns null.
const u8* asset_get_ptr(u8 asset_id);
// Return the descriptor byte length, or zero for an invalid ID.
u16 asset_get_len(u8 asset_id);

#endif
