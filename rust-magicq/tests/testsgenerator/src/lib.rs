use proc_macro::{TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Ident, Lit, Meta, MetaNameValue, LitStr};
use walkdir::WalkDir;

#[proc_macro]
pub fn generate_tests(input: TokenStream) -> TokenStream {
    // Parse the input as a `LitStr`
    let dir = parse_macro_input!(input as LitStr);

    // Get the directory path as a string
    let dir_path = dir.value();

    // Find all show files in the directory
    let shw_files = WalkDir::new(&dir_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().map(|ext| ext == "shw").unwrap_or(false))
        .map(|entry| entry.path().to_owned())
        .collect::<Vec<_>>();

    // Generate a test for each file
    let tests = shw_files.iter().map(|file| {
        let file_name = file.to_string_lossy();
        let file_path = format!("{}/{}", dir_path, file_name);

        let test_name = Ident::new(&format!("test{}", file_name.replace(&['/', '.', '\\', '-', ' '][..], "_")), proc_macro2::Span::call_site());

        quote! {
            #[test]
            fn #test_name() {
                // Run the function on the file contents
                let input = std::fs::read_to_string(&#file_path).unwrap();
                let result = showfile_parser(&input).finish();
                if let Err(e) = result {
                    panic!("Error: {}", convert_error(input.as_str(), e));
                };
            }
        }
    });

    // Concatenate all the tests into a single `TokenStream`
    let output = quote! {
        #(#tests)*
    };

    // Return the generated code as a `TokenStream`
    TokenStream::from(output)
}