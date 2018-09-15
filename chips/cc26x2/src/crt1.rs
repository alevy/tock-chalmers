use cortexm4::{generic_isr, nvic, svc_handler, systick_handler};

extern "C" {
    // Symbols defined in the linker file
    static mut _erelocate: u32;
    static mut _etext: u32;
    static mut _ezero: u32;
    static mut _srelocate: u32;
    static mut _szero: u32;
    fn reset_handler();

    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

unsafe extern "C" fn unhandled_interrupt() {
    'loop0: loop {}
}

unsafe extern "C" fn hard_fault_handler() {
    use cortexm4;
    use core;
    use core::intrinsics::offset;

    let faulting_stack: *mut u32;
    let kernel_stack: bool;

    asm!(
        "mov    r1, 0                       \n\
         tst    lr, #4                      \n\
         itte   eq                          \n\
         mrseq  r0, msp                     \n\
         addeq  r1, 1                       \n\
         mrsne  r0, psp                     "
        : "={r0}"(faulting_stack), "={r1}"(kernel_stack)
        :
        : "r0", "r1"
        :
        );

    if kernel_stack {
        let stacked_r0: u32 = *offset(faulting_stack, 0);
        let stacked_r1: u32 = *offset(faulting_stack, 1);
        let stacked_r2: u32 = *offset(faulting_stack, 2);
        let stacked_r3: u32 = *offset(faulting_stack, 3);
        let stacked_r12: u32 = *offset(faulting_stack, 4);
        let stacked_lr: u32 = *offset(faulting_stack, 5);
        let stacked_pc: u32 = *offset(faulting_stack, 6);
        let stacked_xpsr: u32 = *offset(faulting_stack, 7);

        let mode_str = "Kernel";

        let shcsr: u32 = core::ptr::read_volatile(0xE000ED24 as *const u32);
        let cfsr: u32 = core::ptr::read_volatile(0xE000ED28 as *const u32);
        let hfsr: u32 = core::ptr::read_volatile(0xE000ED2C as *const u32);
        let mmfar: u32 = core::ptr::read_volatile(0xE000ED34 as *const u32);
        let bfar: u32 = core::ptr::read_volatile(0xE000ED38 as *const u32);

        let iaccviol = (cfsr & 0x01) == 0x01;
        let daccviol = (cfsr & 0x02) == 0x02;
        let munstkerr = (cfsr & 0x08) == 0x08;
        let mstkerr = (cfsr & 0x10) == 0x10;
        let mlsperr = (cfsr & 0x20) == 0x20;
        let mmfarvalid = (cfsr & 0x80) == 0x80;

        let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
        let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
        let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
        let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
        let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
        let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
        let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

        let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
        let invstate = ((cfsr >> 16) & 0x02) == 0x02;
        let invpc = ((cfsr >> 16) & 0x04) == 0x04;
        let nocp = ((cfsr >> 16) & 0x08) == 0x08;
        let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
        let divbysero = ((cfsr >> 16) & 0x200) == 0x200;

        let vecttbl = (hfsr & 0x02) == 0x02;
        let forced = (hfsr & 0x40000000) == 0x40000000;

        let ici_it = (((stacked_xpsr >> 25) & 0x3) << 6) | ((stacked_xpsr >> 10) & 0x3f);
        let thumb_bit = ((stacked_xpsr >> 24) & 0x1) == 1;
        let exception_number = (stacked_xpsr & 0x1ff) as usize;

        panic!(
            "{} HardFault.\n\
             \tKernel version {}\n\
             \tr0  0x{:x}\n\
             \tr1  0x{:x}\n\
             \tr2  0x{:x}\n\
             \tr3  0x{:x}\n\
             \tr12 0x{:x}\n\
             \tlr  0x{:x}\n\
             \tpc  0x{:x}\n\
             \tprs 0x{:x} [ N {} Z {} C {} V {} Q {} GE {}{}{}{} ; ICI.IT {} T {} ; Exc {}-{} ]\n\
             \tsp  0x{:x}\n\
             \ttop of stack     0x{:x}\n\
             \tbottom of stack  0x{:x}\n\
             \tSHCSR 0x{:x}\n\
             \tCFSR  0x{:x}\n\
             \tHSFR  0x{:x}\n\
             \tInstruction Access Violation:       {}\n\
             \tData Access Violation:              {}\n\
             \tMemory Management Unstacking Fault: {}\n\
             \tMemory Management Stacking Fault:   {}\n\
             \tMemory Management Lazy FP Fault:    {}\n\
             \tInstruction Bus Error:              {}\n\
             \tPrecise Data Bus Error:             {}\n\
             \tImprecise Data Bus Error:           {}\n\
             \tBus Unstacking Fault:               {}\n\
             \tBus Stacking Fault:                 {}\n\
             \tBus Lazy FP Fault:                  {}\n\
             \tUndefined Instruction Usage Fault:  {}\n\
             \tInvalid State Usage Fault:          {}\n\
             \tInvalid PC Load Usage Fault:        {}\n\
             \tNo Coprocessor Usage Fault:         {}\n\
             \tUnaligned Access Usage Fault:       {}\n\
             \tDivide By Zero:                     {}\n\
             \tBus Fault on Vector Table Read:     {}\n\
             \tForced Hard Fault:                  {}\n\
             \tFaulting Memory Address: (valid: {}) {:#010X}\n\
             \tBus Fault Address:       (valid: {}) {:#010X}\n\
             ",
            mode_str,
            env!("TOCK_KERNEL_VERSION"),
            stacked_r0,
            stacked_r1,
            stacked_r2,
            stacked_r3,
            stacked_r12,
            stacked_lr,
            stacked_pc,
            stacked_xpsr,
            (stacked_xpsr >> 31) & 0x1,
            (stacked_xpsr >> 30) & 0x1,
            (stacked_xpsr >> 29) & 0x1,
            (stacked_xpsr >> 28) & 0x1,
            (stacked_xpsr >> 27) & 0x1,
            (stacked_xpsr >> 19) & 0x1,
            (stacked_xpsr >> 18) & 0x1,
            (stacked_xpsr >> 17) & 0x1,
            (stacked_xpsr >> 16) & 0x1,
            ici_it,
            thumb_bit,
            exception_number,
            cortexm4::ipsr_isr_number_to_str(exception_number),
            faulting_stack as u32,
            (_estack as *const ()) as u32,
            (&_ezero as *const u32) as u32,
            shcsr,
            cfsr,
            hfsr,
            iaccviol,
            daccviol,
            munstkerr,
            mstkerr,
            mlsperr,
            ibuserr,
            preciserr,
            impreciserr,
            unstkerr,
            stkerr,
            lsperr,
            undefinstr,
            invstate,
            invpc,
            nocp,
            unaligned,
            divbysero,
            vecttbl,
            forced,
            mmfarvalid,
            mmfar,
            bfarvalid,
            bfar
        );
    } else {
        // hard fault occurred in an app, not the kernel. The app should be
        //  marked as in an error state and handled by the kernel
        asm!(
            "ldr r0, =SYSCALL_FIRED
              mov r1, #1
              str r1, [r0, #0]

              ldr r0, =APP_FAULT
              str r1, [r0, #0]

              /* Read the SCB registers. */
              ldr r0, =SCB_REGISTERS
              ldr r1, =0xE000ED14
              ldr r2, [r1, #0] /* CCR */
              str r2, [r0, #0]
              ldr r2, [r1, #20] /* CFSR */
              str r2, [r0, #4]
              ldr r2, [r1, #24] /* HFSR */
              str r2, [r0, #8]
              ldr r2, [r1, #32] /* MMFAR */
              str r2, [r0, #12]
              ldr r2, [r1, #36] /* BFAR */
              str r2, [r0, #16]

              /* Set thread mode to privileged */
              mov r0, #0
              msr CONTROL, r0

              movw LR, #0xFFF9
              movt LR, #0xFFFF"
        );
    }
}

#[link_section = ".vectors"]
// used Ensures that the symbol is kept until the final binary
#[used]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 50] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MPU fault
    unhandled_interrupt, // Bus fault
    unhandled_interrupt, // Usage fault
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    svc_handler,         // SVC
    unhandled_interrupt, // Debug monitor,
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // PendSV
    systick_handler,     // Systick
    generic_isr,         // GPIO Int handler
    generic_isr,         // I2C
    generic_isr,         // RF Core Command & Packet Engine 1
    generic_isr,         // AON SpiSplave Rx, Tx and CS
    generic_isr,         // AON RTC
    generic_isr,         // UART0 Rx and Tx
    generic_isr,         // AUX software event 0
    generic_isr,         // SSI0 Rx and Tx
    generic_isr,         // SSI1 Rx and Tx
    generic_isr,         // RF Core Command & Packet Engine 0
    generic_isr,         // RF Core Hardware
    generic_isr,         // RF Core Command Acknowledge
    generic_isr,         // I2S
    generic_isr,         // AUX software event 1
    generic_isr,         // Watchdog timer
    generic_isr,         // Timer 0 subtimer A
    generic_isr,         // Timer 0 subtimer B
    generic_isr,         // Timer 1 subtimer A
    generic_isr,         // Timer 1 subtimer B
    generic_isr,         // Timer 2 subtimer A
    generic_isr,         // Timer 2 subtimer B
    generic_isr,         // Timer 3 subtimer A
    generic_isr,         // Timer 3 subtimer B
    generic_isr,         // Crypto Core Result available
    generic_isr,         // uDMA Software
    generic_isr,         // uDMA Error
    generic_isr,         // Flash controller
    generic_isr,         // Software Event 0
    generic_isr,         // AUX combined event
    generic_isr,         // AON programmable 0
    generic_isr,         // Dynamic Programmable interrupt
    // source (Default: PRCM)
    generic_isr, // AUX Comparator A
    generic_isr, // AUX ADC new sample or ADC DMA
    // done, ADC underflow, ADC overflow
    generic_isr, // TRNG event
];

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;

    // Move the relocate segment. This assumes it is located after the text
    // segment, which is where the storm linker file puts it
    p_src = &mut _etext as (*mut u32);
    p_dest = &mut _srelocate as (*mut u32);
    if p_src != p_dest {
        current_block = 1;
    } else {
        current_block = 2;
    }
    'loop1: loop {
        if current_block == 1 {
            if !(p_dest < &mut _erelocate as (*mut u32)) {
                current_block = 2;
                continue;
            }
            *{
                let _old = p_dest;
                p_dest = p_dest.offset(1isize);
                _old
            } = *{
                let _old = p_src;
                p_src = p_src.offset(1isize);
                _old
            };
            current_block = 1;
        } else {
            p_dest = &mut _szero as (*mut u32);
            break;
        }
    }
    'loop3: loop {
        if !(p_dest < &mut _ezero as (*mut u32)) {
            break;
        }
        *{
            let _old = p_dest;
            p_dest = p_dest.offset(1isize);
            _old
        } = 0u32;
    }
    nvic::enable_all();
}
