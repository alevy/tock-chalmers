#![no_std]
#![no_main]
#![feature(used, lang_items, panic_implementation)]
//! CCFG - Customer Configuration
//!
//! For details see p. 710 in the cc2650 technical reference manual.
//!
//! Currently setup to use the default settings.

use core::fmt::Arguments;

#[repr(C)]
pub struct Ccfg {
    ext_lf_clk: u32,
    mode_conf_1: u32,
    size_and_dis_flags: u32,
    mode_conf: u32,
    volt_load_0: u32,
    volt_load_1: u32,
    rtc_offset: u32,
    freq_offset: u32,
    ieee_mac_0: u32,
    ieee_mac_1: u32,
    ieee_ble_0: u32,
    ieee_ble_1: u32,
    bl_config: u32,
    erase_conf: u32,
    ccfg_ti_options: u32,
    ccfg_tap_dap_0: u32,
    ccfg_tap_dap_1: u32,
    image_valid_conf: u32,
    ccfg_prot_31_0: u32,
    ccfg_prot_63_32: u32,
    ccfg_prot_95_64: u32,
    ccfg_prot_127_96: u32,
}

#[used]
#[link_section = ".init"]
pub static CCFG_CONF: Ccfg = Ccfg {
    ext_lf_clk: 0x01800000,
    mode_conf_1: 0xFF820010,
    size_and_dis_flags: 0x0058FFFD,
    mode_conf: 0xF3FFFF3A,
    volt_load_0: 0xFFFFFFFF,
    volt_load_1: 0xFFFFFFFF,
    rtc_offset: 0xFFFFFFFF,
    freq_offset: 0xFFFFFFFF,
    ieee_mac_0: 0xFFFFFFFF,
    ieee_mac_1: 0xFFFFFFFF,
    ieee_ble_0: 0xFFFFFFFF,
    ieee_ble_1: 0xFFFFFFFF,
    bl_config: 0xC5FE0EC5,
    erase_conf: 0xFFFFFFFF,
    ccfg_ti_options: 0xFFFFFF00,
    ccfg_tap_dap_0: 0xFFC5C5C5,
    ccfg_tap_dap_1: 0xFFC5C5C5,
    image_valid_conf: 0x00000000,
    ccfg_prot_31_0: 0xFFFFFFFF,
    ccfg_prot_63_32: 0xFFFFFFFF,
    ccfg_prot_95_64: 0xFFFFFFFF,
    ccfg_prot_127_96: 0xFFFFFFFF,
};

#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn panic_fmt(_args: Arguments, _file: &'static str, _line: u32) -> ! {
    loop {}
}
