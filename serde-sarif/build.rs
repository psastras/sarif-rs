use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use schemafy_lib::Expander;
use schemafy_lib::Schema;

// Add additional items to the generated sarif.rs file
// Currently adds: derive(Builder) to each struct
// and apprpriate use statements at the top of the file
// todo: this (and other parts) need a refactor and tests
fn process_token_stream(input: proc_macro2::TokenStream) -> syn::File {
  let mut ast: syn::File = syn::parse2(input).unwrap();

  // add use directives to top of the file
  ast.items.insert(
    0,
    syn::parse_quote! {
      use serde::{Serialize, Deserialize};
    },
  );
  ast.items.insert(
    0,
    syn::parse_quote! {
      use derive_builder::Builder;
    },
  );

  // Checks if the type is an Option type (returns true if yes, false otherwise)
  fn path_is_option(path: &syn::Path) -> bool {
    let idents_of_path =
      path
        .segments
        .iter()
        .into_iter()
        .fold(String::new(), |mut acc, v| {
          acc.push_str(&v.ident.to_string());
          acc.push('|');
          acc
        });

    vec!["Option|", "std|option|Option|", "core|option|Option|"]
      .into_iter()
      .find(|s| idents_of_path == *s)
      != None
  }

  (&mut ast.items).iter_mut().for_each(|ref mut item| {
    if let syn::Item::Struct(s) = item {
      // add builder attributes to each struct
      s.attrs.extend(vec![
        syn::parse_quote! {
          #[derive(Builder)]
        },
        syn::parse_quote! {
          #[builder(setter(into, strip_option))]
        },
      ]);

      // for each struct field, if that field is Optional, set None
      // as the default value when using the builder
      (&mut s.fields).into_iter().for_each(|ref mut field| {
        if let syn::Type::Path(typepath) = &field.ty {
          if path_is_option(&typepath.path) {
            field.attrs.push(syn::parse_quote! {
              #[builder(setter(into, strip_option), default)]
            })
          }
        }
      })
    }
  });

  ast
}

fn main() -> Result<()> {
  // Rerun if the schema changes
  println!("cargo:rerun-if-changed=src/schema.json");
  let path = Path::new("src/schema.json");

  // Generate the Rust schema struct
  let json = std::fs::read_to_string(path).unwrap();
  let schema: Schema = serde_json::from_str(&json)?;
  let path_str = path.to_str().unwrap();
  let mut expander = Expander::new(Some("Sarif"), path_str, &schema);
  let generated = process_token_stream(expander.expand(&schema));

  // Write the struct to the $OUT_DIR/sarif.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  let mut file = File::create(out_path.join("sarif.rs"))?;
  file.write_all(prettyplease::unparse(&generated).as_bytes())?;

  Ok(())
}
