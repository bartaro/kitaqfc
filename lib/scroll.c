/*
 * KITAQFC phase 9 NES scroll / camera helper.
 *
 * Intent:
 * - turn 16-bit camera coordinates into PPUCTRL nametable bits + PPUSCROLL writes
 * - keep the helper usable from the current KITAQFC C subset
 * - avoid mapper / mirroring policy assumptions beyond 2x2 nametable addressing
 */

__location(0x2000) unsigned char PPUCTRL;
__location(0x2002) unsigned char PPUSTATUS;
__location(0x2005) unsigned char PPUSCROLL;

unsigned char nes_scroll_ctrl_base;
unsigned short nes_scroll_camera_x;
unsigned short nes_scroll_camera_y;

void nes_scroll_set_base_ctrl(unsigned char ctrl)
{
    nes_scroll_ctrl_base = (unsigned char)(ctrl & 0xFC);
}

void nes_scroll_set(unsigned short camera_x, unsigned short camera_y)
{
    nes_scroll_camera_x = camera_x;
    nes_scroll_camera_y = camera_y;
}

void nes_scroll_apply(void)
{
    unsigned char ctrl;
    unsigned char x_hi;
    unsigned char y_hi;
    unsigned char latch;

    x_hi = (unsigned char)((nes_scroll_camera_x >> 8) & 1);
    y_hi = (unsigned char)((nes_scroll_camera_y >> 8) & 1);
    ctrl = (unsigned char)(nes_scroll_ctrl_base | x_hi | (unsigned char)(y_hi << 1));

    PPUCTRL = ctrl;

    latch = PPUSTATUS;
    PPUSCROLL = (unsigned char)nes_scroll_camera_x;
    PPUSCROLL = (unsigned char)nes_scroll_camera_y;
    latch = latch;
}

void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y)
{
    nes_scroll_camera_x = target_x - center_x;
    nes_scroll_camera_y = target_y - center_y;
}
