use skyline::hooks::InlineCtx;

use crate::latency;
use crate::utils;

static mut TARGET_FRAME_RATE: u32 = 60;
static mut VSYNC_ENABLED: bool = true;
static mut TICK_FREQ: u64 = 0;

#[skyline::hook(offset = 0x135cad8, inline)]
unsafe fn on_game_speed_calc(_: &InlineCtx) {
    if !latency::is_local_online() {
        return;
    }
    set_internal_framerate(3600 / TARGET_FRAME_RATE as u32);
}

#[skyline::hook(offset = 0x3746afc, inline)]
unsafe fn scene_update(_: &InlineCtx) {
    static mut PREV_TICK: Option<skyline::nn::os::Tick> = None;
    if !latency::is_local_online() {
        return;
    }
    set_framerate_target(TARGET_FRAME_RATE);
    set_vsync_enabled(VSYNC_ENABLED);
    if VSYNC_ENABLED {
        return;
    }
    let target_ticks = TICK_FREQ / TARGET_FRAME_RATE as u64;
    if let Some(prev_tick) = PREV_TICK {
        loop {
            let elapsed_ticks = skyline::nn::os::GetSystemTick() - prev_tick;
            if elapsed_ticks >= target_ticks {
                break;
            }
        }
    }
    PREV_TICK = Some(skyline::nn::os::GetSystemTick());
}

unsafe fn set_swap_interval(swap_interval: i32) {
    let base_addr = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
    let r = *((base_addr + 0x06D41430) as *const u64);
    let r = *((r + 0x10) as *const u64);
    let r = (r + 0xF14) as *mut i32;
    *r = swap_interval;
}

unsafe fn set_internal_framerate(internal_framerate: u32) {
    let base_addr = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
    let internal_frame_rate_addr = base_addr + 0x523b004;
    *(internal_frame_rate_addr as *mut u32) = internal_framerate
}

pub unsafe fn set_framerate_target(framerate_target: u32) {
    TARGET_FRAME_RATE = framerate_target;
    set_internal_framerate(3600 / TARGET_FRAME_RATE as u32);
}

pub unsafe fn set_vsync_enabled(enabled: bool) {
    VSYNC_ENABLED = enabled;
    match (VSYNC_ENABLED, TARGET_FRAME_RATE >= 120) {
        (true, true) => set_swap_interval(-1 * ((TARGET_FRAME_RATE / 120) - 1) as i32),
        (false, _) => set_swap_interval(-499),
        _ => set_swap_interval(1),
    }
}

pub unsafe fn framerate_target() -> u32 {
    return TARGET_FRAME_RATE;
}

pub unsafe fn vsync_enabled() -> bool {
    return VSYNC_ENABLED;
}

pub unsafe fn poll() {
    let pressed_buttons = utils::poll_buttons(vec![
        ninput::Buttons::UP,
        ninput::Buttons::DOWN,
        ninput::Buttons::X,
    ]);
    match pressed_buttons {
        ninput::Buttons::UP => {
            if VSYNC_ENABLED {
                TARGET_FRAME_RATE *= 2;
            }
        }
        ninput::Buttons::DOWN => {
            if VSYNC_ENABLED {
                TARGET_FRAME_RATE /= 2;
            }
        }
        ninput::Buttons::X => {
            if TARGET_FRAME_RATE == 60 {
                VSYNC_ENABLED = !VSYNC_ENABLED;
            }
        }
        _ => (),
    }
    TARGET_FRAME_RATE = TARGET_FRAME_RATE.clamp(60, 240);
}

pub fn install() {
    skyline::install_hooks!(scene_update, on_game_speed_calc,);

    #[skyline::from_offset(0x39bf580)]
    fn get_tick_freq() -> u64;
    unsafe {
        TICK_FREQ = get_tick_freq();
    }
}
