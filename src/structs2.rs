use quote::format_ident;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::{Pair, Punctuated},
    token::{Brace, Colon, Comma, Eq, Struct},
    Attribute, Expr, Field, FieldValue, FnArg, Generics, Ident, ImplItem, ItemImpl, ItemStruct,
    Result, Token, Type, Visibility,
};

// NewItemStruct -> ItemStruct
#[derive(Debug)]
pub struct NewItemStruct {
    pub attrs:        Vec<Attribute>,
    pub vis:          Visibility,
    pub struct_token: Struct,
    pub ident:        Ident,
    pub generics:     Generics,
    // pub fields:       Fields,
    pub fields:       NewFields,
    // pub semi_token:   Option<Semi>,
}

// NewFields -> FieldsNamed -> Fields
#[derive(Debug)]
pub struct NewFields {
    pub paren_token: Brace,
    pub named:       Punctuated<NewLocal, Comma>,
}

// NewLocal -> Field
#[derive(Debug)]
pub struct NewLocal {
    pub attrs:       Vec<Attribute>,
    pub vis:         Visibility,
    pub ident:       Ident,
    pub colon_token: Colon,
    pub ty:          Type,
    pub init:        Option<(Eq, Box<Expr>)>,
}

impl Parse for NewFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self { paren_token: braced!(content in input),
                  named:       content.parse_terminated(NewLocal::parse)?, })
    }
}

impl Parse for NewLocal {
    fn parse(input: ParseStream) -> Result<Self> {
        fn parse_init(input: ParseStream) -> Option<(Eq, Box<Expr>)> {
            match (input.parse(), input.parse()) {
                (Ok(eq), Ok(expr)) => Some((eq, Box::new(expr))),
                _ => None,
            }
        }
        Ok(Self { attrs:       input.call(Attribute::parse_outer)?,
                  vis:         input.parse()?,
                  ident:       input.parse()?,
                  colon_token: input.parse()?,
                  ty:          input.parse()?,
                  init:        parse_init(input), })
    }
}

impl Parse for NewItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        fn parse_generics(input: ParseStream) -> Result<Generics> {
            let mut generics: Generics = input.parse()?;
            if input.peek(Token![where]) {
                generics.where_clause = input.parse()?;
            }
            Ok(generics)
        }
        Ok(Self { attrs:        input.call(Attribute::parse_outer)?,
                  vis:          input.parse()?,
                  struct_token: input.parse()?,
                  ident:        input.parse()?,
                  generics:     parse_generics(input)?,
                  fields:       input.parse()?, })
    }
}

impl NewLocal {
    fn split(self) -> ((Option<FnArg>, FieldValue), Pair<Field, Comma>) {
        let (ident, ty, expr) = (self.ident, self.ty, self.init.map(|(_, e)| *e));
        let fnarg = if expr.is_none() { Some(parse_quote!( #ident : #ty )) } else { None };
        let expr = expr.unwrap_or_else(|| parse_quote!( #ident ));
        let field_value = parse_quote!( #ident : #expr );
        let field = Field { attrs: self.attrs,
                            vis: self.vis,
                            ident: Some(ident),
                            colon_token: Some(self.colon_token),
                            ty };
        let pair = Pair::Punctuated(field, Comma::default());
        ((fnarg, field_value), pair)
    }
}

impl NewItemStruct {
    /// 把 NewItemStruct 拆分成一个结构体和 new 方法的实现
    pub fn split(self) -> (ItemStruct, ItemImpl) {
        let ((fnargs, field_values), named): ((Vec<_>, Vec<_>), Punctuated<Field, Comma>) =
            self.fields.named.into_iter().map(NewLocal::split).unzip();
        // 构造结构体
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let (attrs, vis, struct_ident) = (self.attrs, self.vis, self.ident);
        let item_struct = parse_quote! {
            #(#attrs)*
            #vis struct #struct_ident #impl_generics #where_clause {
                #named
            }
        };
        let return_type = format_ident!("Self");
        // let mut inputs: Punctuated<FnArg, Comma> = Punctuated::new();
        // inputs.extend(fnargs.into_iter().flatten());
        let fnargs = fnargs.into_iter().flatten();
        // let mut fields: Punctuated<FieldValue, Comma> = Punctuated::new();
        // fields.extend(field_values.into_iter().map(|f| Pair::Punctuated(f, Comma::default())));
        // let fields: Punctuated<FieldValue, Comma> = parse_quote!(#(#field_values),*);
        let impl_item_ident = format_ident!("new");
        let impl_item: ImplItem = parse_quote! {
            #vis fn #impl_item_ident (#(#fnargs),*) -> #return_type {
                #return_type { #(#field_values),* }
            }
        };
        let item_impl = parse_quote! {
            impl #impl_generics #struct_ident #ty_generics #where_clause {
                #impl_item
            }
        };
        (item_struct, item_impl)
    }

    /// 解析以 `;` 分隔的多项
    pub fn parse_multi(input: ParseStream) -> Result<Punctuated<Self, Token![;]>> {
        Punctuated::parse_terminated(input)
    }
}
