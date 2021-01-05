use proc_macro::TokenStream;

use quote::quote;
use syn::ItemFn;


/// Sugar for "wrapping" entry routines.
///
/// This will produce `"panic_handler"` and `"eh_personality"` functions as well
/// as a naked function named `"__kmain"` (where assembly bootstrap stubs should
/// jump to)
///
/// The generated kmain will set a logger using `log` and invoke
/// architecture specific boot code with a `multiboot2::BootInformation`
/// struct before calling into the decorated entry function.
///
/// Interrupts will be disabled and a heap will **not** be provided. 
#[proc_macro_attribute]
pub fn entry(_args: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        sig,
        attrs,
        vis,
        block,
    } = syn::parse_macro_input!(item as ItemFn);

    let ident = &sig.ident;

    if !sig.inputs.is_empty() {
        let msg = "the main function cannot accept arguments";
        return syn::Error::new_spanned(&sig.inputs, msg)
            .to_compile_error()
            .into();
    }

    if sig.asyncness.is_some() {
        let msg = "entry function can not be async.";
        return syn::Error::new_spanned(sig.asyncness, msg).to_compile_error().into();
    }

    let result = quote! {
        use ::mem::*;

        #(#attrs)*
        #vis #sig {
            #block
        }

        /// Panic handler impl.
        #[panic_handler]
        fn panic(info: &::core::panic::PanicInfo) -> ! {
            ::arch::prelude::panic_handler(info)
        }

        /// Exception handling personality (used to implement unwinding behaviour.)
        #[lang = "eh_personality"]
        #[no_mangle]
        pub extern "C" fn eh_personality() {
            todo!()
        }

        /// Automagically generated from the `entry` macro.
        #[no_mangle]
        pub unsafe extern "C" fn __kmain(multiboot_address: usize) {
            let level = match ::core::option_env!("LOG_LEVEL") {
                None => ::log::LevelFilter::Trace,
                Some(level) => match level {
                    "trace" => ::log::LevelFilter::Trace,
                    "debug" => ::log::LevelFilter::Debug,
                    "info" => ::log::LevelFilter::Info,
                    "warn" => ::log::LevelFilter::Warn,
                    "error" => ::log::LevelFilter::Error,
                    _ => ::log::LevelFilter::Trace,
                }
            };

            ::arch::prelude::install_logger(level);

            let boot_info = multiboot2::load(multiboot_address);

            ::arch::prelude::boot(boot_info);

            #ident();
        }
    };

    TokenStream::from(result)
}

/// Attribute macro for decorating functions that are only supposed to run once.
#[proc_macro_attribute]
pub fn once(_args: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        sig,
        attrs,
        vis,
        block,
    } = syn::parse_macro_input!(item as ItemFn);

    let ident = &sig.ident;
    let panic_msg = format!("FATAL! Tried to call function twice ({:?})", ident.to_string());

    let result = quote! {
        #(#attrs)*
        #vis #sig {
            static __ONCE_GUARD: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(true);

            if __ONCE_GUARD.compare_and_swap(true, false, ::core::sync::atomic::Ordering::SeqCst) {
                #block
            } else {
                panic!(#panic_msg);
            }
        }
    };

    TokenStream::from(result)
}
