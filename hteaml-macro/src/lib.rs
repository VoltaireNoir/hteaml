use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::Speculative, Parse},
    parse_macro_input,
    spanned::Spanned,
    Token,
};

#[proc_macro]
pub fn hteaml(stream: TokenStream) -> TokenStream {
    let html = parse_macro_input!(stream as Html);
    quote! {
        {
            #html
        }
    }
    .into()
}

enum Html {
    Tag(Tag),
    Expr(BracedExpr),
    Seq(Vec<Html>),
}

impl Parse for Html {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let element = input
            .parse()
            .map(Html::Tag)
            .or_else(|_| input.parse().map(Html::Expr))?;
        let mut seq: Vec<_> = vec![];
        if input.peek(syn::token::Paren) || input.peek(syn::token::Brace) {
            seq.push(element);
            while !input.is_empty() {
                let html = input.parse::<Html>()?;
                seq.push(html);
            }
            return Ok(Self::Seq(seq));
        }
        Ok(element)
    }
}

impl ToTokens for Html {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Html::Expr(e) => quote! {
                ::hteaml::Html::from(#e)
            },
            Html::Tag(t) => quote! {
                ::hteaml::Html::Tag(#t)
            },
            Html::Seq(s) => {
                let tag = s.iter();
                quote! {
                    ::hteaml::Html::Html(vec![
                       #(#tag.into()),*
                    ])
                }
            }
        }
        .to_tokens(tokens)
    }
}

struct Tag {
    name: Value,
    attrs: Vec<Attr>,
    cont: Content,
}

impl ToTokens for Tag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let attrs = self.attrs.iter();
        let content = &self.cont;
        let tag = quote! {
            ::hteaml::Tag::new(#name)
            #(#attrs)*
            #content
        };
        tag.to_tokens(tokens);
    }
}

impl Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let name = content.parse()?;
        let mut attrs = vec![];
        let fork = content.fork();
        while let Ok(attr) = fork.parse() {
            attrs.push(attr);
        }
        if !attrs.is_empty() {
            content.advance_to(&fork);
        }
        let cont = match content.peek(Token![=]) {
            true => {
                content.parse::<Token![=]>()?;
                content.parse()?
            }
            false => content.parse::<Content>().ok().unwrap_or(Content::None),
        };
        Ok(Self { name, attrs, cont })
    }
}

#[derive(Clone)]
struct Attr {
    key: Value,
    val: Option<Value>,
}

impl ToTokens for Attr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let key = &self.key;
        let empty_val = Value::Str(syn::LitStr::new("", tokens.span()));
        let val = self.val.as_ref().unwrap_or(&empty_val);
        quote!(.attr(#key, #val)).to_tokens(tokens);
    }
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = input.parse();
        let mut val = None;
        if input.parse::<Token![:]>().is_ok() {
            val = input.parse::<Value>().ok();
            if key.is_err() || val.is_none() {
                return Err(syn::Error::new(
                    input.span(),
                    "expected key:value pairs for attributes",
                ));
            }
        }
        Ok(Self { key: key?, val })
    }
}

#[derive(Clone)]
enum Value {
    Expr(BracedExpr),
    Ident(syn::Ident),
    Str(syn::LitStr),
}

impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input.parse::<syn::Ident>() {
            Ok(val) => Ok(Self::Ident(val)),
            Err(_) => match input.parse::<syn::LitStr>() {
                Ok(s) => Ok(Self::Str(s)),
                Err(_) => {
                    let expr = input.parse::<BracedExpr>().map_err(|_| {
                        syn::Error::new(
                            input.span(),
                            "expected either a string literal, token or a { braced Rust expression }",
                        )
                    })?;
                    Ok(Self::Expr(expr))
                }
            },
        }
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Value::Expr(e) => e.to_tokens(tokens),
            Value::Ident(id) => quote!(stringify!(#id)).to_tokens(tokens),
            Value::Str(s) => s.to_tokens(tokens),
        }
    }
}

enum Content {
    Str(syn::LitStr),
    Expr(BracedExpr),
    Html(Box<Html>),
    Seq(Vec<Content>),
    None,
}

impl Parse for Content {
    // fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    //     match input.parse::<syn::LitStr>() {
    //         Ok(s) => Ok(Self::Str(s)),
    //         Err(_) => match input.parse::<Html>() {
    //             Ok(h) => Ok(Self::Html(Box::new(h))),
    //             Err(_) => match input.parse::<BracedExpr>() {
    //                 Ok(e) => Ok(Self::Expr(e)),
    //                 Err(_) => Err(syn::Error::new(
    //                     input.span(),
    //                     "expected either a string literal or a nested tag or a { Rust expression }",
    //                 )),
    //             },
    //         },
    //     }
    // }

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content = input
            .parse()
            .map(Content::Str)
            .or_else(|_| input.parse().map(Content::Expr))
            .or_else(|_| input.parse().map(|h: Html| Content::Html(Box::new(h))))
            .map_err(|_| {
                syn::Error::new(
                    input.span(),
                    "expected a string literal, a Rust expression or atag",
                )
            })?;
        let mut seq = vec![];
        if input.peek(syn::token::Paren) || input.peek(syn::token::Brace) || input.peek(syn::LitStr)
        {
            seq.push(content);
            while !input.is_empty() {
                seq.push(input.parse()?);
            }
            return Ok(Self::Seq(seq));
        }
        Ok(content)
    }
}

impl ToTokens for Content {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Content::Str(s) => quote!(.content(#s)),
            Content::Html(h) => quote!(.content(#h)),
            Content::Expr(e) => quote!(.content(#e)),
            Content::None => quote!(.self_closing()),
            Content::Seq(s) => return s.iter().for_each(|e| e.to_tokens(tokens)),
        }
        .to_tokens(tokens);
    }
}

#[derive(Clone)]
struct BracedExpr(syn::Expr);

impl Parse for BracedExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);
        Ok(Self(content.parse()?))
    }
}

impl ToTokens for BracedExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}
