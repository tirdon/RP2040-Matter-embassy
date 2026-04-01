#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Endpoint, State};
use embassy_time::{Duration, Timer};
use defmt_rtt as _;
use panic_probe as _;

use rs_matter::data_model::device_types::device_type::DeviceType;
use rs_matter::data_model::device_types::identify::Identify;
use rs_matter::data_model::objects::Cluster;
use rs_matter::error::Error;
use rs_matter::fabric::FabricMgr;
use rs_matter::transport::network::NetworkCommissioning;
use rs_matter::transport::udp::UdpServer;
use rs_matter::utils::rand::rand_config::RandConfig;
use rs_matter::utils::rand::rand_core::RngCore;
use rs_matter::utils::rand::thread_rng;
use rs_matter::Matter;

use core::mem::MaybeUninit;

// ----------------------------------------------------------------------------
// 1. Global Allocator (Required for Matter)
// ----------------------------------------------------------------------------
#[global_allocator]
static ALLOCATOR: embedded_alloc::Heap = embedded_alloc::Heap::empty();

// ----------------------------------------------------------------------------
// 2. USB Serial Setup (for defmt logging)
// ----------------------------------------------------------------------------
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Initialize USB
    let usb_state = MaybeUninit::<State>::uninit();
    let usb_state = unsafe { usb_state.assume_init() };
    let (mut _reader, mut writer) = embassy_rp::usb::new_endpoint_pair(
        p.USB,
        usb_state,
        &mut [0u8; 256],
        &mut [0u8; 256],
    );

    // Initialize Allocator
    let mut heap_mem = [0u8; 16384]; // 16KB Heap
    unsafe { ALLOCATOR.init(&mut heap_mem) };

    // Spawn the main logic task
    spawner.spawn(app_task()).unwrap();
}

// ----------------------------------------------------------------------------
// 3. Application Task
// ----------------------------------------------------------------------------
#[embassy_executor::task]
async fn app_task() {
    let p = embassy_rp::init();

    // --- Hardware Setup ---
    let mut led = Output::new(p.PIN_25, Level::High);

    // --- Matter Initialization ---
    let mut rand_config = RandConfig::new();
    let mut rng = thread_rng(&mut rand_config);

    // Create Fabrics
    let mut fabric_mgr = FabricMgr::new();
    let mut fabric_storage = [0u8; 4096];
    let mut fabric_storage = rs_matter::utils::storage::Storage::new(&mut fabric_storage);
    let mut fabric_storage = rs_matter::utils::storage::StorageBackedFabricStorage::new(
        &mut fabric_storage,
    );
    let mut fabric_storage = rs_matter::utils::storage::FabricStorage::new(
        &mut fabric_storage,
        &mut rng,
    );
    let mut fabric_storage = rs_matter::utils::storage::FabricStorage::new(
        &mut fabric_storage,
        &mut rng,
    );

    // Create Matter Instance
    let mut matter_storage = [0u8; 8192];
    let mut matter_storage = rs_matter::utils::storage::Storage::new(&mut matter_storage);
    let mut matter_storage = rs_matter::utils::storage::StorageBackedMatterStorage::new(
        &mut matter_storage,
    );
    let mut matter_storage = rs_matter::utils::storage::MatterStorage::new(
        &mut matter_storage,
        &mut rng,
    );

    let mut matter = Matter::new(
        &mut matter_storage,
        &mut fabric_storage,
        &mut rng,
        None,
    )
    .await
    .unwrap();

    // Add Device Type (Example: On/Off Light)
    let mut device_types = Vec::new();
    device_types.push(DeviceType::OnOfLight);
    matter.add_device_type(&device_types).await.unwrap();

    // Add Clusters
    let mut identify_cluster = Identify::new();
    matter.add_cluster(Cluster::Identify(identify_cluster)).await.unwrap();

    // Add Network Commissioning
    let mut network_commissioning = NetworkCommissioning::new();
    matter.add_cluster(Cluster::NetworkCommissioning(network_commissioning)).await.unwrap();

    // Initialize UDP Server
    let mut udp_server = UdpServer::new().await.unwrap();
    matter.add_transport(Box::new(udp_server)).await.unwrap();

    // Start Matter
    matter.start().await.unwrap();

    // --- Main Loop ---
    loop {
        // Blink LED
        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;
    }
}
