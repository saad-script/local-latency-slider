mod framerate;
mod ldn;
mod utils;

use skyline::nn::ui2d::Pane;

use utils::{PaneExt, TextBoxExt};

#[skyline::hook(offset = 0x1a12f40)]
unsafe fn update_css(arg: u64) {
    if ldn::is_local_online() {
        ldn::latency_slider::poll();
        framerate::poll();
        let delay_str = ldn::latency_slider::current_input_delay().to_string();
        let framerate_config = framerate::get_framerate_config();
        let room_net_diag = ldn::net::get_room_net_diag();
        let ping_str = match room_net_diag.get_avg_ping() {
            Some(ping) => format!(" {}ms", ping),
            None => String::from(""),
        };
        let banner_display_str = format!(
            "Buffer: {} [{}]{}\0",
            delay_str,
            framerate_config.to_string(),
            ping_str
        );
        let (r, g, b, a) = match room_net_diag.get_network_stability() {
            ldn::net::interface::NetworkStability::Stable => (0, 255, 0, 255),
            ldn::net::interface::NetworkStability::Inconsistent => (255, 255, 0, 255),
            ldn::net::interface::NetworkStability::Unstable => (255, 0, 0, 255),
        };
        drop(room_net_diag);

        // pointer to p1's title text pane
        let p1_pane = (*((*((arg + 0xe58) as *const u64) + 0x10) as *const u64)) as *mut Pane;
        let p1_pane = &mut *p1_pane;
        p1_pane.as_textbox().set_default_material_colors();
        p1_pane.as_textbox().set_color(r, g, b, a);
        p1_pane.as_textbox().set_text_string(&banner_display_str);

        let p1_pane_bg = p1_pane.parent().unwrap().traverse_backward(2).unwrap();
        p1_pane_bg.set_visible(false);

        let p2_pane = p1_pane_bg
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .prev()
            .unwrap();
        p2_pane.as_textbox().set_default_material_colors();
        p2_pane.as_textbox().set_color(r, g, b, a);
        p2_pane.as_textbox().set_text_string(&banner_display_str);

        let panel_root = p2_pane.parent().unwrap().traverse_forward(4).unwrap();

        for player_index in 0..8 {
            let player_panel_root = panel_root
                .get_child(&format!("set_panel_{}p", player_index + 1), false)
                .unwrap();

            let player_panel = player_panel_root.children().unwrap();

            let player_panel_name = player_panel
                .get_child(&format!("set_btn_panel"), false)
                .unwrap();
            let player_panel_name = player_panel_name.children().unwrap().next().unwrap();
            let player_panel_name = player_panel_name
                .get_child(&format!("set_txt_00"), true)
                .unwrap();

            let player_net_info = ldn::net::get_player_net_info(player_index);
            match player_net_info.is_connected() {
                true => {
                    let is_stick_pressed = ninput::any::is_press(ninput::Buttons::STICK_L)
                        || ninput::any::is_press(ninput::Buttons::STICK_R);
                    let is_triggers_pressed = ninput::any::is_press(ninput::Buttons::L)
                        && ninput::any::is_press(ninput::Buttons::R);
                    if is_stick_pressed || is_triggers_pressed {
                        let avg_ping = match player_net_info
                            .net_diagnostics
                            .lock()
                            .unwrap()
                            .get_avg_ping()
                        {
                            Some(p) => format!("{}ms", p),
                            None => String::from("???"),
                        };
                        player_panel_name
                            .as_textbox()
                            .set_text_string(&format!("{}", avg_ping));
                    } else {
                        player_panel_name.as_textbox().set_text_string(&format!(
                            "{}, {}",
                            player_net_info.framerate_config.to_string(),
                            player_net_info.delay.to_string()
                        ));
                    }
                }
                false => {
                    player_panel_name
                        .as_textbox()
                        .set_text_string(&format!("P{}", player_index + 1));
                }
            }
        }
    }
    call_original!(arg);
}

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
