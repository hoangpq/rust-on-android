extern crate proc_macro;
use proc_macro::TokenStream;
use syn;

#[macro_use]
extern crate quote;
extern crate v8;

#[proc_macro_attribute]
pub fn v8_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::ItemFn);
    let name = ast.ident;
    let inputs = ast.decl.inputs;
    let block = ast.block;
    let vis = ast.vis;

    (quote! {
        #[no_mangle]
        #vis extern "C" fn #name(args: &v8::CallbackInfo) {
            (|#inputs|#block)(args);
        }
    })
    .into()
}
