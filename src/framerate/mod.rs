use std::sync::Mutex;

use crate::ldn;
use crate::utils;
use skyline::hooks::InlineCtx;

const DEFAULT_TARGET_FRAMERATE: u32 = 60;
const MAX_TARGET_FRAMERATE: u32 = 240;
const TARGET_FRAMERATE_INC: u32 = 60;

#[derive(Debug, Clone)]
pub struct FramerateConfig {
    target_framerate: u32,
    is_vsync_enabled: bool,
}

impl FramerateConfig {
    pub fn to_string(&self) -> String {
        let vsync_indicator = match self.is_vsync_enabled {
            true => "",
            false => "++",
        };
        format!("{} FPS{}", self.target_framerate, vsync_indicator)
    }
}

static FRAMERATE_CONFIG: Mutex<FramerateConfig> = Mutex::new(FramerateConfig {
    target_framerate: 60,
    is_vsync_enabled: true,
});

#[skyline::hook(offset = 0x135caf8, inline)]
unsafe fn on_game_speed_calc(_: &InlineCtx) {
    if !ldn::is_local_online() {
        return;
    }
    let target_framerate = FRAMERATE_CONFIG.lock().unwrap().target_framerate;
    set_internal_framerate(3600 / target_framerate);
}

#[skyline::hook(offset = 0x374777c, inline)]
unsafe fn scene_update(_: &InlineCtx) {
    static mut PREV_TICK: Option<skyline::nn::os::Tick> = None;
    if !ldn::is_local_online() {
        return;
    }
    let guard = FRAMERATE_CONFIG.lock().unwrap();
    let target_framerate = guard.target_framerate;
    let vsync_enabled = guard.is_vsync_enabled;
    drop(guard);
    set_framerate_target(target_framerate);
    set_vsync_enabled(vsync_enabled);
    if vsync_enabled {
        return;
    }
    let target_ticks = utils::get_tick_freq() / target_framerate as u64;
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
    let r = *((base_addr + 0x6d43430) as *const u64);
    let r = *((r + 0x10) as *const u64);
    let r = (r + 0xF14) as *mut i32;
    *r = swap_interval;
}

unsafe fn set_internal_framerate(internal_framerate: u32) {
    let base_addr = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
    let internal_frame_rate_addr = base_addr + 0x523d004;
    *(internal_frame_rate_addr as *mut u32) = internal_framerate
}

pub fn set_framerate_target(framerate_target: u32) {
    unsafe {
        let mut guard = FRAMERATE_CONFIG.lock().unwrap();
        guard.target_framerate = framerate_target;
        set_internal_framerate(3600 / guard.target_framerate);
    }
}

pub fn set_vsync_enabled(enabled: bool) {
    unsafe {
        let mut guard = FRAMERATE_CONFIG.lock().unwrap();
        guard.is_vsync_enabled = enabled;
        let target_framerate = guard.target_framerate;
        let vsync_enabled = guard.is_vsync_enabled;
        match (vsync_enabled, target_framerate == 60) {
            (true, false) => set_swap_interval(((target_framerate as f64 / 60.0) * 100.0) as i32),
            (false, _) => set_swap_interval(10000),
            _ => set_swap_interval(1),
        }
    }
}

pub fn get_framerate_config() -> FramerateConfig {
    FRAMERATE_CONFIG.lock().unwrap().clone()
}

pub fn poll() {
    let pressed_buttons = utils::poll_buttons(&[
        ninput::Buttons::UP,
        ninput::Buttons::DOWN,
        ninput::Buttons::X,
    ]);
    let mut guard = FRAMERATE_CONFIG.lock().unwrap();
    match pressed_buttons {
        ninput::Buttons::UP => {
            if guard.is_vsync_enabled {
                guard.target_framerate += TARGET_FRAMERATE_INC;
            }
        }
        ninput::Buttons::DOWN => {
            if guard.is_vsync_enabled {
                guard.target_framerate -= TARGET_FRAMERATE_INC;
            }
        }
        ninput::Buttons::X => {
            if guard.target_framerate == DEFAULT_TARGET_FRAMERATE {
                guard.is_vsync_enabled = !guard.is_vsync_enabled;
            }
        }
        _ => (),
    }
    guard.target_framerate = guard.target_framerate.clamp(DEFAULT_TARGET_FRAMERATE, MAX_TARGET_FRAMERATE);
}

pub fn install() {
    skyline::install_hooks!(scene_update, on_game_speed_calc);
}
