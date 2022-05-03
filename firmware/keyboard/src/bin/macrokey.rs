#![no_main]
#![no_std]

use core::convert::Infallible;

use bsp::hal::{self, usb::UsbBus};
use cortex_m::{asm, prelude::*};
use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_time::duration::Extensions as _;
use hal::pac;
use heapless::Deque;
use panic_probe as _;
use rp_pico as bsp;
use usb_device as usbd;
use usbd::{
    class_prelude::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
};

use usbd_hid::{
    descriptor::{KeyboardReport, SerializedDescriptor},
    hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    },
};

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[entry]
fn main() -> ! {
    let mut p = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(p.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        p.XOSC,
        p.CLOCKS,
        p.PLL_SYS,
        p.PLL_USB,
        &mut p.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let timer = hal::Timer::new(p.TIMER, &mut p.RESETS);

    let bus = UsbBus::new(
        p.USBCTRL_REGS,
        p.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut p.RESETS,
    );
    let bus_allocator = UsbBusAllocator::new(bus);
    let vid_pid = UsbVidPid(0x6666, 0x0789);
    let mut hid = HIDClass::new_with_settings(
        &bus_allocator,
        KeyboardReport::desc(),
        10,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Keyboard,
            config: ProtocolModeConfig::ForceReport,
            locale: HidCountryCode::NotSupported,
        },
    );
    let mut dev = UsbDeviceBuilder::new(&bus_allocator, vid_pid)
        .manufacturer("KOBA789")
        .product("RustyKeys")
        .serial_number("789")
        .build();

    let sio = hal::Sio::new(p.SIO);
    let pins = bsp::Pins::new(p.IO_BANK0, p.PADS_BANK0, sio.gpio_bank0, &mut p.RESETS);

    let col1 = pins.gpio18.into_pull_up_input();
    let col2 = pins.gpio17.into_pull_up_input();
    let col3 = pins.gpio16.into_pull_up_input();
    let mut row1 = pins.gpio22.into_push_pull_output();
    let mut row2 = pins.gpio21.into_push_pull_output();
    let cols: &[Column] = &[&col1, &col2, &col3];
    let rows: &mut [Row] = &mut [&mut row1, &mut row2];

    let mut scan_countdown = timer.count_down();
    scan_countdown.start(10.milliseconds());

    let mut macro_queue = Deque::<KeyboardReport, 32>::new();
    let mut is_macro_pressed = false;

    loop {
        dev.poll(&mut [&mut hid]);

        if scan_countdown.wait().is_ok() {
            if let Some(report) = macro_queue.pop_front() {
                hid.push_input(&report).ok();
            } else {
                let state = scan_keys(rows, cols);
                if state[0][0] {
                    if !is_macro_pressed {
                        for report in MACRO_SEQUENCE_KOBA789 {
                            macro_queue.push_back(report.clone()).ok();
                        }
                    }
                    is_macro_pressed = true;
                } else {
                    is_macro_pressed = false;
                }
            }
        }
        // drop received data
        hid.pull_raw_output(&mut [0; 64]).ok();
    }
}

const MACRO_SEQUENCE_KOBA789: &[KeyboardReport] = &[
    KeyboardReport {
        modifier: 0x02, // LEFT_SHIT
        reserved: 0,
        leds: 0,
        keycodes: [0x0, 0, 0, 0, 0, 0],
    },
    KeyboardReport {
        modifier: 0x02, // LEFT_SHIT
        reserved: 0,
        leds: 0,
        keycodes: [0x0e, 0, 0, 0, 0, 0], // K
    },
    KeyboardReport {
        modifier: 0x02, // LEFT_SHIT
        reserved: 0,
        leds: 0,
        keycodes: [0x12, 0, 0, 0, 0, 0], // O
    },
    KeyboardReport {
        modifier: 0x02, // LEFT_SHIT
        reserved: 0,
        leds: 0,
        keycodes: [0x05, 0, 0, 0, 0, 0], // B
    },
    KeyboardReport {
        modifier: 0x02, // LEFT_SHIT
        reserved: 0,
        leds: 0,
        keycodes: [0x04, 0, 0, 0, 0, 0], // A
    },
    KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0x0, 0, 0, 0, 0, 0],
    },
    KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0x24, 0, 0, 0, 0, 0], // 7
    },
    KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0x25, 0, 0, 0, 0, 0], // 8
    },
    KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0x26, 0, 0, 0, 0, 0], // 9
    },
    KeyboardReport {
        modifier: 0,
        reserved: 0,
        leds: 0,
        keycodes: [0x0, 0, 0, 0, 0, 0],
    },
];

pub type Column<'a> = &'a dyn InputPin<Error = Infallible>;
pub type Row<'a> = &'a mut dyn OutputPin<Error = Infallible>;
pub type StateMatrix = [[bool; 3]; 2];

fn scan_keys(rows: &mut [Row], cols: &[Column]) -> StateMatrix {
    let mut matrix = [[false; 3]; 2];
    for (row_pin, row_state) in rows.iter_mut().zip(matrix.iter_mut()) {
        row_pin.set_low().unwrap();
        asm::delay(10);
        for (col_pin, col_state) in cols.iter().zip(row_state.iter_mut()) {
            *col_state = col_pin.is_low().unwrap();
        }
        row_pin.set_high().unwrap();
        asm::delay(10);
    }
    matrix
}
