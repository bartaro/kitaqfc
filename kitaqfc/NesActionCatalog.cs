using System;
using System.Collections.Generic;

// NES gameplay-operation catalog used by KITAQFC diagnostics, codegen comments,
// and KUROSAKI hand-off metadata.
//
// Intent: expose "what the game is doing" instead of only exposing 6502 bytes.
// This is the main hook for KITAQFC's NES-specific DSL direction.
static class NesActionCatalog
{
    public enum TimingClass
    {
        Any,
        NmiOrRenderingOff,
        NmiOnly,
        Irq,
        MapperSpecific,
        FdsOnly,
    }

    public sealed class ActionInfo
    {
        public string Name;
        public string Category;
        public string Operation;
        public string Description;
        public TimingClass Timing;
        public bool DirectPpuAccess;
        public bool QueuePpuAccess;
        public bool UsesOamShadow;
        public bool PerformsOamDma;
        public bool RequiresFds;
        public string MapperRequirement;
        public int OamIndexArgument = -1;
        public int OamSpriteCountArgument = -1;
        public string KurosakiKind;
    }

    static readonly Dictionary<string, ActionInfo> _map = Build();

    public static bool TryGet(string name, out ActionInfo info)
    {
        if (string.IsNullOrEmpty(name)) { info = null; return false; }
        return _map.TryGetValue(name, out info);
    }

    public static IEnumerable<ActionInfo> All => _map.Values;

    static Dictionary<string, ActionInfo> Build()
    {
        var m = new Dictionary<string, ActionInfo>(StringComparer.Ordinal);

        void Add(string name, string category, string op, string desc, TimingClass timing = TimingClass.Any,
            bool directPpu = false, bool queuePpu = false, bool oamShadow = false, bool oamDma = false,
            bool fds = false, string mapper = "", int oamIndexArg = -1, int oamSpriteCountArg = -1,
            string kurosakiKind = "")
        {
            m[name] = new ActionInfo
            {
                Name = name,
                Category = category,
                Operation = op,
                Description = desc,
                Timing = timing,
                DirectPpuAccess = directPpu,
                QueuePpuAccess = queuePpu,
                UsesOamShadow = oamShadow,
                PerformsOamDma = oamDma,
                RequiresFds = fds,
                MapperRequirement = mapper ?? "",
                OamIndexArgument = oamIndexArg,
                OamSpriteCountArgument = oamSpriteCountArg,
                KurosakiKind = string.IsNullOrWhiteSpace(kurosakiKind) ? category : kurosakiKind,
            };
        }

        // PPU direct writes. These should normally occur inside __nes_nmi(), vblank-only code,
        // or with rendering disabled.
        Add("__ppu_on", "ppu", "ppu_on", "Enable PPU rendering using the runtime PPUCTRL/PPUMASK mirrors.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "ppu_control");
        Add("__ppu_off", "ppu", "ppu_off", "Disable PPU rendering.", TimingClass.Any, directPpu: true, kurosakiKind: "ppu_control");
        Add("__ppu_mask_set", "ppu", "ppu_mask_set", "Set PPUMASK and its runtime shadow.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "ppu_control");
        Add("__ppu_ctrl_set", "ppu", "ppu_ctrl_set", "Set PPUCTRL and its runtime shadow.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "ppu_control");
        Add("__ppu_addr", "ppu", "ppu_addr", "Write PPUADDR.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "ppu_register");
        Add("__ppu_data", "ppu", "ppu_data", "Write PPUDATA.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "ppu_register");
        Add("__vram_write", "vram", "vram_write", "Direct VRAM copy through $2006/$2007.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "vram_direct");
        Add("__vram_fill", "vram", "vram_fill", "Direct VRAM fill through $2006/$2007.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "vram_direct");
        Add("__nametable_put", "nametable", "nametable_put", "Direct nametable tile write.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "nametable_direct");
        Add("__nametable_put_nt", "nametable", "nametable_put_nt", "Direct nametable tile write with nametable id.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "nametable_direct");
        Add("__nametable_rect", "nametable", "nametable_rect", "Direct rectangular nametable fill.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "nametable_direct");
        Add("__nametable_rect_nt", "nametable", "nametable_rect_nt", "Direct rectangular nametable fill with nametable id.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "nametable_direct");
        Add("__attr_set", "attribute", "attr_set", "Direct attribute table write.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "attribute_direct");
        Add("__attr_set_nt", "attribute", "attr_set_nt", "Direct attribute table write with nametable id.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "attribute_direct");
        Add("__palette_bg_load", "palette", "palette_bg_load", "Direct background palette load.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "palette_direct");
        Add("__palette_sp_load", "palette", "palette_sp_load", "Direct sprite palette load.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "palette_direct");

        // NMI queue. Queueing is safe from normal code; executing the queue is NMI/vblank work.
        Add("__vramq_clear", "vram_queue", "vramq_clear", "Clear the runtime VRAM update queue.", queuePpu: true, kurosakiKind: "vram_queue");
        Add("__vramq_put", "vram_queue", "vramq_put", "Queue a single-byte VRAM write for the default NMI handler.", queuePpu: true, kurosakiKind: "vram_queue");
        Add("__vramq_copy", "vram_queue", "vramq_copy", "Queue a VRAM copy for the default NMI handler.", queuePpu: true, kurosakiKind: "vram_queue");
        Add("__vramq_fill", "vram_queue", "vramq_fill", "Queue a VRAM fill for the default NMI handler.", queuePpu: true, kurosakiKind: "vram_queue");
        Add("__vramq_commit", "vram_queue", "vramq_commit", "Mark the queued VRAM commands ready for the default NMI handler.", queuePpu: true, kurosakiKind: "vram_queue");
        Add("__vramq_exec", "vram_queue", "vramq_exec", "Execute queued VRAM commands now.", TimingClass.NmiOrRenderingOff, directPpu: true, queuePpu: true, kurosakiKind: "vram_queue_exec");

        // OAM and metasprites.
        Add("__oam_clear", "oam", "oam_clear", "Clear the OAM shadow buffer.", oamShadow: true, kurosakiKind: "oam_shadow");
        Add("__oam_dma", "oam", "oam_dma", "Perform OAM DMA from the default shadow page.", TimingClass.NmiOnly, oamDma: true, kurosakiKind: "oam_dma");
        Add("__oam_dma_page", "oam", "oam_dma_page", "Perform OAM DMA from a selected page.", TimingClass.NmiOnly, oamDma: true, kurosakiKind: "oam_dma");
        Add("__sprite_set", "oam", "sprite_set", "Write one sprite entry in the OAM shadow buffer.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "oam_shadow");
        Add("__sprite_move", "oam", "sprite_move", "Move one sprite entry in the OAM shadow buffer.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "oam_shadow");
        Add("__sprite_tile", "oam", "sprite_tile", "Change one sprite tile in the OAM shadow buffer.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "oam_shadow");
        Add("__sprite_attr", "oam", "sprite_attr", "Change one sprite attribute byte in the OAM shadow buffer.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "oam_shadow");
        Add("__sprite_hide", "oam", "sprite_hide", "Hide one sprite entry in the OAM shadow buffer.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "oam_shadow");
        Add("__metasprite_draw", "metasprite", "metasprite_draw", "Draw a metasprite byte stream into the OAM shadow buffer and return next OAM index.", oamShadow: true, oamIndexArg: 0, kurosakiKind: "metasprite");

        // Mapper/IRQ/split screen actions.
        Add("__mapper_irq_set", "mapper", "mapper_irq_set", "Set mapper scanline IRQ latch/reload value.", TimingClass.MapperSpecific, mapper: "mmc3", kurosakiKind: "mapper_irq");
        Add("__irq_scanline_set", "mapper", "mapper_irq_set", "Set MMC3 scanline IRQ latch/reload value.", TimingClass.MapperSpecific, mapper: "mmc3", kurosakiKind: "mapper_irq");
        Add("__mapper_irq_enable", "mapper", "mapper_irq_enable", "Enable mapper scanline IRQ.", TimingClass.MapperSpecific, mapper: "mmc3", kurosakiKind: "mapper_irq");
        Add("__mapper_irq_disable", "mapper", "mapper_irq_disable", "Disable mapper scanline IRQ.", TimingClass.MapperSpecific, mapper: "mmc3", kurosakiKind: "mapper_irq");
        Add("__mapper_irq_ack", "mapper", "mapper_irq_ack", "Acknowledge mapper scanline IRQ.", TimingClass.MapperSpecific, mapper: "mmc3", kurosakiKind: "mapper_irq");
        Add("__split_scroll_sprite0", "split_scroll", "split_scroll_sprite0", "Wait for sprite-0 hit and change scroll mid-frame.", TimingClass.NmiOrRenderingOff, directPpu: true, kurosakiKind: "split_scroll");

        // FDS.
        Add("__fds_wave_load", "fds_sound", "fds_wave_load", "Load 64-byte FDS wavetable into $4040-$407F.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_sound");
        Add("__fds_mod_load", "fds_sound", "fds_mod_load", "Load FDS modulation table data.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_sound");
        Add("__fds_freq_set", "fds_sound", "fds_freq_set", "Set FDS channel frequency.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_sound");
        Add("__fds_volume_set", "fds_sound", "fds_volume_set", "Set FDS channel volume envelope register.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_sound");
        Add("__fds_env_set", "fds_sound", "fds_env_set", "Set FDS volume/modulation envelope parameters.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_sound");
        Add("__fds_load_file", "fds_file", "fds_load_file", "Load an FDS file through the BIOS.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_file");
        Add("__fds_save_file", "fds_file", "fds_save_file", "Save an FDS file through the BIOS.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_file");
        Add("__fds_load_overlay", "fds_overlay", "fds_load_overlay", "Load a non-boot overlay file by FDS file id.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_overlay");
        Add("__fds_load_bank", "fds_overlay", "fds_load_bank", "Load a KITAQFC generated FDS overlay bank.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_overlay");
        Add("__fds_overlay_farcall", "fds_overlay", "fds_overlay_farcall", "Load an overlay bank as needed and call a target function.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_overlay_call");
        Add("__fds_farcall", "fds_overlay", "fds_farcall", "Alias for FDS overlay farcall.", TimingClass.FdsOnly, fds: true, kurosakiKind: "fds_overlay_call");

        return m;
    }
}
