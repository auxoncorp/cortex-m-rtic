//! Trace point generators

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Path};

pub fn tp_trace_start(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TRACE_START::version=1,task={task}");
    quote!(
        defmt::trace!(#fmt_str);
    )
}

pub fn tp_idle_task_enter(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_ENTER::task={task},priority=0");
    quote!(
        defmt::trace!(#fmt_str);
    )
}

pub fn tp_interrupt_enter(device: &Path, priority: u8, interrupt: &Ident) -> TokenStream2 {
    let fmt_str =
        format!("AUXON_INTERRUPT_ENTER::interrupt={{=u32}},priority={priority},isr={interrupt}");
    quote!(
        defmt::trace!(#fmt_str, #device::Interrupt::#interrupt as u32);
    )
}

pub fn tp_interrupt_exit() -> TokenStream2 {
    quote!(
        defmt::trace!("AUXON_INTERRUPT_EXIT");
    )
}

pub fn tp_sw_task_enter(
    task: &Ident,
    priority: u8,
    dispatcher: &Ident,
    arg_cnt: u8,
) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_ENTER::task={task},priority={priority},dispatcher={dispatcher},arg_cnt={arg_cnt},queue_index={{=u8}}");
    quote!(
        defmt::trace!(#fmt_str, index);
    )
}

pub fn tp_hw_task_enter(task: &Ident, priority: u8, interrupt: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_ENTER::task={task},priority={priority},isr={interrupt}");
    quote!(
        defmt::trace!(#fmt_str);
    )
}

pub fn tp_task_exit() -> TokenStream2 {
    quote!(
        defmt::trace!("AUXON_TASK_EXIT");
    )
}

pub fn tp_task_spawn(
    device: &Path,
    enum_: &Ident,
    task: &Ident,
    interrupt: &Ident,
    arg_cnt: u8,
) -> TokenStream2 {
    let fmt_str = format!(
        "AUXON_TASK_SPAWN::task={task},interrupt={{=u32}},isr={interrupt},arg_cnt={arg_cnt},queue_index={{=u8}}"
    );
    quote!(
        defmt::trace!(#fmt_str, #device::#enum_::#interrupt as u32, index);
    )
}

pub fn tp_task_spawn_failed(
    device: &Path,
    enum_: &Ident,
    task: &Ident,
    interrupt: &Ident,
    arg_cnt: u8,
) -> TokenStream2 {
    let fmt_str = format!(
        "AUXON_TASK_SPAWN_FAILED::task={task},interrupt={{=u32}},isr={interrupt},arg_cnt={arg_cnt}"
    );
    quote!(
        defmt::trace!(#fmt_str, #device::#enum_::#interrupt as u32);
    )
}

pub fn tp_task_spawn_after(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_SPAWN_AFTER::task={task},instant={{=u64}},duration={{=u64}}");
    quote!(
        defmt::trace!(#fmt_str, instant.ticks(), duration.ticks());
    )
}

pub fn tp_task_spawn_at(task: &Ident, arg_cnt: u8) -> TokenStream2 {
    let fmt_str = format!(
        "AUXON_TASK_SPAWN_AT::task={task},instant={{=u64}},arg_cnt={arg_cnt},queue_index={{=u8}}"
    );
    quote!(
        defmt::trace!(#fmt_str, instant.ticks(), index);
    )
}

pub fn tp_task_spawn_at_failed(task: &Ident, arg_cnt: u8) -> TokenStream2 {
    let fmt_str =
        format!("AUXON_TASK_SPAWN_AT_FAILED::task={task},instant={{=u64}},arg_cnt={arg_cnt}");
    quote!(
        defmt::trace!(#fmt_str, instant.ticks());
    )
}

pub fn tp_task_cancel(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_CANCEL::task={task}");
    quote!(
        defmt::trace!(#fmt_str);
    )
}

pub fn tp_task_reschedule_after(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_RESCHEDULE_AFTER::task={task},duration={{=u64}}");
    quote!(
        defmt::trace!(#fmt_str, duration.ticks());
    )
}

pub fn tp_task_reschedule_at(task: &Ident) -> TokenStream2 {
    let fmt_str = format!("AUXON_TASK_RESCHEDULE_AT::task={task},instant={{=u64}}");
    quote!(
        defmt::trace!(#fmt_str, instant.ticks());
    )
}
