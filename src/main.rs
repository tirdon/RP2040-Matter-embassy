#![no_std]
#![no_main]

use cyw43::aligned_bytes;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap;
use static_cell::StaticCell;
use {defmt::unwrap, panic_rtt_target as _};

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

type Bus = cyw43_pio::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>;

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Bus>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    
    // CYW43 setup for LED control on Pico W
    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    let btfw = aligned_bytes!("../cyw43-firmware/43439A0_btfw.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    
    // PioSpi::new signature: (common, sm, freq, irq, cs, mosi, clk, dma)
    // Frequency is FixedU32, 1MHz is a safe default.
    let spi = PioSpi::new(
        &mut pio.common, 
        pio.sm0, 
        embassy_rp::pio::Fixed::from_bits(0x01000), // Approximate 1MHz if bits are scaled
        pio.irq0, 
        cs, 
        p.PIN_24, 
        p.PIN_29, 
        p.DMA_CH0
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, btfw).await;
    
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    defmt::info!("Hello World! Blinking LED...");

    loop {
        control.gpio_set(0, true).await; // LED On
        Timer::after(Duration::from_millis(500)).await;
        control.gpio_set(0, false).await; // LED Off
        Timer::after(Duration::from_millis(500)).await;
    }
}
