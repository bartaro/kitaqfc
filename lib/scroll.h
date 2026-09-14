/*
 * Optional declarations for the phase 9 NES scroll helper.
 */

extern unsigned char nes_scroll_ctrl_base;
extern unsigned short nes_scroll_camera_x;
extern unsigned short nes_scroll_camera_y;

// Store PPUCTRL bits other than the two nametable-selection bits for later application.
void nes_scroll_set_base_ctrl(unsigned char ctrl);
// Store camera coordinates without touching hardware registers.
void nes_scroll_set(unsigned short camera_x, unsigned short camera_y);
// Combine bit 8 of each camera coordinate with base PPUCTRL, reset the latch
// by reading PPUSTATUS, and write low-byte X/Y scroll. This uses 256-unit
// wrapping on both axes rather than normalizing Y to a 240-pixel nametable height.
// Initialize base control to the intended PPUCTRL value, including NMI enable, before applying.
// Only coordinate bits 0..8 are used; arrange safe PPU timing in the caller.
void nes_scroll_apply(void);
// Subtract the screen anchor from the world point into unsigned camera
// coordinates. Underflow wraps; world-edge clamping belongs to the game.
void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y);
