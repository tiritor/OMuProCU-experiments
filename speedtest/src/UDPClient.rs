use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::time::SystemTime;
use log::{debug, info, error};
use std::path::Path;
use std::fs;
use std::fs::OpenOptions;
use core_affinity;
use yaml_rust::YamlLoader;
use threadpool::ThreadPool;
use udpbenchmark::udp_application::{UDPApplication, UDPApplicationEnum};
use std::sync::Mutex;

#[derive(PartialEq)]
enum SpeedtestEnum {
    ByDurationCustomBitrate,
    ByDuration,
    ByPacketCount,
    Ping,
}

impl SpeedtestEnum {
    fn from_string(speedtest_mode: &str) -> SpeedtestEnum {
        match speedtest_mode {
            "duration_custom_bitrate" => SpeedtestEnum::ByDurationCustomBitrate,
            "duration" => SpeedtestEnum::ByDuration,
            "packet_count" => SpeedtestEnum::ByPacketCount,
            "ping" => SpeedtestEnum::Ping,
            _ => SpeedtestEnum::ByDuration,
        }
    }
    fn to_string(&self) -> &str {
        match self {
            SpeedtestEnum::ByDurationCustomBitrate => "duration_custom_bitrate",
            SpeedtestEnum::ByDuration => "duration",
            SpeedtestEnum::ByPacketCount => "packet_count",
            SpeedtestEnum::Ping => "ping",
        }
    }
}

#[derive(Debug)]
struct PacketStats {
    session_id: u16,
    sent_time: u128,
    received_time: u128,
}

#[derive(Debug)]
struct RTTTimes {
    rtt_times: Vec<PacketStats>,
}

impl RTTTimes {
    pub fn new() -> Self {
        RTTTimes {
            rtt_times: Vec::new(),
        }
    }

    pub fn is_session_id_present(&self, session_id: u16) -> bool {
        for packet in &self.rtt_times {
            if packet.session_id == session_id {
                return true;
            }
        }
        false
    }

    pub fn add(&mut self, session_id: u16, sent_time: u128, received_time: u128) {
        self.rtt_times.push(PacketStats {
            session_id,
            sent_time,
            received_time,
        });
    }

    pub fn set_received_time(&mut self, session_id: u16, received_time: u128) {
        for packet in &mut self.rtt_times {
            if packet.session_id == session_id {
                packet.received_time = received_time;
            }
        }
    }

    pub fn set_sent_time(&mut self, session_id: u16, sent_time: u128) {
        for packet in &mut self.rtt_times {
            if packet.session_id == session_id {
                packet.sent_time = sent_time;
            }
        }
    }

    pub fn get_rtt(&self, session_id: u16) -> u128 {
        for packet in &self.rtt_times {
            if packet.session_id == session_id {
                return packet.received_time - packet.sent_time;
            }
        }
        0
    }

    pub fn get_rtts(&self) -> Vec<u128> {
        let mut rtt_values = Vec::new();
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            rtt_values.push(rtt);
        }
        rtt_values
    }

    pub fn get_average_rtt(&self) -> f64 {
        let mut total_rtt = 0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            total_rtt += rtt;
        }
        total_rtt as f64 / self.rtt_times.len() as f64
    }

    pub fn get_median_rtt(&self) -> u128 {
        let mut rtt_values = Vec::new();
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            rtt_values.push(rtt);
        }
        rtt_values.sort();
        let mid = rtt_values.len() / 2;
        rtt_values[mid]
    }

    pub fn get_min_rtt(&self) -> u128 {
        let mut min_rtt = std::u128::MAX;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            if rtt < min_rtt {
                min_rtt = rtt;
            }
        }
        min_rtt
    }

    pub fn get_max_rtt(&self) -> u128 {
        let mut max_rtt = 0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            if rtt > max_rtt {
                max_rtt = rtt;
            }
        }
        max_rtt
    }

    pub fn get_variance_rtt(&self) -> f64 {
        let mut total_rtt = 0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            total_rtt += rtt;
        }
        let average_rtt = total_rtt / self.rtt_times.len() as u128;

        let mut variance = 0.0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            variance += (rtt as f64 - average_rtt as f64).powf(2.0);
        }
        variance = variance / self.rtt_times.len() as f64
    }

    pub fn get_stddev_rtt(&self) -> f64 {
        let mut total_rtt = 0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            total_rtt += rtt;
        }
        let average_rtt = total_rtt / self.rtt_times.len() as u128;

        let mut variance = 0.0;
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            variance += (rtt as f64 - average_rtt as f64).powf(2.0);
        }
        let stddev = (variance as f64 / self.rtt_times.len() as f64).sqrt();
        stddev
    }

    pub fn get_percentile_rtt(&self, percentile: f64) -> f64 {
        let mut rtt_values = Vec::new();
        for stats in &self.rtt_times {
            if stats.received_time == 0 || stats.sent_time == 0 {
                continue;
            }
            let rtt = stats.received_time - stats.sent_time;
            rtt_values.push(rtt as u64);
        }
        rtt_values.sort();
        let index = (percentile * rtt_values.len() as f64) as usize;
        rtt_values[index] as f64
    }
}

enum BitrateScale {
    Bps = 1,
    Kbps = 1024,
    Mbps = 1048576,
    Gbps = 1073741824,
    
}

fn get_bitrate_scale(bitrate_scale: &str) -> BitrateScale {
    match bitrate_scale {
        "bps" => BitrateScale::Bps,
        "kbps" => BitrateScale::Kbps,
        "Kbps" => BitrateScale::Kbps,
        "K" => BitrateScale::Kbps,
        "mbps" => BitrateScale::Mbps,
        "Mbps" => BitrateScale::Mbps,
        "M" => BitrateScale::Mbps,
        "gbps" => BitrateScale::Gbps,
        "Gbps" => BitrateScale::Gbps,
        "G" => BitrateScale::Gbps,
        _ => BitrateScale::Bps,
    }
}

fn speedtest_simple_ping(duration: Duration, server_addr: &str, client_addr: &str, payload_size: usize, interval : Duration, rtt_times : &Arc<Mutex<RTTTimes>>) {
    info!("Starting UDP Speedtest client in duration mode");
    info!("Sending packets with payload size {} bytes for {:?}", payload_size, duration);

    let server_addr = server_addr.to_string();
    let client_addr = client_addr.to_string();

    info!("Connecting to server at {}", server_addr);

    // Bind the client socket
    debug!("Binding to client address: {}", client_addr);
    let socket = Arc::new(UdpSocket::bind(&client_addr).expect("Couldn't bind to address"));
    info!("Socket bound to address: {}", client_addr);
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("Couldn't set read timeout");

    let payload = vec![0u8; payload_size];
    let total_time = Arc::new(AtomicUsize::new(0));
    let total_bytes_sent = Arc::new(AtomicUsize::new(0));
    let total_bytes_received = Arc::new(AtomicUsize::new(0));

    // Create a thread pool
    let pool = ThreadPool::new(2); // Adjust the number of threads based on your CPU cores

    // Get the list of available CPU cores
    let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");

    // Thread for receiving packets
    for (_, core_id) in cores.iter().enumerate().take(1) {
        let core_id = core_id.clone(); // Clone the core ID to move into the thread
        let socket_clone = Arc::clone(&socket);
        let total_bytes_received_clone = Arc::clone(&total_bytes_received);
        let rtt_times_clone: Arc<Mutex<RTTTimes>> = Arc::clone(&rtt_times); // Provide explicit type annotation

        debug!("Starting receiving thread for core: {:?}", core_id);

        pool.execute(move || {
            // Pin this thread to a specific core
            core_affinity::set_for_current(core_id);

            let mut buf = [0; 131072];
            let start_time = Instant::now();
            while start_time.elapsed() < duration {
                debug!("Waiting to receive data...");
                match socket_clone.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        debug!("Received data: {} bytes", len);
                        // debug!("Received response: {:?}", &buf[..len]);
                        let response_packet = UDPApplication::new(&buf[..len]);
                        debug!("Received response: {}", response_packet.summary());
                        total_bytes_received_clone.fetch_add(len, Ordering::Relaxed);
                        let receive_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error").as_micros() as u128;
                        debug!("Receive time: {}", receive_time);
                        let mut rtt_times = rtt_times_clone.lock().unwrap();
                        if rtt_times.is_session_id_present(response_packet.session_id) {
                            debug!("Setting received time for session ID: {}", response_packet.session_id);
                            rtt_times.set_received_time(response_packet.session_id, receive_time);
                        } else {
                            debug!("Missing Session ID! Adding session ID: {}", response_packet.session_id);
                            rtt_times.add(response_packet.session_id, 0, receive_time);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        debug!("No data received yet, continuing...");
                    }
                    Err(e) => {
                        error!("Error receiving packet: {:?}", e);
                    }
                }
            }
        });
    }

    let total_bytes_sent_clone = Arc::clone(&total_bytes_sent);
    let total_time_clone = Arc::clone(&total_time);
    
    let session_id_counter = Arc::new(AtomicUsize::new(0));
    let rtt_times_clone = Arc::clone(&rtt_times);

    // Thread for sending packets
    pool.execute(move || {
        let start_time = Instant::now();
        while start_time.elapsed() < duration {
            let session_id_counter_clone = Arc::clone(&session_id_counter);
            
            let request_packet = UDPApplication {
                type_field: UDPApplicationEnum::REQUEST as u16,
                session_id: session_id_counter_clone.fetch_add(1, Ordering::SeqCst) as u16, // Session ID can be set as needed
                test_payload: payload.clone(),
            };
            let packet_start_time = Instant::now();
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
            debug!("Sending request: {}", request_packet.summary());
            let packet_bytes = request_packet.to_bytes();
            socket.send_to(&packet_bytes, server_addr.clone()).expect("Couldn't send data");

            let mut rtt_times = rtt_times_clone.lock().unwrap();
            rtt_times.add(request_packet.session_id, now.as_micros(), 0);

            let elapsed_time = packet_start_time.elapsed().as_micros() as usize;
            total_time_clone.fetch_add(elapsed_time, Ordering::Relaxed);
            total_bytes_sent_clone.fetch_add(packet_bytes.len(), Ordering::Relaxed);

            // Sleep for the calculated interval to control the bitrate
            thread::sleep(interval);
        }
    });

    pool.join();

    // println!("RTT: {:?}", rtt_times);
    let total_time_value = total_time.load(Ordering::Relaxed);
    let total_bytes_sent_value = total_bytes_sent.load(Ordering::Relaxed);
    let total_bytes_received_value = total_bytes_received.load(Ordering::Relaxed);

    let throughput_sent = total_bytes_sent_value as f64 / total_time_value as f64;
    let throughput_received = total_bytes_received_value as f64 / total_time_value as f64;

    debug!("Total bytes sent: {}", total_bytes_sent_value);
    debug!("Total bytes received: {}", total_bytes_received_value);
    debug!("Throughput sent: {:.2} bytes/sec ({:.2} MBps)", throughput_sent, throughput_sent / 1024.0 / 1024.0);
    debug!("Throughput received: {:.2} bytes/sec ({:.2} MBps)", throughput_received, throughput_received / 1024.0 / 1024.0);

}

fn speedtest_by_duration(duration: Duration, server_addr: &str, client_addr: &str, payload_size: usize) {
    info!("Starting UDP Speedtest client in duration mode");
    info!("Sending packets with payload size {} bytes for {:?}", payload_size, duration);

    let server_addr = server_addr.to_string();
    let client_addr = client_addr.to_string();

    info!("Connecting to server at {}", server_addr);
    
    // Bind the client socket
    debug!("Binding to client address: {}", client_addr);
    let socket = Arc::new(UdpSocket::bind(client_addr).expect("Couldn't bind to address"));
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("Couldn't set read timeout");

    let payload = vec![0u8; payload_size];
    let total_time = Arc::new(AtomicUsize::new(0));
    let total_bytes_sent = Arc::new(AtomicUsize::new(0));
    let total_bytes_received = Arc::new(AtomicUsize::new(0));
    let session_id_counter = Arc::new(AtomicUsize::new(0));
    let rtt_times = Arc::new(Mutex::new(RTTTimes::new()));

    // Create a thread pool
    let pool = ThreadPool::new(8); // Adjust the number of threads based on your CPU cores

    // Get the list of available CPU cores
    let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");

    // Thread for receiving packets
    for (_, core_id) in cores.iter().enumerate().take(4) {
        let core_id = core_id.clone(); // Clone the core ID to move into the thread
        let socket_clone = Arc::clone(&socket);
        let total_bytes_received_clone = Arc::clone(&total_bytes_received);
        let total_time_clone = Arc::clone(&total_time);
        let rtt_times_clone: Arc<Mutex<RTTTimes>> = Arc::clone(&rtt_times); // Provide explicit type annotation
    
        pool.execute(move || {
            // Pin this thread to a specific core
            core_affinity::set_for_current(core_id);
    
            let mut buf = [0; 131072];
            let start_time = Instant::now();
            while start_time.elapsed() < duration {
                match socket_clone.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        let mut rtt_times = rtt_times_clone.lock().unwrap();
                        let response_packet = UDPApplication::new(&buf[..len]);
                        if rtt_times.is_session_id_present(response_packet.session_id) {
                            rtt_times.set_received_time(response_packet.session_id, Instant::now().elapsed().as_micros() as u128);
                        } else {
                            rtt_times.add(response_packet.session_id, 0, Instant::now().elapsed().as_micros() as u128);
                        }
                        total_bytes_received_clone.fetch_add(len, Ordering::Relaxed);
                    }
                    
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Handle the WouldBlock error by retrying
                        thread::sleep(Duration::from_millis(10)); // Optional: Add a small sleep to avoid busy-waiting
                    }
                    Err(e) => {
                        eprintln!("Error receiving data: {}", e);
                        break;
                    }
                }
            
            total_time_clone.fetch_add(start_time.elapsed().as_secs() as usize, Ordering::SeqCst);
            }
        });
    }

    for (_, core_id) in cores.iter().enumerate().take(4) {
        let core_id = core_id.clone(); // Clone the core ID to move into the thread
        let socket_clone = Arc::clone(&socket);
        let total_bytes_sent_clone = Arc::clone(&total_bytes_sent);
        let total_time_clone = Arc::clone(&total_time);
        let server_addr_clone = server_addr.clone(); // Clone server_addr for each thread
        let payload_clone = payload.clone(); // Clone payload for each thread
        let session_id_counter_clone = Arc::clone(&session_id_counter); // Clone session_id_counter for each thread
        let rtt_times_clone = Arc::clone(&rtt_times); // Clone rtt_times inside the loop
    
        pool.execute(move || {
            // Pin this thread to a specific core
            core_affinity::set_for_current(core_id);
    
            let start_time = Instant::now();
            while start_time.elapsed() < duration {
                let mut batch = vec![];
                for _ in 0..10 { // Batch size of 10
                    let request_packet = UDPApplication {
                        type_field: UDPApplicationEnum::REQUEST as u16,
                        session_id: session_id_counter_clone.fetch_add(1, Ordering::SeqCst) as u16, // Session ID can be set as needed
                        test_payload: payload_clone.clone(),
                    };
                    let mut rtt_times = rtt_times_clone.lock().unwrap();
                    rtt_times.add(request_packet.session_id, Instant::now().elapsed().as_micros() as u128, 0);
                    batch.push(request_packet.to_bytes());
                }
    
                for packet_bytes in batch {
                    let packet_start_time = Instant::now();
                    socket_clone.send_to(&packet_bytes, server_addr_clone.clone()).expect("Couldn't send data");
                    let elapsed_time = packet_start_time.elapsed().as_micros() as usize;
                    total_time_clone.fetch_add(elapsed_time, Ordering::Relaxed);
                    total_bytes_sent_clone.fetch_add(packet_bytes.len(), Ordering::Relaxed);
                }
            }
        });
    }

    pool.join();


    // let total_time_value = total_time.load(Ordering::Relaxed);
    let total_bytes_sent_value = total_bytes_sent.load(Ordering::Relaxed);
    let total_bytes_received_value = total_bytes_received.load(Ordering::Relaxed);

    let throughput_sent = total_bytes_sent_value as f64 / duration.as_secs_f64();
    let throughput_received = total_bytes_received_value as f64 / duration.as_secs_f64();

    debug!("Total bytes sent: {}", total_bytes_sent_value);
    debug!("Total bytes received: {}", total_bytes_received_value);
    debug!("Throughput sent: {:.2} bytes/sec ({:.2} MBps)", throughput_sent, throughput_sent / 1024.0 / 1024.0);
    debug!("Throughput received: {:.2} bytes/sec ({:.2} MBps)", throughput_received, throughput_received / 1024.0 / 1024.0);
}

fn speedtest_bitrate_by_duration(duration: Duration, bitrate: i64, server_addr: &str, client_addr: &str, payload_size: usize, bitrate_scale: BitrateScale, rtt_times : &Arc<Mutex<RTTTimes>>) {
    debug!("Starting UDP Speedtest client in duration mode");
    debug!("Sending packets with payload size {} bytes for {:?}", payload_size, duration);

    let server_addr = server_addr.to_string();
    let client_addr = client_addr.to_string();

    debug!("Connecting to server at {}", server_addr);
    
    // Bind the client socket
    debug!("Binding to client address: {}", client_addr);
    let socket = Arc::new(UdpSocket::bind(client_addr).expect("Couldn't bind to address"));
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("Couldn't set read timeout");

    let payload = vec![0u8; payload_size];
    let total_time = Arc::new(AtomicUsize::new(0));
    let total_bytes_sent = Arc::new(AtomicUsize::new(0));
    let total_bytes_received = Arc::new(AtomicUsize::new(0));

    // let socket_clone = Arc::clone(&socket);
    let total_time_clone = Arc::clone(&total_time);
    // let total_bytes_received_clone = Arc::clone(&total_bytes_received);

    // Create a thread pool
    let pool = ThreadPool::new(8); // Adjust the number of threads based on your CPU cores

    // Get the list of available CPU cores
    let cores = core_affinity::get_core_ids().expect("Couldn't get core IDs");

    // Thread for receiving packets
    for (_, core_id) in cores.iter().enumerate().take(4) {
        let core_id = core_id.clone(); // Clone the core ID to move into the thread
        let socket_clone = Arc::clone(&socket);
        let total_bytes_received_clone = Arc::clone(&total_bytes_received);
        let total_time_clone = Arc::clone(&total_time);
        let rtt_times_clone: Arc<Mutex<RTTTimes>> = Arc::clone(&rtt_times); // Provide explicit type annotation
    
        pool.execute(move || {
            // Pin this thread to a specific core
            core_affinity::set_for_current(core_id);
    
            let mut buf = [0; 131072];
            let start_time = Instant::now();
            while start_time.elapsed() < duration {
                match socket_clone.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        let response_packet = UDPApplication::new(&buf[..len]);
                        let receive_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error").as_micros() as u128;
                        debug!("Received response: {}", response_packet.summary());
                        let mut rtt_times = rtt_times_clone.lock().unwrap();
                        if rtt_times.is_session_id_present(response_packet.session_id) {
                            rtt_times.set_received_time(response_packet.session_id, receive_time);
                        } else {
                            rtt_times.add(response_packet.session_id, 0, receive_time);
                        }
                        total_bytes_received_clone.fetch_add(len, Ordering::Relaxed);
                    }
                    
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Handle the WouldBlock error by retrying
                        thread::sleep(Duration::from_millis(10)); // Optional: Add a small sleep to avoid busy-waiting
                    }
                    Err(e) => {
                        eprintln!("Error receiving data: {}", e);
                        break;
                    }
                }
            
            total_time_clone.fetch_add(start_time.elapsed().as_secs() as usize, Ordering::SeqCst);
            }
        });
    }

    let total_bytes_sent_clone = Arc::clone(&total_bytes_sent);

    let bitrate = bitrate * bitrate_scale as i64;
    debug!("Bitrate: {} bps", bitrate);
    // Calculate the interval between sending packets
    let packet_size_bits = (payload_size * 8) as u64; // Convert payload size to bits
    debug!("Packet size in bits: {}", packet_size_bits);
    let interval = Duration::from_secs_f64(packet_size_bits as f64 / bitrate as f64);
    debug!("Interval between packets: {:?}", interval);

    let session_id_counter = Arc::new(AtomicUsize::new(0));
    let rtt_times_clone = Arc::clone(&rtt_times);

    // Thread for sending packets
    pool.execute(move || {
        let start_time = Instant::now();
        while start_time.elapsed() < duration {
            let session_id_counter_clone = Arc::clone(&session_id_counter);
            
            let request_packet = UDPApplication {
                type_field: UDPApplicationEnum::REQUEST as u16,
                session_id: session_id_counter_clone.fetch_add(1, Ordering::SeqCst) as u16, // Session ID can be set as needed
                test_payload: payload.clone(),
            };
            let packet_start_time = Instant::now();
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
            debug!("Sending request: {}", request_packet.summary());
            let packet_bytes = request_packet.to_bytes();
            socket.send_to(&packet_bytes, server_addr.clone()).expect("Couldn't send data");

            let mut rtt_times = rtt_times_clone.lock().unwrap();
            rtt_times.add(request_packet.session_id, now.as_micros(), 0);

            let elapsed_time = packet_start_time.elapsed().as_micros() as usize;
            total_time_clone.fetch_add(elapsed_time, Ordering::Relaxed);
            total_bytes_sent_clone.fetch_add(packet_bytes.len(), Ordering::Relaxed);

            // Sleep for the calculated interval to control the bitrate
            thread::sleep(interval);
        }
    });

    pool.join();


    // let total_time_value = total_time.load(Ordering::Relaxed);
    let total_bytes_sent_value = total_bytes_sent.load(Ordering::Relaxed);
    let total_bytes_received_value = total_bytes_received.load(Ordering::Relaxed);

    let throughput_sent = total_bytes_sent_value as f64 * 8 as f64 / duration.as_secs_f64();
    let throughput_received = total_bytes_received_value as f64 * 8 as f64 / duration.as_secs_f64();

    debug!("Total bytes sent: {}", total_bytes_sent_value);
    debug!("Total bytes received: {}", total_bytes_received_value);
    debug!("Throughput sent: {:.2} bytes/sec ({:.2} MBps)", throughput_sent, throughput_sent / 1024.0 / 1024.0);
    debug!("Throughput received: {:.2} bytes/sec ({:.2} MBps)", throughput_received, throughput_received / 1024.0 / 1024.0);
}

fn speedtest_by_packet_count(server_addr: &str, client_addr: &str, payload_size: usize, packet_count: usize) {
    debug!("Starting UDP Speedtest client in packet count mode");
    debug!("Sending {} packets with payload size {} bytes", packet_count, payload_size);

    let server_addr = server_addr.to_string();
    let client_addr = client_addr.to_string();

    debug!("Connecting to server at {}", server_addr);
    
    // Bind the client socket
    debug!("Binding to client address: {}", client_addr);
    let socket = Arc::new(UdpSocket::bind(client_addr).expect("Couldn't bind to address"));
    socket.set_read_timeout(Some(Duration::from_secs(5))).expect("Couldn't set read timeout");

    let payload = vec![0u8; payload_size];
    let total_time = Arc::new(AtomicUsize::new(0));
    let total_bytes_sent = Arc::new(AtomicUsize::new(0));
    let total_bytes_received = Arc::new(AtomicUsize::new(0));

    let socket_clone = Arc::clone(&socket);
    let total_time_clone = Arc::clone(&total_time);
    let total_bytes_received_clone = Arc::clone(&total_bytes_received);

    
    // Create a thread pool
    let pool = ThreadPool::new(4); // Adjust the number of threads based on your CPU cores

    // Thread for receiving packets
    pool.execute(move || {
        let mut buf = [0; 131072];
        let mut received_packets = 0;
        while received_packets < packet_count {
            match socket_clone.recv_from(&mut buf) {
                Ok((len, _addr)) => {
                    let packet = UDPApplication::new(&buf[..len]);
                    if packet.type_field == UDPApplicationEnum::RESPONSE as u16 {
                        received_packets += 1;
                        total_bytes_received_clone.fetch_add(len, Ordering::Relaxed);
                    }
                }
                Err(e) => {
                    debug!("Error receiving packet: {:?}", e);
                }
            }
        }
    });

    let total_bytes_sent_clone = Arc::clone(&total_bytes_sent);

    // Thread for sending packets
    pool.execute(move || {
        for i in 0..packet_count {
            let request_packet = UDPApplication {
                type_field: UDPApplicationEnum::REQUEST as u16,
                session_id: i as u16,
                test_payload: payload.clone(),
            };
    
            let start_time = Instant::now();
            debug!("Sending request: {}", request_packet.summary());
            let packet_bytes = request_packet.to_bytes();
            socket.send_to(&packet_bytes, server_addr.clone()).expect("Couldn't send data");
    
            let elapsed_time = start_time.elapsed().as_micros() as usize;
            total_time_clone.fetch_add(elapsed_time, Ordering::Relaxed);
            total_bytes_sent_clone.fetch_add(packet_bytes.len(), Ordering::Relaxed);
        }
    });

    // Wait for the thread pool to finish
    pool.join();

    let total_time_value = total_time.load(Ordering::Relaxed);
    let average_time = total_time_value / packet_count;
    debug!("Average round-trip time: {:?}", average_time);

    let total_bytes_sent_value = total_bytes_sent.load(Ordering::Relaxed);
    let total_bytes_received_value = total_bytes_received.load(Ordering::Relaxed);

    let throughput_sent = total_bytes_sent_value as f64 / total_time_value as f64;
    let throughput_received = total_bytes_received_value as f64 / total_time_value as f64;

    debug!("Total bytes sent: {}", total_bytes_sent_value);
    debug!("Total bytes received: {}", total_bytes_received_value);
    debug!("Throughput sent: {:.2} bytes/sec ({:.2} MBps)", throughput_sent, throughput_sent / 1024.0 / 1024.0);
    debug!("Throughput received: {:.2} bytes/sec ({:.2} MBps)", throughput_received, throughput_received / 1024.0 / 1024.0);

}

// fn load_settings() {
//     let settings = YamlLoader::load_from_str(include_str!("../client_config.yaml")).unwrap();
//     println!("{:?}", settings);

//     let server_addr = settings[0]["server_addr"].as_str().unwrap();
//     let client_addr = settings[0]["client_addr"].as_str().unwrap();

//     // println!("Payload Size: {:?}", settings[0]["payload_size"].as_i64());

//     let payload_size = settings[0]["payload_size"].as_i64().unwrap() as usize;
//     let packet_count = settings[0]["packet_count"].as_i64().unwrap() as usize;
//     let duration = Duration::from_secs(settings[0]["speedtest_duration"].as_i64().unwrap() as u64);

//     let speedtest_setting = settings[0]["speedtest_mode"].as_str().unwrap().to_lowercase();
//     let bitrate_scale_setting = settings[0]["bitrate_scale"].as_str().unwrap().to_lowercase();
//     let scale = get_bitrate_scale(&bitrate_scale_setting);

//     let mut speedtest_mode = SpeedtestEnum::ByDuration;
//     if speedtest_setting == "duration" {
//         speedtest_mode = SpeedtestEnum::ByDuration;
//     } else if speedtest_setting == "packet_count" {
//         speedtest_mode = SpeedtestEnum::ByPacketCount;
//     } else if speedtest_setting == "duration_custom_bitrate" {
//         speedtest_mode = SpeedtestEnum::ByDurationCustomBitrate;
//     }

//     // return (server_addr, client_addr, payload_size, packet_count, duration, speedtest_mode, scale);
    
// }

fn evaluate_rtt(rtt_times: &Arc<Mutex<RTTTimes>>) {
    let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
    debug!("RTT times: {:?}", rtt_times_value);
    let avg_rtt = rtt_times.lock().unwrap().get_average_rtt();
    let median_rtt = rtt_times.lock().unwrap().get_median_rtt();
    let min_rtt = rtt_times.lock().unwrap().get_min_rtt();
    let max_rtt = rtt_times.lock().unwrap().get_max_rtt();
    let stddev_rtt = rtt_times.lock().unwrap().get_stddev_rtt();
    let variance_rtt = rtt_times.lock().unwrap().get_variance_rtt();
    let percentile_rtt = rtt_times.lock().unwrap().get_percentile_rtt(0.95);
    debug!("Average RTT: {:.2} microseconds ({:.2} ms)", avg_rtt, avg_rtt / 1000.0);
    debug!("Median RTT: {:.2} microseconds ({:.2} ms)", median_rtt, median_rtt as f64 / 1000.0);
    debug!("Minimum RTT: {:.2} microseconds ({:.2} ms)", min_rtt, min_rtt as f64 / 1000.0);
    debug!("Maximum RTT: {:.2} microseconds ({:.2} ms)", max_rtt, max_rtt as f64 / 1000.0);
    debug!("Variance RTT: {:.2} microseconds ({:.2} ms)", variance_rtt,  variance_rtt / 1000.0);
    debug!("Standard Deviation RTT: {:.2} microseconds ({:.2} ms)", stddev_rtt, stddev_rtt / 1000.0);
    debug!("95th Percentile RTT: {:.2} microseconds ({:.2} ms)", percentile_rtt, percentile_rtt / 1000.0);
}

fn write_raw_data_to_csv(rtt_times : &Arc<Mutex<RTTTimes>>) {
    let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
    let mut wtr = csv::Writer::from_path("rtt_times.csv").unwrap();
    for rtt in rtt_times_value {
        wtr.write_record(&[rtt.to_string()]).unwrap();
    }
    wtr.flush().unwrap();
}

fn write_evaluated_data_to_csv(server_addr: &str, speedtest_mode: &SpeedtestEnum, rtt_times : &Arc<Mutex<RTTTimes>>) {
    let results_dir = "results/";
    if !Path::new(&results_dir).exists() {
        let _ = fs::create_dir_all(results_dir);
    }
    let file_name = format!("{}/rtt_times_{}_{}.csv", results_dir, server_addr, speedtest_mode.to_string());

    let avg_rtt = rtt_times.lock().unwrap().get_average_rtt();
    let median_rtt = rtt_times.lock().unwrap().get_median_rtt();
    let min_rtt = rtt_times.lock().unwrap().get_min_rtt();
    let max_rtt = rtt_times.lock().unwrap().get_max_rtt();
    let stddev_rtt = rtt_times.lock().unwrap().get_stddev_rtt();
    let variance_rtt = rtt_times.lock().unwrap().get_variance_rtt();
    let percentile_rtt = rtt_times.lock().unwrap().get_percentile_rtt(0.95);

    // Check if the file exists and append entry to the file
    let file_exists = Path::new(&file_name).exists();

    let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .append(true)
    .open(file_name)
    .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    if !file_exists {
        wtr.write_record(&["Server Address", "Speedtest-Mode", "Average RTT", "Median RTT", "Minimum RTT", "Maximum RTT", "Variance RTT", "Standard Deviation RTT", "95th Percentile RTT"]).unwrap();
    }
    wtr.write_record(&[server_addr.to_string(), speedtest_mode.to_string().into(), avg_rtt.to_string(), median_rtt.to_string(), min_rtt.to_string(), max_rtt.to_string(), variance_rtt.to_string(), stddev_rtt.to_string(), percentile_rtt.to_string()]).unwrap();
}

fn main() {
    env_logger::init();

    info!("Starting UDP Speedtest client");

    let settings = YamlLoader::load_from_str(include_str!("../client_config.yaml")).unwrap();
    debug!("{:?}", settings);

    let server_addr = settings[0]["server_addr"].as_str().unwrap();
    let client_addr = settings[0]["client_addr"].as_str().unwrap();

    // println!("Payload Size: {:?}", settings[0]["payload_size"].as_i64());

    let payload_size = settings[0]["payload_size"].as_i64().unwrap() as usize;
    let packet_count = settings[0]["packet_count"].as_i64().unwrap() as usize;
    let duration = Duration::from_secs(settings[0]["speedtest_duration"].as_i64().unwrap() as u64);

    let speedtest_setting = settings[0]["speedtest_mode"].as_str().unwrap().to_lowercase();
    let bitrate_scale_setting = settings[0]["bitrate_scale"].as_str().unwrap();
    let scale = get_bitrate_scale(&bitrate_scale_setting);

    let mut speedtest_mode = SpeedtestEnum::ByDuration;
    if speedtest_setting == "duration" {
        speedtest_mode = SpeedtestEnum::ByDuration;
    } else if speedtest_setting == "packet_count" {
        speedtest_mode = SpeedtestEnum::ByPacketCount;
    } else if speedtest_setting == "duration_custom_bitrate" {
        speedtest_mode = SpeedtestEnum::ByDurationCustomBitrate;
        match scale {
            BitrateScale::Bps => debug!("Bitrate scale is Bps"),
            BitrateScale::Kbps => debug!("Bitrate scale is Kbps"),
            BitrateScale::Mbps => debug!("Bitrate scale is Mbps"),
            BitrateScale::Gbps => debug!("Bitrate scale is Gbps"),
        }
    } else if speedtest_setting == "ping" {
        speedtest_mode = SpeedtestEnum::Ping;
    }

    let experiment_mode = settings[0]["experiment_mode"].as_bool().unwrap();
    // let experiment_count = ;
    if experiment_mode {
        info!("Starting experiment mode");
        let experiment_count = settings[0]["experiment_count"].as_i64().unwrap() as usize;
        let experiment_interval = settings[0]["experiment_interval"].as_i64().unwrap() as u64;
        let experiment_servers = settings[0]["experiment_servers"].as_vec().unwrap();
        let experiment_payload_sizes = settings[0]["experiment_payload_sizes"].as_vec().unwrap();

        for count in 0..experiment_count {
            info!("Starting experiment iteration: {}", count);
            for server in experiment_servers {
                info!("Starting experiment for server: {}", server.as_str().unwrap());
                for payload_size in experiment_payload_sizes {
                    let scale = get_bitrate_scale(&bitrate_scale_setting);
                    info!("Starting experiment for payload size: {}", payload_size.as_i64().unwrap());
                    let server_addr = server.as_str().unwrap();
                    let payload_size = payload_size.as_i64().unwrap() as usize;
                    if speedtest_mode == SpeedtestEnum::ByDuration {
                        speedtest_by_duration(duration, server_addr, client_addr, payload_size);
                    } else if speedtest_mode == SpeedtestEnum::ByDurationCustomBitrate {
                        let bitrate = settings[0]["bitrate"].as_i64().unwrap();
                        let rtt_times = Arc::new(Mutex::new(RTTTimes::new()));
                        speedtest_bitrate_by_duration(duration, bitrate, server_addr, client_addr, payload_size, scale, &rtt_times);
                        let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
                        debug!("RTT times: {:?}", rtt_times_value);
                        evaluate_rtt(&rtt_times);
                        write_evaluated_data_to_csv(&server_addr, &speedtest_mode, &rtt_times);
                    } else if speedtest_mode == SpeedtestEnum::ByPacketCount {
                        speedtest_by_packet_count(server_addr, client_addr, payload_size, packet_count);
                    } else if speedtest_mode == SpeedtestEnum::Ping {
                        let interval = Duration::from_secs(settings[0]["ping_interval"].as_i64().unwrap() as u64);
                        let rtt_times = Arc::new(Mutex::new(RTTTimes::new()));
                        speedtest_simple_ping(duration, server_addr, client_addr, payload_size, interval, &rtt_times); 
                        let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
                        debug!("RTT times: {:?}", rtt_times_value);
                        // debug!("Packet stats: {:?}", rtt_times);
                        evaluate_rtt(&rtt_times);
                        write_evaluated_data_to_csv(&server_addr, &speedtest_mode, &rtt_times);
                    }
                    thread::sleep(Duration::from_secs(experiment_interval));
                }
            }
        }
    } else {
        info!("Starting single mode");
        if speedtest_mode == SpeedtestEnum::ByDuration {
            speedtest_by_duration(duration, server_addr, client_addr, payload_size);
        } else if speedtest_mode == SpeedtestEnum::ByDurationCustomBitrate {
            let bitrate = settings[0]["bitrate"].as_i64().unwrap();
            let rtt_times = Arc::new(Mutex::new(RTTTimes::new()));
            speedtest_bitrate_by_duration(duration, bitrate, server_addr, client_addr, payload_size, scale, &rtt_times);
            let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
            debug!("RTT times: {:?}", rtt_times_value);
            evaluate_rtt(&rtt_times);
            write_evaluated_data_to_csv(&server_addr, &speedtest_mode, &rtt_times);

        } else if speedtest_mode == SpeedtestEnum::ByPacketCount {
            speedtest_by_packet_count(server_addr, client_addr, payload_size, packet_count);
        
        } else if speedtest_mode == SpeedtestEnum::Ping {
            let interval = Duration::from_secs(settings[0]["ping_interval"].as_i64().unwrap() as u64);
            let rtt_times = Arc::new(Mutex::new(RTTTimes::new()));
            speedtest_simple_ping(duration, server_addr, client_addr, payload_size, interval, &rtt_times); 
            let rtt_times_value = rtt_times.lock().unwrap().get_rtts();
            debug!("RTT times: {:?}", rtt_times_value);
            // debug!("Packet stats: {:?}", rtt_times);
            evaluate_rtt(&rtt_times);
            write_evaluated_data_to_csv(&server_addr, &speedtest_mode, &rtt_times);
        }

    }

}