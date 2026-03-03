use std::mem::swap;

pub fn get_bits(mut lim1: u8, mut lim2: u8, source: u32) -> u32 {
    if lim1 > 63 || lim2 > 63 {
        panic!("can not deal with numbers with more than 64 bits");
    }

    if lim2 > lim1 {
        swap(&mut lim1, &mut lim2);
    }

    let length = lim1 - lim2 + 1;
    let mask = if length == 64 {
        !0
    } else {
        (1u32 << length) - 1
    };

    (source >> lim2) & mask
}

pub fn set_bits(value: u32, mut lim1: u8, mut lim2: u8, source: u32) -> u32 {
    if lim1 > 63 || lim2 > 63 {
        panic!("can not deal with numbers with more than 64 bits");
    }

    if lim2 > lim1 {
        swap(&mut lim1, &mut lim2);
    }

    let length = lim1 - lim2 + 1;

    if value.checked_shr(length as u32).unwrap_or(0) > 0 {
        panic!("the value is greater than the range size");
    }

    let ones = u32::MAX >> (64 - length);
    let clearing_mask = !(ones << lim2);
    let source_cleared = source & clearing_mask;

    source_cleared | value << lim2
}

pub fn sign_extend_128(value: u128, original_size: u8) -> u128 {
    if original_size == 0 || original_size > 128 {
        panic!("cannot sign-extend value with original size of {original_size}")
    }

    let msb = value & (1 << (original_size - 1));

    if msb == 0 {
        value
    } else {
        value | (u128::MAX << original_size)
    }
}

pub fn sign_extend_64(value: u64, original_size: u8) -> u64 {
    if original_size == 0 || original_size > 64 {
        panic!("cannot sign-extend value with original size of {original_size}")
    }

    let msb = value & (1 << (original_size - 1));

    if msb == 0 {
        value
    } else {
        value | (u64::MAX << original_size)
    }
}

pub fn sign_extend_32(value: u32, original_size: u8) -> u32 {
    if original_size == 0 || original_size > 64 {
        panic!("cannot sign-extend value with original size of {original_size}")
    }

    let msb = value & (1 << (original_size - 1));

    if msb == 0 {
        value
    } else {
        value | (u32::MAX << original_size)
    }
}

pub mod rounding_modes {

    const E63 : f32 = 9223372036854775808.0; // 2^63
    const RM_NEAR : u32 = 0x1F80;
    const RM_ZERO: u32 = 0x7F80;
    const RM_DOWN: u32 = 0x3F80;
    const RM_UP: u32 = 0x5F80;

    use std::arch::asm;

    pub fn float_mul(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_mul_rm(a, b, RM_NEAR),
            0b001 => float_mul_rm(a, b, RM_ZERO),
            0b010 => float_mul_rm(a, b, RM_DOWN),
            0b011 => float_mul_rm(a, b, RM_UP),
            0b100 => float_mul_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_sqrt(a: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_sqrt_rm(a, RM_NEAR),
            0b001 => float_sqrt_rm(a, RM_ZERO),
            0b010 => float_sqrt_rm(a, RM_DOWN),
            0b011 => float_sqrt_rm(a, RM_UP),
            0b100 => float_sqrt_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_add(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_add_rm(a, b, RM_NEAR),
            0b001 => float_add_rm(a, b, RM_ZERO),
            0b010 => float_add_rm(a, b, RM_DOWN),
            0b011 => float_add_rm(a, b, RM_UP),
            0b100 => float_add_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_sub(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_sub_rm(a, b, RM_NEAR),
            0b001 => float_sub_rm(a, b, RM_ZERO),
            0b010 => float_sub_rm(a, b, RM_DOWN),
            0b011 => float_sub_rm(a, b, RM_UP),
            0b100 => float_sub_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_div(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_div_rm(a, b, RM_NEAR),
            0b001 => float_div_rm(a, b, RM_ZERO),
            0b010 => float_div_rm(a, b, RM_DOWN),
            0b011 => float_div_rm(a, b, RM_UP),
            0b100 => float_div_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fma(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fma_rm(a, b, c, RM_NEAR),
            0b001 => float_fma_rm(a, b, c, RM_ZERO),
            0b010 => float_fma_rm(a, b, c, RM_DOWN),
            0b011 => float_fma_rm(a, b, c, RM_UP),
            0b100 => float_fma_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fms(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fms_rm(a, b, c, RM_NEAR),
            0b001 => float_fms_rm(a, b, c, RM_ZERO),
            0b010 => float_fms_rm(a, b, c, RM_DOWN),
            0b011 => float_fms_rm(a, b, c, RM_UP),
            0b100 => float_fms_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fnms(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fnms_rm(a, b, c, RM_NEAR),
            0b001 => float_fnms_rm(a, b, c, RM_ZERO),
            0b010 => float_fnms_rm(a, b, c, RM_DOWN),
            0b011 => float_fnms_rm(a, b, c, RM_UP),
            0b100 => float_fnms_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fnma(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fnma_rm(a, b, c, RM_NEAR),
            0b001 => float_fnma_rm(a, b, c, RM_ZERO),
            0b010 => float_fnma_rm(a, b, c, RM_DOWN),
            0b011 => float_fnma_rm(a, b, c, RM_UP),
            0b100 => float_fnma_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_mul_rm(a: f32, b: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "mulss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in (reg) rm
            );
        }
        res
    }

    pub fn float_fma_rm(a: f32, b: f32, c: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in (reg) rm
            );
        }
        res
    }

    pub fn float_fnma_rm(a: f32, b: f32, c: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_fms_rm(a: f32, b: f32, c: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_fnms_rm(a: f32, b: f32, c: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_add_rm(a: f32, b: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "addss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_sub_rm(a: f32, b: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "subss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_div_rm(a: f32, b: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "divss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_sqrt_rm(a: f32, rm: u32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "sqrtss {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_mul(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_mul_rm(a, b, RM_NEAR),
            0b001 => double_mul_rm(a, b, RM_ZERO),
            0b010 => double_mul_rm(a, b, RM_DOWN),
            0b011 => double_mul_rm(a, b, RM_UP),
            0b100 => double_mul_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_sqrt(a: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_sqrt_rm(a, RM_NEAR),
            0b001 => double_sqrt_rm(a, RM_ZERO),
            0b010 => double_sqrt_rm(a, RM_DOWN),
            0b011 => double_sqrt_rm(a, RM_UP),
            0b100 => double_sqrt_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_add(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_add_rm(a, b, RM_NEAR),
            0b001 => double_add_rm(a, b, RM_ZERO),
            0b010 => double_add_rm(a, b, RM_DOWN),
            0b011 => double_add_rm(a, b, RM_UP),
            0b100 => double_add_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_sub(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_sub_rm(a, b, RM_NEAR),
            0b001 => double_sub_rm(a, b, RM_ZERO),
            0b010 => double_sub_rm(a, b, RM_DOWN),
            0b011 => double_sub_rm(a, b, RM_UP),
            0b100 => double_sub_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_div(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_div_rm(a, b, RM_NEAR),
            0b001 => double_div_rm(a, b, RM_ZERO),
            0b010 => double_div_rm(a, b, RM_DOWN),
            0b011 => double_div_rm(a, b, RM_UP),
            0b100 => double_div_rm(a, b, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fma(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fma_rm(a, b, c, RM_NEAR),
            0b001 => double_fma_rm(a, b, c, RM_ZERO),
            0b010 => double_fma_rm(a, b, c, RM_DOWN),
            0b011 => double_fma_rm(a, b, c, RM_UP),
            0b100 => double_fma_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fms(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fms_rm(a, b, c, RM_NEAR),
            0b001 => double_fms_rm(a, b, c, RM_ZERO),
            0b010 => double_fms_rm(a, b, c, RM_DOWN),
            0b011 => double_fms_rm(a, b, c, RM_UP),
            0b100 => double_fms_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fnms(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fnms_rm(a, b, c, RM_NEAR),
            0b001 => double_fnms_rm(a, b, c, RM_ZERO),
            0b010 => double_fnms_rm(a, b, c, RM_DOWN),
            0b011 => double_fnms_rm(a, b, c, RM_UP),
            0b100 => double_fnms_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fnma(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fnma_rm(a, b, c, RM_NEAR),
            0b001 => double_fnma_rm(a, b, c, RM_ZERO),
            0b010 => double_fnma_rm(a, b, c, RM_DOWN),
            0b011 => double_fnma_rm(a, b, c, RM_UP),
            0b100 => double_fnma_rm(a, b, c, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_mul_rm(a: f64, b: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "mulsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_fnms_rm(a: f64, b: f64, c: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_fnma_rm(a: f64, b: f64, c: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_fma_rm(a: f64, b: f64, c: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_fms_rm(a: f64, b: f64, c: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "vfmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_add_rm(a: f64, b: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "addsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_sub_rm(a: f64, b: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "subsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_div_rm(a: f64, b: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "divsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                rm = in(reg) rm
            );
        }
        res
    }

    pub fn double_sqrt_rm(a: f64, rm: u32) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "sqrtsd {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_to_i32_rm(a: f32, rm: u32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_to_i64_rm(a: f32, rm: u32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn float_to_u64_rm(a: f32, rm: u32) -> u64 {
        let res : i64;
        if a < E63 {
            println!("up");
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtss2si {res}, {a}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(reg) res,
                    a = in(xmm_reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            let xmm1: f32 = E63;
            let high_bit: i64 = i64::MIN;
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "subss {a}, {xmm1}",
                    "cvtss2si {res:r}, {a}",
                    "xor {res:r}, {high_bit:r}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(reg) res,
                    high_bit = in (reg) high_bit,
                    a = in(xmm_reg) a,
                    xmm1 = in(xmm_reg) xmm1,
                    rm = in(reg) rm,
                );
            }
        }
        res as u64
    }

    pub fn i32_to_float_rm(a: i32, rm: u32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn u32_to_float_rm(a: u32, rm: u32) -> f32 {
        let mut res : f32 = 0.0;
        if (a & 0b10000000) != 1 { // if it is "negative"
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {a:e}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    a = in(reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            // divide number so it fits + lsb to ensure precision
            let half_lsb = (a<<2) | (a & 0b1);
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {half_lsb:e}",
                    "addss {res}, {res}",       // double value back
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    half_lsb = in(reg) half_lsb,
                    rm = in(reg) rm,
                );
            }
        }
        res
    }

    pub fn i64_to_float_rm(a: i64, rm: u32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:r}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn u64_to_float_rm(a: u64, rm: u32) -> f32 {
        let mut res : f32 = 0.0;
        if (a & 0b10000000) != 1 { // if it is "negative"
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {a:r}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    a = in(reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            // divide number so it fits + lsb to ensure precision
            let half_lsb = (a<<2) | (a & 0b1);
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {half_lsb:r}",
                    "addss {res}, {res}",       // double value back
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    half_lsb = in(reg) half_lsb,
                    rm = in(reg) rm,
                );
            }
        }
        res
    }

    pub fn float_to_i32 (a: f32, rm: u8) -> i32 {
        match rm {
            0b000 => float_to_i32_rm(a, RM_NEAR),
            0b001 => float_to_i32_rm(a, RM_ZERO),
            0b010 => float_to_i32_rm(a, RM_DOWN),
            0b011 => float_to_i32_rm(a, RM_UP),
            0b100 => float_to_i32_rm(a, RM_NEAR),
            0b101..0b111 => { 
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_to_u32 (a: f32, rm: u8) -> u32 {
        float_to_i64(a, rm) as u32
    }

    pub fn float_to_i64 (a: f32, rm: u8) -> i64 {
        match rm {
            0b000 => float_to_i64_rm(a, RM_NEAR),
            0b001 => float_to_i64_rm(a, RM_ZERO),
            0b010 => float_to_i64_rm(a, RM_DOWN),
            0b011 => float_to_i64_rm(a, RM_UP),
            0b100 => float_to_i64_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_to_u64 (a: f32, rm: u8) -> u64 {
        match rm {
            0b000 => float_to_u64_rm(a, RM_NEAR),
            0b001 => float_to_u64_rm(a, RM_ZERO),
            0b010 => float_to_u64_rm(a, RM_DOWN),
            0b011 => float_to_u64_rm(a, RM_UP),
            0b100 => float_to_u64_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn i32_to_float (a: i32, rm: u8) -> f32 {
        match rm {
            0b000 => i32_to_float_rm(a, RM_NEAR),
            0b001 => i32_to_float_rm(a, RM_ZERO),
            0b010 => i32_to_float_rm(a, RM_DOWN),
            0b011 => i32_to_float_rm(a, RM_UP),
            0b100 => i32_to_float_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u32_to_float (a: u32, rm: u8) -> f32 {
        match rm {
            0b000 => u32_to_float_rm(a, RM_NEAR),
            0b001 => u32_to_float_rm(a, RM_ZERO),
            0b010 => u32_to_float_rm(a, RM_DOWN),
            0b011 => u32_to_float_rm(a, RM_UP),
            0b100 => u32_to_float_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn i64_to_float (a: i64, rm: u8) -> f32 {
        match rm {
            0b000 => i64_to_float_rm(a, RM_NEAR),
            0b001 => i64_to_float_rm(a, RM_ZERO),
            0b010 => i64_to_float_rm(a, RM_DOWN),
            0b011 => i64_to_float_rm(a, RM_UP),
            0b100 => i64_to_float_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u64_to_float (a: u64, rm: u8) -> f32 {
        match rm {
            0b000 => u64_to_float_rm(a, RM_NEAR),
            0b001 => u64_to_float_rm(a, RM_ZERO),
            0b010 => u64_to_float_rm(a, RM_DOWN),
            0b011 => u64_to_float_rm(a, RM_UP),
            0b100 => u64_to_float_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_i32_rm(a: f64, rm: u32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn i32_to_double_rm(a: i32, rm: u32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn u32_to_double_rm(a: u32, rm: u32) -> f64 {
        let mut res : f64 = 0.0;
        if (a & 0b10000000) != 1 { // if it is "negative"
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {a:e}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    a = in(reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            // divide number so it fits + lsb to ensure precision
            let half_lsb = (a<<2) | (a & 0b1);
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {half_lsb:e}",
                    "addss {res}, {res}",       // double value back
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    half_lsb = in(reg) half_lsb,
                    rm = in(reg) rm,
                );
            }
        }
        res
    }

    pub fn double_to_i64_rm(a: f64, rm: u32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_to_u64_rm(a: f64, rm: u32) -> u64 {
        let res : i64;
        if a < (E63 as f64) {
            println!("up");
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtss2si {res}, {a}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(reg) res,
                    a = in(xmm_reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            let xmm1: f64 = E63 as f64;
            let high_bit: i64 = i64::MIN;
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "subss {a}, {xmm1}",
                    "cvtss2si {res:r}, {a}",
                    "xor {res:r}, {high_bit:r}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(reg) res,
                    high_bit = in (reg) high_bit,
                    a = in(xmm_reg) a,
                    xmm1 = in(xmm_reg) xmm1,
                    rm = in(reg) rm,
                );
            }
        }
        res as u64
    }

    pub fn i64_to_double_rm(a: i64, rm: u32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn u64_to_double_rm(a: u64, rm: u32) -> f64 {
        let mut res : f64 = 0.0;
        if (a & 0b10000000) != 1 { // if it is "negative"
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {a:r}",
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    a = in(reg) a,
                    rm = in(reg) rm,
                );
            }
        } else {
            // divide number so it fits + lsb to ensure precision
            let half_lsb = (a<<2) | (a & 0b1);
            unsafe {
                asm!(
                    "sub rsp, 8",
                    "stmxcsr [rsp]",
                    "mov dword ptr [rsp + 4], {rm:e}",
                    "ldmxcsr [rsp + 4]",
                    "cvtsi2ss {res}, {half_lsb:r}",
                    "addss {res}, {res}",       // double value back
                    "ldmxcsr [rsp]",  // restore MXCSR 
                    "add rsp, 8",
                    res = out(xmm_reg) res,
                    half_lsb = in(reg) half_lsb,
                    rm = in(reg) rm,
                );
            }
        }
        res
    }

    pub fn double_to_i32 (a: f64, rm: u8) -> i32 {
        match rm {
            0b000 => double_to_i32_rm(a, RM_NEAR),
            0b001 => double_to_i32_rm(a, RM_ZERO),
            0b010 => double_to_i32_rm(a, RM_DOWN),
            0b011 => double_to_i32_rm(a, RM_UP),
            0b100 => double_to_i32_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_u32 (a: f64, rm: u8) -> u32 {
        double_to_i32(a, rm) as u32
    }

    pub fn i32_to_double (a: i32, rm: u8) -> f64 {
        match rm {
            0b000 => i32_to_double_rm(a, RM_NEAR),
            0b001 => i32_to_double_rm(a, RM_ZERO),
            0b010 => i32_to_double_rm(a, RM_DOWN),
            0b011 => i32_to_double_rm(a, RM_UP),
            0b100 => i32_to_double_rm(a, RM_NEAR),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u32_to_double (a: u32, rm: u8) -> f64 {
        match rm {
            0b000 => u32_to_double_rm(a, RM_NEAR),
            0b001 => u32_to_double_rm(a, RM_ZERO),
            0b010 => u32_to_double_rm(a, RM_DOWN),
            0b011 => u32_to_double_rm(a, RM_UP),
            0b100 => u32_to_double_rm(a, RM_NEAR),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_i64 (a: f64, rm: u8) -> i64 {
        match rm {
            0b000 => double_to_i64_rm(a, RM_NEAR),
            0b001 => double_to_i64_rm(a, RM_ZERO),
            0b010 => double_to_i64_rm(a, RM_DOWN),
            0b011 => double_to_i64_rm(a, RM_UP),
            0b100 => double_to_i64_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_u64 (a: f64, rm: u8) -> u64 {
        match rm {
            0b000 => double_to_u64_rm(a, RM_NEAR),
            0b001 => double_to_u64_rm(a, RM_ZERO),
            0b010 => double_to_u64_rm(a, RM_DOWN),
            0b011 => double_to_u64_rm(a, RM_UP),
            0b100 => double_to_u64_rm(a, RM_NEAR),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn i64_to_double (a: i64, rm: u8) -> f64 {
        match rm {
            0b000 => i64_to_double_rm(a, RM_NEAR),
            0b001 => i64_to_double_rm(a, RM_ZERO),
            0b010 => i64_to_double_rm(a, RM_DOWN),
            0b011 => i64_to_double_rm(a, RM_UP),
            0b100 => i64_to_double_rm(a, RM_NEAR),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u64_to_double (a: u64, rm: u8) -> f64 {
        match rm {
            0b000 => u64_to_double_rm(a, RM_NEAR),
            0b001 => u64_to_double_rm(a, RM_ZERO),
            0b010 => u64_to_double_rm(a, RM_DOWN),
            0b011 => u64_to_double_rm(a, RM_UP),
            0b100 => u64_to_double_rm(a, RM_NEAR),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_float_rm(a: f64, rm: u32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], {rm:e}",
                "ldmxcsr [rsp + 4]",
                "cvtsd2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(xmm_reg) a,
                rm = in(reg) rm,
            );
        }
        res
    }

    pub fn double_to_float (a: f64, rm: u8) -> f32 {
         match rm {
            0b000 => double_to_float_rm(a, RM_NEAR),
            0b001 => double_to_float_rm(a, RM_ZERO),
            0b010 => double_to_float_rm(a, RM_DOWN),
            0b011 => double_to_float_rm(a, RM_UP),
            0b100 => double_to_float_rm(a, RM_NEAR),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }
}
