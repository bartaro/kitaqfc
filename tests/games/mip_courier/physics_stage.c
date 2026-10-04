#include "physics_stage.h"
#include "assets.h"
#include "hardware.h"

u8 mip_physics_collision_index;
u8 mip_physics_collision_result;

/*
 * KITAQFC physics2d uses Q5.3 channels: a whole-pixel coordinate, a 1/8-pixel
 * fraction, an unsigned speed, and a direction.  This avoids signed-compare
 * ambiguity on the 6502 while preserving smooth acceleration and inertia.
 */

static void physics_integrate_player(void)
{
    u8 magnitude;

    if (mip_game.player_dir_x == KQ2D_DIR_POSITIVE) {
        mip_game.player_sub_x =
            (u8)(mip_game.player_sub_x + mip_game.player_speed_x);
        mip_game.player_x =
            (u8)(mip_game.player_x +
                (mip_game.player_sub_x >> KQ2D_SUBPIXEL_BITS));
        mip_game.player_sub_x =
            (u8)(mip_game.player_sub_x & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.player_dir_x == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.player_speed_x;
        while (magnitude != 0) {
            if (magnitude <= mip_game.player_sub_x) {
                mip_game.player_sub_x =
                    (u8)(mip_game.player_sub_x - magnitude);
                magnitude = 0;
            } else {
                magnitude =
                    (u8)(magnitude - (mip_game.player_sub_x + 1));
                mip_game.player_x = (u8)(mip_game.player_x - 1);
                mip_game.player_sub_x = KQ2D_SUBPIXEL_MASK;
            }
        }
    }

    if (mip_game.player_dir_y == KQ2D_DIR_POSITIVE) {
        mip_game.player_sub_y =
            (u8)(mip_game.player_sub_y + mip_game.player_speed_y);
        mip_game.player_y =
            (u8)(mip_game.player_y +
                (mip_game.player_sub_y >> KQ2D_SUBPIXEL_BITS));
        mip_game.player_sub_y =
            (u8)(mip_game.player_sub_y & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.player_dir_y == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.player_speed_y;
        while (magnitude != 0) {
            if (magnitude <= mip_game.player_sub_y) {
                mip_game.player_sub_y =
                    (u8)(mip_game.player_sub_y - magnitude);
                magnitude = 0;
            } else {
                magnitude =
                    (u8)(magnitude - (mip_game.player_sub_y + 1));
                mip_game.player_y = (u8)(mip_game.player_y - 1);
                mip_game.player_sub_y = KQ2D_SUBPIXEL_MASK;
            }
        }
    }
}

static void physics_integrate_drone0(void)
{
    u8 magnitude;

    if (mip_game.drone0_dir_x == KQ2D_DIR_POSITIVE) {
        mip_game.drone0_sub_x =
            (u8)(mip_game.drone0_sub_x + mip_game.drone0_speed_x);
        mip_game.drone0_x =
            (u8)(mip_game.drone0_x +
                (mip_game.drone0_sub_x >> KQ2D_SUBPIXEL_BITS));
        mip_game.drone0_sub_x =
            (u8)(mip_game.drone0_sub_x & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.drone0_dir_x == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.drone0_speed_x;
        while (magnitude != 0) {
            if (magnitude <= mip_game.drone0_sub_x) {
                mip_game.drone0_sub_x =
                    (u8)(mip_game.drone0_sub_x - magnitude);
                magnitude = 0;
            } else {
                magnitude = (u8)(magnitude - (mip_game.drone0_sub_x + 1));
                mip_game.drone0_x = (u8)(mip_game.drone0_x - 1);
                mip_game.drone0_sub_x = KQ2D_SUBPIXEL_MASK;
            }
        }
    }

    if (mip_game.drone0_dir_y == KQ2D_DIR_POSITIVE) {
        mip_game.drone0_sub_y =
            (u8)(mip_game.drone0_sub_y + mip_game.drone0_speed_y);
        mip_game.drone0_y =
            (u8)(mip_game.drone0_y +
                (mip_game.drone0_sub_y >> KQ2D_SUBPIXEL_BITS));
        mip_game.drone0_sub_y =
            (u8)(mip_game.drone0_sub_y & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.drone0_dir_y == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.drone0_speed_y;
        while (magnitude != 0) {
            if (magnitude <= mip_game.drone0_sub_y) {
                mip_game.drone0_sub_y =
                    (u8)(mip_game.drone0_sub_y - magnitude);
                magnitude = 0;
            } else {
                magnitude = (u8)(magnitude - (mip_game.drone0_sub_y + 1));
                mip_game.drone0_y = (u8)(mip_game.drone0_y - 1);
                mip_game.drone0_sub_y = KQ2D_SUBPIXEL_MASK;
            }
        }
    }
}

static void physics_integrate_drone1(void)
{
    u8 magnitude;

    if (mip_game.drone1_dir_x == KQ2D_DIR_POSITIVE) {
        mip_game.drone1_sub_x =
            (u8)(mip_game.drone1_sub_x + mip_game.drone1_speed_x);
        mip_game.drone1_x =
            (u8)(mip_game.drone1_x +
                (mip_game.drone1_sub_x >> KQ2D_SUBPIXEL_BITS));
        mip_game.drone1_sub_x =
            (u8)(mip_game.drone1_sub_x & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.drone1_dir_x == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.drone1_speed_x;
        while (magnitude != 0) {
            if (magnitude <= mip_game.drone1_sub_x) {
                mip_game.drone1_sub_x =
                    (u8)(mip_game.drone1_sub_x - magnitude);
                magnitude = 0;
            } else {
                magnitude = (u8)(magnitude - (mip_game.drone1_sub_x + 1));
                mip_game.drone1_x = (u8)(mip_game.drone1_x - 1);
                mip_game.drone1_sub_x = KQ2D_SUBPIXEL_MASK;
            }
        }
    }

    if (mip_game.drone1_dir_y == KQ2D_DIR_POSITIVE) {
        mip_game.drone1_sub_y =
            (u8)(mip_game.drone1_sub_y + mip_game.drone1_speed_y);
        mip_game.drone1_y =
            (u8)(mip_game.drone1_y +
                (mip_game.drone1_sub_y >> KQ2D_SUBPIXEL_BITS));
        mip_game.drone1_sub_y =
            (u8)(mip_game.drone1_sub_y & KQ2D_SUBPIXEL_MASK);
    } else if (mip_game.drone1_dir_y == KQ2D_DIR_NEGATIVE) {
        magnitude = mip_game.drone1_speed_y;
        while (magnitude != 0) {
            if (magnitude <= mip_game.drone1_sub_y) {
                mip_game.drone1_sub_y =
                    (u8)(mip_game.drone1_sub_y - magnitude);
                magnitude = 0;
            } else {
                magnitude = (u8)(magnitude - (mip_game.drone1_sub_y + 1));
                mip_game.drone1_y = (u8)(mip_game.drone1_y - 1);
                mip_game.drone1_sub_y = KQ2D_SUBPIXEL_MASK;
            }
        }
    }
}

void physics_stage_reset_positions(void)
{
    mip_game.player_x = 32;
    mip_game.player_y = 48;
    mip_game.player_sub_x = 0;
    mip_game.player_sub_y = 0;
    mip_game.player_speed_x = 0;
    mip_game.player_speed_y = 0;
    mip_game.player_dir_x = KQ2D_DIR_NONE;
    mip_game.player_dir_y = KQ2D_DIR_NONE;

    mip_game.drone0_x = 208;
    mip_game.drone0_y = 72;
    mip_game.drone0_sub_x = 0;
    mip_game.drone0_sub_y = 0;
    mip_game.drone0_speed_x = 0;
    mip_game.drone0_speed_y = 0;
    mip_game.drone0_dir_x = KQ2D_DIR_NONE;
    mip_game.drone0_dir_y = KQ2D_DIR_NONE;

    mip_game.drone1_x = 120;
    mip_game.drone1_y = 184;
    mip_game.drone1_sub_x = 0;
    mip_game.drone1_sub_y = 0;
    mip_game.drone1_speed_x = 0;
    mip_game.drone1_speed_y = 0;
    mip_game.drone1_dir_x = KQ2D_DIR_NONE;
    mip_game.drone1_dir_y = KQ2D_DIR_NONE;
}

void physics_stage_init(void)
{
    physics_stage_reset_positions();
}

u8 physics_stage_update_player(void)
{
    u8 accel_x;
    u8 accel_y;
    u8 accel_dir_x;
    u8 accel_dir_y;
    u8 speed_limit;
    u8 event;
    u8 curve;
    u8 drag_amount;
    u8 drag_shift;
    u8 bumper_index;
    u8 bumper_base;
    u8 center_x;
    u8 center_y;
    u8 distance_x;
    u8 distance_y;

    accel_x = 0;
    accel_y = 0;
    accel_dir_x = KQ2D_DIR_NONE;
    accel_dir_y = KQ2D_DIR_NONE;
    event = PHYS_EVENT_NONE;
    curve = 5;

    if ((mip_pad_current & PAD_LEFT) != 0) {
        accel_x = 3;
        accel_dir_x = KQ2D_DIR_NEGATIVE;
        mip_game.last_dir = PAD_LEFT;
        curve = 7;
    }
    if ((mip_pad_current & PAD_RIGHT) != 0) {
        accel_x = 3;
        accel_dir_x = KQ2D_DIR_POSITIVE;
        mip_game.last_dir = PAD_RIGHT;
        curve = 7;
    }
    if ((mip_pad_current & PAD_UP) != 0) {
        accel_y = 3;
        accel_dir_y = KQ2D_DIR_NEGATIVE;
        mip_game.last_dir = PAD_UP;
        curve = 7;
    }
    if ((mip_pad_current & PAD_DOWN) != 0) {
        accel_y = 3;
        accel_dir_y = KQ2D_DIR_POSITIVE;
        mip_game.last_dir = PAD_DOWN;
        curve = 7;
    }
    if ((mip_pad_current & PAD_B) != 0) curve = 2;

    if (mip_game.boost_timer != 0) speed_limit = 40;
    else speed_limit = 24;

    if ((mip_pad_pressed & PAD_A) != 0 && mip_game.boosts != 0) {
        mip_game.boosts = (u8)(mip_game.boosts - 1);
        mip_game.boost_timer = 18;
        if (mip_game.last_dir == PAD_LEFT) {
            accel_x = (u8)(accel_x + 20);
            accel_dir_x = KQ2D_DIR_NEGATIVE;
        } else if (mip_game.last_dir == PAD_RIGHT) {
            accel_x = (u8)(accel_x + 20);
            accel_dir_x = KQ2D_DIR_POSITIVE;
        } else if (mip_game.last_dir == PAD_UP) {
            accel_y = (u8)(accel_y + 20);
            accel_dir_y = KQ2D_DIR_NEGATIVE;
        } else {
            accel_y = (u8)(accel_y + 20);
            accel_dir_y = KQ2D_DIR_POSITIVE;
        }
        event = (u8)(event | PHYS_EVENT_BOOST);
    }

    if (accel_x != 0) {
        if (mip_game.player_dir_x == KQ2D_DIR_NONE ||
            mip_game.player_dir_x == accel_dir_x) {
            mip_game.player_dir_x = accel_dir_x;
            mip_game.player_speed_x =
                (u8)(mip_game.player_speed_x + accel_x);
        } else if (mip_game.player_speed_x > accel_x) {
            mip_game.player_speed_x =
                (u8)(mip_game.player_speed_x - accel_x);
        } else {
            mip_game.player_speed_x =
                (u8)(accel_x - mip_game.player_speed_x);
            mip_game.player_dir_x = accel_dir_x;
        }
        if (mip_game.player_speed_x > speed_limit) {
            mip_game.player_speed_x = speed_limit;
        }
    }

    if (accel_y != 0) {
        if (mip_game.player_dir_y == KQ2D_DIR_NONE ||
            mip_game.player_dir_y == accel_dir_y) {
            mip_game.player_dir_y = accel_dir_y;
            mip_game.player_speed_y =
                (u8)(mip_game.player_speed_y + accel_y);
        } else if (mip_game.player_speed_y > accel_y) {
            mip_game.player_speed_y =
                (u8)(mip_game.player_speed_y - accel_y);
        } else {
            mip_game.player_speed_y =
                (u8)(accel_y - mip_game.player_speed_y);
            mip_game.player_dir_y = accel_dir_y;
        }
        if (mip_game.player_speed_y > speed_limit) {
            mip_game.player_speed_y = speed_limit;
        }
    }

    if (curve == 2) drag_shift = KQ2D_DRAG_BRAKE;
    else if (curve == 5) drag_shift = KQ2D_DRAG_COAST;
    else drag_shift = KQ2D_DRAG_THRUST;

    if (mip_game.player_speed_x != 0) {
        drag_amount = (u8)(mip_game.player_speed_x >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.player_speed_x <= drag_amount) {
            mip_game.player_speed_x = 0;
            mip_game.player_dir_x = KQ2D_DIR_NONE;
        } else {
            mip_game.player_speed_x =
                (u8)(mip_game.player_speed_x - drag_amount);
        }
    }
    if (mip_game.player_speed_y != 0) {
        drag_amount = (u8)(mip_game.player_speed_y >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.player_speed_y <= drag_amount) {
            mip_game.player_speed_y = 0;
            mip_game.player_dir_y = KQ2D_DIR_NONE;
        } else {
            mip_game.player_speed_y =
                (u8)(mip_game.player_speed_y - drag_amount);
        }
    }

    physics_integrate_player();

    if (mip_game.player_x < 22) {
        mip_game.player_x = 22;
        mip_game.player_sub_x = 0;
        mip_game.player_dir_x = KQ2D_DIR_POSITIVE;
        mip_game.player_speed_x = (u8)(mip_game.player_speed_x >> 1);
        event = (u8)(event | PHYS_EVENT_BOUNCE);
    } else if (mip_game.player_x > 234) {
        mip_game.player_x = 234;
        mip_game.player_sub_x = 0;
        mip_game.player_dir_x = KQ2D_DIR_NEGATIVE;
        mip_game.player_speed_x = (u8)(mip_game.player_speed_x >> 1);
        event = (u8)(event | PHYS_EVENT_BOUNCE);
    }
    if (mip_game.player_y < 46) {
        mip_game.player_y = 46;
        mip_game.player_sub_y = 0;
        mip_game.player_dir_y = KQ2D_DIR_POSITIVE;
        mip_game.player_speed_y = (u8)(mip_game.player_speed_y >> 1);
        event = (u8)(event | PHYS_EVENT_BOUNCE);
    } else if (mip_game.player_y > 210) {
        mip_game.player_y = 210;
        mip_game.player_sub_y = 0;
        mip_game.player_dir_y = KQ2D_DIR_NEGATIVE;
        mip_game.player_speed_y = (u8)(mip_game.player_speed_y >> 1);
        event = (u8)(event | PHYS_EVENT_BOUNCE);
    }

    if (mip_game.bumper_cooldown == 0) {
        bumper_index = (u8)(mip_game.frame & 3);
        if (bumper_index >= BUMPER_COUNT) bumper_index = 0;
        bumper_base = (u8)(bumper_index << 1);
        center_x =
            (u8)((mip_bumpers[(__safe_index u8)bumper_base] << 3) + 4);
        center_y =
            (u8)((mip_bumpers[(__safe_index u8)(bumper_base + 1)] << 3) + 4);
        if (mip_game.player_x >= center_x) {
            distance_x = (u8)(mip_game.player_x - center_x);
        } else {
            distance_x = (u8)(center_x - mip_game.player_x);
        }
        if (mip_game.player_y >= center_y) {
            distance_y = (u8)(mip_game.player_y - center_y);
        } else {
            distance_y = (u8)(center_y - mip_game.player_y);
        }
        if (distance_x < 11 && distance_y < 11) {
            if (distance_x >= distance_y) {
                if (mip_game.player_x < center_x) {
                    mip_game.player_dir_x = KQ2D_DIR_NEGATIVE;
                } else {
                    mip_game.player_dir_x = KQ2D_DIR_POSITIVE;
                }
                mip_game.player_speed_x = 28;
            } else {
                if (mip_game.player_y < center_y) {
                    mip_game.player_dir_y = KQ2D_DIR_NEGATIVE;
                } else {
                    mip_game.player_dir_y = KQ2D_DIR_POSITIVE;
                }
                mip_game.player_speed_y = 28;
            }
            mip_game.bumper_cooldown = 20;
            event = (u8)(event | PHYS_EVENT_BOUNCE);
        }
    }

    return event;
}

static void physics_update_drone0(void)
{
    u8 accel_x;
    u8 accel_y;
    u8 accel_dir_x;
    u8 accel_dir_y;
    u8 limit;
    u8 drag_amount;
    u8 drag_shift;

    accel_x = 0;
    accel_y = 0;
    accel_dir_x = KQ2D_DIR_NONE;
    accel_dir_y = KQ2D_DIR_NONE;
    if (mip_game.drone0_stun == 0) {
        accel_x = (u8)(2 + mip_game.stage);
        accel_y = accel_x;
        if (mip_game.drone0_x < mip_game.player_x) {
            accel_dir_x = KQ2D_DIR_POSITIVE;
        } else {
            accel_dir_x = KQ2D_DIR_NEGATIVE;
        }
        if (mip_game.drone0_y < mip_game.player_y) {
            accel_dir_y = KQ2D_DIR_POSITIVE;
        } else {
            accel_dir_y = KQ2D_DIR_NEGATIVE;
        }
    }

    limit = (u8)(14 + (mip_game.stage << 1));
    if (accel_x != 0) {
        if (mip_game.drone0_dir_x == KQ2D_DIR_NONE ||
            mip_game.drone0_dir_x == accel_dir_x) {
            mip_game.drone0_dir_x = accel_dir_x;
            mip_game.drone0_speed_x =
                (u8)(mip_game.drone0_speed_x + accel_x);
        } else if (mip_game.drone0_speed_x > accel_x) {
            mip_game.drone0_speed_x =
                (u8)(mip_game.drone0_speed_x - accel_x);
        } else {
            mip_game.drone0_speed_x =
                (u8)(accel_x - mip_game.drone0_speed_x);
            mip_game.drone0_dir_x = accel_dir_x;
        }
        if (mip_game.drone0_speed_x > limit) {
            mip_game.drone0_speed_x = limit;
        }
    }
    if (accel_y != 0) {
        if (mip_game.drone0_dir_y == KQ2D_DIR_NONE ||
            mip_game.drone0_dir_y == accel_dir_y) {
            mip_game.drone0_dir_y = accel_dir_y;
            mip_game.drone0_speed_y =
                (u8)(mip_game.drone0_speed_y + accel_y);
        } else if (mip_game.drone0_speed_y > accel_y) {
            mip_game.drone0_speed_y =
                (u8)(mip_game.drone0_speed_y - accel_y);
        } else {
            mip_game.drone0_speed_y =
                (u8)(accel_y - mip_game.drone0_speed_y);
            mip_game.drone0_dir_y = accel_dir_y;
        }
        if (mip_game.drone0_speed_y > limit) {
            mip_game.drone0_speed_y = limit;
        }
    }

    if (mip_game.drone0_stun != 0) drag_shift = KQ2D_DRAG_STUN;
    else drag_shift = KQ2D_DRAG_THRUST;
    if (mip_game.drone0_speed_x != 0) {
        drag_amount = (u8)(mip_game.drone0_speed_x >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.drone0_speed_x <= drag_amount) {
            mip_game.drone0_speed_x = 0;
            mip_game.drone0_dir_x = KQ2D_DIR_NONE;
        } else {
            mip_game.drone0_speed_x =
                (u8)(mip_game.drone0_speed_x - drag_amount);
        }
    }
    if (mip_game.drone0_speed_y != 0) {
        drag_amount = (u8)(mip_game.drone0_speed_y >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.drone0_speed_y <= drag_amount) {
            mip_game.drone0_speed_y = 0;
            mip_game.drone0_dir_y = KQ2D_DIR_NONE;
        } else {
            mip_game.drone0_speed_y =
                (u8)(mip_game.drone0_speed_y - drag_amount);
        }
    }

    physics_integrate_drone0();
    if (mip_game.drone0_x < 22) {
        mip_game.drone0_x = 22;
        mip_game.drone0_dir_x = KQ2D_DIR_POSITIVE;
    } else if (mip_game.drone0_x > 234) {
        mip_game.drone0_x = 234;
        mip_game.drone0_dir_x = KQ2D_DIR_NEGATIVE;
    }
    if (mip_game.drone0_y < 46) {
        mip_game.drone0_y = 46;
        mip_game.drone0_dir_y = KQ2D_DIR_POSITIVE;
    } else if (mip_game.drone0_y > 210) {
        mip_game.drone0_y = 210;
        mip_game.drone0_dir_y = KQ2D_DIR_NEGATIVE;
    }
}

static void physics_update_drone1(void)
{
    u8 accel_x;
    u8 accel_y;
    u8 accel_dir_x;
    u8 accel_dir_y;
    u8 limit;
    u8 drag_amount;
    u8 drag_shift;

    accel_x = 0;
    accel_y = 0;
    accel_dir_x = KQ2D_DIR_NONE;
    accel_dir_y = KQ2D_DIR_NONE;
    if (mip_game.drone1_stun == 0) {
        accel_x = (u8)(2 + mip_game.stage);
        accel_y = accel_x;
        if (mip_game.drone1_x < mip_game.player_x) {
            accel_dir_x = KQ2D_DIR_POSITIVE;
        } else {
            accel_dir_x = KQ2D_DIR_NEGATIVE;
        }
        if (mip_game.drone1_y < mip_game.player_y) {
            accel_dir_y = KQ2D_DIR_POSITIVE;
        } else {
            accel_dir_y = KQ2D_DIR_NEGATIVE;
        }
    }

    limit = (u8)(14 + (mip_game.stage << 1));
    if (accel_x != 0) {
        if (mip_game.drone1_dir_x == KQ2D_DIR_NONE ||
            mip_game.drone1_dir_x == accel_dir_x) {
            mip_game.drone1_dir_x = accel_dir_x;
            mip_game.drone1_speed_x =
                (u8)(mip_game.drone1_speed_x + accel_x);
        } else if (mip_game.drone1_speed_x > accel_x) {
            mip_game.drone1_speed_x =
                (u8)(mip_game.drone1_speed_x - accel_x);
        } else {
            mip_game.drone1_speed_x =
                (u8)(accel_x - mip_game.drone1_speed_x);
            mip_game.drone1_dir_x = accel_dir_x;
        }
        if (mip_game.drone1_speed_x > limit) {
            mip_game.drone1_speed_x = limit;
        }
    }
    if (accel_y != 0) {
        if (mip_game.drone1_dir_y == KQ2D_DIR_NONE ||
            mip_game.drone1_dir_y == accel_dir_y) {
            mip_game.drone1_dir_y = accel_dir_y;
            mip_game.drone1_speed_y =
                (u8)(mip_game.drone1_speed_y + accel_y);
        } else if (mip_game.drone1_speed_y > accel_y) {
            mip_game.drone1_speed_y =
                (u8)(mip_game.drone1_speed_y - accel_y);
        } else {
            mip_game.drone1_speed_y =
                (u8)(accel_y - mip_game.drone1_speed_y);
            mip_game.drone1_dir_y = accel_dir_y;
        }
        if (mip_game.drone1_speed_y > limit) {
            mip_game.drone1_speed_y = limit;
        }
    }

    if (mip_game.drone1_stun != 0) drag_shift = KQ2D_DRAG_STUN;
    else drag_shift = KQ2D_DRAG_THRUST;
    if (mip_game.drone1_speed_x != 0) {
        drag_amount = (u8)(mip_game.drone1_speed_x >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.drone1_speed_x <= drag_amount) {
            mip_game.drone1_speed_x = 0;
            mip_game.drone1_dir_x = KQ2D_DIR_NONE;
        } else {
            mip_game.drone1_speed_x =
                (u8)(mip_game.drone1_speed_x - drag_amount);
        }
    }
    if (mip_game.drone1_speed_y != 0) {
        drag_amount = (u8)(mip_game.drone1_speed_y >> drag_shift);
        if (drag_amount == 0) drag_amount = 1;
        if (mip_game.drone1_speed_y <= drag_amount) {
            mip_game.drone1_speed_y = 0;
            mip_game.drone1_dir_y = KQ2D_DIR_NONE;
        } else {
            mip_game.drone1_speed_y =
                (u8)(mip_game.drone1_speed_y - drag_amount);
        }
    }

    physics_integrate_drone1();
    if (mip_game.drone1_x < 22) {
        mip_game.drone1_x = 22;
        mip_game.drone1_dir_x = KQ2D_DIR_POSITIVE;
    } else if (mip_game.drone1_x > 234) {
        mip_game.drone1_x = 234;
        mip_game.drone1_dir_x = KQ2D_DIR_NEGATIVE;
    }
    if (mip_game.drone1_y < 46) {
        mip_game.drone1_y = 46;
        mip_game.drone1_dir_y = KQ2D_DIR_POSITIVE;
    } else if (mip_game.drone1_y > 210) {
        mip_game.drone1_y = 210;
        mip_game.drone1_dir_y = KQ2D_DIR_NEGATIVE;
    }
}

void physics_stage_update_drones(void)
{
    if ((mip_game.frame & 1) == 0) physics_update_drone0();
    else physics_update_drone1();
}

void physics_stage_check_cargo(void)
{
    u8 source_index;
    u8 tile_x;
    u8 tile_y;
    u8 center_x;
    u8 center_y;
    u8 distance_x;
    u8 distance_y;

    mip_physics_collision_result = 0;
    source_index =
        (u8)((mip_game.stage << 3) + mip_physics_collision_index);
    /*
     * Collision coordinates are kept immediate in this switch.  Rendering
     * still owns the shared table; the hot physics bank does not dereference
     * cross-bank PRG data.
     */
    tile_x = 4;
    tile_y = 7;
    if (source_index == 1) { tile_x = 15; tile_y = 6; }
    else if (source_index == 2) { tile_x = 25; tile_y = 7; }
    else if (source_index == 3) { tile_x = 6; tile_y = 15; }
    else if (source_index == 4) { tile_x = 17; tile_y = 14; }
    else if (source_index == 5) { tile_x = 27; tile_y = 18; }
    else if (source_index == 6) { tile_x = 4; tile_y = 24; }
    else if (source_index == 7) { tile_x = 17; tile_y = 22; }
    else if (source_index == 8) { tile_x = 7; tile_y = 8; }
    else if (source_index == 9) { tile_x = 12; tile_y = 12; }
    else if (source_index == 10) { tile_x = 18; tile_y = 7; }
    else if (source_index == 11) { tile_x = 27; tile_y = 12; }
    else if (source_index == 12) { tile_x = 3; tile_y = 18; }
    else if (source_index == 13) { tile_x = 18; tile_y = 18; }
    else if (source_index == 14) { tile_x = 11; tile_y = 24; }
    else if (source_index == 15) { tile_x = 25; tile_y = 24; }
    else if (source_index == 16) { tile_x = 4; tile_y = 10; }
    else if (source_index == 17) { tile_x = 12; tile_y = 6; }
    else if (source_index == 18) { tile_x = 20; tile_y = 8; }
    else if (source_index == 19) { tile_x = 27; tile_y = 6; }
    else if (source_index == 20) { tile_x = 4; tile_y = 22; }
    else if (source_index == 21) { tile_x = 18; tile_y = 16; }
    else if (source_index == 22) { tile_x = 26; tile_y = 20; }
    else if (source_index == 23) { tile_x = 13; tile_y = 24; }
    center_x = (u8)((tile_x << 3) + 4);
    center_y = (u8)((tile_y << 3) + 4);
    if (mip_game.player_x >= center_x) {
        distance_x = (u8)(mip_game.player_x - center_x);
    } else {
        distance_x = (u8)(center_x - mip_game.player_x);
    }
    if (mip_game.player_y >= center_y) {
        distance_y = (u8)(mip_game.player_y - center_y);
    } else {
        distance_y = (u8)(center_y - mip_game.player_y);
    }
    if (distance_x < 11 && distance_y < 11) {
        mip_physics_collision_result = 1;
    }
}

void physics_stage_check_gate(void)
{
    u8 distance_x;
    u8 distance_y;

    mip_physics_collision_result = 0;
    if (mip_game.player_x >= 224) {
        distance_x = (u8)(mip_game.player_x - 224);
    } else {
        distance_x = (u8)(224 - mip_game.player_x);
    }
    if (mip_game.player_y >= 200) {
        distance_y = (u8)(mip_game.player_y - 200);
    } else {
        distance_y = (u8)(200 - mip_game.player_y);
    }
    if (distance_x < 15 && distance_y < 15) {
        mip_physics_collision_result = 1;
    }
}

void physics_stage_check_drone(void)
{
    u8 drone_x;
    u8 drone_y;
    u8 distance_x;
    u8 distance_y;

    mip_physics_collision_result = 0;
    if (mip_physics_collision_index == 0) {
        drone_x = mip_game.drone0_x;
        drone_y = mip_game.drone0_y;
    } else {
        drone_x = mip_game.drone1_x;
        drone_y = mip_game.drone1_y;
    }
    if (mip_game.player_x >= drone_x) {
        distance_x = (u8)(mip_game.player_x - drone_x);
    } else {
        distance_x = (u8)(drone_x - mip_game.player_x);
    }
    if (mip_game.player_y >= drone_y) {
        distance_y = (u8)(mip_game.player_y - drone_y);
    } else {
        distance_y = (u8)(drone_y - mip_game.player_y);
    }
    if (distance_x < 12 && distance_y < 12) {
        mip_physics_collision_result = 1;
    }
}

void physics_stage_knock_selected_drone(void)
{
    if (mip_physics_collision_index == 0) {
        if (mip_game.drone0_dir_x == KQ2D_DIR_NEGATIVE) {
            mip_game.drone0_dir_x = KQ2D_DIR_POSITIVE;
        } else if (mip_game.drone0_dir_x == KQ2D_DIR_POSITIVE) {
            mip_game.drone0_dir_x = KQ2D_DIR_NEGATIVE;
        }
        if (mip_game.drone0_dir_y == KQ2D_DIR_NEGATIVE) {
            mip_game.drone0_dir_y = KQ2D_DIR_POSITIVE;
        } else if (mip_game.drone0_dir_y == KQ2D_DIR_POSITIVE) {
            mip_game.drone0_dir_y = KQ2D_DIR_NEGATIVE;
        }
        mip_game.drone0_stun = 90;
    } else {
        if (mip_game.drone1_dir_x == KQ2D_DIR_NEGATIVE) {
            mip_game.drone1_dir_x = KQ2D_DIR_POSITIVE;
        } else if (mip_game.drone1_dir_x == KQ2D_DIR_POSITIVE) {
            mip_game.drone1_dir_x = KQ2D_DIR_NEGATIVE;
        }
        if (mip_game.drone1_dir_y == KQ2D_DIR_NEGATIVE) {
            mip_game.drone1_dir_y = KQ2D_DIR_POSITIVE;
        } else if (mip_game.drone1_dir_y == KQ2D_DIR_POSITIVE) {
            mip_game.drone1_dir_y = KQ2D_DIR_NEGATIVE;
        }
        mip_game.drone1_stun = 90;
    }
}
