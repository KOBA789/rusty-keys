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
use panic_probe as _;
use rp_pico as bsp;
use usb_device as usbd;
use usbd::{
    class_prelude::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
};

use usbd_hid::{
    descriptor::{MouseReport, SerializedDescriptor},
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
        MouseReport::desc(),
        10,
        HidClassSettings {
            subclass: HidSubClass::NoSubClass,
            protocol: HidProtocol::Mouse,
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

    loop {
        dev.poll(&mut [&mut hid]);

        if scan_countdown.wait().is_ok() {
            let state = scan_keys(rows, cols);
            let report = build_report(&state);
            hid.push_input(&report).ok();
        }
        // drop received data
        hid.pull_raw_output(&mut [0; 64]).ok();
    }
}

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

fn build_report(matrix: &StateMatrix) -> MouseReport {
    let mut buttons = 0;
    let mut x = 0;
    let mut y = 0;

    if matrix[1][0] {
        x -= 5;
    }
    if matrix[1][2] {
        x += 5;
    }
    if matrix[0][1] {
        y -= 5;
    }
    if matrix[1][1] {
        y += 5;
    }
    if matrix[0][0] {
        buttons += 0b1 << 0; // LEFT CLICK
    }
    if matrix[0][2] {
        buttons += 0b1 << 1; // RIGHT CLICK
    }

    MouseReport {
        buttons,
        x,
        y,
        wheel: 0,
        pan: 0,
    }
}
