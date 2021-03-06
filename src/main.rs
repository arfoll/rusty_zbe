use bitflags::bitflags;
use clap::{Arg, Command};
use futures_util::stream::StreamExt;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio_socketcan::{CANFilter, CANFrame, CANSocket, Error};
use uinput_tokio::event::keyboard;

bitflags! {
    #[derive(Default)]
    struct ZbeKeys:u64 {
        const MAP =    0b0001;
        const NAV =    0b100000000000;
        const COM =    0b000010000000000000000000;
        const MEDIA =  0b000000000000100000000000;
        const MENU =   0b00000100000000000000000000000000;
        const BACK =   0b00100000000000000000000000000000;
        const OPTION = 0b00000000000000010000000000000000;
        const ENTER = 0b100000; // FIXME
    }
}

lazy_static! {
    static ref KEYMAPING: HashMap<&'static ZbeKeys, &'static keyboard::Key> = {
        let mut map = HashMap::new();
        map.insert(&ZbeKeys::MAP, &keyboard::Key::A);
        map.insert(&ZbeKeys::NAV, &keyboard::Key::N);
        map.insert(&ZbeKeys::COM, &keyboard::Key::C);
        map.insert(&ZbeKeys::MEDIA, &keyboard::Key::E);
        map.insert(&ZbeKeys::MENU, &keyboard::Key::M);
        map.insert(&ZbeKeys::BACK, &keyboard::Key::Esc);
        map.insert(&ZbeKeys::OPTION, &keyboard::Key::O);
        map.insert(&ZbeKeys::ENTER, &keyboard::Key::Enter);
        map
    };
}

// CAN ID in extended format is 29bit max in extended format
const IDRIVE_CAN_DATA_ID: u32 = 0x25B;
const IUK_CAN_NM3_MSG_ID: u32 = 0x510;
const IUK_CAN_NM3_MSG_PAYLOAD: &'static [u8] =  &[0x40, 0x10, 0x40, 0x00, 0x0F, 0x9F, 0x19, 0x00];
// Not clear what timeout should be, but 1s seems safe
const IUK_CAN_NM3_TIMEOUT: u64 = 1000;
// /usr/include/linux/can.h:#define CAN_SFF_MASK 0x000007FFU /* standard frame format (SFF) */
const CAN_SFF_MASK: u32 = 0x000007FF;
const CAN_IF_ARG: &str = "canif";
const CAN_IF_DEFAULT: &str = "can0";

fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

async fn keepalive(canif:String) {
    let cansock_tx = CANSocket::open(&canif).unwrap();
    let nm3frame = CANFrame::new(IUK_CAN_NM3_MSG_ID, IUK_CAN_NM3_MSG_PAYLOAD, false, false).unwrap();

    loop {
        println!("Writing on {}", canif);
        // why do I need to unwrap the result and '?' doesn't work in a fn?
        let nm3 = cansock_tx.write_frame(nm3frame).unwrap();
        nm3.await;
        println!("Waiting 1 seconds");
        sleep(Duration::from_millis(IUK_CAN_NM3_TIMEOUT)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = Command::new("rusty-zgw")
        .version("0.0.1")
        .about("Creates a uinput device from a BMW idrive controller")
        .author("Brendan Le Foll, brendan@fridu.org")
        .arg(
            Arg::new(CAN_IF_ARG)
                .short('c')
                .long(CAN_IF_ARG) // allow --canif
                .takes_value(true)
                .help("can interface to use")
                .default_value(CAN_IF_DEFAULT)
         )
         .get_matches();

    let canif: String = app.value_of(CAN_IF_ARG).unwrap().to_string();
    println!("Going to use, {}!", canif);
    // wtf? Why can I not clone directly at least?
    let caniflocal = canif.clone();

    tokio::spawn(async move {
        // is good?
        keepalive(caniflocal).await;
    });

    // on arch the udev uinput detection seems brocken in the lib -> need to investigate
    let mut device = uinput_tokio::open("/dev/uinput")
        .unwrap()
        .name("test")
        .unwrap()
        .event(uinput_tokio::event::Keyboard::All)
        .unwrap()
        .create()
        .await
        .unwrap();


    // doesn't support copy so I have to do this twice? Berk
    let mut cansock_rx = CANSocket::open(&canif).unwrap();
    println!("Going to use, {}!", canif);

    let idrive_filter = CANFilter::new(IDRIVE_CAN_DATA_ID, CAN_SFF_MASK).unwrap();
    let can_filters = [idrive_filter];

    cansock_rx.set_filter(&can_filters);

    // why is LOOP not needed?
    // Use BCM to filter messages?
    while let Some(next) = cansock_rx.next().await {
        println!("{:#?}", next);

        // RAW can frame without ID
        let canframe = next.unwrap();
        let mut candata = canframe.data();

        // our CAN data should always be 8 bytes, otherwise ignore it
        if candata.len() == 8 {
            println!("{:X?}", candata);
            let canbitdata = read_be_u64(&mut candata);
            println!("{:X}", canbitdata);
            // truncating from bits doesn't need unwrap
            let derived_data: ZbeKeys = ZbeKeys::from_bits_truncate(canbitdata);

            if KEYMAPING.contains_key(&derived_data) {
                let key = KEYMAPING.get(&derived_data).unwrap();
                device.click(*key).await.unwrap();
            }

            // Not sure why you need to but if you dont sync then udev will wait for 4 chars to arrive and then send
            device.synchronize().await.unwrap();
        }
    }

    Ok(())
}
