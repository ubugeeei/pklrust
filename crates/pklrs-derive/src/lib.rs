extern crate proc_macro;

mod from_pkl;
mod pkl_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

/// Derive macro that generates a `from_pkl_value` method for deserializing Pkl values.
///
/// # Attributes
///
/// - `#[pkl(rename = "...")]` — rename the field
/// - `#[pkl(default)]` — use Default::default() for missing fields
/// - `#[pkl(default = "path")]` — use a custom default function
#[proc_macro_derive(FromPkl, attributes(pkl))]
pub fn derive_from_pkl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = from_pkl::impl_from_pkl(&input);
    TokenStream::from(expanded)
}

/// Evaluate inline PKL source written as tokens.
///
/// Converts the token stream into PKL source and generates code that
/// evaluates it at runtime, returning `pklrs::Result<pklrs::PklValue>`.
///
/// # Supported constructs
///
/// - Properties: `host = "localhost"`
/// - Nested objects: `database { url = "..." }`
/// - Classes: `class Server { host: String }`
/// - Functions: `function add(a, b) = a + b`
/// - Imports: `import("pkl:json")`
/// - Pipe operators: `items |> filter(...)`
/// - For/when generators: `for (x in xs) { ... }`
/// - Type annotations: `port: UInt16 = 8080`
/// - Modifiers: `local`, `hidden`, `fixed`, `const`, etc.
///
/// # Limitations
///
/// - PKL raw strings (`#"..."#`) — use regular strings instead
/// - PKL string interpolation (`\(expr)`) — not supported
/// - PKL multi-line strings (`"""..."""`) — not supported
///
/// # Example
///
/// ```ignore
/// use pklrs::pkl;
///
/// let value = pkl! {
///     host = "localhost"
///     port = 8080
///     database {
///         url = "postgres://localhost/mydb"
///         maxConnections = 10
///     }
/// }?;
/// ```
#[proc_macro]
pub fn pkl(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let trees: Vec<TokenTree> = input2.into_iter().collect();
    let pkl_source = pkl_macro::tokens_to_pkl(&trees);

    let expanded = quote! {
        ::pklrs::evaluate_text(#pkl_source)
    };

    expanded.into()
}
