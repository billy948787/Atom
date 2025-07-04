use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;

#[proc_macro_derive(AtomVertex)]
pub fn derive_atom_vertex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named.iter().collect::<Vec<_>>(),
            syn::Fields::Unnamed(fields) => &fields.unnamed.iter().collect::<Vec<_>>(),
            syn::Fields::Unit => return proc_macro::TokenStream::new(),
        },
        _ => panic!("AtomVertex can only be derived for structs"),
    };

    let eqs = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            self.#ident == other.#ident
        }
    });

    let hashs = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            self.#ident.to_bits().hash(state);
        }
    });

    let mins = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident.min(other.#ident)
        }
    });

    let maxs = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident.max(other.#ident)
        }
    });

    let adds = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident + other.#ident
        }
    });

    let length = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            self.#ident.to_bits() as f32
        }
    });

    let subs = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident - other.#ident
        }
    });

    let muls = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident * rhs
        }
    });

    let divs = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident / rhs
        }
    });

    let expanded = quote! {
        impl std::cmp::PartialEq for #name{
            fn eq(&self, other: &Self) -> bool{
                true #(&& #eqs)*
            }
        }

        impl std::cmp::Eq for #name{

        }

        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #(#hashs)*
            }
        }

        impl<T> std::ops::Mul<T> for #name
        where
            T: Into<f32> + Copy,
        {
            type Output = Self;

            fn mul(self, rhs: T) -> Self {
                let rhs: f32 = rhs.into();
                Self {
                    #(#muls),*
                }
            }
        }

        impl<T> std::ops::Div<T> for #name
        where
            T: Into<f32> + Copy,
        {
            type Output = Self;

            fn div(self, rhs: T) -> Self {
                let rhs: f32 = rhs.into();
                Self {
                    #(#divs),*
                }
            }
        }

        impl std::ops::Add for #name {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                Self {
                    #(#adds),*
                }
            }
        }

        impl std::ops::Sub for #name {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                Self {
                    #(#subs),*
                }
            }
        }

        impl #name {
            pub fn min(self, other: Self) -> Self {
                Self {
                    #(#mins),*
                }
            }

            pub fn max(self, other: Self) -> Self {
                Self {
                    #(#maxs),*
                }
            }

            pub fn length(&self) -> f32{
                let mut len = 0.0;
                #(
                    len += #length;
                )*
                len
            }
        }
    };

    TokenStream::from(expanded)
}
