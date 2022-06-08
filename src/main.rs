//use futures_timer::Delay;
//use std::time::Duration;
use futures_util::stream::StreamExt;
use tokio::time::{sleep, Duration};
use tokio_socketcan::{CANFrame, CANSocket, Error};

async fn keepalive(canif:String) {
    let cansock_tx = CANSocket::open(&canif).unwrap();
    let nm3frame = CANFrame::new(0x1, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], false, false).unwrap();

    loop {
        println!("Writing on vcan0");
        // why do I need to unwrap the result and '?' doesn't work in a fn?
        let nm3 = cansock_tx.write_frame(nm3frame).unwrap();
        nm3.await;
        sleep(Duration::from_millis(1000)).await;
        println!("Waiting 3 seconds");
        //Delay::new(Duration::from_secs(3)).await;
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

// LOOP not needed!
// Use BCM to filter messages?
    loop {
//        let frame = CANFrame::new(0xFF, &[0], false, false).unwrap();
//        cansock.write_frame(frame)?.await?;
//        sleep(Duration::from_millis(1000)).await;
        while let Some(next) = cansock_rx.next().await {
            println!("{:#?}", next);
        }
        println!("endofloop!!!");
    }
}
