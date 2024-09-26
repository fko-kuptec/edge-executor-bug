use std::net::{TcpListener, TcpStream};

use async_io_mini::Async;
use embassy_time::Timer;
use esp_idf_hal::task::block_on;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{self, AccessPointConfiguration, BlockingWifi, EspWifi},
};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use futures_util::task::SpawnExt;

const SSID: &str = "esp-animation";
const PASSWORD: &str = "hello-world";

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Enable the use of the `eventfd` syscall for async runtimes
    let _eventfd = esp_idf_svc::io::vfs::MountedEventfs::mount(5).unwrap();

    // Get peripherals
    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // Initialize wifi peripheral
    let sysloop = EspSystemEventLoop::take().unwrap();
    let partition = EspDefaultNvsPartition::take().unwrap();
    let wifi = EspWifi::new(peripherals.modem, sysloop.clone(), Some(partition))
        .expect("failed to initialize WIFI");
    let mut wifi = BlockingWifi::wrap(wifi, sysloop).unwrap();

    // Start WIFI network
    wifi.set_configuration(&wifi::Configuration::AccessPoint(
        AccessPointConfiguration {
            ssid: SSID.try_into().unwrap(),
            auth_method: wifi::AuthMethod::WPA2Personal,
            password: PASSWORD.try_into().unwrap(),
            ..Default::default()
        },
    ))
    .expect("failed to configure WIFI");
    wifi.start().expect("failed to start WIFI");

    // Run one of the HTTP server variants
    http_edge_executor().unwrap(); // ❌
                                   //http_async_executor().unwrap(); // ❌
                                   //http_emwait().unwrap(); // ✔️
                                   //http_futures_executor().unwrap(); // ✔️
}

fn http_edge_executor() -> anyhow::Result<()> {
    let executor: edge_executor::LocalExecutor = Default::default();
    let task = async {
        let socket = Async::<TcpListener>::bind(([0, 0, 0, 0], 80))?;
        log::info!("[HTTP] server started");

        loop {
            let (stream, address) = socket.accept().await?;
            log::info!("[HTTP] request from {address}");
            executor
                .spawn(async move {
                    if let Err(error) = handle_client(stream).await {
                        log::error!("[HTTP] communication with {address} failed: {error}");
                    }
                })
                .detach();
        }
    };

    block_on(executor.run(task))
}

fn http_async_executor() -> anyhow::Result<()> {
    let executor: async_executor::LocalExecutor = Default::default();
    let task = async {
        let socket = Async::<TcpListener>::bind(([0, 0, 0, 0], 80))?;
        log::info!("[HTTP] server started");

        loop {
            let (stream, address) = socket.accept().await?;
            log::info!("[HTTP] request from {address}");
            executor
                .spawn(async move {
                    if let Err(error) = handle_client(stream).await {
                        log::error!("[HTTP] communication with {address} failed: {error}");
                    }
                })
                .detach();
        }
    };

    block_on(executor.run(task))
}

/*fn http_emwait() -> anyhow::Result<()> {
    let executor: emwait::LocalExecutor = Default::default();
    let task = async {
        let socket = Async::<TcpListener>::bind(([0, 0, 0, 0], 80))?;
        log::info!("[HTTP] server started");

        loop {
            let (stream, address) = socket.accept().await?;
            log::info!("[HTTP] request from {address}");
            executor.spawn(async move {
                if let Err(error) = handle_client(stream).await {
                    log::error!("[HTTP] communication with {address} failed: {error}");
                }
            });
        }
    };

    block_on(executor.run_until(task))
}*/

fn http_futures_executor() -> anyhow::Result<()> {
    let mut executor: futures_executor::LocalPool = Default::default();
    let spawner = executor.spawner();
    let task = async {
        let socket = Async::<TcpListener>::bind(([0, 0, 0, 0], 80))?;
        log::info!("[HTTP] server started");

        loop {
            let (stream, address) = socket.accept().await?;
            log::info!("[HTTP] request from {address}");
            spawner
                .spawn(async move {
                    if let Err(error) = handle_client(stream).await {
                        log::error!("[HTTP] communication with {address} failed: {error}");
                    }
                })
                .unwrap();
        }
    };

    executor.run_until(task)
}

async fn handle_client(mut stream: Async<TcpStream>) -> anyhow::Result<()> {
    // Consume complete request until the end of the headers
    let mut buffer = vec![0; 2048];
    let mut index = 0;
    while let Ok(len) = stream.read(&mut buffer[index..]).await {
        if len == 0 {
            break;
        } else {
            log::info!("[HTTP] received {len} bytes");
            index += len;

            // Try to find the end of the headers
            if let Ok(request) = std::str::from_utf8(&buffer[..index]) {
                if request.contains("\r\n\r\n") {
                    break;
                }
            }
        }
    }

    log::info!("[HTTP] request received");

    // Send MJPEG response header
    stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
    stream
        .write_all(b"CacheControl: no-cache, no-store, must-revalidate, max-age=0\r\n")
        .await?;
    stream.write_all(b"Pragma: no-cache\r\n").await?;
    stream.write_all(b"Expires: 0\r\n").await?;
    stream.write_all(b"Age: 0\r\n").await?;
    stream
        .write_all(b"Content-Type: multipart/x-mixed-replace; boundary=\"000.000.000\"\r\n")
        .await?;
    stream.write_all(b"\r\n").await?;

    // Send JPEG frames
    for frame in std::iter::repeat(FRAMES).flatten() {
        // Boundary line
        stream.write_all(b"\r\n--000.000.000\r\n").await?;

        // Frame header
        stream.write_all(b"Content-Type: image/jpeg\r\n").await?;
        stream
            .write_all(format!("Content-Length: {}\r\n", frame.len()).as_bytes())
            .await?;
        stream.write_all(b"\r\n").await?;

        // Frame body
        stream.write_all(frame).await?;

        // Frame time
        Timer::after(embassy_time::Duration::from_millis(40)).await;
    }

    Ok(())
}

static FRAMES: [&[u8]; 18] = [
    include_bytes!("../animation/frame00.jpg"),
    include_bytes!("../animation/frame01.jpg"),
    include_bytes!("../animation/frame02.jpg"),
    include_bytes!("../animation/frame03.jpg"),
    include_bytes!("../animation/frame04.jpg"),
    include_bytes!("../animation/frame05.jpg"),
    include_bytes!("../animation/frame06.jpg"),
    include_bytes!("../animation/frame07.jpg"),
    include_bytes!("../animation/frame08.jpg"),
    include_bytes!("../animation/frame09.jpg"),
    include_bytes!("../animation/frame10.jpg"),
    include_bytes!("../animation/frame11.jpg"),
    include_bytes!("../animation/frame12.jpg"),
    include_bytes!("../animation/frame13.jpg"),
    include_bytes!("../animation/frame14.jpg"),
    include_bytes!("../animation/frame15.jpg"),
    include_bytes!("../animation/frame16.jpg"),
    include_bytes!("../animation/frame17.jpg"),
];
