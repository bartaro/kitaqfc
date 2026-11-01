#ifndef NES_GAME_H
#define NES_GAME_H

#include "core.h"
#include "intrinsics.h"

/*
 * AI-friendly NES operation aliases.
 *
 * These names intentionally describe gameplay-side operations.  They are thin
 * aliases over KITAQFC intrinsics so the compiler can still record the exact
 * NES action for diagnostics and KUROSAKI metadata.
 */
#define nes_vram_put(addr, value)          __vramq_put((addr), (value))
#define nes_vram_copy(addr, src, len)      __vramq_copy((addr), (src), (len))
#define nes_vram_fill(addr, value, len)    __vramq_fill((addr), (value), (len))
#define nes_vram_commit()                  __vramq_commit()
#define nes_vram_clear_queue()             __vramq_clear()

#define nes_oam_clear()                    __oam_clear()
#define nes_oam_dma()                      __oam_dma()
#define nes_sprite_set(i,x,y,t,a)          __sprite_set((i),(x),(y),(t),(a))
#define nes_sprite_move(i,x,y)             __sprite_move((i),(x),(y))
#define nes_sprite_hide(i)                 __sprite_hide((i))
#define nes_metasprite_draw(i,x,y,data)    __metasprite_draw((i),(x),(y),(data))

#define nes_pad1()                         __pad_read1_safe()
#define nes_pad2()                         __pad_read2_safe()

#define nes_mapper_irq_set(scanline)       __mapper_irq_set((scanline))
#define nes_mapper_irq_enable()            __mapper_irq_enable()
#define nes_mapper_irq_disable()           __mapper_irq_disable()
#define nes_mapper_irq_ack()               __mapper_irq_ack()
#define nes_split_scroll_sprite0(x,y)      __split_scroll_sprite0((x),(y))

#define nes_fds_wave_load(wave64)          __fds_wave_load((wave64))
#define nes_fds_load_bank(bank)            __fds_load_bank((bank))
#define nes_fds_farcall(bank, func)        __fds_overlay_farcall((bank), (func))

#endif
