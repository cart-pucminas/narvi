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

    use std::arch::asm;

    pub fn float_mul(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_mul_near(a, b),
            0b001 => float_mul_zero(a, b),
            0b010 => float_mul_down(a, b),
            0b011 => float_mul_up(a, b),
            0b100 => float_mul_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_sqrt(a: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_sqrt_near(a),
            0b001 => float_sqrt_zero(a),
            0b010 => float_sqrt_down(a),
            0b011 => float_sqrt_up(a),
            0b100 => float_sqrt_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_add(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_add_near(a, b),
            0b001 => float_add_zero(a, b),
            0b010 => float_add_down(a, b),
            0b011 => float_add_up(a, b),
            0b100 => float_add_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_sub(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_sub_near(a, b),
            0b001 => float_sub_zero(a, b),
            0b010 => float_sub_down(a, b),
            0b011 => float_sub_up(a, b),
            0b100 => float_sub_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_div(a: f32, b: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_div_near(a, b),
            0b001 => float_div_zero(a, b),
            0b010 => float_div_down(a, b),
            0b011 => float_div_up(a, b),
            0b100 => float_div_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fma(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fma_near(a, b, c),
            0b001 => float_fma_zero(a, b, c),
            0b010 => float_fma_down(a, b, c),
            0b011 => float_fma_up(a, b, c),
            0b100 => float_fma_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fms(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fms_near(a, b, c),
            0b001 => float_fms_zero(a, b, c),
            0b010 => float_fms_down(a, b, c),
            0b011 => float_fms_up(a, b, c),
            0b100 => float_fms_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fnms(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fnms_near(a, b, c),
            0b001 => float_fnms_zero(a, b, c),
            0b010 => float_fnms_down(a, b, c),
            0b011 => float_fnms_up(a, b, c),
            0b100 => float_fnms_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_fnma(a: f32, b: f32, c: f32, rm: u8) -> f32 {
        match rm {
            0b000 => float_fnma_near(a, b, c),
            0b001 => float_fnma_zero(a, b, c),
            0b010 => float_fnma_down(a, b, c),
            0b011 => float_fnma_up(a, b, c),
            0b100 => float_fnma_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_mul_down(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "mulss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_mul_up(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "mulss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_mul_near(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "mulss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_mul_zero(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "mulss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_fma_down(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fma_up(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fma_near(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fma_zero(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnma_down(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnma_up(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnma_near(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnma_zero(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fms_down(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fms_up(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fms_near(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fms_zero(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnms_down(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnms_up(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnms_near(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_fnms_zero(a: f32, b: f32, c: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213ss {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn float_add_down(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "addss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_add_up(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "addss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_add_near(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "addss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_add_zero(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "addss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_sub_down(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "subss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_sub_up(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "subss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_sub_near(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "subss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_sub_zero(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "subss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_div_down(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "divss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_div_up(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "divss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_div_near(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "divss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_div_zero(a: f32, b: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "divss {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn float_sqrt_down(a: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "sqrtss {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn float_sqrt_up(a: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "sqrtss {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn float_sqrt_near(a: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "sqrtss {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn float_sqrt_zero(a: f32) -> f32 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "sqrtss {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn double_mul(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_mul_near(a, b),
            0b001 => double_mul_zero(a, b),
            0b010 => double_mul_down(a, b),
            0b011 => double_mul_up(a, b),
            0b100 => double_mul_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_sqrt(a: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_sqrt_near(a),
            0b001 => double_sqrt_zero(a),
            0b010 => double_sqrt_down(a),
            0b011 => double_sqrt_up(a),
            0b100 => double_sqrt_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_add(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_add_near(a, b),
            0b001 => double_add_zero(a, b),
            0b010 => double_add_down(a, b),
            0b011 => double_add_up(a, b),
            0b100 => double_add_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_sub(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_sub_near(a, b),
            0b001 => double_sub_zero(a, b),
            0b010 => double_sub_down(a, b),
            0b011 => double_sub_up(a, b),
            0b100 => double_sub_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_div(a: f64, b: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_div_near(a, b),
            0b001 => double_div_zero(a, b),
            0b010 => double_div_down(a, b),
            0b011 => double_div_up(a, b),
            0b100 => double_div_near(a, b),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fma(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fma_near(a, b, c),
            0b001 => double_fma_zero(a, b, c),
            0b010 => double_fma_down(a, b, c),
            0b011 => double_fma_up(a, b, c),
            0b100 => double_fma_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fms(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fms_near(a, b, c),
            0b001 => double_fms_zero(a, b, c),
            0b010 => double_fms_down(a, b, c),
            0b011 => double_fms_up(a, b, c),
            0b100 => double_fms_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fnms(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fnms_near(a, b, c),
            0b001 => double_fnms_zero(a, b, c),
            0b010 => double_fnms_down(a, b, c),
            0b011 => double_fnms_up(a, b, c),
            0b100 => double_fnms_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_fnma(a: f64, b: f64, c: f64, rm: u8) -> f64 {
        match rm {
            0b000 => double_fnma_near(a, b, c),
            0b001 => double_fnma_zero(a, b, c),
            0b010 => double_fnma_down(a, b, c),
            0b011 => double_fnma_up(a, b, c),
            0b100 => double_fnma_near(a, b, c),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_mul_down(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "mulsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_mul_up(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "mulsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_mul_near(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "mulsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_mul_zero(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "mulsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_fnms_down(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnms_up(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnms_near(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnms_zero(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfnmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnma_down(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnma_up(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnma_near(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fnma_zero(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfnmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fma_down(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fma_up(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fma_near(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fma_zero(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfmadd213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fms_down(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fms_up(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fms_near(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_fms_zero(a: f64, b: f64, c: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "vfmsub213sd {res}, {b}, {c}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b,
                c = in(xmm_reg) c
            );
        }
        res
    }

    pub fn double_add_down(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "addsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_add_up(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "addsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_add_near(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "addsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_add_zero(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "addsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_sub_down(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "subsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_sub_up(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "subsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_sub_near(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "subsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_sub_zero(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "subsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_div_down(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "divsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_div_up(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "divsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_div_near(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "divsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_div_zero(a: f64, b: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "divsd {res}, {b}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
                b = in(xmm_reg) b
            );
        }
        res
    }

    pub fn double_sqrt_down(a: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "sqrtsd {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn double_sqrt_up(a: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "sqrtsd {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn double_sqrt_near(a: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "sqrtsd {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn double_sqrt_zero(a: f64) -> f64 {
        let mut res = a;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "sqrtsd {res}, {res}",
                "ldmxcsr [rsp]",  // restore MXCSR
                "add rsp, 8",
                res = inout(xmm_reg) res,
            );
        }
        res
    }

    pub fn float_to_i32_down(a: f32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i32_up(a: f32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i32_near(a: f32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i32_zero(a: f32) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i64_down(a: f32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i64_up(a: f32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i64_near(a: f32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn float_to_i64_zero(a: f32) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtss2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn i32_to_float_down(a: i32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_float_up(a: i32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_float_near(a: i32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_float_zero(a: i32) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_float_down(a: i64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:r}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_float_up(a: i64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a:r}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_float_near(a: i64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_float_zero(a: i64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn float_to_i32 (a: f32, rm: u8) -> i32 {
        match rm {
            0b000 => float_to_i32_near(a),
            0b001 => float_to_i32_zero(a),
            0b010 => float_to_i32_down(a),
            0b011 => float_to_i32_up(a),
            0b100 => float_to_i32_near(a),
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
            0b000 => float_to_i64_near(a),
            0b001 => float_to_i64_zero(a),
            0b010 => float_to_i64_down(a),
            0b011 => float_to_i64_up(a),
            0b100 => float_to_i64_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn float_to_u64 (a: f32, rm: u8) -> u64 {
        float_to_i64(a, rm) as u64
    }

    pub fn i32_to_float (a: i32, rm: u8) -> f32 {
        match rm {
            0b000 => i32_to_float_near(a),
            0b001 => i32_to_float_zero(a),
            0b010 => i32_to_float_down(a),
            0b011 => i32_to_float_up(a),
            0b100 => i32_to_float_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u32_to_float (a: u32, rm: u8) -> f32 {
        i64_to_float(a as i64, rm)
    }

    pub fn i64_to_float (a: i64, rm: u8) -> f32 {
        match rm {
            0b000 => i64_to_float_near(a),
            0b001 => i64_to_float_zero(a),
            0b010 => i64_to_float_down(a),
            0b011 => i64_to_float_up(a),
            0b100 => i64_to_float_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u64_to_float (a: u64, rm: u8) -> f32 {
        i64_to_float(a as i64, rm)
    }

    pub fn double_to_i32_down(a: f64) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i32_up(a: f64) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i32_near(a: f64) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i32_zero(a: f64) -> i32 {
        let mut res : i32 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:e}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }
    pub fn i32_to_double_down(a: i32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_double_up(a: i32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_double_near(a: i32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i32_to_double_zero(a: i32) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a:e}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn double_to_i64_down(a: f64) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i64_up(a: f64) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i64_near(a: f64) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_i64_zero(a: f64) -> i64 {
        let mut res : i64 = 0x0000;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2si {res:r}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }
    pub fn i64_to_double_down(a: i64) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_double_up(a: i64) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_double_near(a: i64) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn i64_to_double_zero(a: i64) -> f64 {
        let mut res : f64 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsi2sd {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(reg) a
            );
        }
        res
    }

    pub fn double_to_i32 (a: f64, rm: u8) -> i32 {
        match rm {
            0b000 => double_to_i32_near(a),
            0b001 => double_to_i32_zero(a),
            0b010 => double_to_i32_down(a),
            0b011 => double_to_i32_up(a),
            0b100 => double_to_i32_near(a),
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
            0b000 => i32_to_double_near(a),
            0b001 => i32_to_double_zero(a),
            0b010 => i32_to_double_down(a),
            0b011 => i32_to_double_up(a),
            0b100 => i32_to_double_near(a),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u32_to_double (a: u32, rm: u8) -> f64 {
        i32_to_double(a as i32, rm)
    }

    pub fn double_to_i64 (a: f64, rm: u8) -> i64 {
        match rm {
            0b000 => double_to_i64_near(a),
            0b001 => double_to_i64_zero(a),
            0b010 => double_to_i64_down(a),
            0b011 => double_to_i64_up(a),
            0b100 => double_to_i64_near(a),
            0b101..0b111 => {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Returned 0.");
                0
            },
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn double_to_u64 (a: f64, rm: u8) -> u64 {
        double_to_i64(a, rm) as u64
    }

    pub fn i64_to_double (a: i64, rm: u8) -> f64 {
        match rm {
            0b000 => i64_to_double_near(a),
            0b001 => i64_to_double_zero(a),
            0b010 => i64_to_double_down(a),
            0b011 => i64_to_double_up(a),
            0b100 => i64_to_double_near(a),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f64::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }

    pub fn u64_to_double (a: u64, rm: u8) -> f64 {
        i64_to_double(a as i64, rm)
    }

    pub fn double_to_float_down(a: f64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x3F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_float_up(a: f64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x5F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }
    
    pub fn double_to_float_near(a: f64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x1F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_float_zero(a: f64) -> f32 {
        let mut res : f32 = 0.0;
        unsafe {
            asm!(
                "sub rsp, 8",
                "stmxcsr [rsp]",
                "mov dword ptr [rsp + 4], 0x7F80",
                "ldmxcsr [rsp + 4]",
                "cvtsd2ss {res}, {a}",
                "ldmxcsr [rsp]",  // restore MXCSR 
                "add rsp, 8",
                res = inout(xmm_reg) res,
                a = in(xmm_reg) a
            );
        }
        res
    }

    pub fn double_to_float (a: f64, rm: u8) -> f32 {
         match rm {
            0b000 => double_to_float_near(a),
            0b001 => double_to_float_zero(a),
            0b010 => double_to_float_down(a),
            0b011 => double_to_float_up(a),
            0b100 => double_to_float_near(a),
            0b101..0b111 =>  {
                eprintln!("WARNING: Reserved RM used ({rm:03b}). Canonical NaN returned.");
                f32::NAN
            }
            _ => unreachable!("Should not be possible, RM is 3 bits only."),
        }
    }
}
