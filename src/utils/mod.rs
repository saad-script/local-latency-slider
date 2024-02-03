use std::time::Instant;

#[skyline::from_offset(0x37a1270)]
pub fn set_text_string(pane: u64, string: *const u8);

pub fn poll_buttons(buttons: Vec<ninput::Buttons>) -> ninput::Buttons {
    static mut PRESS_COOLDOWN: Option<Instant> = None;
    for button in buttons {
        let is_pressed = ninput::any::is_press(button);
        unsafe {
            match (PRESS_COOLDOWN, is_pressed) {
                (Some(t), _) => {
                    if t.elapsed().as_millis() > 167 {
                        PRESS_COOLDOWN = None;
                    }
                }
                (None, true) => {
                    PRESS_COOLDOWN = Some(Instant::now());
                    return button;
                }
                _ => PRESS_COOLDOWN = None,
            }
        }
    }
    return ninput::Buttons::empty();
}
