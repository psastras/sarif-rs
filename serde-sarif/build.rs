use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use schemafy_lib::Expander;
use schemafy_lib::Schema;
use syn::parse::Parser;

// Add additional items to the generated sarif.rs file
// Currently adds: derive(TypedBuilder) to each struct
// and appropriate use statements at the top of the file
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
      use typed_builder::TypedBuilder;
    },
  );
  ast.items.insert(
    0,
    syn::parse_quote! {
      use std::collections::BTreeMap;
    },
  );

  // Checks if the type is an Option type (returns true if yes, false otherwise)
  fn path_is_option(path: &syn::Path) -> bool {
    let idents_of_path =
      path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
      });

    vec!["Option|", "std|option|Option|", "core|option|Option|"]
      .into_iter()
      .any(|s| idents_of_path == *s)
  }

  // Checks if the type is a collection type (returns true if yes, false otherwise)
  fn path_is_vec(path: &syn::Path) -> bool {
    let idents_of_path =
      path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
      });

    vec!["Vec|", "std|vec|Vec|"]
      .into_iter()
      .any(|s| idents_of_path.starts_with(s))
  }

  ast.items.iter_mut().for_each(|ref mut item| {
    if let syn::Item::Struct(s) = item {
      // add builder attributes to each struct
      s.attrs.extend(vec![
        syn::parse_quote! {
          #[derive(TypedBuilder)]
        },
        syn::parse_quote! {
          #[builder(field_defaults(setter(into)))]
        },
      ]);

      // Add a field to PropertyBag to allow arbitrary JSON data
      // The proper way to do this would be to modify the JSON schema parsing library to properly
      // output this. This is a workaround since there's only one struct in the SARIF schema that requires this.
      if s.ident == "PropertyBag" {
        if let syn::Fields::Named(fields) = &mut s.fields {
          fields.named.push(
            syn::Field::parse_named
              .parse2(syn::parse_quote! {
                #[doc = r"Arbitrary properties to include in the PropertyBag"]
                #[serde(flatten)]
                #[builder(default = ::std::collections::BTreeMap::new())]
                pub additional_properties: BTreeMap<String, serde_json::Value>
              })
              .unwrap(),
          );
        }
      }

      // Rewrite Result::kind and Result::level to use ResultKind and
      // ResultLevel instead of serde_json::Value.
      // This is a workaround for schemafy's inability to produce appropriate
      // exhaustive enums here.
      if s.ident == "Result" {
        if let syn::Fields::Named(fields) = &mut s.fields {
          for field in fields.named.iter_mut() {
            if field.ident.as_ref().unwrap() == "kind" {
              field.ty = syn::parse_quote! { Option<ResultKind> };
            } else if field.ident.as_ref().unwrap() == "level" {
              field.ty = syn::parse_quote! { Option<ResultLevel> };
            }
          }
        }
      }

      // for each struct field, if that field is Optional, set None
      // as the default value when using the builder
      (&mut s.fields).into_iter().for_each(|ref mut field| {
        if let syn::Type::Path(typepath) = &field.ty {
          if path_is_option(&typepath.path) {
            #[cfg(not(feature = "opt-builder"))]
            field.attrs.push(syn::parse_quote! {
              #[builder(setter(strip_option), default)]
            });

            #[cfg(feature = "opt-builder")]
            field.attrs.push(syn::parse_quote! {
              #[builder(setter(strip_option(fallback_prefix = "opt_")), default)]
            });
          } else if path_is_vec(&typepath.path) {
            field.attrs.push(syn::parse_quote! {
              #[builder(default)]
            })
          }
        }
      });
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
