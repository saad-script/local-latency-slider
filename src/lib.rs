mod framerate;
mod ldn;
mod utils;

use skyline::nn::ui2d::Pane;

#[skyline::hook(offset = 0x1a12f40)]
unsafe fn update_css(arg: u64) {
    if ldn::is_local_online() {
        // pointer to p1's text pane
        let p1_pane = (*((*((arg + 0xe58) as *const u64) + 0x10) as *const u64)) as *mut Pane;

        // going up the layout.arc hierarchy to get p2's text pane
        let p2_pane_node = (*(*(*(*p1_pane).parent).parent).parent).link.prev;
        let p2_pane = ((p2_pane_node as *mut u64).sub(1)) as *mut Pane;

        ldn::latency_slider::poll();
        framerate::poll();
        let delay_str = ldn::latency_slider::current_input_delay().to_string();
        let framerate = framerate::framerate_target();
        let vsync_str = match framerate::vsync_enabled() {
            false => String::from("++"),
            true => String::from(""),
        };
        let ping_str = match ldn::net::get_ping() {
            Some(ping) => format!(" {}ms", ping),
            None => String::from(""),
        };
        utils::set_text_string(
            p1_pane,
            format!(
                "Buffer: {} [{} FPS{}]{}\0",
                delay_str, framerate, vsync_str, ping_str
            )
            .as_ptr(),
        );
        utils::set_text_string(
            p2_pane,
            format!(
                "Buffer: {} [{} FPS{}]{}\0",
                delay_str, framerate, vsync_str, ping_str
            )
            .as_ptr(),
        );
    }
    call_original!(arg);
}

// #[skyline::hook(offset = 0x4b640, inline)]
// unsafe fn on_draw_ui2d(ctx: &skyline::hooks::InlineCtx) {
//     let layout = *ctx.registers[0].x.as_ref() as *mut skyline::nn::ui2d::Layout;
//     let layout_name = skyline::from_c_str((*layout).layout_name);
//     let root_pane = (*layout).root_pane;

//     if layout_name == "local_top" {
//         let result = find_pane_by_name_recursive(root_pane, "set_txt_title_00\0".as_ptr());
//         if result != std::ptr::null_mut() {
//             let tb = result as u64;
//             update_latency_display("Input Delay: ", tb);
//         }
//     }
// }

#[skyline::main(name = "local-latency-slider")]
pub fn main() {
    if !utils::is_yuzu_emulator() {
        skyline::error::show_error(
            1,
            "Compatibility Error",
            "Local Latency Slider mod is currently only supported on yuzu emulator",
        );
        return;
    }

    framerate::install();
    ldn::install();
    skyline::install_hook!(update_css);
}
