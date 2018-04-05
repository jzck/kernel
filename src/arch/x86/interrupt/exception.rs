// https://wiki.osdev.org/Exceptions

macro_rules! exception {
    ($name:ident, $func:block) => {
        pub extern "x86-interrupt" fn $name(stack_frame: &mut ExceptionStackFrame)
        {
            println!("Exception: {}", stringify!($name));
            println!("{:#?}", stack_frame);
            flush!();

            #[allow(unused_variables)]
            fn inner(stack: &mut ExceptionStackFrame) {
                $func
            }
            inner(stack_frame);
        }
    }
}

macro_rules! exception_err {
    ($name:ident, $func:block) => {
        pub extern "x86-interrupt" fn $name(
            stack_frame: &mut ExceptionStackFrame, _error_code: u32)
        {
            println!("Exception: {}", stringify!($name));
            println!("{:#?}", stack_frame);
            flush!();

            #[allow(unused_variables)]
            fn inner(stack: &mut ExceptionStackFrame) {
                $func
            }
            inner(stack_frame);
        }
    }
}

use x86::structures::idt::*;

exception!(divide_by_zero, {});
exception!(debug, {});
exception!(non_maskable, {});
exception!(breakpoint, {
    println!("testing here dont mind me");
    flush!();
});
exception!(overflow, {});
exception!(bound_range, {});
exception!(invalid_opcode, {});
exception!(device_not_available, {});
exception_err!(double_fault, {});
exception!(coprocessor_segment_overrun, {});
exception_err!(invalid_tss, {});
exception_err!(segment_not_present, {});
exception_err!(stack_segment, {});
exception_err!(general_protection, {});

pub extern "x86-interrupt" fn page_fault(
    stack_frame: &mut ExceptionStackFrame, code: PageFaultErrorCode)
{
    println!("Exception: page_fault");
    println!("Error code: {:#b}", code);
    println!("{:#?}", stack_frame);
    flush!();
}

exception!(x87_fpu, {});
exception_err!(alignment_check, {});
exception!(machine_check, {});
exception!(simd, {});
exception!(virtualization, {});
exception_err!(security, {});