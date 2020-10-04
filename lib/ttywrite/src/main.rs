mod parsers;

use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::Xmodem;
use xmodem::Progress;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn progress_fn(progress: Progress) {
    println!("Progress: {:?}", progress);
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader, Write, Read};

    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    let mut settings = port.read_settings().expect("valid");
    settings.set_baud_rate(opt.baud_rate).expect("valid baud rate");
    settings.set_char_size(opt.char_width);
    settings.set_stop_bits(opt.stop_bits);
    settings.set_flow_control(opt.flow_control);
    port.write_settings(&settings).expect("valid settings");
    port.set_timeout(Duration::from_secs(opt.timeout)).expect("valid timeout");

    // FIXME: Implement the `ttywrite` utility.
    match opt.input {
        None => {
            loop {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer).expect("valid read");
                if opt.raw {
                    (&mut port).write_all(&buffer.as_bytes()).expect("valid write");
                } else {
                    Xmodem::transmit_with_progress(buffer.as_bytes(), &mut port, progress_fn).expect("valid transmit");
                }
            }
        },
        Some(path) => {
            let mut input = BufReader::new(File::open(path).expect("file should exist"));
            if opt.raw {
                std::io::copy(&mut input, &mut port).expect("valid copy");
            } else {
                Xmodem::transmit_with_progress(input, &mut port, progress_fn).expect("valid transmit");
            }
        },
    }

}
