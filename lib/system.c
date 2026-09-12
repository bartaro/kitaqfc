#include "system.h"

u16 kq_system_frame;
static SystemCallback system_vblank_callback;

void system_init(void)
{
    kq_system_frame = 0;
    system_vblank_callback = 0;
    __nmi_enable();
}

void system_wait_vblank(void)
{
    __nmi_wait();
    kq_system_frame = (u16)(kq_system_frame + 1);
}

void system_set_vblank_callback(SystemCallback callback)
{
    system_vblank_callback = callback;
}

u16 system_get_frame(void)
{
    return kq_system_frame;
}

u8 system_get_frame8(void)
{
    return (u8)kq_system_frame;
}

void system_enable_interrupts(void)
{
    __irq_enable();
}

void system_disable_interrupts(void)
{
    __irq_disable();
}
