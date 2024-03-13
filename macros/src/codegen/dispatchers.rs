use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{
    analyze::Analysis,
    check::Extra,
    codegen::{tracing, util},
};

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis, extra: &Extra) -> Vec<TokenStream2> {
    let mut items = vec![];

    let interrupts = &analysis.interrupts;

    for (&level, channel) in &analysis.channels {
        let mut stmts = vec![];

        let variants = channel
            .tasks
            .iter()
            .map(|name| {
                let cfgs = &app.software_tasks[name].cfgs;

                quote!(
                    #(#cfgs)*
                    #name
                )
            })
            .collect::<Vec<_>>();

        // For future use
        // let doc = format!(
        //     "Software tasks to be dispatched at priority level {}",
        //     level,
        // );
        let t = util::spawn_t_ident(level);
        items.push(quote!(
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            // #[doc = #doc]
            #[doc(hidden)]
            pub enum #t {
                #(#variants,)*
            }
        ));

        let n = util::capacity_literal(channel.capacity as usize + 1);
        let rq = util::rq_ident(level);
        let (rq_ty, rq_expr) = {
            (
                quote!(rtic::export::SCRQ<#t, #n>),
                quote!(rtic::export::Queue::new()),
            )
        };

        // For future use
        // let doc = format!(
        //     "Queue of tasks ready to be dispatched at priority level {}",
        //     level
        // );
        items.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            static #rq: rtic::RacyCell<#rq_ty> = rtic::RacyCell::new(#rq_expr);
        ));

        let interrupt = util::suffixed(&interrupts[&level].0.to_string());
        let attribute = &interrupts[&level].1.attrs;

        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let fq = util::fq_ident(name);
                let inputs = util::inputs_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);
                let arg_cnt = pats.len() as u8;
                let tp_task_enter = tracing::tp_sw_task_enter(name, level, &interrupt, arg_cnt);
                let tp_task_exit = tracing::tp_task_exit();

                quote!(
                    #(#cfgs)*
                    #t::#name => {
                        let #tupled =
                            (&*#inputs
                            .get())
                            .get_unchecked(usize::from(index))
                            .as_ptr()
                            .read();
                        (&mut *#fq.get_mut()).split().0.enqueue_unchecked(index);
                        let priority = &rtic::export::Priority::new(PRIORITY);
                        #tp_task_enter
                        #name(
                            #name::Context::new(priority)
                            #(,#pats)*
                        );
                        #tp_task_exit
                    }
                )
            })
            .collect::<Vec<_>>();

        stmts.push(quote!(
            while let Some((task, index)) = (&mut *#rq.get_mut()).split().1.dequeue() {
                match task {
                    #(#arms)*
                }
            }
        ));

        let doc = format!("Interrupt handler to dispatch tasks at priority {}", level);
        let device = &extra.device;
        let tp_int_enter = tracing::tp_interrupt_enter(device, level, &interrupt);
        let tp_int_exit = tracing::tp_interrupt_exit();
        items.push(quote!(
            #[allow(non_snake_case)]
            #[doc = #doc]
            #[no_mangle]
            #(#attribute)*
            unsafe fn #interrupt() {
                /// The priority of this interrupt handler
                const PRIORITY: u8 = #level;
                #tp_int_enter
                rtic::export::run(PRIORITY, || {
                    #(#stmts)*
                });
                #tp_int_exit
            }
        ));
    }

    items
}
