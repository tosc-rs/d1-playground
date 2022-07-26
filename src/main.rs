#![no_std]
#![no_main]

use panic_halt as _;

mod de;

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

    // Blink LED
    loop { unsafe {
        println!("Hello, world!");

        gpio.pc_dat.write(|w| w.bits(2));

        riscv::asm::delay(100_000_000);

        gpio.pc_dat.write(|w| w.bits(0));

        riscv::asm::delay(100_000_000);
    }}
}
