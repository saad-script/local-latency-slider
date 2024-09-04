use std::sync::atomic::{AtomicI8, Ordering};

use skyline::hooks::InlineCtx;

use crate::ldn;
use crate::utils;

const MAX_INPUT_BUFFER: u8 = 25;

#[derive(Debug)]
pub struct Delay {
    buffer: AtomicI8,
    last_auto: AtomicI8,
}

impl Delay {
    fn next(&self) {
        let prev_delay = self.buffer.load(Ordering::SeqCst);
        self.buffer.store(
            (prev_delay + 1).min(MAX_INPUT_BUFFER as i8),
            Ordering::SeqCst,
        );
    }
    fn prev(&self) {
        let prev_delay = self.buffer.load(Ordering::SeqCst);
        self.buffer
            .store((prev_delay - 1).max(-1), Ordering::SeqCst);
    }
    pub fn load_from(&self, delay: &Delay) {
        self.buffer.store(delay.buffer.load(Ordering::SeqCst), Ordering::SeqCst);
        self.last_auto.store(delay.last_auto.load(Ordering::SeqCst), Ordering::SeqCst);
    }
    pub const fn default() -> Self {
        Delay { 
            buffer: AtomicI8::new(4), 
            last_auto: AtomicI8::new(-1),
        }
    }
}

impl Clone for Delay {
    fn clone(&self) -> Self {
        Delay {
            buffer: AtomicI8::new(self.buffer.load(Ordering::SeqCst)),
            last_auto: AtomicI8::new(self.last_auto.load(Ordering::SeqCst)),
        }
    }
}

impl ToString for Delay {
    fn to_string(&self) -> String {
        let buffer = self.buffer.load(Ordering::SeqCst);
        let last_auto = self.last_auto.load(Ordering::SeqCst);
        match (buffer >= 0, last_auto >= 0) {
            (false, false) => String::from("Auto"),
            (false, true) => format!("Auto ({}f)", last_auto).to_string(),
            (true, _) => format!("{}f", buffer).to_string(),
        }
    }
}

static CURRENT_INPUT_DELAY: Delay = Delay::default();

#[skyline::hook(offset = 0x16ccc58, inline)]
unsafe fn set_online_latency(ctx: &InlineCtx) {
    if ldn::is_local_online() {
        let auto = *(*ctx.registers[19].x.as_ref() as *mut u8);
        let buffer = CURRENT_INPUT_DELAY.buffer.load(Ordering::SeqCst);
        CURRENT_INPUT_DELAY
            .last_auto
            .store(auto as i8, Ordering::SeqCst);
        if buffer >= 0 {
            *(*ctx.registers[19].x.as_ref() as *mut u8) = buffer as u8;
        }
    }
}

pub fn current_input_delay() -> &'static Delay {
    &CURRENT_INPUT_DELAY
}

pub fn poll() {
    let pressed_buttons = utils::poll_buttons(&[ninput::Buttons::LEFT, ninput::Buttons::RIGHT]);
    match pressed_buttons {
        ninput::Buttons::LEFT => CURRENT_INPUT_DELAY.prev(),
        ninput::Buttons::RIGHT => CURRENT_INPUT_DELAY.next(),
        _ => (),
    }
}

pub(super) fn install() {
    skyline::install_hook!(set_online_latency);
}
