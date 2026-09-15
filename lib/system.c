#include "system.h"

u16 kq_system_frame;
static SystemCallback system_vblank_callback;

// Reset the software frame counter and callback storage, then enable NMI.
void system_init(void)
{
    kq_system_frame = 0;
    system_vblank_callback = 0;
    // NMI waits require a running PPU and its enabled NMI source, independently of the CPU IRQ mask.
    __nmi_enable();
}

// Wait for NMI, increment the software counter, then invoke the registered callback synchronously.
void system_wait_vblank(void)
{
    __nmi_wait();
    kq_system_frame = (u16)(kq_system_frame + 1);
    if (system_vblank_callback != 0) system_vblank_callback();
}

// Replace the callback run by system_wait_vblank; null disables callback dispatch.
void system_set_vblank_callback(SystemCallback callback)
{
    system_vblank_callback = callback;
}

// Return the wrapping 16-bit count of completed waits through this wrapper.
u16 system_get_frame(void)
{
    return kq_system_frame;
}

// Return the low byte of the software frame counter.
u8 system_get_frame8(void)
{
    return (u8)kq_system_frame;
}

// Enable maskable CPU IRQs through the intrinsic; NMI is controlled separately.
void system_enable_interrupts(void)
{
    __irq_enable();
}

// Disable maskable CPU IRQs; this does not disable the PPU's NMI source.
void system_disable_interrupts(void)
{
    __irq_disable();
}
