use skyline::hooks::InlineCtx;
use skyline::nn::ui2d::{Pane, Layout};

static mut CURRENT_INPUT_BUFFER: isize = 4;
static mut MOST_RECENT_AUTO: isize = -1;
static mut IS_LOCAL_ONLINE: bool = false;

const MAX_INPUT_BUFFER: isize = 25;
const MIN_INPUT_BUFFER: isize = -1;

#[skyline::from_offset(0x37a1270)]
unsafe fn set_text_string(pane: u64, string: *const u8);

#[skyline::from_offset(0x59970)]
pub unsafe fn find_pane_by_name_recursive(
    pane: *const Pane,
    s: *const u8,
) -> *mut Pane;

unsafe fn poll_input_update_delay() {
    static mut CURRENT_COUNTER: usize = 0;
    if ninput::any::is_press(ninput::Buttons::RIGHT) {
        if CURRENT_COUNTER == 0 {
            CURRENT_INPUT_BUFFER += 1;
        }
        CURRENT_COUNTER = (CURRENT_COUNTER + 1) % 10;
    } else if ninput::any::is_press(ninput::Buttons::LEFT) {
        if CURRENT_COUNTER == 0 {
            CURRENT_INPUT_BUFFER -= 1;
        }
        CURRENT_COUNTER = (CURRENT_COUNTER + 1) % 10;
    } else {
        CURRENT_COUNTER = 0;
    }
    CURRENT_INPUT_BUFFER = CURRENT_INPUT_BUFFER.clamp(MIN_INPUT_BUFFER, MAX_INPUT_BUFFER);
}

unsafe fn update_latency_display(header: &str, pane_handle: u64) {
    if CURRENT_INPUT_BUFFER == -1 {
        if MOST_RECENT_AUTO == -1 {
            set_text_string(
                pane_handle,
                format!("{}Auto\0", header).as_ptr(),
            );
        } else {
            set_text_string(
                pane_handle,
                format!("{}Auto ({}f)\0", header, MOST_RECENT_AUTO).as_ptr()
            )
        }
    } else {
        set_text_string(
            pane_handle, 
            format!("{}{}f\0",header, CURRENT_INPUT_BUFFER).as_ptr()
        );
    }
}

#[skyline::hook(offset = 0x16cdb08, inline)]
unsafe fn set_online_latency(ctx: &InlineCtx) {
    let auto = *(*ctx.registers[19].x.as_ref() as *mut u8);
    if IS_LOCAL_ONLINE {
        MOST_RECENT_AUTO = auto as isize;
        if CURRENT_INPUT_BUFFER != -1 {
            *(*ctx.registers[19].x.as_ref() as *mut u8) = CURRENT_INPUT_BUFFER as u8;
        }
    }
}

#[skyline::hook(offset = 0x22d91f4, inline)]
unsafe fn online_melee_any_scene_create(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
}

#[skyline::hook(offset = 0x22d9124, inline)]
unsafe fn bg_matchmaking_seq(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
}

#[skyline::hook(offset = 0x23599b0, inline)]
unsafe fn main_menu(_: &InlineCtx) {
    IS_LOCAL_ONLINE = false;
}

// called on local online menu init
static mut LOCAL_ROOM_PANE_HANDLE: u64 = 0;
#[skyline::hook(offset = 0x1bd3ae0, inline)]
unsafe fn store_local_menu_pane(ctx: &InlineCtx) {
    IS_LOCAL_ONLINE = true;
    LOCAL_ROOM_PANE_HANDLE = *((*((*ctx.registers[0].x.as_ref() + 8) as *const u64) + 0x10) as *const u64);
}

#[skyline::hook(offset = 0x1bd6f40, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    if LOCAL_ROOM_PANE_HANDLE == 0 {
        return;
    }
    poll_input_update_delay();
    update_latency_display("", LOCAL_ROOM_PANE_HANDLE);
}

// only works for host (p1) for some reason.
static mut IS_IN_CSS: bool = false;
#[skyline::hook(offset = 0x1a12460)]
unsafe fn update_css(arg: u64) {
    if IS_LOCAL_ONLINE {
        let pane = *((*((arg + 0xe58) as *const u64) + 0x10) as *const u64);
        poll_input_update_delay();
        update_latency_display("Input Delay: ", pane);
    }
    IS_IN_CSS = true;
    call_original!(arg);
}

// current workaround for css delay display not showing on client (p2).
#[skyline::hook(offset = 0x4b640, inline)]
unsafe fn on_draw_ui2d(ctx: &InlineCtx) {

    if !IS_LOCAL_ONLINE || !IS_IN_CSS {
        return;
    }

    let layout = *ctx.registers[0].x.as_ref() as *mut Layout;
    let layout_name = skyline::from_c_str((*layout).layout_name);
    let root_pane = (*layout).root_pane;

    if layout_name == "chara_select_base" {
        let result = find_pane_by_name_recursive(root_pane, "set_txt_title_00\0".as_ptr());
        if result != std::ptr::null_mut() {
            let tb = result as u64;
            update_latency_display("Input Delay: ", tb);
        }
    }

    IS_IN_CSS = layout_name == "chara_select_base" || 
                layout_name == "chara_select" || 
                layout_name == "select_bg" ||
                layout_name == "tournament_bg";
}

#[skyline::main(name = "local-latency-slider")]
pub unsafe fn main() {
    skyline::install_hooks!(
        main_menu,
        online_melee_any_scene_create,
        bg_matchmaking_seq,
        store_local_menu_pane,
        update_local_menu,
        set_online_latency,
        update_css,
        on_draw_ui2d,
    );
}