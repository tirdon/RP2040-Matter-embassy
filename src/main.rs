//! Simple Matter On/Off Light for Raspberry Pi Pico W.
//!
//! Uses WiFi as the main transport and BLE for commissioning.
//! Based on the rs-matter-embassy rp example.

#![no_std]
#![no_main]
#![recursion_limit = "256"]

use core::mem::MaybeUninit;
use core::pin::pin;
use core::ptr::addr_of_mut;

#[cfg(not(feature = "skip-cyw43-firmware"))]
use cyw43::{aligned_bytes, Aligned, A4};
use embassy_executor::Spawner;

use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::dma;
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::InterruptHandler;

use embedded_alloc::LlffHeap;

use panic_rtt_target as _;

use defmt::{info, unwrap};

use rs_matter_embassy::epoch::epoch;
use rs_matter_embassy::matter::crypto::{default_crypto, Crypto};
use rs_matter_embassy::matter::dm::clusters::desc::{self, ClusterHandler as _};
use rs_matter_embassy::matter::dm::clusters::on_off::test::TestOnOffDeviceLogic;
use rs_matter_embassy::matter::dm::clusters::on_off::{self, OnOffHooks};
use rs_matter_embassy::matter::dm::devices::test::{
    DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET,
};
use rs_matter_embassy::matter::dm::devices::DEV_TYPE_ON_OFF_LIGHT;
use rs_matter_embassy::matter::dm::{Async, Dataver, EmptyHandler, Endpoint, EpClMatcher, Node};
use rs_matter_embassy::matter::utils::init::InitMaybeUninit;
use rs_matter_embassy::matter::{clusters, devices};
use rs_matter_embassy::stack::persist::DummyKvBlobStore;
use rs_matter_embassy::stack::rand::reseeding_csprng;
use rs_matter_embassy::wireless::rp::RpWifiDriver;
use rs_matter_embassy::wireless::{EmbassyWifi, EmbassyWifiMatterStack};

macro_rules! mk_static {
    ($t:ty) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        STATIC_CELL.uninit()
    }};
    ($t:ty,$val:expr) => {{
        mk_static!($t).write($val)
    }};
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

/// Memory for the rs-matter-stack bump allocator.
/// Increase if you get panics during stack init.
const BUMP_SIZE: usize = 16500;

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

const LOG_RINGBUF_SIZE: usize = 2048;

/// Endpoint 0 is the root (Matter system clusters), so our light is on Endpoint 1.
const LIGHT_ENDPOINT_ID: u16 = 1;

/// The Matter Light device Node
const NODE: Node = Node {
    id: 0,
    endpoints: &[
        EmbassyWifiMatterStack::<0, ()>::root_endpoint(),
        Endpoint {
            id: LIGHT_ENDPOINT_ID,
            device_types: devices!(DEV_TYPE_ON_OFF_LIGHT),
            clusters: clusters!(desc::DescHandler::CLUSTER, TestOnOffDeviceLogic::CLUSTER),
        },
    ],
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Heap needed by the x509 crate used inside rs-matter
    {
        const HEAP_SIZE: usize = 8192;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());

    rtt_target::rtt_init_defmt!(rtt_target::ChannelMode::NoBlockSkip, LOG_RINGBUF_SIZE);

    info!("Starting Matter light...");

    // CYW43 firmware blobs (auto-downloaded by build.rs)
    #[cfg(feature = "skip-cyw43-firmware")]
    let (fw, clm, btfw, nvram) = (
        Option::<&Aligned<A4, [u8]>>::None,
        Option::<&Aligned<A4, [u8]>>::None,
        Option::<&Aligned<A4, [u8]>>::None,
        Option::<&Aligned<A4, [u8]>>::None,
    );

    #[cfg(not(feature = "skip-cyw43-firmware"))]
    let (fw, clm, btfw, nvram) = (
        Option::<&Aligned<A4, [u8]>>::Some(aligned_bytes!("../cyw43-firmware/43439A0.bin")),
        Option::<&Aligned<A4, [u8]>>::Some(aligned_bytes!("../cyw43-firmware/43439A0_clm.bin")),
        Option::<&Aligned<A4, [u8]>>::Some(aligned_bytes!("../cyw43-firmware/43439A0_btfw.bin")),
        Option::<&Aligned<A4, [u8]>>::Some(aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin")),
    );

    // Allocate the Matter stack statically (~35-50KB footprint)
    let stack = mk_static!(EmbassyWifiMatterStack<BUMP_SIZE, ()>).init_with(
        EmbassyWifiMatterStack::init(&TEST_DEV_DET, TEST_DEV_COMM, &TEST_DEV_ATT, epoch),
    );

    // Crypto provider using the RP2040 ROSC TRNG
    let crypto = default_crypto(reseeding_csprng(RoscRng, 1000).unwrap(), DAC_PRIVKEY);
    let mut weak_rand = crypto.weak_rand().unwrap();

    // On/Off light cluster
    let on_off = on_off::OnOffHandler::new_standalone(
        Dataver::new_rand(&mut weak_rand),
        LIGHT_ENDPOINT_ID,
        TestOnOffDeviceLogic::new(true),
    );

    // Chain endpoint cluster handlers
    let handler = EmptyHandler
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(TestOnOffDeviceLogic::CLUSTER.id),
            ),
            on_off::HandlerAsyncAdaptor(&on_off),
        )
        .chain(
            EpClMatcher::new(Some(LIGHT_ENDPOINT_ID), Some(desc::DescHandler::CLUSTER.id)),
            Async(desc::DescHandler::new(Dataver::new_rand(&mut weak_rand)).adapt()),
        );

    let persist = stack
        .create_persist_with_comm_window(&crypto, DummyKvBlobStore)
        .await
        .unwrap();

    // Run the Matter stack with WiFi + BLE coexistence
    let matter = pin!(stack.run_coex(
        EmbassyWifi::new(
            RpWifiDriver::new(
                p.PIN_23, p.PIN_25, p.PIN_24, p.PIN_29, p.DMA_CH0, p.PIO0, Irqs, fw, clm, btfw,
                nvram,
            ),
            crypto.rand().unwrap(),
            true,
            stack,
        ),
        &persist,
        &crypto,
        (NODE, handler),
        (),
    ));

    unwrap!(matter.await);
}
