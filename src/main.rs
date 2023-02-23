#![no_std]
#![no_main]

use esp32c3_hal::{
    clock::ClockControl,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    Delay,
    Rtc,
};
use esp_backtrace as _;

use smart_leds::SmartLedsWrite;
use smart_leds::RGB8;
use smart_leds::brightness;
use ws2812_timer_delay as ws2812;
const NUM_LEDS: usize = 1;

#[entry]
fn main() -> ! 
{
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the watchdog timers. For the ESP32-C3, this includes the Super WDT,
    // the RTC WDT, and the TIMG WDTs.
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Set GPIO5 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio5.into_push_pull_output();
    let neopixel_pin = io.pins.gpio8.into_push_pull_output();
    let mut neopixel = ws2812::Ws2812::new(timer_group0.timer0, neopixel_pin);
    let mut data = [RGB8::default(); NUM_LEDS];

    led.set_high().unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);
    fn wheel(mut wheel_pos: u8) -> RGB8 {
        wheel_pos = 255 - wheel_pos;
        if wheel_pos < 85 {
            return (255 - wheel_pos * 3, 0, wheel_pos * 3).into()
        }
        if wheel_pos < 170 {
            wheel_pos -=85;
            return (0, wheel_pos * 3, 255 - wheel_pos * 3).into()
        }
        wheel_pos -= 170;
        (wheel_pos*3, 255 - wheel_pos * 3, 0).into()
    }
    loop {
        led.toggle().unwrap();
        for j in 0..(256*5) 
        {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
        neopixel.write(brightness(data.iter().cloned(), 32)).unwrap();
        delay.delay_ms(5u8);
        }
        delay.delay_ms(100u32);
    }
}