use skyline::nn::ui2d::{Pane, TextBox};
use std::time::Instant;

extern "C" {
    #[link_name = "\u{1}_ZN2nn2os22GetSystemTickFrequencyEv"]
    pub fn get_tick_freq() -> u64;
}

extern "C" {
    #[link_name = "\u{1}_ZN3app14sv_information11is_ready_goEv"]
    pub fn is_ready_go() -> bool;
}

#[skyline::from_offset(0x37a1ef0)]
fn set_text_string(pane: *mut Pane, string: *const u8);

#[skyline::from_offset(0x59970)]
fn find_pane_by_name_recursive(pane: *const Pane, s: *const u8) -> *mut Pane;

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


pub trait PaneExt {
    fn next(&self) -> Option<&mut Pane>;
    fn prev(&self) -> Option<&mut Pane>;
    fn parent(&self) -> Option<&mut Pane>;
    fn children(&self) -> Option<&mut Pane>;
    fn traverse_forward(&self, steps: usize) -> Option<&mut Pane>;
    fn traverse_backward(&self, steps: usize) -> Option<&mut Pane>;
    fn get_child(&self, name: &str, recursive: bool) -> Option<&mut Pane>;
}

impl PaneExt for Pane {
    fn next(&self) -> Option<&mut Pane> {
        unsafe {
            let node = self.link.next;
            let pane = ((node as *mut u64).sub(1)) as *mut Pane;
            match pane.is_null() || ((*pane).children_list.next.is_null() && (*pane).children_list.prev.is_null()) {
                true => None,
                false => Some(&mut *pane),
            }
        }
    }

    fn prev(&self) -> Option<&mut Pane> {
        unsafe {
            let node = self.link.prev;
            let pane = ((node as *mut u64).sub(1)) as *mut Pane;
            match pane.is_null() || ((*pane).children_list.next.is_null() && (*pane).children_list.prev.is_null()) {
                true => None,
                false => Some(&mut *pane),
            }
        }
    }

    fn parent(&self) -> Option<&mut Pane> {
        unsafe {
            let p = self.parent;
            match p.is_null() {
                true => None,
                false => Some(&mut *p),
            }
        }
    }

    fn children(&self) -> Option<&mut Pane> {
        unsafe {
            let node = self.children_list.next;
            let pane = ((node as *mut u64).sub(1)) as *mut Pane;
            match pane.is_null() || ((*pane).children_list.next.is_null() && (*pane).children_list.prev.is_null()) {
                true => None,
                false => Some(&mut *pane),
            }
        }
    }

    fn traverse_forward(&self, steps: usize) -> Option<&mut Pane> {
        let mut i = 0;
        let mut current = self.next();
        while let Some(p) = current {
            i += 1;
            if i == steps {
                return Some(p);
            }
            current = p.next();
        }
        return None;
    }

    fn traverse_backward(&self, steps: usize) -> Option<&mut Pane> {
        let mut i = 0;
        let mut current = self.prev();
        while let Some(p) = current {
            i += 1;
            if i == steps {
                return Some(p);
            }
            current = p.prev();
        }
        return None;
    }

    fn get_child(&self, name: &str, recursive: bool) -> Option<&mut Pane> {
        if recursive {
            let child = unsafe { find_pane_by_name_recursive(self as *const Pane, format!("{}\0", name).as_ptr()) };
            match child.is_null() {
                true => return None,
                false => return unsafe { Some(&mut *child) },
            }
        }

        let mut current = self.children();
        while let Some(p) = current {
            if p.get_name() == name {
                return Some(p);
            }
            current = p.next();
        }
        return None;
    }

}

pub trait TextBoxExt {
    fn set_text_string(&mut self, text: &str);
}

impl TextBoxExt for TextBox {
    fn set_text_string(&mut self, text: &str) {
        unsafe {
            set_text_string(self as *mut TextBox as *mut Pane, format!("{}\0", text).as_ptr());
        }
    }
}
