use skyline::hooks::InlineCtx;

static mut CURRENT_INPUT_BUFFER: isize = 4;
static mut MOST_RECENT_AUTO: isize = -1;
static mut IS_LOCAL_ONLINE: bool = false;

const MAX_INPUT_BUFFER: isize = 25;
const MIN_INPUT_BUFFER: isize = -1;

#[skyline::from_offset(0x37a1270)]
unsafe fn set_text_string(pane: u64, string: *const u8);

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

unsafe fn update_latency_display(pane_handle: u64) {
    if CURRENT_INPUT_BUFFER == -1 {
        if MOST_RECENT_AUTO == -1 {
            set_text_string(pane_handle, format!("Auto\0").as_ptr());
        } else {
            set_text_string(
                pane_handle,
                format!("Auto({}f)\0", MOST_RECENT_AUTO).as_ptr(),
            )
        }
    } else {
        set_text_string(pane_handle, format!("{}f\0", CURRENT_INPUT_BUFFER).as_ptr());
    }
}

#[skyline::hook(offset = 0x37a0c40, inline)]
unsafe fn update_pane(ctx: &InlineCtx) {
    let pane_ptr = *ctx.registers[0].x.as_ref() as *const u64;
    let buffer = *ctx.registers[1].x.as_ref() as *mut u16; // &u64 -> *mut u16
    let length = *ctx.registers[2].w.as_ref() as u32;

    let raw = std::slice::from_raw_parts(buffer, length as usize);

    if raw.iter().all(|&n| n > 31 && n < 123) {
        let str: String = raw.iter().copied().map(|c| c as u8 as char).collect();
        println!("PANE {:p} CONTAINS {} {:?}", pane_ptr, str, raw);
    }

    let solo_battle = b"Solo Battle";
    if raw.len() == solo_battle.len()
        && raw
            .iter()
            .zip(solo_battle)
            .all(|(wide, short)| *wide == *short as u16)
    {
        println!("{:p}->{}", pane_ptr, *pane_ptr);
    }
}

// This only updates the host's (p1) latency display. Can't find a reference to p2's pane handle :(
// #[skyline::hook(offset = 0x1a12460)]
// unsafe fn update_chara_select_screen(arg: u64) {
//     println!("IS IN CHAR SELECT SCREEN");
//     if IS_LOCAL_ONLINE {
//         poll_input_update_delay();
//         let pane_handle = *((*((arg + 0xe58) as *const u64) + 0x10) as *const u64);
//         update_latency_display(pane_handle);
//         let pane_handle2 = *((*((arg + 0xe68) as *const u64) + 0x10) as *const u64);
//         update_latency_display(pane_handle2);
//     }
//     call_original!(arg);
// }

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
    LOCAL_ROOM_PANE_HANDLE =
        *((*((*ctx.registers[0].x.as_ref() + 8) as *const u64) + 0x10) as *const u64);
}

#[skyline::hook(offset = 0x1bd6f40, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    if LOCAL_ROOM_PANE_HANDLE == 0 {
        return;
    }
    poll_input_update_delay();
    update_latency_display(LOCAL_ROOM_PANE_HANDLE);
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
        update_pane,
    );
}
