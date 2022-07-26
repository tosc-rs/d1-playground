#![no_std]
#![no_main]

use core::sync::atomic::{compiler_fence, Ordering};

use d1_pac::{PLIC, TIMER, UART0};
use panic_halt as _;

use d1_playground::timer::{Timer, TimerMode, TimerPrescaler, TimerSource, Timers};

static HOUND: &str = include_str!("../hound.txt");

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
    ccu.uart_bgr
        .write(|w| w.uart0_gating().pass().uart0_rst().deassert());

    // DMAC enable
    let dmac = &p.DMAC;
    ccu.dma_bgr.write(|w| w.gating().pass().rst().deassert());

    // Set PC1 LED to output.
    let gpio = &p.GPIO;
    gpio.pc_cfg0
        .write(|w| w.pc1_select().output().pc0_select().ledc_do());

    // Set PB8 and PB9 to function 6, UART0, internal pullup.
    gpio.pb_cfg1
        .write(|w| w.pb8_select().uart0_tx().pb9_select().uart0_rx());
    gpio.pb_pull0
        .write(|w| w.pc8_pull().pull_up().pc9_pull().pull_up());

    // Configure UART0 for 115200 8n1.
    // By default APB1 is 24MHz, use divisor 13 for 115200.
    let uart0 = p.UART0;

    // UART Mode
    // No Auto Flow Control
    // No Loop Back
    // No RTS_N
    // No DTR_N
    uart0.mcr.write(|w| unsafe { w.bits(0) });

    // RCVR INT Trigger: 1 char in FIFO
    // TXMT INT Trigger: FIFO Empty
    // DMA Mode 0 - (???)
    // FIFOs Enabled
    uart0.fcr().write(|w| w.fifoe().set_bit());

    // TX Halted
    // Also has some DMA relevant things? Not set currently
    uart0.halt.write(|w| w.halt_tx().enabled());

    // Enable control of baudrates
    uart0.lcr.write(|w| w.dlab().divisor_latch());

    // Baudrates
    uart0.dll().write(|w| unsafe { w.dll().bits(13) });
    uart0.dlh().write(|w| unsafe { w.dlh().bits(0) });

    // Unlatch baud rate, set width
    uart0.lcr.write(|w| w.dlab().rx_buffer().dls().eight());

    // Re-enable sending
    uart0.halt.write(|w| w.halt_tx().disabled());
    unsafe { PRINTER = Some(Uart(uart0)) };

    // Set up timers
    let Timers {
        mut timer0,
        mut timer1,
        ..
    } = Timers::new(p.TIMER);

    timer0.set_source(TimerSource::OSC24_M);
    timer1.set_source(TimerSource::OSC24_M);

    timer0.set_prescaler(TimerPrescaler::P8); // 24M / 8:  3.00M ticks/s
    timer1.set_prescaler(TimerPrescaler::P32); // 24M / 32: 0.75M ticks/s

    timer0.set_mode(TimerMode::SINGLE_COUNTING);
    timer1.set_mode(TimerMode::SINGLE_COUNTING);

    let _ = timer0.get_and_clear_interrupt();
    let _ = timer1.get_and_clear_interrupt();

    unsafe {
        riscv::interrupt::enable();
        riscv::register::mie::set_mext();
    }

    // yolo
    timer0.set_interrupt_en(true);
    timer1.set_interrupt_en(true);
    let plic = &p.PLIC;

    plic.prio[75].write(|w| w.priority().p1());
    plic.prio[76].write(|w| w.priority().p1());
    plic.mie[2].write(|w| unsafe { w.bits((1 << 11) | (1 << 12)) });

    let mut descriptor = Descriptor {
        configuration: 0,
        source_address: 0,
        destination_address: 0,
        byte_counter: 0,
        parameter: 0,
        link: 0
    };
    let desc_addr: *mut u8 = &mut descriptor as *mut Descriptor as *mut u8;
    let thr_addr = unsafe { &*UART0::PTR }.thr() as *const _ as usize as u64;


    for chunk in HOUND.lines() {

        descriptor.set_source(chunk.as_ptr() as usize as u64);
        descriptor.set_dest(thr_addr);
        descriptor.byte_counter = chunk.len() as u32;

        // I think? DMAC_CFG_REGN
        descriptor.configuration = 0;
        descriptor.configuration |= 0b0 << 30;  // BMODE_SEL: Normal
        descriptor.configuration |= 0b00 << 25; // DEST_WIDTH: 8-bit
        descriptor.configuration |= 0b1 << 24;  // DMA_ADDR_MODE: Dest IO Mode
        descriptor.configuration |= 0b00 << 22; // Dest block size: 1
        descriptor.configuration |= 0b001110 << 16; // !!! Dest DRQ Type - UART0
        descriptor.configuration |= 0b00 << 9; // Source width 8 bit
        descriptor.configuration |= 0b0 << 8; // Source Linear Mode
        descriptor.configuration |= 0b00 << 6; // Source block size 1
        descriptor.configuration |= 0b000001 << 0; // Source DRQ type - DRAM

        descriptor.end_link();

        compiler_fence(Ordering::SeqCst); //////

        dmac.dmac_desc_addr_reg0.write(|w| {
            w.dma_desc_addr().variant((desc_addr as usize >> 2) as u32);
            w.dma_desc_high_addr().variant(((desc_addr as usize >> 32) as u8) & 0b11);
            w
        });
        dmac.dmac_en_reg0.write(|w| w.dma_en().enabled());

        compiler_fence(Ordering::SeqCst); //////

        timer0.start_counter(1_500_000);
        unsafe { riscv::asm::wfi() };

        println!("");
        dmac.dmac_en_reg0.write(|w| w.dma_en().disabled());
    }
    panic!();
}

#[export_name = "MachineExternal"]
fn im_an_interrupt() {
    let plic = unsafe { &*PLIC::PTR };
    let timer = unsafe { &*TIMER::PTR };

    let claim = plic.mclaim.read().mclaim();
    // println!("INTERRUPT! claim: {}", claim.bits());

    match claim.bits() {
        75 => {
            timer
                .tmr_irq_sta
                .modify(|_r, w| w.tmr0_irq_pend().set_bit());
            // Wait for the interrupt to clear to avoid repeat interrupts
            while timer.tmr_irq_sta.read().tmr0_irq_pend().bit_is_set() {}
        }
        76 => {
            timer
                .tmr_irq_sta
                .modify(|_r, w| w.tmr1_irq_pend().set_bit());
            // Wait for the interrupt to clear to avoid repeat interrupts
            while timer.tmr_irq_sta.read().tmr1_irq_pend().bit_is_set() {}
        }
        x => {
            println!("Unexpected claim: {}", x);
            panic!();
        }
    }

    // Release claim
    plic.mclaim.write(|w| w.mclaim().variant(claim.bits()));
}

#[repr(C, align(4))]
// This gets written to DMAC_DESC_ADDR_REGN in a funky way
pub struct Descriptor {
    configuration: u32,
    source_address: u32,
    destination_address: u32,
    byte_counter: u32,
    parameter: u32,
    link: u32,
}

impl Descriptor {
    fn set_source(&mut self, source: u64) {
        assert!(source < (1 << 34));
        self.source_address = source as u32;
        //                  332222222222 11 11 11111100 00000000
        //                  109876543210 98 76 54321098 76543210
        self.parameter &= 0b111111111111_11_00_11111111_11111111;
        self.parameter |= (((source >> 32) & 0b11) << 16) as u32;
    }

    fn set_dest(&mut self, dest: u64) {
        assert!(dest < (1 << 34));
        self.destination_address = dest as u32;
        //                  332222222222 11 11 11111100 00000000
        //                  109876543210 98 76 54321098 76543210
        self.parameter &= 0b111111111111_00_11_11111111_11111111;
        self.parameter |= (((dest >> 32) & 0b11) << 18) as u32;
    }

    fn end_link(&mut self) {
        self.link = 0xFFFF_F800;
    }
}

// Main config register:
// DMAC_CFG_REGN
// Mode:
// DMAC_MODE_REGN
