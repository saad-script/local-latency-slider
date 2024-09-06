pub mod latency_slider;
pub mod net;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::framerate;
use crate::ldn::net::interface::{get_network_role, NetworkRole};
use crate::utils::TextBoxExt;
use skyline::hooks::InlineCtx;
use skyline::nn::ui2d::Pane;

static mut LOCAL_ROOM_PANE_HANDLE: Option<*mut Pane> = None;
static mut CUSTOM_CSS_NUM_PLAYERS_FLAG: bool = false;

// workaround for sv_information::is_ready_go() being unreliable for ldn in some cases
static IN_GAME: AtomicBool = AtomicBool::new(false);

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
    update_in_game_flag(false);
    CUSTOM_CSS_NUM_PLAYERS_FLAG = true;
    let handle = *((*((*ctx.registers[0].x.as_ref() + 8) as *const u64) + 0x10) as *const u64);
    LOCAL_ROOM_PANE_HANDLE = Some(handle as *mut Pane);
}

#[skyline::hook(offset = 0x1bd7a60, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    if let Some(p) = LOCAL_ROOM_PANE_HANDLE {
        latency_slider::poll();
        let delay_str = latency_slider::current_input_delay().to_string();
        (*p).as_textbox().set_text_string(&format!("{}", delay_str));
    }
}

#[skyline::hook(offset = 0x1a261e0)]
unsafe fn css_player_pane_num_changed(param_1: i64, prev_num: i32, changed_by_player: u32) {
    if is_local_online() 
        && CUSTOM_CSS_NUM_PLAYERS_FLAG
        && changed_by_player == 0 
        && get_network_role() == NetworkRole::Host {
        CUSTOM_CSS_NUM_PLAYERS_FLAG = false;
        *((param_1 + 0x160) as *mut i32) = 2;
    }
    call_original!(param_1, prev_num, changed_by_player);
}

#[skyline::hook(offset = 0x1345558, inline)]
unsafe fn on_match_start(_: &InlineCtx) {
    if !is_local_online() {
        return;
    }
    update_in_game_flag(true);
}

#[skyline::hook(offset = 0x1d68b74, inline)]
unsafe fn on_match_end(_: &InlineCtx) {
    if !is_local_online() {
        return;
    }
    update_in_game_flag(false);
}

fn update_in_game_flag(new_in_game_flag: bool) {
    let _ = IN_GAME.compare_exchange(!new_in_game_flag, new_in_game_flag, Ordering::SeqCst, Ordering::SeqCst);
}

pub fn is_local_online() -> bool {
    unsafe {
        return LOCAL_ROOM_PANE_HANDLE.is_some();
    }
}

pub fn is_in_game() -> bool {
    IN_GAME.load(Ordering::SeqCst)
}

pub fn install() {
    skyline::install_hooks!(
        online_melee_any_scene_create,
        bg_matchmaking_seq,
        main_menu,
        store_local_menu_pane,
        update_local_menu,
        css_player_pane_num_changed,
        on_match_start,
        on_match_end,
    );
    latency_slider::install();
    net::install();
}
