use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, TypePath};

#[proc_macro]
pub fn assert_sync_proc(t: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(t as TypePath);
    quote! {{struct _AssertSync where #ty: Sync;}}.into()
}

// 与 `assert_sync` 等价
#[proc_macro]
pub fn assert_sync_proc_spanned(t: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(t as TypePath);
    let assert_sync = quote_spanned! {ty.span()=>
        {struct _AssertSync where #ty: Sync;}
    };
    // dbg!(&ty);
    println!("{}", assert_sync);
    assert_sync.into()
}

#[proc_macro]
pub fn test(_: TokenStream) -> TokenStream {
    let range = 1..4;
    let q = quote!( 0 #(+ #range)*); // q = TokenStream [ ... ]
    println!("{}", q); // 0 + 1i32 + 2i32 + 3i32
    q.into()
}

#[proc_macro]
pub fn struct_new(input: TokenStream) -> TokenStream {
    let new_struct: structs::NewItemStruct = parse_macro_input!(input);
    let (item_struct, item_impl) = new_struct.split();
    // dbg!(&normal);
    TokenStream::from(quote! {#item_struct #item_impl})
}

/// struct D<T> where T: Copy {
///     foo: u8,
///     pub bar: T,
///     abc: u8 = 255,
/// }
/// 暂时忽略字段前的属性
mod structs {
    use quote::format_ident;
    use syn::{
        braced,
        parse::{Parse, ParseStream},
        punctuated::{Pair, Punctuated},
        token::{Brace, Colon, Comma, Eq, Impl, Paren, Struct},
        AngleBracketedGenericArguments, Attribute, Block, Expr, ExprPath, ExprStruct, Field,
        FieldValue, Fields, FieldsNamed, FnArg, GenericArgument, GenericParam, Generics, Ident,
        ImplItem, ImplItemMethod, ItemImpl, ItemStruct, Member, Pat, PatIdent, PatType, Path,
        PathArguments, PathSegment, Result, ReturnType, Stmt, Token, Type, TypePath, Visibility,
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
            Ok(Self { attrs:       input.call(Attribute::parse_outer)?,
                      vis:         input.parse()?,
                      ident:       input.parse()?,
                      colon_token: input.parse()?,
                      ty:          input.parse()?,
                      init:        {
                          match (input.parse(), input.parse()) {
                              (Ok(eq), Ok(expr)) => Some((eq, Box::new(expr))),
                              _ => None,
                          }
                      }, })
        }
    }

    impl Parse for NewItemStruct {
        fn parse(input: ParseStream) -> Result<Self> {
            Ok(Self { attrs:        input.call(Attribute::parse_outer)?,
                      vis:          input.parse()?,
                      struct_token: input.parse()?,
                      ident:        input.parse()?,
                      generics:     dbg!(input.parse()?), // TODO: 识别 where 语句
                      fields:       input.parse()?, })
        }
    }

    impl NewLocal {
        fn split_punct(self) -> (NewMethod, Pair<Field, Comma>) {
            fn default_expr(ident: Ident) -> Expr {
                Expr::Path(ExprPath { attrs: Vec::new(),
                                      qself: None,
                                      path:  ident_into_path(ident), })
            }

            let ident = self.ident;
            let expr = self.init.map(|(_, e)| *e);
            let colon_token = expr.as_ref().map(|_| Colon::default());
            let arg = expr.is_none();
            let expr = expr.unwrap_or_else(|| default_expr(ident.clone()));
            let field_value = FieldValue { attrs: Vec::new(),
                                           member: Member::Named(ident.clone()),
                                           colon_token,
                                           expr };
            let arg_ident = PatIdent { attrs:      Vec::new(),
                                       by_ref:     None,
                                       mutability: None,
                                       ident:      ident.clone(),
                                       subpat:     None, };
            let method = NewMethod { arg_ident,
                                     arg_ty: self.ty.clone(),
                                     arg,
                                     field_value };
            let field = Field { attrs:       self.attrs,
                                vis:         self.vis,
                                ident:       Some(ident),
                                colon_token: Some(self.colon_token),
                                ty:          self.ty, };
            let pair = Pair::Punctuated(field, Comma::default());
            (method, pair)
        }
    }

    /// new 方法所需的部分
    struct NewMethod {
        arg_ident:   PatIdent,
        arg_ty:      Type,
        arg:         bool,
        field_value: FieldValue,
    }

    // Ident T -> Path T
    fn ident_into_path(ident: Ident) -> Path {
        let mut segments = Punctuated::new();
        segments.push(PathSegment { ident,
                                    arguments: syn::PathArguments::None });
        Path { leading_colon: None,
               segments }
    }

    // Ident A<T> -> Path A<T>
    fn ident_into_path_with_generics(ident: Ident, generics: Generics) -> Path {
        fn extract_generic_ident(generics: Generics) -> Punctuated<GenericArgument, Comma> {
            generics.params
                    .into_iter()
                    .map(|t| match t {
                        GenericParam::Type(type_param) => type_param.ident,
                        _ => unreachable!(),
                    })
                    .map(|ident| {
                        // ident > PathSegment > Path > TypePath > Type > GenericArgument
                        GenericArgument::Type(TypePath { qself: None,
                                                         path:  ident.into(), }.into())
                    })
                    .map(|g| Pair::Punctuated(g, Comma::default()))
                    .collect()
        }
        let args = extract_generic_ident(generics);
        let generics = AngleBracketedGenericArguments { colon2_token: None,
                                                        lt_token: <Token![<]>::default(),
                                                        args,
                                                        gt_token: <Token![>]>::default() };
        let mut segments = Punctuated::new();
        segments.push(PathSegment { ident,
                                    arguments: PathArguments::AngleBracketed(generics) });
        Path { leading_colon: None,
               segments }
    }

    impl NewItemStruct {
        /// 把 NewItemStruct 拆分成一个结构体和 new 方法的实现
        pub fn split(self) -> (ItemStruct, ItemImpl) {
            let (method, named): (Vec<_>, Punctuated<Field, Comma>) =
                self.fields.named.into_iter().map(NewLocal::split_punct).unzip();
            // 构造结构体
            let fields = Fields::Named(FieldsNamed { brace_token: self.fields.paren_token,
                                                     named });
            let item_struct = ItemStruct { attrs: self.attrs,
                                           vis: self.vis.clone(),
                                           struct_token: Struct::default(),
                                           ident: self.ident.clone(),
                                           generics: self.generics.clone(),
                                           fields,
                                           semi_token: None };
            // 构造结构体的 new 实现：注意带上泛型参数
            let struct_path = ident_into_path(self.ident.clone());
            let self_ty = TypePath { qself: None,
                                     path:  ident_into_path_with_generics(self.ident.clone(),
                                                                          self.generics.clone()), };
            let struct_type = Type::Path(TypePath { qself: None,
                                                    path:  format_ident!("Self").into(), }); //  PathArguments > PathSegment > Path > TypePath > Type
                                                                                             // TODO: 改进
            let (fnargs, field_values): (Vec<_>, Vec<_>) =
                method.into_iter()
                      .map(|NewMethod { arg_ident,
                                        arg_ty,
                                        arg,
                                        field_value, }| {
                               (if arg {
                                    Some(FnArg::Typed(PatType { attrs:       Vec::new(),
                                                                pat:
                                                                    Box::new(Pat::Ident(arg_ident)),
                                                                colon_token: <Token![:]>::default(),
                                                                ty:          Box::new(arg_ty), }))
                                } else {
                                    None
                                },
                                field_value)
                           })
                      .unzip();
            let mut inputs: Punctuated<FnArg, Comma> = Punctuated::new();
            inputs.extend(fnargs.into_iter().flatten());
            let sig = syn::Signature { constness: None,
                                       asyncness: None,
                                       unsafety: None,
                                       abi: None,
                                       fn_token: <Token![fn]>::default(),
                                       ident: format_ident!("new"),
                                       generics: self.generics.clone(),
                                       paren_token: Paren::default(),
                                       inputs,
                                       variadic: None,
                                       output: ReturnType::Type(<Token![->]>::default(),
                                                                Box::new(struct_type)) };
            let mut fields = Punctuated::new();
            fields.extend(field_values.into_iter().map(|f| Pair::Punctuated(f, Comma::default())));
            let stmt = Stmt::Expr(ExprStruct { attrs: Vec::new(),
                                               path: struct_path,
                                               brace_token: Brace::default(),
                                               fields,
                                               dot2_token: None,
                                               rest: None }.into());
            let block = Block { brace_token: Brace::default(),
                                stmts:       vec![stmt], };
            let impl_item = ImplItem::Method(ImplItemMethod { attrs: Vec::new(),
                                                              vis: self.vis,
                                                              defaultness: None,
                                                              sig,
                                                              block }); // Signature + Block > ImplItemMethod > ImplItem
            let item_impl = ItemImpl { attrs:       Vec::new(),
                                       defaultness: None,
                                       unsafety:    None,
                                       impl_token:  Impl::default(),
                                       generics:    self.generics,
                                       trait_:      None,
                                       self_ty:     Box::new(self_ty.into()),
                                       brace_token: Brace::default(),
                                       items:       vec![impl_item], };
            (dbg!(item_struct), dbg!(item_impl))
        }
    }
}
