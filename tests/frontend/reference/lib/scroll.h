/*
 * Optional declarations for the phase 9 NES scroll helper.
 */

extern unsigned char nes_scroll_ctrl_base;
extern unsigned short nes_scroll_camera_x;
extern unsigned short nes_scroll_camera_y;

void nes_scroll_set_base_ctrl(unsigned char ctrl);
void nes_scroll_set(unsigned short camera_x, unsigned short camera_y);
void nes_scroll_apply(void);
void nes_camera_follow_center(unsigned short target_x, unsigned short target_y, unsigned char center_x, unsigned char center_y);
