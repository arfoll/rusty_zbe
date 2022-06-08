use futures_util::stream::StreamExt;
use tokio::time::{sleep, Duration};
use tokio_socketcan::{CANFilter, CANFrame, CANSocket, Error};

// Random constants
// CAN ID in extended format is 29bit max in extended format
const IDRIVE_CAN_DATA_ID: u32 = 0x25B;
const IUK_CAN_NM3_MSG_ID: u32 = 0x010;
const IUK_CAN_NM3_MSG_PAYLOAD: &'static [u8] =  &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
// /usr/include/linux/can.h:#define CAN_SFF_MASK 0x000007FFU /* standard frame format (SFF) */
const CAN_SFF_MASK: u32 = 0x000007FF;

async fn keepalive(canif:String) {
    let cansock_tx = CANSocket::open(&canif).unwrap();
    let nm3frame = CANFrame::new(IDRIVE_CAN_DATA_ID, IUK_CAN_NM3_MSG_PAYLOAD, false, false).unwrap();

    loop {
        println!("Writing on vcan0");
        // why do I need to unwrap the result and '?' doesn't work in a fn?
        let nm3 = cansock_tx.write_frame(nm3frame).unwrap();
        nm3.await;
        sleep(Duration::from_millis(1000)).await;
        println!("Waiting 3 seconds");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let canif = "vcan0";

    tokio::spawn(async move {
        // is good?
        keepalive(canif.to_string()).await;
    });

    // doesn't support copy so I have to do this twice? Berk
    let mut cansock_rx = CANSocket::open(&canif).unwrap();

    let idrive_filter = CANFilter::new(IDRIVE_CAN_DATA_ID, CAN_SFF_MASK).unwrap();
    let can_filters = [idrive_filter];

    cansock_rx.set_filter(&can_filters);

// why is LOOP not needed?
// Use BCM to filter messages?
    while let Some(next) = cansock_rx.next().await {
        println!("{:#?}", next);
    }

    Ok(())
}
