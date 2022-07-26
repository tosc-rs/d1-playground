#![no_std]
#![no_main]

use panic_halt as _;

mod de;

use d1_playground::timer::{Timers, Timer, TimerSource, TimerPrescaler, TimerMode};

struct Uart(d1_pac::UART0);
static mut PRINTER: Option<Uart> = None;
impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            self.0.thr().write(|w| unsafe { w.thr().bits(*byte) });
            while self.0.usr.read().tfnf().bit_is_clear() {}
        }
        Ok(())
    }
}
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        PRINTER.as_mut().unwrap().write_fmt(args).ok();
    }
}
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(core::format_args!($($arg)*));
    }
}
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::_print(core::format_args!($($arg)*));
        $crate::print!("\r\n");
    }
}

#[riscv_rt::entry]
fn main() -> ! {
    let p = d1_pac::Peripherals::take().unwrap();

    // Enable UART0 clock.
    let ccu = &p.CCU;
    ccu.uart_bgr.write(|w| w.uart0_gating().pass().uart0_rst().deassert());

    // Set PC1 LED to output.
    let gpio = &p.GPIO;
    gpio.pc_cfg0.write(|w| {
        w.pc1_select().output()
        .pc0_select().ledc_do()
    });

    // Set PB8 and PB9 to function 6, UART0, internal pullup.
    gpio.pb_cfg1.write(|w| w.pb8_select().uart0_tx().pb9_select().uart0_rx());
    gpio.pb_pull0.write(|w| w.pc8_pull().pull_up().pc9_pull().pull_up());

    // Configure UART0 for 115200 8n1.
    // By default APB1 is 24MHz, use divisor 13 for 115200.
    let uart0 = p.UART0;
    uart0.mcr.write(|w| unsafe { w.bits(0) });
    uart0.fcr().write(|w| w.fifoe().set_bit());
    uart0.halt.write(|w| w.halt_tx().enabled());
    uart0.lcr.write(|w| w.dlab().divisor_latch());
    uart0.dll().write(|w| unsafe { w.dll().bits(13)});
    uart0.dlh().write(|w| unsafe { w.dlh().bits(0) });
    uart0.lcr.write(|w| w.dlab().rx_buffer().dls().eight());
    uart0.halt.write(|w| w.halt_tx().disabled());
    unsafe { PRINTER = Some(Uart(uart0)) };

    // Set up timers
    let Timers { mut timer0, mut timer1, .. } = Timers::new(p.TIMER);

    timer0.set_source(TimerSource::OSC24_M);
    timer1.set_source(TimerSource::OSC24_M);

    timer0.set_prescaler(TimerPrescaler::P8);  // 24M / 8:  3.00M ticks/s
    timer1.set_prescaler(TimerPrescaler::P32); // 24M / 32: 0.75M ticks/s

    timer0.set_mode(TimerMode::SINGLE_COUNTING);
    timer1.set_mode(TimerMode::SINGLE_COUNTING);

    // Blink LED
    loop { unsafe {
        println!("Hello, world!");

        // Start both counters for 3M ticks: that's 1s for timer 0
        // and 4s for timer 1.
        timer0.start_counter(3_000_000);
        timer1.start_counter(3_000_000);
        gpio.pc_dat.write(|w| w.bits(2));

        while timer0.current_value() != 0 { }

        gpio.pc_dat.write(|w| w.bits(0));

        while timer1.current_value() != 0 { }
    }}
}
