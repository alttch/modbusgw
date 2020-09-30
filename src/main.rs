#[macro_use]
extern crate lazy_static;

use crc16::*;
use serial::prelude::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

use argparse::{ArgumentParser, Store};

const HOMEPAGE: &str = "https://github.com/alttch/modbusgw";
const VERSION: &str = "1.0.1";

struct Task {
    frame: Vec<u8>,
    reply_ch: mpsc::Sender<Vec<u8>>,
    broadcast: bool,
}

struct DataChannel {
    tx: Mutex<mpsc::Sender<Task>>,
    rx: Mutex<mpsc::Receiver<Task>>,
}

impl DataChannel {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

lazy_static! {
    static ref DC: DataChannel = DataChannel::new();
}

fn main() {
    let greeting = format!("TCP<->RTU Modbus Gateway v{} ({})", VERSION, HOMEPAGE);
    let mut listen = "0.0.0.0:5502".to_owned();
    let mut port_dev = "".to_owned();
    let mut baud_rate = "9600".to_owned();
    let mut char_size = "8".to_owned();
    let mut parity = "N".to_owned();
    let mut stop_bits = "1".to_owned();
    let mut timeout = "2".to_owned();
    let mut delay = "0.02".to_owned();
    let mut ap = ArgumentParser::new();
    ap.set_description(&greeting);
    ap.refer(&mut port_dev)
        .add_option(&["-p", "--port"], Store, "serial port device (REQUIRED)")
        .required();
    ap.refer(&mut listen).add_option(
        &["-l", "--listen"],
        Store,
        "host:port to listen (default: 0.0.0.0:5502)",
    );
    ap.refer(&mut baud_rate).add_option(
        &["-b", "--baud-rate"],
        Store,
        "serial port baud rate (default: 9600)",
    );
    ap.refer(&mut char_size).add_option(
        &["--char-size"],
        Store,
        "serial port char size (default: 8)",
    );
    ap.refer(&mut parity)
        .add_option(&["--parity"], Store, "serial port parity (default: N)");
    ap.refer(&mut stop_bits).add_option(
        &["--stop-bits"],
        Store,
        "serial port stop bits (default: 1)",
    );
    ap.refer(&mut timeout)
        .add_option(&["--timeout"], Store, "serial port timeout (default: 1s)");
    ap.refer(&mut delay)
        .add_option(&["--delay"], Store, "delay between frames (default: 0.02s)");
    ap.parse_args_or_exit();
    drop(ap);
    let frame_delay = Duration::from_millis((delay.parse::<f32>().unwrap() * 1000f32) as u64);
    let serial_band_rate = match baud_rate.as_str() {
        "110" => serial::Baud110,
        "300" => serial::Baud300,
        "600" => serial::Baud600,
        "1200" => serial::Baud1200,
        "2400" => serial::Baud2400,
        "4800" => serial::Baud4800,
        "9600" => serial::Baud9600,
        "19200" => serial::Baud19200,
        "38400" => serial::Baud38400,
        "57600" => serial::Baud57600,
        "115200" => serial::Baud115200,
        _ => unimplemented!("specified baud rate not supported"),
    };
    let serial_char_size = match char_size.as_str() {
        "5" => serial::Bits5,
        "6" => serial::Bits6,
        "7" => serial::Bits7,
        "8" => serial::Bits8,
        _ => unimplemented!("specified char size not supported"),
    };
    let serial_parity = match parity.as_str() {
        "N" => serial::ParityNone,
        "E" => serial::ParityEven,
        "O" => serial::ParityOdd,
        _ => unimplemented!("specified parity not supported"),
    };
    let serial_stop_bits = match stop_bits.as_str() {
        "1" => serial::Stop1,
        "2" => serial::Stop2,
        _ => unimplemented!("specified stop bits not supported"),
    };
    let listener = TcpListener::bind(&listen).unwrap();
    let mut port = serial::open(&port_dev).unwrap();
    port.reconfigure(&|settings| {
        (settings.set_baud_rate(serial_band_rate).unwrap());
        settings.set_char_size(serial_char_size);
        settings.set_parity(serial_parity);
        settings.set_stop_bits(serial_stop_bits);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .unwrap();
    port.set_timeout(Duration::from_secs(timeout.parse().unwrap()))
        .unwrap();
    thread::spawn(move || loop {
        let rx = DC.rx.lock().unwrap();
        let task: Task = rx.recv().unwrap();
        port.write(&task.frame).unwrap();
        let mut buf = [0u8; 3];
        let mut response = Vec::new();
        let func = task.frame[1];
        if !task.broadcast {
            let len = port.read(&mut buf).unwrap_or(0);
            if len == 3 {
                let remaining = match func == buf[1] {
                    true => match func {
                        1 | 2 | 3 | 4 => buf[2] + 2,
                        5 | 6 | 15 | 16 => 5,
                        _ => 0,
                    },
                    false => 2,
                };
                if remaining > 0 {
                    let mut rest = vec![0u8; remaining as usize];
                    let len = port.read(&mut rest).unwrap_or(0);
                    if len == remaining as usize {
                        response.extend_from_slice(&buf);
                        response.extend(rest);
                    }
                }
            }
        }
        task.reply_ch.send(response).unwrap();
        thread::sleep(frame_delay);
    });
    println!("Modbus gateway tcp:{} <-> rtu:{}", listen, port_dev);
    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.unwrap();
            loop {
                let mut buf = [0; 256];
                let mut response = Vec::new();
                let len = stream.read(&mut buf).unwrap_or(0);
                if len < 6 {
                    return;
                }
                let unit_id = buf[6];
                let broadcast = unit_id == 0 || unit_id == 255;
                if !broadcast {
                    response.extend_from_slice(&buf[0..4]);
                }
                let proto_id = u16::from_be_bytes([buf[2], buf[3]]);
                let length = u16::from_be_bytes([buf[4], buf[5]]);
                if proto_id != 0 || length < 6 || length > 250 {
                    eprintln!("client frame broken");
                    return;
                }
                let (tx, rx) = mpsc::channel();
                let mut rtu_frame = Vec::new();
                rtu_frame.extend_from_slice(&buf[6..len]);
                let crc = State::<MODBUS>::calculate(&rtu_frame);
                rtu_frame.extend_from_slice(&crc.to_le_bytes());
                DC.tx
                    .lock()
                    .unwrap()
                    .send(Task {
                        frame: rtu_frame,
                        reply_ch: tx,
                        broadcast: broadcast,
                    })
                    .unwrap();
                let resp = rx.recv().unwrap();
                let len = resp.len();
                macro_rules! response_error {
                    ($err:expr) => {
                        response.extend_from_slice(&[0, 3, unit_id, buf[7] + 0x80, $err]);
                    };
                }
                if len > 0 {
                    if len > 4 {
                        let end = resp.len() - 2;
                        let crc = State::<MODBUS>::calculate(&resp[..end]);
                        if crc == u16::from_le_bytes([resp[len - 2], resp[len - 1]]) {
                            response.extend_from_slice(&(end as u16).to_be_bytes());
                            response.extend_from_slice(&resp[..end]);
                        } else {
                            eprintln!("unit {} reply crc error", unit_id);
                            response_error!(0x0B);
                        }
                    } else {
                        eprintln!("unit {} invalid response", unit_id);
                        response_error!(0x0B);
                    }
                } else if !broadcast {
                    eprintln!("unit {} no response", unit_id);
                    response_error!(0x0B);
                }
                if response.len() > 0 {
                    if stream.write(&response).is_err() {
                        return;
                    };
                }
            }
        });
    }
}
