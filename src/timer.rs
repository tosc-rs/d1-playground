use d1_pac::TIMER;
pub use d1_pac::timer::tmr_ctrl::{
    TMR_CLK_SRC_A as TimerSource,
    TMR_CLK_PRES_A as TimerPrescaler,
    TMR_MODE_A as TimerMode,
};

pub struct Timers {
    pub timer0: Timer0,
    pub timer1: Timer1,
}

mod sealed {
    use d1_pac::{generic::Reg, timer::{tmr_ctrl::TMR_CTRL_SPEC, tmr_intv_value::TMR_INTV_VALUE_SPEC, tmr_cur_value::TMR_CUR_VALUE_SPEC}};

    use super::*;

    pub trait TimerSealed {
        fn ctrl(&self) -> &Reg<TMR_CTRL_SPEC>;
        fn interval(&self) -> &Reg<TMR_INTV_VALUE_SPEC>;
        fn value(&self) -> &Reg<TMR_CUR_VALUE_SPEC>;
    }

    impl TimerSealed for Timer0 {
        #[inline(always)]
        fn ctrl(&self) -> &Reg<TMR_CTRL_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr0_ctrl
        }

        #[inline(always)]
        fn interval(&self) -> &Reg<TMR_INTV_VALUE_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr0_intv_value
        }

        #[inline(always)]
        fn value(&self) -> &Reg<TMR_CUR_VALUE_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr0_cur_value
        }
    }

    impl TimerSealed for Timer1 {
        #[inline(always)]
        fn ctrl(&self) -> &Reg<TMR_CTRL_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr1_ctrl
        }

        #[inline(always)]
        fn interval(&self) -> &Reg<TMR_INTV_VALUE_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr1_intv_value
        }

        #[inline(always)]
        fn value(&self) -> &Reg<TMR_CUR_VALUE_SPEC> {
            let timer = unsafe { &*TIMER::PTR };
            &timer.tmr1_cur_value
        }
    }

    impl Timer for Timer0 { }
    impl Timer for Timer1 { }
}

pub struct Timer0 {
    _x: (),
}

pub struct Timer1 {
    _x: (),
}

pub trait Timer: sealed::TimerSealed {
    #[inline]
    fn set_source(&mut self, variant: TimerSource) {
        self.ctrl().modify(|_r, w| {
            w.tmr_clk_src().variant(variant);
            w
        });
    }

    #[inline]
    fn set_prescaler(&mut self, variant: TimerPrescaler) {
        self.ctrl().modify(|_r, w| {
            w.tmr_clk_pres().variant(variant);
            w
        });
    }

    #[inline]
    fn set_mode(&mut self, variant: TimerMode) {
        self.ctrl().modify(|_r, w| {
            w.tmr_mode().variant(variant);
            w
        });
    }

    #[inline]
    fn start_counter(&mut self, interval: u32) {
        self.interval().write(|w| unsafe {
            w.bits(interval);
            w
        });
        // Set the reload AND enable bits at the same time
        // TODO: Reset status flag or interrupt flag?
        self.ctrl().modify(|_r, w| {
            w.tmr_reload().set_bit();
            w.tmr_en().set_bit();
            w
        });
    }

    #[inline]
    fn current_value(&self) -> u32 {
        self.value().read().bits()
    }
}

impl Timers {
    pub fn new(
        _periph: TIMER,
    ) -> Self {
        // 1. Configure the timer parameters clock source, prescale factor, and timing mode by writing **TMRn_CTRL_REG**. There is no sequence requirement of configuring the parameters.
        // 2. Write the interval value.
        //     * Write TMRn_INTV_VALUE_REG to configure the interval value for the timer.
        //     * Write bit[1] of TMRn_CTRL_REG to load the interval value to the timer. The value of the bit will be cleared automatically after loading the interval value.
        // 3. Write bit[0] of TMRn_CTRL_REG to start the timer. To get the current value of the timer, read
        // TMRn_CUR_VALUE_REG.
        Self {
            timer0: Timer0 { _x: () },
            timer1: Timer1 { _x: () },
        }
    }
}
