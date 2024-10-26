use log::{debug, error, info};
use yaml_rust::yaml;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use udpbenchmark::udp_application::{UDPApplication, UDPApplicationEnum};
use threadpool::ThreadPool;
use core_affinity;
use rand::thread_rng;
use rand::Rng;

enum ServerType {
    Default,
    Jitter,
    Delay,
    Loss,
    Duplicate,
    Reorder,
}

struct UDPServer {
    socket: Arc<Mutex<UdpSocket>>,
    pool: ThreadPool,
}

impl UDPServer {
    pub fn new(addr: &str, port: u16) -> Self {
        let socket = UdpSocket::bind(format!("{}:{}", addr, port)).expect("Couldn't bind to address");
        info!("Server bound to {}:{}", addr, port);
        UDPServer {
            socket: Arc::new(Mutex::new(socket)),
            pool: ThreadPool::new(4), // Adjust the number of threads as needed
        }
    }

    pub fn handle_client_with_loss(&self, loss: u64) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            // Introduce loss to simulate real-world network conditions
                            let loss_ran = thread_rng().gen_range(0..100);
                            if loss_ran < loss {
                                debug!("Dropping packet");
                                continue;
                            }
                            let response_bytes = response_packet.to_bytes();
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                    
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }

    pub fn handle_client_with_jitter(&self, jitter: u64) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            let response_bytes = response_packet.to_bytes();
                            // Introduce jitter to simulate real-world network conditions
                            let jitter = thread_rng().gen_range(0..jitter);
                            std::thread::sleep(std::time::Duration::from_millis(jitter));
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                    
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }

    pub fn handle_client_with_delay(&self, delay: u64) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            let response_bytes = response_packet.to_bytes();
                            // Introduce delay to simulate real-world network conditions
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                    
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }

    pub fn handle_client_with_duplicate(&self, duplicate: u64) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            let response_bytes = response_packet.to_bytes();
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                            // Duplicate the packet to simulate real-world network conditions
                            let duplicate_ran = thread_rng().gen_range(0..100);
                            if duplicate_ran < duplicate {
                                if let Err(e) = socket.send_to(&response_bytes, addr) {
                                    error!("Failed to send response: {}", e);
                                } else {
                                    debug!("Sent response to {}", addr);
                                }
                            }
                            
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }

    pub fn handle_client_with_reorder(&self, reorder: u64, reorder_delay: u64) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            let response_bytes = response_packet.to_bytes();
                            // Reorder the packet to simulate real-world network conditions
                            let reorder_ran = thread_rng().gen_range(0..100);
                            if reorder_ran < reorder {
                                std::thread::sleep(std::time::Duration::from_millis(thread_rng().gen_range(0..reorder_delay)));
                            }
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                    
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }

    pub fn handle_client(&self) {
        let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");
        let socket = Arc::clone(&self.socket);

        for (_, core_id) in cores.iter().enumerate().take(self.pool.max_count()) {
            let socket_clone = Arc::clone(&socket);
            let core_id = core_id.clone();

            self.pool.execute(move || {
                core_affinity::set_for_current(core_id);

                let mut buf = [0; 131072];
                loop {
                    let socket = socket_clone.lock().unwrap();
                    match socket.recv_from(&mut buf) {
                        Ok((len, addr)) => {
                            debug!("Received {} bytes from {}", len, addr);
                            // Process the packet (this is where you can add your custom logic)
                            // For example, you can parse the packet and send a response
                            let request_packet = UDPApplication::from_bytes(&buf[..len]);
                            let response_packet = UDPApplication {
                                type_field: UDPApplicationEnum::RESPONSE as u16,
                                session_id: request_packet.session_id, // Session ID can be set as needed
                                test_payload: request_packet.test_payload,
                            };
                            let response_bytes = response_packet.to_bytes();
                            if let Err(e) = socket.send_to(&response_bytes, addr) {
                                error!("Failed to send response: {}", e);
                            } else {
                                debug!("Sent response to {}", addr);
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive data: {}", e);
                        }
                    }
                }
            });
        }

        // Wait for all threads to finish (this will block indefinitely in this example)
        self.pool.join();
    }
}

fn main() {
    env_logger::init();
    
    let settings = yaml::YamlLoader::load_from_str(include_str!("../server_config.yaml")).unwrap();
    let server_address = settings[0]["server"]["address"].as_str().unwrap();
    let address_parts: Vec<&str> = server_address.split(':').collect::<Vec<&str>>();

    let server_type = match settings[0]["server"]["qos_profile"].as_str().unwrap() {
        "jitter" => ServerType::Jitter,
        "delay" => ServerType::Delay,
        "loss" => ServerType::Loss,
        "duplicate" => ServerType::Duplicate,
        "reorder" => ServerType::Reorder,
        _ => ServerType::Default,
    };

    let server = UDPServer::new(address_parts[0], address_parts[1].parse::<u16>().unwrap());

    match server_type {
        ServerType::Jitter => {
            info!("Server type: Jitter");
            let jitter = settings[0]["qos_profile_config"]["jitter"].as_i64().unwrap_or(0) as u64;
            server.handle_client_with_jitter(jitter);
        }
        ServerType::Delay => {
            info!("Server type: Delay");
            let delay = settings[0]["qos_profile_config"]["delay"].as_i64().unwrap_or(0) as u64;
            server.handle_client_with_delay(delay);
        }
        ServerType::Loss => {
            info!("Server type: Loss");
            let loss = settings[0]["qos_profile_config"]["loss"].as_i64().unwrap_or(0) as u64;
            server.handle_client_with_loss(loss);
        }
        ServerType::Duplicate => {
            info!("Server type: Duplicate");
            let duplicate = settings[0]["qos_profile_config"]["duplicate"].as_i64().unwrap_or(0) as u64;
            server.handle_client_with_duplicate(duplicate);
        }
        ServerType::Reorder => {
            info!("Server type: Reorder");
            let reorder = settings[0]["qos_profile_config"]["reorder"].as_i64().unwrap_or(0) as u64;
            let reorder_delay = settings[0]["qos_profile_config"]["reorder_delay"].as_i64().unwrap_or(0) as u64;
            server.handle_client_with_reorder(reorder, reorder_delay);
        }
        _ => {
            info!("Server type: Default");
            server.handle_client();
        }
        
    }
}