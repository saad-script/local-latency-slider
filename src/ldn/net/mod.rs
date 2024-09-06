pub mod interface;

use interface::*;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use crate::ldn;

const SEND_PORT: u16 = 3070;
const LISTEN_PORT: u16 = 3080;

static ROOM_NET_DIAGNOSTICS: Mutex<NetworkDiagnostics> = Mutex::new(NetworkDiagnostics::new());
static PLAYER_NET_STATS: [PlayerNetInfo; 8] = [
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
    PlayerNetInfo::default(),
];

#[skyline::hook(replace = scan_network)]
unsafe fn on_network_scan(
    network_info: *mut NetworkInfo,
    num_networks_found: *mut i32,
    max_num_networks: i32,
    scan_filter: u64,
    _param_5: i32,
) {
    println!("Scanning network...");
    call_original!(
        network_info,
        num_networks_found,
        max_num_networks,
        scan_filter,
        _param_5
    );
    println!(
        "Found: {}, Max: {}, ?: {}",
        *num_networks_found, max_num_networks, _param_5
    );
}

fn poll_listener(socket: &UdpSocket, buf: &mut [u8]) -> std::io::Result<()> {
    let (packet, mut src_addr) = match socket.read(buf, true) {
        Ok(p) => p,
        Err(e) => {
            return Err(e);
        }
    };

    match packet.packet_type {
        NetworkPacketType::Ping => {
            println!("Responding to Ping packet from {}...", src_addr.to_string());
            src_addr.set_port(LISTEN_PORT);
            let res_packet = NetworkPacket::create_pong_packet(&packet);
            socket.write(&src_addr, res_packet)?;
        }
        NetworkPacketType::Pong => {
            let curr_ping = packet.get_time_elapsed().as_millis();
            ROOM_NET_DIAGNOSTICS
                .lock()
                .unwrap()
                .register_ping(curr_ping as u64);
            println!("Got Pong packet ({}): {}", src_addr.to_string(), curr_ping);

            let network_info = try_get_network_info()?;
            // Check which player sent the packet and update the ping
            let (player_index, _node) =
                match network_info
                    .node_info_array
                    .iter()
                    .enumerate()
                    .find(|(_i, n)| {
                        RawIPv4Address(n.ipv4_address).to_socket_address(LISTEN_PORT) == src_addr
                    }) {
                    Some(v) => v,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "Unable to identify sender address",
                        ))
                    }
                };

            let player_net_info = &PLAYER_NET_STATS[player_index];
            player_net_info.set_connected(true);
            player_net_info.delay.load_from(&packet.delay);
            player_net_info
                .framerate_config
                .load_from(&packet.framerate_config);
            player_net_info
                .net_diagnostics
                .lock()
                .unwrap()
                .register_ping(curr_ping as u64);
        }
    }

    Ok(())
}

fn poll_sender(addr: &SocketAddr, socket: &UdpSocket) -> std::io::Result<()> {
    let network_info = try_get_network_info()?;
    let mut sent = false;
    for (i, node) in network_info.node_info_array.iter().enumerate() {
        if node.is_connected == 0 {
            PLAYER_NET_STATS[i].set_connected(false);
            continue;
        }
        let ping_addr = RawIPv4Address(node.ipv4_address).to_socket_address(LISTEN_PORT);
        if ping_addr.ip() == addr.ip() {
            continue;
        }
        let packet = NetworkPacket::create_ping_packet();
        if let Err(e) = socket.write(&ping_addr, packet) {
            eprintln!("Error sending ping packet to {}: {}", ping_addr, e);
            continue;
        }
        sent = true;
    }
    match sent {
        true => Ok(()),
        false => {
            ROOM_NET_DIAGNOSTICS.lock().unwrap().reset();
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No network nodes found",
            ))
        }
    }
}

fn network_loop(network_role: NetworkRole, thread_type: NetworkThreadType) {
    let port = match thread_type {
        NetworkThreadType::Listener => LISTEN_PORT,
        NetworkThreadType::Sender => SEND_PORT,
    };
    let addr = get_network_address(port);
    let socket = UdpSocket::bind(addr).expect("Unable to bind to socket");
    let mut buf = [0; 1024];
    while get_network_role() == network_role {
        let poll_start_timestamp = Instant::now();
        let r = match thread_type {
            NetworkThreadType::Listener => poll_listener(&socket, &mut buf),
            NetworkThreadType::Sender => poll_sender(&addr, &socket),
        };
        if let Err(e) = r {
            eprintln!("Error in {:?} thread: {:?}", thread_type, e);
        }

        //limit the rate the sender thread sends out packets
        if thread_type == NetworkThreadType::Sender {
            let packet_interval = match ldn::is_in_game() {
                true => Duration::from_secs_f64(0.5),
                false => Duration::from_secs_f64(0.1),
            };
            if poll_start_timestamp.elapsed() < packet_interval {
                thread::sleep(packet_interval - poll_start_timestamp.elapsed());
            }
        }
    }
    for player_net_stat in PLAYER_NET_STATS.iter() {
        player_net_stat.set_connected(false);
    }
}

unsafe fn spawn_network_threads(network_role: NetworkRole) {
    let network_role_clone = network_role.clone();
    thread::spawn(move || {
        skyline::nn::os::ChangeThreadPriority(skyline::nn::os::GetCurrentThread(), 5);
        network_loop(network_role, NetworkThreadType::Listener);
        println!(
            "Listener Network loop exited, with Network State: {:?}",
            get_network_state()
        );
        skyline::nn::os::ChangeThreadPriority(skyline::nn::os::GetCurrentThread(), 24);
    });
    thread::spawn(move || {
        skyline::nn::os::ChangeThreadPriority(skyline::nn::os::GetCurrentThread(), 5);
        network_loop(network_role_clone, NetworkThreadType::Sender);
        println!(
            "Sender Network loop exited, with Network State: {:?}",
            get_network_state()
        );
        skyline::nn::os::ChangeThreadPriority(skyline::nn::os::GetCurrentThread(), 24);
    });
}

#[skyline::hook(replace = create_network)]
unsafe fn on_network_created(network_config: u64, security_config: u64, user_config: u64) {
    println!("Creating network...");
    call_original!(network_config, security_config, user_config);
    spawn_network_threads(NetworkRole::Host);
}

#[skyline::hook(replace = connect_network)]
unsafe fn on_network_connected(
    network_info: *mut NetworkInfo,
    security_config: u64,
    user_config: u64,
    local_comm_ver: i32,
    connect_option: u32,
) {
    println!("Connecting to network...");
    call_original!(
        network_info,
        security_config,
        user_config,
        local_comm_ver,
        connect_option
    );
    spawn_network_threads(NetworkRole::Client);
}

#[skyline::hook(replace = disconnect_network)]
unsafe fn on_network_disconnected() {
    call_original!();
    println!("Network Disconnected");
}

#[skyline::hook(replace = destroy_network)]
unsafe fn on_network_destroyed() {
    call_original!();
    println!("Network Destroyed");
}

pub fn get_player_net_info<'a>(player_index: usize) -> &'static PlayerNetInfo {
    &PLAYER_NET_STATS[player_index]
}

pub fn get_room_net_diag<'a>() -> std::sync::MutexGuard<'a, NetworkDiagnostics> {
    ROOM_NET_DIAGNOSTICS.lock().unwrap()
}

pub(super) fn install() {
    skyline::install_hooks!(
        on_network_scan,
        on_network_created,
        on_network_connected,
        on_network_disconnected,
        on_network_destroyed
    );
}
