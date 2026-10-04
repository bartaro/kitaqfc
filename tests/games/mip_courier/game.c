#include "game.h"
#include "audio.h"
#include "hardware.h"
#include "physics_stage.h"
#include "render.h"

MipGameState mip_game;

void game_reset_cargo(void)
{
    mip_game.cargo_active_mask = 0xFF;
    mip_game.cargo_remaining = CARGO_COUNT;
}

void game_start_title(void)
{
    mip_game.mode = GAME_TITLE;
    mip_game.frame = 0;
    mip_game.result_delay = 0;
    render_title_screen();
    audio_start_song(SONG_TITLE);
}

void game_start_play(void)
{
    u8 index;
    mip_game.mode = GAME_PLAY;
    mip_game.frame = 0;
    mip_game.stage = 0;
    mip_game.score = 0;
    mip_game.health = 3;
    mip_game.boosts = 3;
    mip_game.time_seconds = 90;
    mip_game.second_frames = 0;
    mip_game.combo = 0;
    mip_game.combo_timer = 0;
    mip_game.boost_timer = 0;
    mip_game.invul_timer = 0;
    mip_game.bumper_cooldown = 0;
    mip_game.gate_open = 0;
    mip_game.last_dir = PAD_RIGHT;
    mip_game.result_win = 0;
    mip_game.result_delay = 0;
    index = 0;
    while (index < DRONE_COUNT) {
        if (index == 0) mip_game.drone0_stun = 0;
        else mip_game.drone1_stun = 0;
        index = (u8)(index + 1);
    }
    game_reset_cargo();
    physics_stage_init();
    render_game_screen();
    audio_start_song(SONG_GAME);
}

void game_start_result(u8 win)
{
    mip_game.mode = GAME_RESULT;
    mip_game.result_win = win;
    mip_game.result_delay = 45;
    render_result_screen();
    if (win != 0) {
        audio_start_song(SONG_CLEAR);
        audio_play_sfx(SFX_CLEAR);
    } else {
        audio_start_song(SONG_TITLE);
        audio_play_sfx(SFX_DAMAGE);
    }
}

void game_add_score(u16 value)
{
    if (mip_game.score > (u16)(65535 - value)) mip_game.score = 65535;
    else mip_game.score = (u16)(mip_game.score + value);
    render_queue_score();
}

void game_collect_cargo(void)
{
    u16 value;
    u8 mask;
    mask = (u8)(1 << mip_physics_collision_index);
    mip_game.cargo_active_mask =
        (u8)(mip_game.cargo_active_mask & (u8)(mask ^ 0xFF));
    mip_game.cargo_remaining = (u8)(mip_game.cargo_remaining - 1);
    if (mip_game.combo_timer != 0) {
        if (mip_game.combo < 5) mip_game.combo = (u8)(mip_game.combo + 1);
    } else {
        mip_game.combo = 1;
    }
    mip_game.combo_timer = 120;
    value = (u16)(100 + (u16)(mip_game.combo * 50));
    game_add_score(value);
    if (mip_game.boosts < 3) {
        mip_game.boosts = (u8)(mip_game.boosts + 1);
        render_queue_boosts();
    }
    render_queue_collected_cargo();
    audio_play_sfx(SFX_CARGO);

    if (mip_game.cargo_remaining == 0) {
        mip_game.gate_open = 1;
        render_queue_gate(1);
        game_add_score(500);
        audio_play_sfx(SFX_GATE);
    }
}

void game_advance_stage(void)
{
    if (mip_game.stage >= 2) {
        game_add_score((u16)(mip_game.time_seconds * 25));
        game_start_result(1);
        return;
    }
    mip_game.stage = (u8)(mip_game.stage + 1);
    mip_game.gate_open = 0;
    mip_game.combo = 0;
    mip_game.combo_timer = 0;
    mip_game.boosts = 3;
    if (mip_game.time_seconds < 80) mip_game.time_seconds = (u8)(mip_game.time_seconds + 20);
    else mip_game.time_seconds = 99;
    mip_game.drone0_stun = 0;
    mip_game.drone1_stun = 0;
    game_reset_cargo();
    physics_stage_reset_positions();
    render_queue_next_stage();
    audio_play_sfx(SFX_GATE);
}

void game_damage_player(void)
{
    if (mip_game.invul_timer != 0) return;
    mip_game.invul_timer = 90;
    if (mip_game.health != 0) mip_game.health = (u8)(mip_game.health - 1);
    if (mip_game.time_seconds > 5) mip_game.time_seconds = (u8)(mip_game.time_seconds - 5);
    else mip_game.time_seconds = 0;
    if (mip_game.player_dir_x == KQ2D_DIR_NEGATIVE) {
        mip_game.player_dir_x = KQ2D_DIR_POSITIVE;
    } else if (mip_game.player_dir_x == KQ2D_DIR_POSITIVE) {
        mip_game.player_dir_x = KQ2D_DIR_NEGATIVE;
    }
    if (mip_game.player_dir_y == KQ2D_DIR_NEGATIVE) {
        mip_game.player_dir_y = KQ2D_DIR_POSITIVE;
    } else if (mip_game.player_dir_y == KQ2D_DIR_POSITIVE) {
        mip_game.player_dir_y = KQ2D_DIR_NEGATIVE;
    }
    render_queue_health();
    render_queue_time();
    audio_play_sfx(SFX_DAMAGE);
    if (mip_game.health == 0) {
        game_start_result(0);
    }
}

void game_update_timers(void)
{
    u8 index;
    if (mip_game.combo_timer != 0) mip_game.combo_timer = (u8)(mip_game.combo_timer - 1);
    if (mip_game.boost_timer != 0) mip_game.boost_timer = (u8)(mip_game.boost_timer - 1);
    if (mip_game.invul_timer != 0) mip_game.invul_timer = (u8)(mip_game.invul_timer - 1);
    if (mip_game.bumper_cooldown != 0) {
        mip_game.bumper_cooldown = (u8)(mip_game.bumper_cooldown - 1);
    }
    index = 0;
    while (index < DRONE_COUNT) {
        if (index == 0) {
            if (mip_game.drone0_stun != 0) {
                mip_game.drone0_stun = (u8)(mip_game.drone0_stun - 1);
            }
        } else {
            if (mip_game.drone1_stun != 0) {
                mip_game.drone1_stun = (u8)(mip_game.drone1_stun - 1);
            }
        }
        index = (u8)(index + 1);
    }

    mip_game.second_frames = (u8)(mip_game.second_frames + 1);
    if (mip_game.second_frames >= 60) {
        mip_game.second_frames = 0;
        if (mip_game.time_seconds != 0) {
            mip_game.time_seconds = (u8)(mip_game.time_seconds - 1);
            render_queue_time();
        }
        if (mip_game.time_seconds == 0) {
            game_start_result(0);
        }
    }
}

void game_update_play(void)
{
    u8 event;
    u8 index;
    u8 mask;
    game_update_timers();
    if (mip_game.mode != GAME_PLAY) return;

    event = physics_stage_update_player();
    if ((event & PHYS_EVENT_BOOST) != 0) {
        render_queue_boosts();
        audio_play_sfx(SFX_BOOST);
    } else if ((event & PHYS_EVENT_BOUNCE) != 0) {
        audio_play_sfx(SFX_BOUNCE);
    }
    physics_stage_update_drones();

    /*
     * Collision work is phase-sliced: player motion/OAM still run every frame,
     * while two cargo and one drone are tested per frame. The maximum pickup
     * latency is four logic frames, but controller motion remains smooth.
     */
    index = (u8)(mip_game.frame & 3);
    mask = (u8)(1 << index);
    if ((mip_game.cargo_active_mask & mask) != 0) {
        mip_physics_collision_index = index;
        physics_stage_check_cargo();
        if (mip_physics_collision_result != 0) {
            game_collect_cargo();
        }
    }
    index = (u8)(index + 4);
    mask = (u8)(1 << index);
    if ((mip_game.cargo_active_mask & mask) != 0) {
        mip_physics_collision_index = index;
        physics_stage_check_cargo();
        if (mip_physics_collision_result != 0) {
            game_collect_cargo();
        }
    }

    index = (u8)(mip_game.frame & 1);
    mip_physics_collision_index = index;
    physics_stage_check_drone();
    if (mip_physics_collision_result != 0) {
        if (mip_game.boost_timer != 0) {
            physics_stage_knock_selected_drone();
            game_add_score(250);
            audio_play_sfx(SFX_BOUNCE);
        } else {
            game_damage_player();
            if (mip_game.mode != GAME_PLAY) return;
        }
    }

    if (mip_game.gate_open != 0) {
        physics_stage_check_gate();
        if (mip_physics_collision_result != 0) {
            game_advance_stage();
            if (mip_game.mode != GAME_PLAY) return;
        }
    }
    render_game_sprites();
}

void game_update_title(void)
{
    render_title_sprites();
    if ((mip_pad_pressed & PAD_START) != 0) {
        audio_play_sfx(SFX_GATE);
        game_start_play();
    }
}

void game_update_result(void)
{
    hw_oam_hide_all();
    if (mip_game.result_delay != 0) {
        mip_game.result_delay = (u8)(mip_game.result_delay - 1);
        return;
    }
    if ((mip_pad_pressed & PAD_START) != 0) {
        game_start_play();
    }
}

void game_run(void)
{
    hw_init();
    audio_init();
    game_start_title();
    while (1) {
        hw_wait_frame();
        hw_begin_frame_updates();
        hw_poll_pad();
        mip_game.frame = (u16)(mip_game.frame + 1);
        audio_tick();
        if (mip_game.mode == GAME_TITLE) {
            game_update_title();
        } else if (mip_game.mode == GAME_PLAY) {
            game_update_play();
        } else {
            game_update_result();
        }
        hw_finish_frame_updates();
    }
}
