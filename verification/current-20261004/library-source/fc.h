#ifndef FC_H
#define FC_H

// Convenience header collecting FC interfaces. Inclusion provides declarations
// and aliases, not every implementation; link only the source modules required
// by the selected runtime, avoiding alternative helpers with conflicting symbols.
#include "core.h"
#include "intrinsics.h"
#include "system.h"
#include "input.h"
#include "runtime.h"
#include "ppu.h"
#include "ppu_direct.h"
#include "vram_queue.h"
#include "vram.h"
#include "oam.h"
#include "sprite.h"
#include "pad.h"
#include "input_repeat.h"
#include "palette.h"
#include "scroll.h"
#include "tilemap.h"
#include "nametable_asset.h"
#include "attribute.h"
#include "scene.h"
#include "metasprite.h"
#include "actor.h"
#include "entity.h"
#include "collision.h"
#include "fixed.h"
#include "physics2d.h"
#include "physics3d.h"
#include "bank.h"
#include "asset.h"
#include "debug.h"
#include "chain.h"
#include "audio.h"
#include "audio_vblank.h"
#include "fds_sound.h"
#include "vrc6_sound.h"
#include "vrc7_sound.h"
#include "mapper.h"
// This final include adds function-like macros, including zero-argument nes_oam_dma.
// Use selective headers when calling the page-argument runtime API with the same name.
#include "nes_game.h"

#endif
