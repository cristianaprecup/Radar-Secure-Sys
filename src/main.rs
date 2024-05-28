#![no_std]
#![no_main]
#![allow(unused)]
pub mod hcsr04;

use core::str::from_utf8;
use core::panic::PanicInfo;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::{peripherals::*, Peripherals};
use embassy_time::{Duration, Timer};
use heapless::String;
use log::{info, warn};
use hcsr04::HCSR04;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, IpAddress, IpEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources};
use byte_slice_cast::AsByteSlice;
use cyw43_pio::PioSpi;
use embassy_rp::pio::{InterruptHandler, Pio};
use static_cell::StaticCell;
use embedded_io_async::Write;
use core::fmt::Write as fmtWrite;


// Channel
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};


// PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm};


// USB driver
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Endpoint, InterruptHandler as USBInterruptHandler};
use embassy_rp::bind_interrupts;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => USBInterruptHandler<USB>;
    // PIO interrupt for CYW SPI communication
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

static HCSR04_READING: Channel<ThreadModeRawMutex, f64, 1> = Channel::new();
static SERVO_TASK: Channel<ThreadModeRawMutex, u64, 1> = Channel::new();
static JSON_CHANNEL: Channel<ThreadModeRawMutex, f64, 1> = Channel::new();
static TOP: u16 = 0x8000;



#[embassy_executor::task]
async fn hcsr04_task(mut ultrasonic: HCSR04) {
    loop {
        // wait a delay before measuring
        Timer::after(Duration::from_millis(400)).await;
        let unit: f64;
        unit = match ultrasonic.measure().await {
            Ok(unit) => unit.centimeters,
            Err(_) => -1.0,
        };

        if unit < 0.0 || unit >= 400.0{ // checks for not panicking
            continue;
        }

        // info!("Distance: {:.2} cm", unit);

        JSON_CHANNEL.send(unit).await;
        HCSR04_READING.send(unit).await;

    }
}


fn pulse_width_from_angle(angle: u64) -> u64 {
    1000 + (angle * 1000) / 180
}

async fn generate_pwm(pwm_output: &mut Output<'_>, pulse_width_us: u64, repeat: u32) {
    for _ in 0..repeat {
        pwm_output.set_high();
        Timer::after(Duration::from_micros(pulse_width_us)).await;
        pwm_output.set_low();
        Timer::after(Duration::from_micros(20_000 - pulse_width_us)).await;
    }
}

#[embassy_executor::task]
async fn servo_task(mut pwm_output: Output<'static>) {
    loop {
        // info!("Rotating 90 degrees to the right");
        let pulse_width_90 = pulse_width_from_angle(230);
        generate_pwm(&mut pwm_output, pulse_width_90, 25).await; 
        stop_servo(&mut pwm_output, 1).await; 
        // info!("Rotating back 90 degrees to the left");
        let pulse_width_0 = pulse_width_from_angle(0);
        generate_pwm(&mut pwm_output, pulse_width_0, 25).await; 
        stop_servo(&mut pwm_output, 0).await; 
        Timer::after_millis(200).await;
    }
}

// The task used by the serial port driver over USB
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

fn float_to_string(f: f64) -> String<32> {
    let mut s = String::<32>::new();
    write!(s, "{:.2}", f).unwrap();
    s
}

fn build_http_response(unit: f64) -> String<256> {
    let mut response = String::<256>::new();

    // app status line and headers
    response.push_str("HTTP/1.1 200 OK\r\n").unwrap();
    response.push_str("Content-Type: application/json\r\n").unwrap();
    response.push_str("Connection: close\r\n").unwrap();
    response.push_str("Access-Control-Allow-Origin: *\r\n").unwrap();  
    response.push_str("\r\n").unwrap();  

    // app jsonbody
    response.push_str("{\"object\": \"detected\", \"distance\": ").unwrap();

    response.push_str(&float_to_string(unit)).unwrap();
    
    response.push_str("}\r\n").unwrap();  

    response
}


#[embassy_executor::task]
async fn send_json(stack: &'static Stack<cyw43::NetDriver<'static>>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));
    
        // info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("Accept error: {:?}", e);
            continue;
        }
    
        info!("Received connection from {:?}", socket.remote_endpoint());
    
        let mut buf = [0; 4096];
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                warn!("Read EOF");
                break;
            },
            Ok(n) => n,
            Err(e) => {
                warn!("Read error: {:?}", e);
                break;
            }
        };
    
        // info!("Received: {}", from_utf8(&buf[..n]).unwrap());
    
        if from_utf8(&buf[..n]).unwrap().starts_with("GET") {
            // respond with a redirect
            let mut unit = JSON_CHANNEL.receive().await;
            // asign the unit to 0 if the channel is empty
            if unit.is_nan() {
                unit = 0.0;
            }
            let mut response = build_http_response(unit);
            info!("Sending response: {}", response.as_str());
            match socket.write_all(response.as_bytes()).await {
                Ok(_) => info!(""),
                Err(e) => warn!("Failed to send redirect response: {:?}", e),
            };
        }

        socket.close();
        Timer::after(Duration::from_millis(100)).await;
    }
}


// const WIFI_NETWORK: &str = "Cristianap";
// const WIFI_PASSWORD: &str = "12345678";
const WIFI_NETWORK: &str = "cristianawifi";
const WIFI_PASSWORD: &str = "12345678";

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());
    let driver = Driver::new(peripherals.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    // Configure the GPIO pin for Servo
    let mut pwm_output = Output::new(peripherals.PIN_16, Level::Low);

    let mut ultrasonic = HCSR04::new(
        peripherals.PIN_21, // TRIGGER  Gpio pin 21
        peripherals.PIN_20, // ECHO Gpio pin 20
    )
    .unwrap();

    // buzzer
    let mut config_pwm: PwmConfig = Default::default();
    config_pwm.top = 0xFFFF;
    config_pwm.compare_b = 0;

    let mut buzzer = Pwm::new_output_b(peripherals.PWM_SLICE3, peripherals.PIN_7, config_pwm.clone());
     
    // LED
    //conf for red LED
    let mut led_red_config: PwmConfig = Default::default();
    led_red_config.top = TOP;
    led_red_config.compare_b = led_red_config.top;

    //conf for blue LED
    let mut led_bg_config: PwmConfig = Default::default();
    led_bg_config.top = TOP;
    led_bg_config.compare_b = 0;
    led_bg_config.compare_a = 0;

    //PWM for red led
    let mut pwm_red = Pwm::new_output_b(peripherals.PWM_SLICE0, peripherals.PIN_1, led_red_config.clone());
    //PWM for blue and green led
    let mut pwm_blue_green = Pwm::new_output_ab(peripherals.PWM_SLICE1, peripherals.PIN_2, peripherals.PIN_3, led_bg_config.clone());
   
     
    led_red_config.compare_b = 0;
    led_bg_config.compare_a = 0;
    led_bg_config.compare_b = TOP;
    pwm_red.set_config(&led_red_config);
    pwm_blue_green.set_config(&led_bg_config);


/* ---------------- wifi form lab ----------*/

    // Link CYW43 firmware
    let fw = include_bytes!("../net_drivers/43439A0.bin");
    let clm = include_bytes!("../net_drivers/43439A0_clm.bin");

    // Init SPI for communication with CYW43
    let pwr = Output::new(peripherals.PIN_23, Level::Low);
    let cs = Output::new(peripherals.PIN_25, Level::High);
    let mut pio = Pio::new(peripherals.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        peripherals.PIN_24,
        peripherals.PIN_29,
        peripherals.DMA_CH0,
    );

    // Start Wi-Fi task
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    // Init the device
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let config = Config::dhcpv4(Default::default());
    

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef;

    // Init network stack
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    // Start network stack task
    spawner.spawn(net_task(stack)).unwrap();


    loop {
        // Join WPA2 access point
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status {}", err.status);
            }
        }
    }

    // Wait for DHCP (not necessary when using static IP)
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up {:?}!", stack.config_v4());

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];
    /* ---------------- */

    spawner.spawn(send_json(stack)).unwrap();
    spawner.spawn(servo_task(pwm_output)).unwrap();
    spawner.spawn(hcsr04_task(ultrasonic)).unwrap();
    loop {

        // check if dist sens has a value
        let mut buzzer_on = 0.0;
        buzzer_on = HCSR04_READING.receive().await;
        if buzzer_on > 0.0 && buzzer_on <= 30.0 {

            // buzzer
            let high_freq = 2000; 
            let low_freq = 5; 
    
            let alarm_pattern = [high_freq, low_freq];
            let pattern_duration = 50 + (buzzer_on as u64) * 2; 
            for _ in 0..3 { 
                for &freq in &alarm_pattern {
                    config_pwm.compare_b = config_pwm.top / freq; 
                    buzzer.set_config(&config_pwm);
                    Timer::after(Duration::from_millis(pattern_duration)).await;
                }
            }

            if buzzer_on <= 10.0 && buzzer_on > 0.0 {
                //red
                led_red_config.compare_b = TOP;
                led_bg_config.compare_a = 0;
                led_bg_config.compare_b = 0;
                pwm_red.set_config(&led_red_config);
                pwm_blue_green.set_config(&led_bg_config);
            } else {
                //yellow
                led_red_config.compare_b = TOP;
                led_bg_config.compare_a = 0;
                led_bg_config.compare_b = TOP;
                pwm_red.set_config(&led_red_config);
                pwm_blue_green.set_config(&led_bg_config);
            }
    
        } else {
            //off
            config_pwm.compare_b = 0;
            buzzer.set_config(&config_pwm);
            Timer::after(Duration::from_millis(75)).await;
            led_red_config.compare_b = 0;
            led_bg_config.compare_a = 0;
            led_bg_config.compare_b = TOP;
            pwm_red.set_config(&led_red_config);
            pwm_blue_green.set_config(&led_bg_config);
        }

        
        Timer::after(Duration::from_millis(10)).await;
    }
}

async fn stop_servo(pwm_output: &mut Output<'_>, duration_secs: u64) {
    // info!("Stopping servo");
    if duration_secs == 0 {
        for _ in 0..25 {
            pwm_output.set_high();
            Timer::after(Duration::from_micros(1500)).await; 
            pwm_output.set_low();
            Timer::after(Duration::from_micros(18_500)).await; 
        }
    }
    for _ in 0..(duration_secs * 50) {
        pwm_output.set_high();
        Timer::after(Duration::from_micros(1500)).await; 
        pwm_output.set_low();
        Timer::after(Duration::from_micros(18_500)).await; 
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
