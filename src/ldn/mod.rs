pub mod latency_slider;
pub mod net;

use crate::framerate;
use crate::utils;
use skyline::hooks::InlineCtx;
use skyline::nn::ui2d::Pane;

static mut LOCAL_ROOM_PANE_HANDLE: Option<*mut Pane> = None;

#[skyline::hook(offset = 0x22d9cf0, inline)]
unsafe fn online_melee_any_scene_create(_: &InlineCtx) {
    LOCAL_ROOM_PANE_HANDLE = None;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

#[skyline::hook(offset = 0x22d9c20, inline)]
unsafe fn bg_matchmaking_seq(_: &InlineCtx) {
    LOCAL_ROOM_PANE_HANDLE = None;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

#[skyline::hook(offset = 0x235a630, inline)]
unsafe fn main_menu(_: &InlineCtx) {
    LOCAL_ROOM_PANE_HANDLE = None;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

// called on local online menu init
#[skyline::hook(offset = 0x1bd45c0, inline)]
unsafe fn store_local_menu_pane(ctx: &InlineCtx) {
    let handle = *((*((*ctx.registers[0].x.as_ref() + 8) as *const u64) + 0x10) as *const u64);
    LOCAL_ROOM_PANE_HANDLE = Some(handle as *mut Pane);
}

#[skyline::hook(offset = 0x1bd7a60, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    if let Some(v) = LOCAL_ROOM_PANE_HANDLE {
        latency_slider::poll();
        let delay_str = latency_slider::current_input_delay().to_string();
        utils::set_text_string(v, format!("{}\0", delay_str).as_ptr());
    }
}

pub fn is_local_online() -> bool {
    unsafe {
        return LOCAL_ROOM_PANE_HANDLE.is_some();
    }
}

pub fn install() {
    skyline::install_hooks!(
        online_melee_any_scene_create,
        bg_matchmaking_seq,
        main_menu,
        store_local_menu_pane,
        update_local_menu
    );
    latency_slider::install();
    net::install();
}
