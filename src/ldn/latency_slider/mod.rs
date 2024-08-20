use skyline::hooks::InlineCtx;

use crate::ldn;
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

#[skyline::hook(offset = 0x16ccc58, inline)]
unsafe fn set_online_latency(ctx: &InlineCtx) {
    if ldn::is_local_online() {
        let auto = *(*ctx.registers[19].x.as_ref() as *mut u8);
        CURRENT_INPUT_DELAY.last_auto = Some(auto);
        if let Buffer::Override(v) = CURRENT_INPUT_DELAY.buffer {
            *(*ctx.registers[19].x.as_ref() as *mut u8) = v;
        }
    }
}

pub fn current_input_delay() -> &'static Delay {
    unsafe {
        return &CURRENT_INPUT_DELAY;
    }
}

pub fn poll() {
    let pressed_buttons = utils::poll_buttons(&[ninput::Buttons::LEFT, ninput::Buttons::RIGHT]);
    unsafe {
        match pressed_buttons {
            ninput::Buttons::LEFT => CURRENT_INPUT_DELAY.prev(),
            ninput::Buttons::RIGHT => CURRENT_INPUT_DELAY.next(),
            _ => (),
        }
    }
}

pub(super) fn install() {
    skyline::install_hooks!(set_online_latency,);
}
