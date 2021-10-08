use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use rtic_syntax::ast::App;
use syn::Index;

use crate::{analyze::Analysis, codegen::util};

/// Generates code that runs after `#[init]` returns
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    let mut stmts = vec![];

    // Initialize shared resources
    for (name, res) in &app.shared_resources {
        let mangled_name = util::static_shared_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.shared_resource_locations.get(name).is_some() {
            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(shared_resources.#name);
            ));
        }
    }

    // Initialize local resources
    for (name, res) in &app.local_resources {
        let mangled_name = util::static_local_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.local_resource_locations.get(name).is_some() {
            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(local_resources.#name);
            ));
        }
    }

    // Initialize ModalityProbe resources
    for (task_name, probe) in app.modality_probes().iter() {
        let mangled_storage_name =
            util::declared_static_local_resource_ident(&probe.storage_name, task_name);
        let mangled_probe_name =
            util::declared_static_local_resource_ident(&probe.local_name, task_name);
        let probe_id_const =
            syn::Ident::new(&probe.name.to_string().to_uppercase(), Span::call_site());
        stmts.push(quote!(
            #mangled_probe_name.get_mut_unchecked().as_mut_ptr().write(
                    modality_probe_sys::ModalityProbe::new_with_storage(
                        #mangled_storage_name.get_mut_unchecked(),
                        #probe_id_const,
                        modality_probe_sys::NanosecondResolution::UNSPECIFIED,
                        modality_probe_sys::WallClockId::LOCAL_ONLY,
                        None,
                    ).expect("Failed to initialize ModalityProbe")
                );
        ));
    }

    for (i, (monotonic, _)) in app.monotonics.iter().enumerate() {
        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
        // stmts.push(quote!(#[doc = #doc]));

        let idx = Index {
            index: i as u32,
            span: Span::call_site(),
        };
        stmts.push(quote!(monotonics.#idx.reset();));

        // Store the monotonic
        let name = util::monotonic_ident(&monotonic.to_string());
        stmts.push(quote!(*#name.get_mut_unchecked() = Some(monotonics.#idx);));
    }

    // Enable the interrupts -- this completes the `init`-ialization phase
    stmts.push(quote!(rtic::export::interrupt::enable();));

    stmts
}
