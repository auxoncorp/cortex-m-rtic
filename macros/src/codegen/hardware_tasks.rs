use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{local_resources_struct, module, shared_resources_struct, tracing},
};

/// Generate support code for hardware tasks (`#[exception]`s and `#[interrupt]`s)
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app_hardware_tasks -- interrupt handlers and `${task}Resources` constructors
    Vec<TokenStream2>,
    // root_hardware_tasks -- items that must be placed in the root of the crate:
    // - `${task}Locals` structs
    // - `${task}Resources` structs
    // - `${task}` modules
    Vec<TokenStream2>,
    // user_hardware_tasks -- the `#[task]` functions written by the user
    Vec<TokenStream2>,
) {
    let mut mod_app = vec![];
    let mut root = vec![];
    let mut user_tasks = vec![];
    let device = &extra.device;

    for (name, task) in &app.hardware_tasks {
        let symbol = task.args.binds.clone();
        let priority = task.args.priority;
        let cfgs = &task.cfgs;
        let attrs = &task.attrs;
        let user_hardware_task_isr_doc = &format!(" User HW task ISR trampoline for {name}");
        let tp_int_enter = tracing::tp_interrupt_enter(device, priority, &symbol);
        let tp_int_exit = tracing::tp_interrupt_exit();
        let tp_task_enter = tracing::tp_hw_task_enter(name, priority, &symbol);
        let tp_task_exit = tracing::tp_task_exit();

        mod_app.push(quote!(
            #[allow(non_snake_case)]
            #[no_mangle]
            #[doc = #user_hardware_task_isr_doc]
            #(#attrs)*
            #(#cfgs)*
            unsafe fn #symbol() {
                const PRIORITY: u8 = #priority;
                #tp_int_enter
                rtic::export::run(PRIORITY, || {
                    #tp_task_enter
                    #name(
                        #name::Context::new(&rtic::export::Priority::new(PRIORITY))
                    );
                    #tp_task_exit
                });
                #tp_int_exit
            }
        ));

        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let (item, constructor) = local_resources_struct::codegen(
                Context::HardwareTask(name),
                &mut local_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        // `${task}Resources`
        if !task.args.shared_resources.is_empty() {
            let (item, constructor) = shared_resources_struct::codegen(
                Context::HardwareTask(name),
                &mut shared_needs_lt,
                app,
            );

            root.push(item);

            mod_app.push(constructor);
        }

        root.push(module::codegen(
            Context::HardwareTask(name),
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
            extra,
        ));

        let user_hardware_task_doc = &format!(" User HW task: {name}");
        if !task.is_extern {
            let attrs = &task.attrs;
            let cfgs = &task.cfgs;
            let context = &task.context;
            let stmts = &task.stmts;
            user_tasks.push(quote!(
                #[doc = #user_hardware_task_doc]
                #(#attrs)*
                #(#cfgs)*
                #[allow(non_snake_case)]
                fn #name(#context: #name::Context) {
                    use rtic::Mutex as _;
                    use rtic::mutex::prelude::*;

                    #(#stmts)*
                }
            ));
        }
    }

    (mod_app, root, user_tasks)
}
