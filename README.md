# RustyZBE 

This program interacts using python-can to a BMW ZBE controller connected via a
CAN adapter

## ZBE Controller version

There are many different versions of the BMW idrive controller (a.k.a ZBE). I
have made no real attempt to support older generation controllers, but whilst
the CAN IDs are different, there's no real reason other versions couldn't work
with minimal adaption. Feel free to submit PRs for other ZBE versions, I may
also accept donations of ZBEs in return for software support :D.

The code works currently with the controller found in SP15 vehicles equipped
typically with an NBT evo, the first car of this generation being the G11/G12
7-series.

### ZBE pinout

The pinout for the ZBE is given looking at the controller face/buttons down.

 _________________
|12V GND CANH CANL|
 -----------------

## CAN controller setup

In order to connect to a ZBE you will need a CAN controller. I have used a Peak
System PCAN-USB adapter (IPEH-002021) with a 160ohm terminator. I have no
reason to believe other CAN adapters woudln't work, but I recommend looking for
good quality adapters and avoiding the very cheap TJA1050 adapters. Your
adapter will require support for socketcan in Linux
(https://www.kernel.org/doc/Documentation/networking/can.txt) and support 500k
bitrates (this is very common).

# Shut up and how do I use run this??

```
cargo build
./target/debug/main
```
