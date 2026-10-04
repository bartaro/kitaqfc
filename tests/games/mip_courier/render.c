#include "render.h"
#include "assets.h"
#include "hardware.h"
#include "physics_stage.h"

u8 render_font_tile(u8 character)
{
    if (character >= '0' && character <= '9') {
        return (u8)(TILE_0 + (u8)(character - '0'));
    }
    if (character >= 'A' && character <= 'Z') {
        return (u8)(TILE_A + (u8)(character - 'A'));
    }
    if (character == ':') return TILE_COLON;
    if (character == '-') return TILE_DASH;
    if (character == '/') return TILE_SLASH;
    if (character == '!') return TILE_EXCL;
    if (character == '.') return TILE_DOT;
    return TILE_SPACE;
}

void render_text_direct(u8 x, u8 y, const u8* text)
{
    u8 character;
    character = *text;
    while (character != 0) {
        hw_put_tile_direct(x, y, render_font_tile(character));
        x = (u8)(x + 1);
        text = text + 1;
        character = *text;
    }
}

void render_number_direct(u8 x, u8 y, u16 value, u8 digits)
{
    u8 index;
    u16 divisor;
    divisor = 1;
    index = 1;
    while (index < digits) {
        divisor = (u16)(divisor * 10);
        index = (u8)(index + 1);
    }
    index = 0;
    while (index < digits) {
        u8 digit;
        digit = (u8)(value / divisor);
        value = (u16)(value - (u16)(digit * divisor));
        hw_put_tile_direct((u8)(x + index), y, (u8)(TILE_0 + digit));
        if (divisor > 1) divisor = (u16)(divisor / 10);
        index = (u8)(index + 1);
    }
}

void render_number_queue(u8 x, u8 y, u16 value, u8 digits)
{
    u8 index;
    u16 divisor;
    divisor = 1;
    index = 1;
    while (index < digits) {
        divisor = (u16)(divisor * 10);
        index = (u8)(index + 1);
    }
    index = 0;
    while (index < digits) {
        u8 digit;
        digit = (u8)(value / divisor);
        value = (u16)(value - (u16)(digit * divisor));
        settile((u8)(x + index), y, (u8)(TILE_0 + digit));
        if (divisor > 1) divisor = (u16)(divisor / 10);
        index = (u8)(index + 1);
    }
}

void render_rect_direct(u8 x, u8 y, u8 width, u8 height, u8 tile)
{
    u8 row;
    u8 column;
    row = 0;
    while (row < height) {
        column = 0;
        while (column < width) {
            hw_put_tile_direct((u8)(x + column), (u8)(y + row), tile);
            column = (u8)(column + 1);
        }
        row = (u8)(row + 1);
    }
}

void render_player_metasprite(u8 x, u8 y, u8 base_tile, u8 attr)
{
    hw_oam_set(0, x, y, base_tile, attr);
    hw_oam_set(1, (u8)(x + 8), y, (u8)(base_tile + 1), attr);
    hw_oam_set(2, x, (u8)(y + 8), (u8)(base_tile + 2), attr);
    hw_oam_set(3, (u8)(x + 8), (u8)(y + 8), (u8)(base_tile + 3), attr);
}

void render_drone_metasprite(u8 first_id, u8 x, u8 y, u8 attr)
{
    hw_oam_set(first_id, x, y, DRONE_SPRITE, attr);
    hw_oam_set((u8)(first_id + 1), (u8)(x + 8), y, (u8)(DRONE_SPRITE + 1), attr);
    hw_oam_set((u8)(first_id + 2), x, (u8)(y + 8), (u8)(DRONE_SPRITE + 2), attr);
    hw_oam_set((u8)(first_id + 3), (u8)(x + 8), (u8)(y + 8), (u8)(DRONE_SPRITE + 3), attr);
}

void render_title_screen(void)
{
    u8 x;
    u8 y;
    hw_screen_off();
    hw_clear_updates();
    hw_clear_screen_direct(TILE_SPACE);
    hw_ppu_write(0x3F00, mip_palette, 32);

    x = 2;
    while (x < 30) {
        hw_put_tile_direct(x, 4, TILE_WALL_ALT);
        hw_put_tile_direct(x, 25, TILE_WALL_ALT);
        x = (u8)(x + 1);
    }
    y = 5;
    while (y < 25) {
        hw_put_tile_direct(2, y, TILE_WALL_ALT);
        hw_put_tile_direct(29, y, TILE_WALL_ALT);
        y = (u8)(y + 1);
    }
    render_text_direct(7, 6, mip_text_title);
    render_text_direct(8, 8, mip_text_subtitle);
    render_text_direct(6, 17, mip_text_controls1);
    render_text_direct(10, 19, mip_text_controls2);
    render_text_direct(7, 22, mip_text_start);
    hw_put_tile_direct(5, 12, TILE_CORE);
    hw_put_tile_direct(26, 12, TILE_CORE);
    hw_put_tile_direct(5, 23, TILE_SMALL_SPARK);
    hw_put_tile_direct(26, 23, TILE_SMALL_SPARK);
    hw_screen_on();
}

void render_hud_direct(void)
{
    u8 index;
    render_text_direct(1, 0, mip_text_hud_score);
    render_number_direct(7, 0, mip_game.score, 5);
    render_text_direct(14, 0, mip_text_hud_time);
    render_number_direct(19, 0, mip_game.time_seconds, 2);
    render_text_direct(23, 0, mip_text_hud_round);
    render_number_direct(24, 0, (u16)(mip_game.stage + 1), 1);
    index = 0;
    while (index < 3) {
        if (index < mip_game.health) hw_put_tile_direct((u8)(27 + index), 0, TILE_HEART);
        else hw_put_tile_direct((u8)(27 + index), 0, TILE_SPACE);
        index = (u8)(index + 1);
    }

    hw_put_tile_direct(1, 2, render_font_tile('B'));
    hw_put_tile_direct(2, 2, render_font_tile('O'));
    hw_put_tile_direct(3, 2, render_font_tile('O'));
    hw_put_tile_direct(4, 2, render_font_tile('S'));
    hw_put_tile_direct(5, 2, render_font_tile('T'));
    index = 0;
    while (index < 3) {
        if (index < mip_game.boosts) hw_put_tile_direct((u8)(7 + index), 2, TILE_BAR);
        else hw_put_tile_direct((u8)(7 + index), 2, TILE_SPACE);
        index = (u8)(index + 1);
    }
    hw_put_tile_direct(13, 2, render_font_tile('C'));
    hw_put_tile_direct(14, 2, render_font_tile('A'));
    hw_put_tile_direct(15, 2, render_font_tile('R'));
    hw_put_tile_direct(16, 2, render_font_tile('G'));
    hw_put_tile_direct(17, 2, render_font_tile('O'));
    render_number_direct(19, 2, mip_game.cargo_remaining, 1);
    hw_put_tile_direct(22, 2, render_font_tile('A'));
    hw_put_tile_direct(23, 2, render_font_tile('B'));
    hw_put_tile_direct(25, 2, TILE_DASH);
    hw_put_tile_direct(27, 2, render_font_tile('D'));
    hw_put_tile_direct(28, 2, render_font_tile('R'));
    hw_put_tile_direct(29, 2, render_font_tile('I'));
}

void render_game_screen(void)
{
    u8 x;
    u8 y;
    u8 index;
    hw_screen_off();
    hw_clear_updates();
    hw_clear_screen_direct(TILE_SPACE);
    hw_ppu_write(0x3F00, mip_palette, 32);

    y = 4;
    while (y <= 27) {
        x = 1;
        while (x <= 30) {
            hw_put_tile_direct(x, y, TILE_FLOOR);
            x = (u8)(x + 1);
        }
        y = (u8)(y + 1);
    }
    x = 1;
    while (x <= 30) {
        hw_put_tile_direct(x, 4, TILE_WALL);
        hw_put_tile_direct(x, 27, TILE_WALL);
        x = (u8)(x + 1);
    }
    y = 4;
    while (y <= 27) {
        hw_put_tile_direct(1, y, TILE_WALL);
        hw_put_tile_direct(30, y, TILE_WALL);
        y = (u8)(y + 1);
    }

    index = 0;
    while (index < WALL_COUNT) {
        u8 wall_base;
        wall_base = (u8)(index << 2);
        render_rect_direct(
            mip_wall_rects[(__safe_index u8)wall_base],
            mip_wall_rects[(__safe_index u8)(wall_base + 1)],
            mip_wall_rects[(__safe_index u8)(wall_base + 2)],
            mip_wall_rects[(__safe_index u8)(wall_base + 3)],
            (u8)(TILE_WALL_ALT)
        );
        index = (u8)(index + 1);
    }
    index = 0;
    while (index < BUMPER_COUNT) {
        u8 bumper_base;
        bumper_base = (u8)(index << 1);
        hw_put_tile_direct(
            mip_bumpers[(__safe_index u8)bumper_base],
            mip_bumpers[(__safe_index u8)(bumper_base + 1)],
            TILE_BUMPER
        );
        index = (u8)(index + 1);
    }
    index = 0;
    while (index < CARGO_COUNT) {
        u8 source;
        source = (u8)((mip_game.stage << 3) + index);
        hw_put_tile_direct(
            mip_cargo_x[(__safe_index u8)source],
            mip_cargo_y[(__safe_index u8)source],
            TILE_CARGO
        );
        index = (u8)(index + 1);
    }
    render_rect_direct(27, 24, 2, 2, TILE_GATE_CLOSED);
    render_hud_direct();
    hw_screen_on();
}

void render_result_screen(void)
{
    hw_screen_off();
    hw_clear_updates();
    hw_clear_screen_direct(TILE_SPACE);
    hw_ppu_write(0x3F00, mip_palette, 32);
    render_rect_direct(4, 7, 24, 14, TILE_FLOOR);
    render_rect_direct(4, 7, 24, 1, TILE_WALL_ALT);
    render_rect_direct(4, 20, 24, 1, TILE_WALL_ALT);
    if (mip_game.result_win != 0) {
        render_text_direct(5, 10, mip_text_clear);
        hw_put_tile_direct(15, 13, TILE_GATE_OPEN);
        hw_put_tile_direct(16, 13, TILE_GATE_OPEN);
    } else {
        render_text_direct(9, 10, mip_text_fail);
        hw_put_tile_direct(15, 13, TILE_CROSS);
        hw_put_tile_direct(16, 13, TILE_CROSS);
    }
    render_text_direct(9, 16, mip_text_result_score);
    render_number_direct(13, 18, mip_game.score, 5);
    render_text_direct(8, 23, mip_text_again);
    hw_screen_on();
}

void render_title_sprites(void)
{
    u8 base;
    if ((mip_game.frame & 16) == 0) base = PLAYER_SPRITE_IDLE;
    else base = PLAYER_SPRITE_THRUST;
    render_player_metasprite(120, 88, base, 0);
    hw_oam_set(4, 104, 101, SPARK_SPRITE, 2);
    hw_oam_set(5, 144, 101, SPARK_SPRITE, 2);
    hw_oam_hide_from(6);
}

void render_game_sprites(void)
{
    s16 player_x;
    s16 player_y;
    u8 base;
    u8 index;
    player_x = (s16)mip_game.player_x;
    player_y = (s16)mip_game.player_y;

    if (mip_game.invul_timer != 0 && (mip_game.frame & 4) != 0) {
        hw_oam_set(0, 0, 0xF0, 0, 0);
        hw_oam_set(1, 0, 0xF0, 0, 0);
        hw_oam_set(2, 0, 0xF0, 0, 0);
        hw_oam_set(3, 0, 0xF0, 0, 0);
    } else {
        if (mip_game.boost_timer != 0) base = PLAYER_SPRITE_BOOST;
        else if ((mip_game.frame & 8) != 0) base = PLAYER_SPRITE_THRUST;
        else base = PLAYER_SPRITE_IDLE;
        render_player_metasprite(
            (u8)(player_x - 8),
            (u8)(player_y - 9),
            base,
            0
        );
    }

    index = 0;
    while (index < DRONE_COUNT) {
        s16 drone_x;
        s16 drone_y;
        u8 attr;
        if (index == 0) {
            drone_x = (s16)mip_game.drone0_x;
            drone_y = (s16)mip_game.drone0_y;
        } else {
            drone_x = (s16)mip_game.drone1_x;
            drone_y = (s16)mip_game.drone1_y;
        }
        attr = 1;
        if (index == 0) {
            if (mip_game.drone0_stun != 0) attr = 2;
        } else {
            if (mip_game.drone1_stun != 0) attr = 2;
        }
        render_drone_metasprite(
            (u8)(4 + (index << 2)),
            (u8)(drone_x - 8),
            (u8)(drone_y - 9),
            attr
        );
        index = (u8)(index + 1);
    }

    if (mip_game.boost_timer != 0) {
        hw_oam_set(12, (u8)(player_x - 4), (u8)(player_y + 9), SPARK_SPRITE, 2);
        hw_oam_set(13, (u8)(player_x - 12), (u8)(player_y + 6), HIT_SPRITE, 2);
        hw_oam_hide_from(14);
    } else {
        hw_oam_hide_from(12);
    }
}

void render_queue_score(void)
{
    render_number_queue(7, 0, mip_game.score, 5);
}

void render_queue_time(void)
{
    render_number_queue(19, 0, mip_game.time_seconds, 2);
}

void render_queue_health(void)
{
    u8 index;
    index = 0;
    while (index < 3) {
        if (index < mip_game.health) settile((u8)(27 + index), 0, TILE_HEART);
        else settile((u8)(27 + index), 0, TILE_SPACE);
        index = (u8)(index + 1);
    }
}

void render_queue_boosts(void)
{
    u8 index;
    index = 0;
    while (index < 3) {
        if (index < mip_game.boosts) settile((u8)(7 + index), 2, TILE_BAR);
        else settile((u8)(7 + index), 2, TILE_SPACE);
        index = (u8)(index + 1);
    }
}

void render_queue_stage(void)
{
    settile(24, 0, (u8)(TILE_0 + mip_game.stage + 1));
}

void render_queue_collected_cargo(void)
{
    u8 source;
    source =
        (u8)((mip_game.stage << 3) + mip_physics_collision_index);
    settile(
        mip_cargo_x[(__safe_index u8)source],
        mip_cargo_y[(__safe_index u8)source],
        TILE_FLOOR
    );
    settile(19, 2, (u8)(TILE_0 + mip_game.cargo_remaining));
}

void render_queue_gate(u8 open)
{
    u8 tile;
    if (open != 0) tile = TILE_GATE_OPEN;
    else tile = TILE_GATE_CLOSED;
    settile(27, 24, tile);
    settile(28, 24, tile);
    settile(27, 25, tile);
    settile(28, 25, tile);
}

void render_queue_next_stage(void)
{
    u8 index;
    u8 source;
    render_queue_gate(0);
    index = 0;
    while (index < CARGO_COUNT) {
        source = (u8)((mip_game.stage << 3) + index);
        settile(
            mip_cargo_x[(__safe_index u8)source],
            mip_cargo_y[(__safe_index u8)source],
            TILE_CARGO
        );
        index = (u8)(index + 1);
    }
    settile(19, 2, (u8)(TILE_0 + mip_game.cargo_remaining));
    render_queue_stage();
    render_queue_time();
    render_queue_boosts();
}
