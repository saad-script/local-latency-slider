use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crate::ldn;
use crate::utils;
use skyline::hooks::InlineCtx;

const DEFAULT_TARGET_FRAMERATE: u32 = 60;
const MAX_TARGET_FRAMERATE: u32 = 240;
const TARGET_FRAMERATE_INC: u32 = 60;

#[derive(Debug)]
pub struct FramerateConfig {
    target_framerate: AtomicU32,
    is_vsync_enabled: AtomicBool,
}

impl FramerateConfig {
    pub fn load_from(&self, framerate_config: &FramerateConfig) {
        self.target_framerate.store(
            framerate_config.target_framerate.load(Ordering::SeqCst),
            Ordering::SeqCst,
        );
        self.is_vsync_enabled.store(
            framerate_config.is_vsync_enabled.load(Ordering::SeqCst),
            Ordering::SeqCst,
        );
    }
    pub const fn default() -> Self {
        FramerateConfig {
            target_framerate: AtomicU32::new(60),
            is_vsync_enabled: AtomicBool::new(true),
        }
    }
}

impl Clone for FramerateConfig {
    fn clone(&self) -> Self {
        FramerateConfig {
            target_framerate: AtomicU32::new(self.target_framerate.load(Ordering::SeqCst)),
            is_vsync_enabled: AtomicBool::new(self.is_vsync_enabled.load(Ordering::SeqCst)),
        }
    }
}

impl ToString for FramerateConfig {
    fn to_string(&self) -> String {
        let vsync_indicator = match self.is_vsync_enabled.load(Ordering::SeqCst) {
            true => "",
            false => "++",
        };
        format!(
            "{} FPS{}",
            self.target_framerate.load(Ordering::SeqCst),
            vsync_indicator
        )
    }
}

static FRAMERATE_CONFIG: FramerateConfig = FramerateConfig::default();

#[skyline::hook(offset = 0x135caf8, inline)]
unsafe fn on_game_speed_calc(_: &InlineCtx) {
    if !ldn::is_local_online() {
        return;
    }
    set_internal_framerate(3600 / FRAMERATE_CONFIG.target_framerate.load(Ordering::SeqCst));
}

#[skyline::hook(offset = 0x3747b7c, inline)]
unsafe fn scene_update(_: &InlineCtx) {
    static mut PREV_TICK: Option<skyline::nn::os::Tick> = None;
    if !ldn::is_local_online() {
        return;
    }
    let target_framerate = FRAMERATE_CONFIG.target_framerate.load(Ordering::SeqCst);
    let vsync_enabled = FRAMERATE_CONFIG.is_vsync_enabled.load(Ordering::SeqCst);
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
    let r = *((base_addr + 0x6D42430) as *const u64);
    let r = *((r + 0x10) as *const u64);
    let r = (r + 0xF14) as *mut i32;
    *r = swap_interval;
}

unsafe fn set_internal_framerate(internal_framerate: u32) {
    let base_addr = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
    let internal_frame_rate_addr = base_addr + 0x523C004;
    *(internal_frame_rate_addr as *mut u32) = internal_framerate
}

pub fn set_framerate_target(target_framerate: u32) {
    unsafe {
        FRAMERATE_CONFIG
            .target_framerate
            .store(target_framerate, Ordering::SeqCst);
        set_internal_framerate(3600 / target_framerate);
    }
}

pub fn set_vsync_enabled(vsync_enabled: bool) {
    unsafe {
        let target_framerate = FRAMERATE_CONFIG.target_framerate.load(Ordering::SeqCst);
        FRAMERATE_CONFIG
            .is_vsync_enabled
            .store(vsync_enabled, Ordering::SeqCst);
        match (vsync_enabled, target_framerate == 60) {
            (true, false) => set_swap_interval(((target_framerate as f64 / 60.0) * 100.0) as i32),
            (false, _) => set_swap_interval(10000),
            _ => set_swap_interval(1),
        }
    }
}

pub fn get_framerate_config() -> &'static FramerateConfig {
    &FRAMERATE_CONFIG
}

pub fn poll() {
    let pressed_buttons = utils::poll_buttons(&[
        ninput::Buttons::UP,
        ninput::Buttons::DOWN,
        ninput::Buttons::X,
    ]);
    let mut target_framerate = FRAMERATE_CONFIG.target_framerate.load(Ordering::SeqCst);
    let vsync_enabled = FRAMERATE_CONFIG.is_vsync_enabled.load(Ordering::SeqCst);
    match pressed_buttons {
        ninput::Buttons::UP => {
            if vsync_enabled {
                target_framerate += TARGET_FRAMERATE_INC;
            }
        }
        ninput::Buttons::DOWN => {
            if vsync_enabled {
                target_framerate -= TARGET_FRAMERATE_INC;
            }
        }
        ninput::Buttons::X => {
            if target_framerate == DEFAULT_TARGET_FRAMERATE {
                FRAMERATE_CONFIG
                    .is_vsync_enabled
                    .store(!vsync_enabled, Ordering::SeqCst);
            }
        }
        _ => (),
    }
    let new_target_framerate =
        target_framerate.clamp(DEFAULT_TARGET_FRAMERATE, MAX_TARGET_FRAMERATE);
    FRAMERATE_CONFIG
        .target_framerate
        .store(new_target_framerate, Ordering::SeqCst);
}

pub fn install() {
    skyline::install_hooks!(scene_update, on_game_speed_calc);
}
