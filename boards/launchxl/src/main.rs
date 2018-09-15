#![no_std]
#![no_main]
#![feature(lang_items, asm)]

extern crate capsules;

extern crate cc26x2;

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;

use cc26x2::aon;
use cc26x2::prcm;

#[macro_use]
pub mod io;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 3;
static mut PROCESSES: [Option<&'static mut kernel::procs::Process<'static>>; NUM_PROCS] =
    [None, None, None];

#[link_section = ".app_memory"]
// Give half of RAM to be dedicated APP memory
static mut APP_MEMORY: [u8; 0xA000] = [0; 0xA000];

pub struct Platform {
    gpio: &'static capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
    console: &'static capsules::console::Console<'static, cc26x2::uart::UART>,
    button: &'static capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
    >,
    rng: &'static capsules::rng::SimpleRng<'static, cc26x2::trng::Trng>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            _ => f(None),
        }
    }
}

unsafe fn configure_pins() {
    cc26x2::gpio::PORT[0].enable_gpio();
    cc26x2::gpio::PORT[1].enable_gpio();

    cc26x2::gpio::PORT[2].enable_uart_rx();
    cc26x2::gpio::PORT[3].enable_uart_tx();

    cc26x2::gpio::PORT[4].enable_i2c_scl();
    cc26x2::gpio::PORT[5].enable_i2c_sda();

    cc26x2::gpio::PORT[6].enable_gpio(); // Red LED
    cc26x2::gpio::PORT[7].enable_gpio(); // Green LED

    // SPI MISO cc26x2::gpio::PORT[8]
    // SPI MOSI cc26x2::gpio::PORT[9]
    // SPI CLK  cc26x2::gpio::PORT[10]
    // SPI CS   cc26x2::gpio::PORT[11]

    // PWM      cc26x2::gpio::PORT[12]

    cc26x2::gpio::PORT[13].enable_gpio();
    cc26x2::gpio::PORT[14].enable_gpio();

    cc26x2::gpio::PORT[15].enable_gpio();

    // unused   cc26x2::gpio::PORT[16]
    // unused   cc26x2::gpio::PORT[17]

    // PWM      cc26x2::gpio::PORT[18]
    // PWM      cc26x2::gpio::PORT[19]
    // PWM      cc26x2::gpio::PORT[20]

    cc26x2::gpio::PORT[21].enable_gpio();
    cc26x2::gpio::PORT[22].enable_gpio();

    // analog   cc26x2::gpio::PORT[23]
    // analog   cc26x2::gpio::PORT[24]
    // analog   cc26x2::gpio::PORT[25]
    // analog   cc26x2::gpio::PORT[26]
    // analog   cc26x2::gpio::PORT[27]
    // analog   cc26x2::gpio::PORT[28]
    // analog   cc26x2::gpio::PORT[29]
    // analog   cc26x2::gpio::PORT[30]
}

#[no_mangle]
pub unsafe fn reset_handler() {
    cc26x2::init();

    // Setup AON event defaults
    aon::AON.setup();

    // Power on peripherals (eg. GPIO)
    prcm::Power::enable_domain(prcm::PowerDomain::Peripherals);

    // Wait for it to turn on until we continue
    while !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals) {}

    // Enable the GPIO clocks
    prcm::Clock::enable_gpio();
    configure_pins();

    // LEDs
    let led_pins = static_init!(
        [(
            &'static cc26x2::gpio::GPIOPin,
            capsules::led::ActivationMode
        ); 2],
        [
            (
                &cc26x2::gpio::PORT[6],
                capsules::led::ActivationMode::ActiveHigh
            ), // Red
            (
                &cc26x2::gpio::PORT[7],
                capsules::led::ActivationMode::ActiveHigh
            ), // Green
        ]
    );
    let led = static_init!(
        capsules::led::LED<'static, cc26x2::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // BUTTONS
    let button_pins = static_init!(
        [(&'static cc26x2::gpio::GPIOPin, capsules::button::GpioMode); 2],
        [
            (
                &cc26x2::gpio::PORT[13],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 2
            (
                &cc26x2::gpio::PORT[14],
                capsules::button::GpioMode::LowWhenPressed
            ), // Button 1
        ]
    );
    let button = static_init!(
        capsules::button::Button<'static, cc26x2::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );
    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    // UART
    let console = static_init!(
        capsules::console::Console<cc26x2::uart::UART>,
        capsules::console::Console::new(
            &cc26x2::uart::UART0,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    kernel::hil::uart::UART::set_client(&cc26x2::uart::UART0, console);
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(console), kc);

    // Setup for remaining GPIO pins
    let gpio_pins = static_init!(
        [&'static cc26x2::gpio::GPIOPin; 22],
        [
            &cc26x2::gpio::PORT[1],
            &cc26x2::gpio::PORT[5],
            &cc26x2::gpio::PORT[8],
            &cc26x2::gpio::PORT[9],
            &cc26x2::gpio::PORT[10],
            &cc26x2::gpio::PORT[11],
            &cc26x2::gpio::PORT[12],
            &cc26x2::gpio::PORT[15],
            &cc26x2::gpio::PORT[16],
            &cc26x2::gpio::PORT[17],
            &cc26x2::gpio::PORT[18],
            &cc26x2::gpio::PORT[19],
            &cc26x2::gpio::PORT[20],
            &cc26x2::gpio::PORT[21],
            &cc26x2::gpio::PORT[22],
            &cc26x2::gpio::PORT[23],
            &cc26x2::gpio::PORT[24],
            &cc26x2::gpio::PORT[25],
            &cc26x2::gpio::PORT[26],
            &cc26x2::gpio::PORT[27],
            &cc26x2::gpio::PORT[30],
            &cc26x2::gpio::PORT[31],
        ]
    );
    let gpio = static_init!(
        capsules::gpio::GPIO<'static, cc26x2::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let rtc = &cc26x2::rtc::RTC;
    rtc.start();

    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&cc26x2::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    let virtual_alarm1 = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, cc26x2::rtc::Rtc>,
        >,
        capsules::alarm::AlarmDriver::new(virtual_alarm1, kernel::Grant::create())
    );
    virtual_alarm1.set_client(alarm);

    let rng = static_init!(
        capsules::rng::SimpleRng<'static, cc26x2::trng::Trng>,
        capsules::rng::SimpleRng::new(&cc26x2::trng::TRNG, kernel::Grant::create())
    );
    cc26x2::trng::TRNG.set_client(rng);

    let launchxl = Platform {
        console,
        gpio,
        led,
        button,
        alarm,
        rng,
    };

    let mut chip = cc26x2::chip::Cc26X2::new();

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );

    kernel::main(
        &launchxl,
        &mut chip,
        &mut PROCESSES,
        Some(&kernel::ipc::IPC::new()),
    );
}
