use skyline::nn::ui2d::Pane;

mod framerate;
mod latency;
mod utils;

#[skyline::hook(offset = 0x1a12460)]
unsafe fn update_css(arg: u64) {
    if latency::is_local_online() {
        // pointer to p1's text pane
        let p1_pane = (*((*((arg + 0xe58) as *const u64) + 0x10) as *const u64)) as *mut Pane;

        // going up the layout.arc hierarchy to get p2's text pane
        let p2_pane_node = (*(*(*(*p1_pane).parent).parent).parent).link.prev;
        let p2_pane = ((p2_pane_node as *mut u64).sub(1)) as *mut Pane;

        latency::poll();
        framerate::poll();
        let delay_str = latency::current_input_delay().to_string();
        let framerate = framerate::framerate_target();
        let vsync_str = match framerate::vsync_enabled() {
            false => String::from("++"),
            true => String::from(""),
        };
        utils::set_text_string(
            p1_pane as u64,
            format!("Buffer: {} [{} FPS{}]\0", delay_str, framerate, vsync_str).as_ptr(),
        );
        utils::set_text_string(
            p2_pane as u64,
            format!("Buffer: {} [{} FPS{}]\0", delay_str, framerate, vsync_str).as_ptr(),
        );
    }
    call_original!(arg);
}

#[skyline::main(name = "local-latency-slider")]
pub fn main() {
    framerate::install();
    latency::install();

    skyline::install_hooks!(update_css,);
}
