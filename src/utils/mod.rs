use skyline::nn::ui2d::Pane;
use std::time::Instant;

extern "C" {
    #[link_name = "\u{1}_ZN2nn2os22GetSystemTickFrequencyEv"]
    pub fn get_tick_freq() -> u64;
}

#[skyline::from_offset(0x37a1ef0)]
pub fn set_text_string(pane: *mut Pane, string: *const u8);

pub fn is_yuzu_emulator() -> bool {
    unsafe {
        let base_address = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64;
        return base_address == 0x80004000;
    }
}

pub fn poll_buttons(buttons: &[ninput::Buttons]) -> ninput::Buttons {
    static mut PRESS_COOLDOWN: Option<Instant> = None;
    for button in buttons {
        let is_pressed = ninput::any::is_press(*button);
        unsafe {
            match (PRESS_COOLDOWN, is_pressed) {
                (Some(t), _) => {
                    if t.elapsed().as_millis() > 167 {
                        PRESS_COOLDOWN = None;
                    }
                }
                (None, true) => {
                    PRESS_COOLDOWN = Some(Instant::now());
                    return *button;
                }
                _ => PRESS_COOLDOWN = None,
            }
        }
    }
    return ninput::Buttons::empty();
}
