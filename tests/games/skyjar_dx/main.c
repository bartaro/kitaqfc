/*
 * SKY JAR DX - Risk/Reward Starcatcher for KITAQFC / NES
 * Revised one-screen action game with combo, comet, focus and graze rewards.
 * Graphics, BGM, and SFX are newly created for this sample.
 * Target: NROM + CHR-ROM
 */

#define PAD_A      0x01
#define PAD_B      0x02
#define PAD_SELECT 0x04
#define PAD_START  0x08
#define PAD_UP     0x10
#define PAD_DOWN   0x20
#define PAD_LEFT   0x40
#define PAD_RIGHT  0x80

#define T_SPACE 0x00
#define T_STARBG 0x01
#define T_FLOOR 0x02
#define T_WINDOW 0x03
#define T_BLOCK 0x04

#define T_0 0x10
#define T_1 0x11
#define T_2 0x12
#define T_3 0x13
#define T_4 0x14
#define T_5 0x15
#define T_6 0x16
#define T_7 0x17
#define T_8 0x18
#define T_9 0x19

#define T_A 0x1A
#define T_B 0x1B
#define T_C 0x1C
#define T_D 0x1D
#define T_E 0x1E
#define T_F 0x1F
#define T_G 0x20
#define T_H 0x21
#define T_I 0x22
#define T_J 0x23
#define T_K 0x24
#define T_L 0x25
#define T_M 0x26
#define T_N 0x27
#define T_O 0x28
#define T_P 0x29
#define T_Q 0x2A
#define T_R 0x2B
#define T_S 0x2C
#define T_T 0x2D
#define T_U 0x2E
#define T_V 0x2F
#define T_W 0x30
#define T_X 0x31
#define T_Y 0x32
#define T_Z 0x33
#define T_HEART 0x34
#define T_SHIELD 0x35
#define T_DASH 0x36

#define TILE_PLAYER 0x40
#define TILE_STAR   0x44
#define TILE_ROCK   0x48
#define TILE_SHIELD 0x4C
#define TILE_DASHER 0x50

__location(0x2000) unsigned char PPUCTRL;
__location(0x2001) unsigned char PPUMASK;
__location(0x2002) unsigned char PPUSTATUS;
__location(0x2003) unsigned char OAMADDR;
__location(0x2005) unsigned char PPUSCROLL;
__location(0x2006) unsigned char PPUADDR;
__location(0x2007) unsigned char PPUDATA;
__location(0x4000) unsigned char SQ1_VOL;
__location(0x4001) unsigned char SQ1_SWEEP;
__location(0x4002) unsigned char SQ1_LO;
__location(0x4003) unsigned char SQ1_HI;
__location(0x4004) unsigned char SQ2_VOL;
__location(0x4005) unsigned char SQ2_SWEEP;
__location(0x4006) unsigned char SQ2_LO;
__location(0x4007) unsigned char SQ2_HI;
__location(0x4008) unsigned char TRI_LINEAR;
__location(0x400A) unsigned char TRI_LO;
__location(0x400B) unsigned char TRI_HI;
__location(0x400C) unsigned char NOISE_VOL;
__location(0x400E) unsigned char NOISE_LO;
__location(0x400F) unsigned char NOISE_HI;
__location(0x4014) unsigned char OAMDMA;
__location(0x4015) unsigned char APU_STATUS;
__location(0x4016) unsigned char JOY1;
__location(0x4017) unsigned char JOY2_APU_FRAME;

__location(0x0200) unsigned char oam_shadow[256];
__location(0x0300) unsigned char vram_queue[192];

unsigned char nmi_counter;
unsigned char vram_used;
unsigned char vram_overflow;
unsigned char frame_counter;

unsigned char pad_cur;
unsigned char pad_prev;
unsigned char pad_pressed;

unsigned char game_state;
unsigned char player_x;
unsigned char player_y;
unsigned char score;
unsigned char lives;
unsigned char shield_on;
unsigned char combo;
unsigned char focus_meter;
unsigned char focus_on;
unsigned char last_gain;
unsigned char rng_state;

unsigned char obj_x[6];
unsigned char obj_y[6];
unsigned char obj_type[6];
unsigned char obj_speed[6];

unsigned char hud_digits[2];
unsigned char hud_life[1];
unsigned char hud_shield[1];
unsigned char hud_combo[1];
unsigned char hud_focus[1];

unsigned char music_tick_count;
unsigned char music_step;
unsigned char bass_step;
unsigned char sfx_timer;
unsigned char sfx_kind;

__prg_rom const unsigned char palette_data[32] = {
    0x0F, 0x01, 0x11, 0x30,
    0x0F, 0x05, 0x15, 0x25,
    0x0F, 0x02, 0x12, 0x22,
    0x0F, 0x08, 0x18, 0x28,
    0x0F, 0x17, 0x27, 0x30,
    0x0F, 0x18, 0x28, 0x38,
    0x0F, 0x06, 0x16, 0x26,
    0x0F, 0x0A, 0x1A, 0x2A
};

__prg_rom const unsigned char txt_title1[10] = { T_S, T_K, T_Y, T_SPACE, T_J, T_A, T_R, T_SPACE, T_D, T_X };
__prg_rom const unsigned char txt_title2[11] = { T_C, T_A, T_T, T_C, T_H, T_SPACE, T_S, T_T, T_A, T_R, T_S };
__prg_rom const unsigned char txt_press[11] = { T_P, T_R, T_E, T_S, T_S, T_SPACE, T_S, T_T, T_A, T_R, T_T };
__prg_rom const unsigned char txt_rule1[16] = { T_A, T_SPACE, T_D, T_A, T_S, T_H, T_SPACE, T_SPACE, T_B, T_SPACE, T_F, T_O, T_C, T_U, T_S, T_SPACE };
__prg_rom const unsigned char txt_rule2[18] = { T_G, T_R, T_A, T_Z, T_E, T_SPACE, T_R, T_O, T_C, T_K, T_SPACE, T_F, T_O, T_R, T_SPACE, T_B, T_O, T_N };
__prg_rom const unsigned char txt_score[4] = { T_S, T_T, T_A, T_R };
__prg_rom const unsigned char txt_chain[3] = { T_C, T_H, T_N };
__prg_rom const unsigned char txt_focus[3] = { T_F, T_O, T_C };
__prg_rom const unsigned char txt_life[4] = { T_L, T_I, T_F, T_E };
__prg_rom const unsigned char txt_goal[7] = { T_G, T_O, T_A, T_L, T_SPACE, T_5, T_0 };
__prg_rom const unsigned char txt_win1[8] = { T_J, T_A, T_R, T_SPACE, T_F, T_U, T_L, T_L };
__prg_rom const unsigned char txt_win2[11] = { T_S, T_T, T_A, T_R, T_S, T_SPACE, T_S, T_A, T_F, T_E, T_SPACE };
__prg_rom const unsigned char txt_over1[9] = { T_J, T_A, T_R, T_SPACE, T_B, T_R, T_O, T_K, T_E };
__prg_rom const unsigned char txt_over2[11] = { T_T, T_R, T_Y, T_SPACE, T_A, T_G, T_A, T_I, T_N, T_SPACE, T_SPACE };

__prg_rom const unsigned char note_lo[16] = {
    0xAC, 0x7D, 0x54, 0x1D,
    0xFE, 0xD5, 0xBE, 0xA9,
    0x8E, 0x7E, 0x71, 0x64,
    0x59, 0x50, 0x47, 0x3F
};

__prg_rom const unsigned char note_hi[16] = {
    0x01, 0x01, 0x01, 0x01,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00
};

__prg_rom const unsigned char melody_seq[32] = {
    0, 2, 4, 7, 5, 4, 2, 4,
    3, 5, 7, 10, 8, 7, 5, 7,
    2, 4, 7, 11, 9, 7, 4, 7,
    0, 3, 5, 8, 7, 5, 3, 2
};

__prg_rom const unsigned char bass_seq[8] = { 0, 0, 3, 3, 2, 2, 5, 3 };

unsigned short nt_addr(unsigned char row, unsigned char col)
{
    unsigned short a;
    a = 0x2000;
    a = a + ((unsigned short)row << 5);
    a = a + col;
    return a;
}

void ppu_seek(unsigned short addr)
{
    unsigned char latch;
    latch = PPUSTATUS;
    PPUADDR = (unsigned char)(addr >> 8);
    PPUADDR = (unsigned char)addr;
    latch = latch;
}

void ppu_write(unsigned short addr, unsigned char* src, unsigned char len)
{
    ppu_seek(addr);
    while (len != 0)
    {
        PPUDATA = *src;
        src = src + 1;
        len = len - 1;
    }
}

void ppu_fill(unsigned short addr, unsigned char value, unsigned short len)
{
    ppu_seek(addr);
    while (len != 0)
    {
        PPUDATA = value;
        len = len - 1;
    }
}

void ppu_put(unsigned short addr, unsigned char value)
{
    ppu_seek(addr);
    PPUDATA = value;
}

void vramq_clear(void)
{
    vram_used = 0;
    vram_overflow = 0;
}

unsigned char vramq_write(unsigned short addr, unsigned char* src, unsigned char len)
{
    unsigned char used;
    unsigned char need;
    unsigned char i;
    unsigned char dst;

    used = vram_used;
    need = (unsigned char)(len + 3);
    if ((unsigned char)(used + need) < used)
    {
        vram_overflow = 1;
        return 0;
    }
    if ((unsigned char)(used + need) > 192)
    {
        vram_overflow = 1;
        return 0;
    }

    vram_queue[used] = (unsigned char)(addr >> 8);
    vram_queue[(unsigned char)(used + 1)] = (unsigned char)addr;
    vram_queue[(unsigned char)(used + 2)] = len;
    i = 0;
    dst = (unsigned char)(used + 3);
    while (i != len)
    {
        vram_queue[dst] = src[i];
        dst = dst + 1;
        i = i + 1;
    }
    vram_used = (unsigned char)(used + need);
    return 1;
}

void vramq_flush(void)
{
    unsigned char i;
    unsigned char len;
    unsigned char data_index;
    unsigned char latch;

    i = 0;
    while (i != vram_used)
    {
        latch = PPUSTATUS;
        PPUADDR = vram_queue[i];
        PPUADDR = vram_queue[(unsigned char)(i + 1)];
        len = vram_queue[(unsigned char)(i + 2)];
        data_index = (unsigned char)(i + 3);
        while (len != 0)
        {
            PPUDATA = vram_queue[data_index];
            data_index = data_index + 1;
            len = len - 1;
        }
        i = data_index;
        latch = latch;
    }
    vram_used = 0;
}

void oam_hide_all(void)
{
    unsigned char i;
    unsigned char dst;
    i = 0;
    dst = 0;
    while (i != 64)
    {
        oam_shadow[dst] = 0xF8;
        dst = dst + 4;
        i = i + 1;
    }
}

void oam_hide_from(unsigned char index)
{
    unsigned char dst;
    dst = index + index;
    dst = dst + dst;
    while (dst != 0)
    {
        oam_shadow[dst] = 0xF8;
        dst = dst + 4;
    }
}

void sprite_set(unsigned char index, unsigned char x, unsigned char y, unsigned char tile, unsigned char attr)
{
    unsigned char dst;
    dst = index + index;
    dst = dst + dst;
    oam_shadow[dst] = y;
    oam_shadow[(unsigned char)(dst + 1)] = tile;
    oam_shadow[(unsigned char)(dst + 2)] = attr;
    oam_shadow[(unsigned char)(dst + 3)] = x;
}

unsigned char draw_16(unsigned char index, unsigned char x, unsigned char y, unsigned char tile, unsigned char attr)
{
    sprite_set(index, x, y, tile, attr);
    index = index + 1;
    sprite_set(index, (unsigned char)(x + 8), y, (unsigned char)(tile + 1), attr);
    index = index + 1;
    sprite_set(index, x, (unsigned char)(y + 8), (unsigned char)(tile + 2), attr);
    index = index + 1;
    sprite_set(index, (unsigned char)(x + 8), (unsigned char)(y + 8), (unsigned char)(tile + 3), attr);
    index = index + 1;
    return index;
}

void nes_wait_nmi(void)
{
    unsigned char start;
    start = nmi_counter;
    while (nmi_counter == start)
    {
    }
}

void nes_vblank_wait(void)
{
    while ((PPUSTATUS & 0x80) != 0)
    {
    }
    while ((PPUSTATUS & 0x80) == 0)
    {
    }
}

void __nes_nmi(void)
{
    if (vram_used != 0)
    {
        vramq_flush();
    }
    OAMADDR = 0;
    OAMDMA = 0x02;
    PPUSCROLL = 0;
    PPUSCROLL = 0;
    nmi_counter = nmi_counter + 1;
}

void pad_poll(void)
{
    unsigned char state;
    unsigned char mask;
    unsigned char sample;

    pad_prev = pad_cur;
    JOY1 = 1;
    JOY1 = 0;
    state = 0;
    mask = 1;
    while (mask != 0)
    {
        sample = JOY1;
        if ((sample & 1) != 0)
        {
            state = state | mask;
        }
        mask = mask + mask;
    }
    pad_cur = state;
    pad_pressed = (unsigned char)(state & (unsigned char)(state ^ pad_prev));
    sample = JOY2_APU_FRAME;
    sample = sample;
}

unsigned char rng8(void)
{
    unsigned char x;
    x = rng_state;
    x = (unsigned char)(x ^ (unsigned char)(x << 3));
    x = (unsigned char)(x ^ (unsigned char)(x >> 5));
    x = (unsigned char)(x ^ (unsigned char)(x << 1));
    rng_state = x;
    return x;
}

void apu_init(void)
{
    JOY2_APU_FRAME = 0x40;
    APU_STATUS = 0x0F;
    SQ1_SWEEP = 0x08;
    SQ2_SWEEP = 0x08;
    SQ1_VOL = 0x10;
    SQ2_VOL = 0x10;
    TRI_LINEAR = 0x80;
    NOISE_VOL = 0x10;
}

void pulse1_play(unsigned char idx, unsigned char volume)
{
    SQ1_SWEEP = 0x08;
    SQ1_VOL = volume;
    SQ1_LO = note_lo[idx];
    SQ1_HI = (unsigned char)(0x18 | note_hi[idx]);
}

void pulse2_play(unsigned char idx)
{
    SQ2_SWEEP = 0x08;
    SQ2_VOL = 0x92;
    SQ2_LO = note_lo[idx];
    SQ2_HI = (unsigned char)(0x18 | note_hi[idx]);
}

void triangle_play(unsigned char idx)
{
    TRI_LINEAR = 0x81;
    TRI_LO = note_lo[idx];
    TRI_HI = (unsigned char)(0x08 | note_hi[idx]);
}

void sfx_collect(void)
{
    sfx_kind = 1;
    sfx_timer = 8;
    pulse1_play(11, 0x9C);
}

void sfx_hit(void)
{
    sfx_kind = 2;
    sfx_timer = 14;
    SQ1_SWEEP = 0x08;
    SQ1_VOL = 0x8F;
    SQ1_LO = 0xF0;
    SQ1_HI = 0x1A;
    NOISE_VOL = 0x1F;
    NOISE_LO = 0x05;
    NOISE_HI = 0x18;
}

void sfx_power(void)
{
    sfx_kind = 3;
    sfx_timer = 16;
    pulse1_play(4, 0x9A);
}

void sfx_step(void)
{
    if (sfx_timer != 0)
    {
        sfx_timer = sfx_timer - 1;
        if (sfx_kind == 1)
        {
            if (sfx_timer == 4)
            {
                pulse1_play(14, 0x98);
            }
        }
        if (sfx_kind == 2)
        {
            if (sfx_timer == 4)
            {
                NOISE_VOL = 0x10;
                SQ1_VOL = 0x10;
            }
        }
        if (sfx_kind == 3)
        {
            if (sfx_timer == 10)
            {
                pulse1_play(7, 0x98);
            }
            if (sfx_timer == 5)
            {
                pulse1_play(12, 0x96);
            }
        }
    }
    else
    {
        SQ1_VOL = 0x10;
        NOISE_VOL = 0x10;
        sfx_kind = 0;
    }
}

void music_step_do(void)
{
    unsigned char n;
    unsigned char b;

    n = melody_seq[music_step];
    pulse2_play(n);
    music_step = music_step + 1;
    if (music_step == 32)
    {
        music_step = 0;
    }

    if ((music_step & 3) == 0)
    {
        b = bass_seq[bass_step];
        triangle_play(b);
        bass_step = bass_step + 1;
        if (bass_step == 8)
        {
            bass_step = 0;
        }
    }
}

void music_tick(void)
{
    music_tick_count = music_tick_count + 1;
    if ((music_tick_count & 7) == 0)
    {
        music_step_do();
    }
    sfx_step();
}

void put_text(unsigned char row, unsigned char col, unsigned char* text, unsigned char len)
{
    ppu_write(nt_addr(row, col), text, len);
}

void draw_starfield(void)
{
    unsigned char i;
    unsigned char r;
    unsigned char c;
    unsigned short a;

    i = 0;
    while (i != 38)
    {
        r = (unsigned char)((rng8() & 0x0F) + 4);
        c = (unsigned char)(rng8() & 0x1F);
        a = nt_addr(r, c);
        ppu_put(a, T_STARBG);
        i = i + 1;
    }
}

void draw_skyline(void)
{
    unsigned char col;
    unsigned char h;
    unsigned char row;

    col = 0;
    while (col != 32)
    {
        h = (unsigned char)((rng8() & 3) + 2);
        row = (unsigned char)(26 - h);
        while (row != 27)
        {
            ppu_put(nt_addr(row, col), T_BLOCK);
            if ((row & 1) == 0)
            {
                ppu_put(nt_addr(row, col), T_WINDOW);
            }
            row = row + 1;
        }
        col = col + 1;
    }
    ppu_fill(nt_addr(27, 0), T_FLOOR, 96);
}

void draw_base_screen(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;
    vramq_clear();
    ppu_fill(0x2000, T_SPACE, 960);
    ppu_fill(0x23C0, 0x00, 64);
    draw_starfield();
    draw_skyline();
}

void draw_title_screen(void)
{
    draw_base_screen();
    put_text(8, 11, txt_title1, 10);
    put_text(10, 10, txt_title2, 11);
    put_text(14, 10, txt_press, 11);
    put_text(17, 7, txt_rule1, 16);
    put_text(19, 7, txt_rule2, 18);
    oam_hide_all();
    OAMADDR = 0;
    OAMDMA = 0x02;
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
}

void draw_play_screen(void)
{
    draw_base_screen();
    put_text(1, 2, txt_score, 4);
    put_text(1, 14, txt_goal, 7);
    put_text(1, 23, txt_life, 4);
    put_text(2, 2, txt_chain, 3);
    put_text(2, 10, txt_focus, 3);
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
}

void draw_win_screen(void)
{
    draw_base_screen();
    put_text(10, 12, txt_win1, 8);
    put_text(12, 10, txt_win2, 11);
    put_text(16, 10, txt_press, 11);
    oam_hide_all();
    OAMADDR = 0;
    OAMDMA = 0x02;
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
}

void draw_over_screen(void)
{
    draw_base_screen();
    put_text(10, 11, txt_over1, 9);
    put_text(12, 10, txt_over2, 11);
    put_text(16, 10, txt_press, 11);
    oam_hide_all();
    OAMADDR = 0;
    OAMDMA = 0x02;
    PPUCTRL = 0x80;
    PPUMASK = 0x1E;
}

void respawn_object(unsigned char i)
{
    unsigned char r;
    unsigned char x;

    x = rng8();
    if (x < 16)
    {
        x = x + 16;
    }
    if (x > 224)
    {
        x = x - 32;
    }
    obj_x[i] = x;
    obj_y[i] = (unsigned char)(24 + (rng8() & 31));
    obj_speed[i] = (unsigned char)((rng8() & 3) + 1);

    r = (unsigned char)(rng8() & 7);
    if (r == 0)
    {
        obj_type[i] = 1; /* rock */
    }
    else
    {
        if (r == 1)
        {
            obj_type[i] = 2; /* shield */
        }
        else
        {
            if (r == 2)
            {
                obj_type[i] = 3; /* comet: high risk, high reward */
                obj_speed[i] = (unsigned char)(obj_speed[i] + 2);
            }
            else
            {
                obj_type[i] = 0; /* star */
            }
        }
    }
}

void init_objects(void)
{
    unsigned char i;
    i = 0;
    while (i != 6)
    {
        respawn_object(i);
        obj_y[i] = (unsigned char)(24 + (i << 4));
        i = i + 1;
    }
}

void update_score_digits(void)
{
    unsigned char tens;
    unsigned char ones;

    tens = 0;
    ones = score;
    while (ones >= 10)
    {
        ones = ones - 10;
        tens = tens + 1;
    }
    hud_digits[0] = (unsigned char)(T_0 + tens);
    hud_digits[1] = (unsigned char)(T_0 + ones);
    hud_life[0] = (unsigned char)(T_0 + lives);
    hud_combo[0] = (unsigned char)(T_0 + combo);
    hud_focus[0] = (unsigned char)(T_0 + focus_meter);
    if (shield_on != 0)
    {
        hud_shield[0] = T_SHIELD;
    }
    else
    {
        hud_shield[0] = T_SPACE;
    }
}

void queue_hud(void)
{
    update_score_digits();
    vramq_write(nt_addr(1, 7), hud_digits, 2);
    vramq_write(nt_addr(1, 28), hud_life, 1);
    vramq_write(nt_addr(1, 30), hud_shield, 1);
    vramq_write(nt_addr(2, 6), hud_combo, 1);
    vramq_write(nt_addr(2, 14), hud_focus, 1);
}

void start_game(void)
{
    score = 0;
    lives = 3;
    shield_on = 0;
    combo = 0;
    focus_meter = 3;
    focus_on = 0;
    last_gain = 0;
    player_x = 120;
    player_y = 204;
    init_objects();
    draw_play_screen();
    queue_hud();
    game_state = 1;
}

unsigned char box_hit(unsigned char ax, unsigned char ay, unsigned char bx, unsigned char by)
{
    if ((unsigned char)(ax + 14) < bx)
    {
        return 0;
    }
    if ((unsigned char)(bx + 14) < ax)
    {
        return 0;
    }
    if ((unsigned char)(ay + 14) < by)
    {
        return 0;
    }
    if ((unsigned char)(by + 14) < ay)
    {
        return 0;
    }
    return 1;
}

void update_player(void)
{
    unsigned char spd;
    spd = 2;
    if ((pad_cur & PAD_A) != 0)
    {
        spd = 4;
    }
    if ((pad_cur & PAD_LEFT) != 0)
    {
        if (player_x > spd)
        {
            player_x = player_x - spd;
        }
    }
    if ((pad_cur & PAD_RIGHT) != 0)
    {
        if (player_x < 232)
        {
            player_x = player_x + spd;
        }
    }
}

void add_score(unsigned char gain)
{
    while (gain != 0)
    {
        if (score < 99)
        {
            score = score + 1;
        }
        gain = gain - 1;
    }
}

void collect_star_kind(unsigned char kind)
{
    unsigned char gain;
    gain = 1;
    if (kind == 3)
    {
        gain = 3;
    }
    if ((pad_cur & PAD_A) != 0)
    {
        gain = (unsigned char)(gain + 1);
    }
    if (combo >= 3)
    {
        gain = (unsigned char)(gain + 1);
    }
    if (focus_on != 0)
    {
        if (gain > 1)
        {
            gain = gain - 1;
        }
    }
    add_score(gain);
    last_gain = gain;
    if (combo < 9)
    {
        combo = combo + 1;
    }
    if (focus_meter < 9)
    {
        focus_meter = focus_meter + 1;
    }
    sfx_collect();
    if (score >= 50)
    {
        game_state = 2;
        draw_win_screen();
    }
}

void hurt_player(void)
{
    if (shield_on != 0)
    {
        shield_on = 0;
        combo = 0;
        sfx_hit();
    }
    else
    {
        if (lives != 0)
        {
            lives = lives - 1;
        }
        combo = 0;
        sfx_hit();
        if (lives == 0)
        {
            game_state = 3;
            draw_over_screen();
        }
    }
}

unsigned char near_player_x(unsigned char ox)
{
    if (ox > player_x)
    {
        if ((unsigned char)(ox - player_x) < 28) { return 1; }
    }
    else
    {
        if ((unsigned char)(player_x - ox) < 28) { return 1; }
    }
    return 0;
}

void brave_graze(unsigned char i)
{
    if (obj_type[i] == 1)
    {
        if (obj_y[i] > player_y)
        {
            if (obj_y[i] < (unsigned char)(player_y + 22))
            {
                if (near_player_x(obj_x[i]) != 0)
                {
                    add_score(1);
                    if (combo < 9) { combo = combo + 1; }
                    if (focus_meter < 9) { focus_meter = focus_meter + 1; }
                    sfx_collect();
                    respawn_object(i);
                }
            }
        }
    }
}

void update_objects(void)
{
    unsigned char i;
    unsigned char sp;
    focus_on = 0;
    if ((pad_cur & PAD_B) != 0)
    {
        if (focus_meter != 0)
        {
            focus_on = 1;
        }
    }
    if (focus_on != 0)
    {
        if ((frame_counter & 0x0F) == 0)
        {
            focus_meter = focus_meter - 1;
        }
    }
    i = 0;
    while (i != 6)
    {
        sp = obj_speed[i];
        if (focus_on != 0)
        {
            if (sp > 1)
            {
                sp = sp - 1;
            }
        }
        obj_y[i] = (unsigned char)(obj_y[i] + sp);
        if (obj_y[i] > 224)
        {
            respawn_object(i);
        }
        else
        {
            brave_graze(i);
        }
        if (box_hit(player_x, player_y, obj_x[i], obj_y[i]) != 0)
        {
            if (obj_type[i] == 0)
            {
                collect_star_kind(0);
            }
            else
            {
                if (obj_type[i] == 3)
                {
                    collect_star_kind(3);
                }
                else
                {
                    if (obj_type[i] == 1)
                    {
                        hurt_player();
                    }
                    else
                    {
                        shield_on = 1;
                        if (focus_meter < 9) { focus_meter = focus_meter + 1; }
                        sfx_power();
                    }
                }
            }
            respawn_object(i);
        }
        i = i + 1;
    }
}

void draw_game_objects(void)
{
    unsigned char next;
    unsigned char i;
    unsigned char t;
    unsigned char a;

    next = 0;
    if ((pad_cur & PAD_A) != 0)
    {
        t = TILE_DASHER;
    }
    else
    {
        t = TILE_PLAYER;
    }
    if (shield_on != 0)
    {
        a = 3;
    }
    else
    {
        a = 0;
    }
    next = draw_16(next, player_x, player_y, t, a);

    i = 0;
    while (i != 6)
    {
        if (obj_type[i] == 0)
        {
            t = TILE_STAR;
            a = 1;
        }
        else
        {
            if (obj_type[i] == 3)
            {
                t = TILE_STAR;
                a = 3;
            }
            else
            {
                if (obj_type[i] == 1)
                {
                    t = TILE_ROCK;
                    a = 2;
                }
                else
                {
                    t = TILE_SHIELD;
                    a = 3;
                }
            }
        }
        next = draw_16(next, obj_x[i], obj_y[i], t, a);
        i = i + 1;
    }
    oam_hide_from(next);
}

void update_playing(void)
{
    update_player();
    update_objects();
    if (game_state == 1)
    {
        draw_game_objects();
        queue_hud();
    }
}

void boot(void)
{
    PPUCTRL = 0;
    PPUMASK = 0;
    rng_state = 0xA7;
    nes_vblank_wait();
    nes_vblank_wait();
    ppu_write(0x3F00, palette_data, 32);
    apu_init();
    oam_hide_all();
    draw_title_screen();
    game_state = 0;
}

unsigned char main(void)
{
    boot();
    while (1)
    {
        nes_wait_nmi();
        frame_counter = frame_counter + 1;
        rng_state = (unsigned char)(rng_state + frame_counter);
        music_tick();
        pad_poll();

        if (game_state == 0)
        {
            if ((pad_pressed & PAD_START) != 0)
            {
                start_game();
            }
        }
        else
        {
            if (game_state == 1)
            {
                update_playing();
            }
            else
            {
                if ((pad_pressed & PAD_START) != 0)
                {
                    start_game();
                }
            }
        }
    }
    return 0;
}
