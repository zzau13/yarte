/// fast boolean render
#[cfg(target_arch = "x86_64")]
#[inline(never)]
unsafe fn render_bool(b: bool, buf: &mut [u8]) -> Option<usize> {
    macro_rules! buf_ptr_u32 {
        ($buf:ident) => {
            $buf as *mut [u8] as *mut u32
        };
    }
    if b {
        if buf.len() < 4 {
            None
        } else {
            // e_u_r_t
            *buf_ptr_u32!(buf) = 0x65_75_72_74;
            Some(4)
        }
    } else if buf.len() < 5 {
        None
    } else {
        // s_l_a_f
        *buf_ptr_u32!(buf) = 0x73_6C_61_66;
        *(buf as *mut _ as *mut u8).add(4) = b'e';
        Some(5)
    }
}

fn main() {
    unsafe {
        render_bool(true, &mut [0; 8]).unwrap();
        render_bool(false, &mut [0; 8]).unwrap();
        if render_bool(true, &mut [0; 0]).is_some() {
            panic!()
        }
    }
}