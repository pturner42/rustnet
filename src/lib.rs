extern crate sdl2_net;

static MAX_BUFFER_SIZE: u16 = 512;

pub struct NetworkData{
    write_buffer: [u8; 512],
    buffer_index: u32,
    read_buffer: [u8; 512],
    read_buffer_size: u32,

    c_buffer: Vec<u8>,

    socket_set: sdl2_net::SocketSet,
    socket: sdl2_net::TCPsocket,

    is_server: bool,
}

pub fn rnet_read_socket<F: Fn(u8, u32) -> bool, J: Fn(&NetworkData) -> u32>(net: &mut NetworkData, socket: &sdl2_net::TCPsocket, can_handle: F, func: J) -> bool {
    
    if sdl2_net::socket_ready(socket) {
        let rec_data = sdl2_net::tcp_recv(socket, net.c_buffer.as_mut_ptr(), MAX_BUFFER_SIZE as i32);
        if rec_data > 0 {
            for i in 0..rec_data {
                net.read_buffer[(net.read_buffer_size as i32 + i) as usize] = net.c_buffer[i as usize]
            }
            net.read_buffer_size += rec_data as u32;
            while net.read_buffer_size > 0 {
                if can_handle(peek_byte(net), net.read_buffer_size){
                    func(&net);
                } else { break; }
            }
            return true
        } else {
            remove_socket(&net, &(net.socket));
            sdl2_net::tcp_close(&(net.socket));
            return false
        }
    }

    true
}

pub fn peek_byte(net_data: &NetworkData) -> u8 {
    net_data.read_buffer[0]
}

pub fn read_byte(net_data: &mut NetworkData) -> u8 {
    let b = net_data.read_buffer[0];
    shift_buffer(net_data, 1);
    b
}

fn shift_buffer(net_data: &mut NetworkData, shift: u32) {
    for i in 0..net_data.read_buffer_size {
        net_data.read_buffer[i as usize] = net_data.read_buffer[(i+shift) as usize];
    }
}

pub fn rnet_check_for_new_client(net_data: &NetworkData) -> Option<sdl2_net::TCPsocket> {
    if !net_data.is_server { return None }
    if sdl2_net::socket_ready(&(net_data.socket)) {
        let pos_new_socket = sdl2_net::tcp_accept(&(net_data.socket));

        let new_socket: sdl2_net::TCPsocket;

        match pos_new_socket {
            Some(s) => new_socket = s,
            None => return None,
        }

        sdl2_net::add_socket(&(net_data.socket_set), &(new_socket));

        return Some(new_socket)
    }
    None
}

pub fn rnet_check_sockets(net_data: &NetworkData) -> bool {
    sdl2_net::check_sockets(&(net_data.socket_set), 0) > 0
}

fn remove_socket(net_data: &NetworkData, socket: &sdl2_net::TCPsocket) {
    sdl2_net::del_socket(&(net_data.socket_set), &socket);
}

pub fn rnet_init_server(port: u16, num_clients: u32) -> Option<NetworkData> {
    let possible_ss = rnet_initialize(num_clients as i32);
    
    let socket_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return None,
        Some(ss) => socket_set = ss,
    }

    let possible_ip = sdl2_net::become_host(port);

    let mut ip: sdl2_net::IPaddress;

    match possible_ip {
        Some(i) => ip = i,
        None => return None,
    }

    let possible_socket = sdl2_net::tcp_open(&mut ip);

    let mut socket: sdl2_net::TCPsocket;

    match possible_socket {
        Some(s) => socket = s,
        None => return None,
    }

    sdl2_net::add_socket(&socket_set, &socket);

    Some(new_net_data(socket_set, socket, true))

}

pub fn rnet_init_client(host: &str, port: u16) -> Option<NetworkData> {
    let possible_ss = rnet_initialize(1);

    let socket_set: sdl2_net::SocketSet;

    match possible_ss{
        None => return None,
        Some(ss) => socket_set = ss,
    }

    let possible_ip = sdl2_net::resolve_host(host, port);

    let mut ip: sdl2_net::IPaddress;

    match possible_ip {
        Some(i) => ip = i,
        None => return None,
    }

    let possible_socket = sdl2_net::tcp_open(&mut ip);

    let mut socket: sdl2_net::TCPsocket;

    match possible_socket {
        Some(s) => socket = s,
        None => return None,
    }

    sdl2_net::add_socket(&socket_set, &socket);

    Some(new_net_data(socket_set, socket, false))
}

fn new_net_data(socket_set: sdl2_net::SocketSet, socket: sdl2_net::TCPsocket, is_server: bool) -> NetworkData {
    NetworkData{   write_buffer : [0; 512],    buffer_index : 0,
                        read_buffer : [0; 512],     read_buffer_size : 0,
                        c_buffer : Vec::with_capacity(MAX_BUFFER_SIZE as usize),
                        socket_set : socket_set,    socket : socket,
                        is_server: is_server}
}

fn rnet_initialize(socket_set_size: i32) -> Option<sdl2_net::SocketSet> {
    if !sdl2_net::init() {
        println!("SDLNet init failure.");
        None
    } else {
        Some(sdl2_net::alloc_socket_set(socket_set_size))
    }
}
