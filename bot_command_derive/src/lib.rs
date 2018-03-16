extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(BotCommand)]
pub fn bot_command(input: TokenStream) -> TokenStream {
  // Parse the string representation
  let ast = syn::parse(input).unwrap();

  // Build the impl
  let gen = impl_bot_command(&ast);

  // Return the generated impl
  gen.into()
}

fn impl_bot_command(ast: &syn::DeriveInput) -> quote::Tokens {
  let struct_data = match ast.data {
    syn::Data::Struct(ref s) => s,
    _ => panic!("cannot derive BotCommand on anything but a struct")
  };
  let name = &ast.ident;
  let mut field_name = None;
  for field in struct_data.fields.iter() {
    let path = match field.ty {
      syn::Type::Path(ref p) => p,
      _ => continue
    };
    if path.path.clone().into_tokens().to_string() == "Arc < BotEnv >" {
      field_name = Some(field.ident.expect("cannot derive for tuple structs"));
      break;
    }
  }
  match field_name {
    Some(ident) => quote! {
      impl ::commands::BotCommand for #name {
        #[cfg_attr(feature = "cargo-clippy", allow(redundant_field_names))]
        fn new(env: Arc<::bot::BotEnv>) -> Self {
          #name { #ident: env }
        }
      }
    },
    None => quote! {
      impl ::commands::BotCommand for #name {
        fn new(_: Arc<::bot::BotEnv>) -> Self {
          #name
        }
      }
    }
  }
}
