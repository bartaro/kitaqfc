#include "asset.h"
#include "intrinsics.h"

static u8 asset_table_bank;
static const AssetDesc* asset_table;
static u8 asset_table_count;

void asset_set_table(u8 bank, const AssetDesc* table, u8 count)
{
    asset_table_bank = bank;
    asset_table = table;
    asset_table_count = count;
}

u8 asset_get(u8 asset_id, AssetDesc* out_desc)
{
    if (out_desc == 0) return 0;
    if (asset_id >= asset_table_count) return 0;
    __far_memcpy((u8*)out_desc, asset_table_bank, (u16)(asset_table + asset_id), sizeof(AssetDesc));
    return 1;
}

u8 asset_load_raw(u8 asset_id, void* dst, u16 max_len)
{
    AssetDesc desc;
    u16 len;
    if (asset_get(asset_id, &desc) == 0) return 0;
    len = desc.len;
    if (len > max_len) len = max_len;
    __far_memcpy((u8*)dst, desc.bank, (u16)desc.ptr, len);
    return 1;
}

u8 asset_load_tiles(u8 asset_id, u16 vram_dst)
{
    AssetDesc desc;
    const u8* src;
    u16 len;
    u8 chunk;
    u8 old_bank;

    if (asset_get(asset_id, &desc) == 0) return 0;
    if (desc.type != ASSET_TYPE_TILES && desc.type != ASSET_TYPE_RAW) return 0;

    old_bank = bank_get_current();
    bank_switch(desc.bank);

    src = desc.ptr;
    len = desc.len;
    while (len != 0) {
        if (len > 128) chunk = 128;
        else chunk = (u8)len;
        __vram_write(vram_dst, src, chunk);
        vram_dst = (u16)(vram_dst + chunk);
        src = src + chunk;
        len = (u16)(len - chunk);
    }

    bank_switch(old_bank);
    return 1;
}

u8 asset_get_bank(u8 asset_id)
{
    AssetDesc desc;
    if (asset_get(asset_id, &desc) == 0) return 0;
    return desc.bank;
}

const u8* asset_get_ptr(u8 asset_id)
{
    AssetDesc desc;
    if (asset_get(asset_id, &desc) == 0) return 0;
    return desc.ptr;
}

u16 asset_get_len(u8 asset_id)
{
    AssetDesc desc;
    if (asset_get(asset_id, &desc) == 0) return 0;
    return desc.len;
}
