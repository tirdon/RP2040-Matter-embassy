#![no_std]
#![no_main]

use cyw43::aligned_bytes;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use embedded_alloc::LlffHeap;
use static_cell::StaticCell;
use {panic_rtt_target as _};

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

// Bind PIO and DMA interrupts for the CYW43 SPI interface
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

// Type alias for the CYW43 SPI bus (PIO-based bit-bang SPI)
type Bus = cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>;

// Background task that drives the CYW43 WiFi chip event loop
#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Bus>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize a small heap allocator (1KB, enough for blink demo)
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());

    // Load CYW43 firmware blobs (auto-downloaded by build.rs on first build)
    let fw = aligned_bytes!("../cyw43-firmware/43439A0.bin");       // WiFi firmware
    let clm = aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");  // CLM (regulatory) data
    let nvram = aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin"); // Board-specific NVRAM config

    // Configure CYW43 control pins and PIO-based SPI
    let pwr = Output::new(p.PIN_23, Level::Low);   // WL_ON: power control
    let cs = Output::new(p.PIN_25, Level::High);    // SPI chip select
    let mut pio = Pio::new(p.PIO0, Irqs);

    let dma = dma::Channel::new(p.DMA_CH0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        cyw43_pio::DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,   // SPI data/MOSI
        p.PIN_29,   // SPI clock
        dma,
    );

    // Initialize CYW43 driver — the Pico W LED is controlled through this chip
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;

    // Spawn the CYW43 event loop as a background task
    spawner.spawn(wifi_task(runner).unwrap());

    // Finish CYW43 init with CLM data and enable power saving
    control.init(clm).await;
    control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

    // GPIO17 held low, GPIO16 toggled alongside LED
    let _gpio17 = Output::new(p.PIN_16, Level::Low);
    let mut gpio16 = Output::new(p.PIN_17, Level::Low);

    defmt::info!("Hello World! Blinking LED...");

    // Blink the onboard LED (active high on CYW43 GPIO 0) and toggle GPIO17
    let mut count: u32 = 0;
    loop {
        control.gpio_set(0, true).await;
        gpio16.set_high();
        defmt::info!("LED on  ({})", count);
        Timer::after(Duration::from_millis(500)).await;
        control.gpio_set(0, false).await;
        gpio16.set_low();
        defmt::info!("LED off ({})", count);
        Timer::after(Duration::from_millis(500)).await;
        count += 1;
    }
}
