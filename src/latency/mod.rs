use skyline::hooks::InlineCtx;

use crate::framerate;
use crate::utils;

const MAX_INPUT_BUFFER: u8 = 25;

#[derive(Debug)]
pub struct Delay {
    buffer: Buffer,
    last_auto: Option<u8>,
}

#[derive(Debug)]
pub enum Buffer {
    Auto,
    Override(u8),
}

impl Delay {
    fn next(&mut self) {
        self.buffer = match &self.buffer {
            Buffer::Auto => Buffer::Override(0),
            Buffer::Override(v) => Buffer::Override((*v + 1).min(MAX_INPUT_BUFFER)),
        }
    }
    fn prev(&mut self) {
        self.buffer = match &self.buffer {
            Buffer::Auto => Buffer::Auto,
            Buffer::Override(v) => match v {
                0 => Buffer::Auto,
                _ => Buffer::Override(*v - 1),
            },
        }
    }
    pub fn to_string(&self) -> String {
        match self.buffer {
            Buffer::Auto => match self.last_auto {
                None => "Auto".to_string(),
                Some(v) => format!("Auto ({}f)", v).to_string(),
            },
            Buffer::Override(v) => format!("{}f", v).to_string(),
        }
    }
}

static mut CURRENT_INPUT_DELAY: Delay = Delay {
    buffer: Buffer::Override(4),
    last_auto: None,
};
static mut IS_LOCAL_ONLINE: bool = false;
static mut LOCAL_ROOM_PANE_HANDLE: Option<u64> = None;

#[skyline::hook(offset = 0x16ccc58, inline)]
unsafe fn set_online_latency(ctx: &InlineCtx) {
    if IS_LOCAL_ONLINE {
        let auto = *(*ctx.registers[19].x.as_ref() as *mut u8);
        CURRENT_INPUT_DELAY.last_auto = Some(auto);
        if let Buffer::Override(v) = CURRENT_INPUT_DELAY.buffer {
            *(*ctx.registers[19].x.as_ref() as *mut u8) = v;
        }
    }
}

#[skyline::hook(offset = 0x22d9cf0, inline)]
unsafe fn online_melee_any_scene_create(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

#[skyline::hook(offset = 0x22d9c20, inline)]
unsafe fn bg_matchmaking_seq(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

#[skyline::hook(offset = 0x235a630, inline)]
unsafe fn main_menu(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
    framerate::set_framerate_target(60);
    framerate::set_vsync_enabled(true);
}

// called on local online menu init
#[skyline::hook(offset = 0x1bd45c0, inline)]
unsafe fn store_local_menu_pane(ctx: &InlineCtx) {
    IS_LOCAL_ONLINE = true;
    let handle = *((*((*ctx.registers[0].x.as_ref() + 8) as *const u64) + 0x10) as *const u64);
    LOCAL_ROOM_PANE_HANDLE = Some(handle);
}

#[skyline::hook(offset = 0x1bd7a60, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    if let Some(v) = LOCAL_ROOM_PANE_HANDLE {
        poll();
        let delay_str = CURRENT_INPUT_DELAY.to_string();
        utils::set_text_string(v, format!("{}\0", delay_str).as_ptr());
    }
}

pub unsafe fn is_local_online() -> bool {
    return IS_LOCAL_ONLINE;
}

pub unsafe fn current_input_delay() -> &'static Delay {
    return &CURRENT_INPUT_DELAY;
}

pub unsafe fn poll() {
    let pressed_buttons = utils::poll_buttons(&[ninput::Buttons::LEFT, ninput::Buttons::RIGHT]);
    match pressed_buttons {
        ninput::Buttons::LEFT => CURRENT_INPUT_DELAY.prev(),
        ninput::Buttons::RIGHT => CURRENT_INPUT_DELAY.next(),
        _ => (),
    }
}

pub fn install() {
    skyline::install_hooks!(
        main_menu,
        online_melee_any_scene_create,
        bg_matchmaking_seq,
        store_local_menu_pane,
        set_online_latency,
        update_local_menu,
    );
}
