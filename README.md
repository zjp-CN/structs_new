# structs_new

使用函数式过程宏，给结构体生成 `new` 构造函数。完整例子见 
[`struct_new2-multi`](https://github.com/zjp-CN/structs_new/blob/main/tests/ui/struct_new2-multi.rs)。

你可以通过 `cargo run --bin struct_new2-multi` 命令运行。

注意：这是我学习编写 Rust 过程宏的小案例 ——
我可能会围绕它写一点教程（目前暂且先把零碎的东西记录在此 README 中）。

## 学习过程宏的经验

过程宏是 Rust 中进阶的内容，它是声明宏的拓展。但它的学习资料比较少，我整理了一些：你可以在
[此链接](https://www.yuque.com/zhoujiping/programming/rust-materials) 网页中搜索【过程宏】找到。

如果你想编写过程宏，那么以下内容是必须掌握的：
1. Rust 几乎所有的语法：因为 Rust 的宏就是在操作 AST，所以掌握源代码的语法结构是第一步。当你能自如地查阅 
    [Reference](https://doc.rust-lang.org/nightly/reference) 一书，那么这一步就成功了。
2. Rust 的声明宏：声明宏可以单独学习，它完全不涉及过程宏；但过程宏涉及声明宏，而且它们之间有许多相似的地方。
   你可以不需要掌握声明宏高阶模式部分，但你至少要掌握声明宏最通用的部分。
   当你对 [The Little Book of Rust Macros](https://zjp-cn.github.io/tlborm/)
   一书中的大部分内容熟悉时，你可以准备迎接过程宏了。


对我来说，学习过程宏的过程：
1. 从 [Rust Book: ch19-06-macros](https://doc.rust-lang.org/book/ch19-06-macros.html) 中知道过程宏的分类。
2. 从 [Reference: procedural-macros](https://doc.rust-lang.org/nightly/reference/procedural-macros.html)
    中知道过程宏真正的编写框架。
3. 从 [syn/examples](https://github.com/dtolnay/syn/tree/master/examples) 
    中学习如何在特定任务下真实地编写过程宏。你对这四个例子理解地越仔细，那么你就能越快地上手过程宏。
4. 最重要的资料是文档：[quote](https://docs.rs/quote/latest/quote/) 和 [syn](https://docs.rs/syn/latest/syn/)
    。过程宏不像声明宏那样开箱即用，你需要引入别的库，所以你需要掌握这两个库。


## 例子由来

此仓库的例子来源于[这个帖子](https://rustcc.cn/article?id=5dbddd4b-4a25-48bd-a78d-8e8d0a952346)提出的问题：
> 如何通过声明宏自动生成 `new` 方法

你可以通过完整的
[样例代码](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4e92b67453f00ba53e3082cdf47000f7)
感受其实际用途。

核心设计：
1. 通过自定义语法，把结构体的字段分成两种：带默认值和不带默认值
2. `new` 方法的参数接收不带默认值的字段的值
3. 结构体及其字段需要保留其属性和可见性

这个问题并不算复杂，除了这个现成的、不太完美的、使用声明宏的方案（它不完美在很难解析 
`where` 语句），此外，存在一个功能齐全的 derive 过程宏方案 —— [derive-new](https://crates.io/crates/derive-new)。

> 但这里的重点并不在于解决原问题，而在于面对同一个需求时，函数式过程宏与声明宏之间的区别。

## 例子的价值

现在把关注点拉回到这个仓库里的代码。

### structs.rs

[structs.rs](https://github.com/zjp-CN/structs_new/blob/main/src/structs.rs)
展示了利用 `syn` 中的类型来纯手写构建 AST 的过程。这个构建过程的意义：
1. 让编写者清楚地知道每一处类型对应于源代码的哪种语法，从而领略到，声明宏的 
    [13 种片段分类符](https://zjp-cn.github.io/tlborm/macros/minutiae/fragment-specifiers.html)
    远远不足以完整描述和精确控制语法解析和生成。
2. 表明了过程宏以完全结构化的方式组织起语法，而不是像声明宏那样“所见即所得”。

从而轻松解决原贴里的难以解析 `where` 语句的问题。

当然，结构化组织的代价是，源代码中的一个事物被分到多个类型上，比如我必须写这样的转化函数：

```rust
// Ident T -> Path T
// 等价于
// ```rust
// fn ident_into_path(ident: Ident) -> Path {
//     let mut segments = Punctuated::new();
//     segments.push(PathSegment { ident,
//                                 arguments: syn::PathArguments::None });
//     Path { leading_colon: None,
//            segments }
// }
// ```
fn ident_into_path(ident: Ident) -> Path { ident.into() } // 这里多亏了泛型

// Ident A<T> -> Path A<T>
fn ident_into_path_with_generics(ident: Ident, generics: Generics) -> Path {
    fn generic_into_argument(generics: Generics) -> Punctuated<GenericArgument, Comma> {
        generics.params
                .into_iter()
                .map(|g| match g {
                    GenericParam::Type(type_param) => {
                        // ident > PathSegment > Path > TypePath > Type > GenericArgument
                        GenericArgument::Type(TypePath { qself: None,
                                                         path:  type_param.ident.into(), }.into())
                    }
                    GenericParam::Lifetime(lifetime) => {
                        GenericArgument::Lifetime(lifetime.lifetime)
                    }
                    GenericParam::Const(_) => unreachable!(),
                })
                .map(|g| Pair::Punctuated(g, Comma::default()))
                .collect()
    }

    let args = generic_into_argument(generics);
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
```

来把 `Ident` 放入不同语法情境中。

### structs2.rs

手动构建 AST 太繁琐了，但 `syn` 提供了方便的 
[`parse_quote!`](https://docs.rs/syn/latest/syn/macro.parse_quote.html)
声明宏，利用插值解决不同语境的类型转化问题。

在 [structs2.rs](https://github.com/zjp-CN/structs_new/blob/main/src/structs2.rs)
的例子中，你只需要编写如下代码 —— 它看起来很像 Rust 源代码（也像声明宏的展开部分[^macro-rules-like]）
—— 就能避免大多数手动构建 AST 类型[^avoid]。

```rust
let fnarg = if expr.is_none() { Some(parse_quote!( #ident : #ty )) } else { None };
let expr = expr.unwrap_or_else(|| parse_quote!( #ident ));
let field_value = parse_quote!( #ident : #expr );

let item_struct = parse_quote! {
    #(#attrs)*
    #vis struct #struct_ident #impl_generics #where_clause {
        #named
    }
};
let return_type = format_ident!("Self");
let fnargs = fnargs.into_iter().flatten();
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
```

从代码数量上看，利用 `parse_quote!` 比手写的方式减少了一半代码（仅限此例）。

[^macro-rules-like]: 这不是偶然的，因为 `parse_quote!` 背后是 `quote::quote!` 
声明宏的[功劳](https://docs.rs/quote/latest/src/quote/lib.rs.html)。

[^avoid]: 当然，手动构建 AST 类型依然是基本功。

---

此外，还有一点值得注意，`structs_new2!` 中的 `parse_multi` 方法让解析同类型的变长参数得以轻松实现。

其实质是利用 `Punctuated<Self, Token![;]>` 类型进行解析输入。

## 例子说明

在声明宏的解决方案中，你可以实现
[`struct_new2-multi`](https://github.com/zjp-CN/structs_new/blob/main/tests/ui/struct_new2-multi.rs)。
中的 A、B、C、D 的功能，但面对更复杂的泛型结构体 E、F、G，声明宏就很难做到。

原因在于，声明宏的分类符过于笼统，比如面对 `Box<dyn Fn(T, T2) -> (R, R2)>`，当你需要把它作为类型进行解析时，
`syn` 可以非常详细地控制解析的精确度。你可以看看它在 `syn` 中作为 `Type` 的[呈现](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8349f2624bbfef68958e1a1895fe30eb)。

而声明宏对此除了用拆解的方式解析，还可以用方便的 `$ty` 分类符匹配到它，然而一旦编译器不接受 `$tt`
之外的分类符时，[事情会变得很棘手](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=bddd4fe5776be6ffeb1fc6abae68a1ff)。

`syn` 的 [`Type`](https://docs.rs/syn/latest/syn/enum.Type.html)
类型让你无需特别当心语法多变的泛型：比如 `I: Iterator`、`I::Item: Copy`、`'c: 'a+'b`。
当你需要这些泛型的一部分时，也可以提取出来。尤其是 `.split_for_impl()` 技巧：

```rust
let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

let item_struct = parse_quote! {
    #(#attrs)*
    #vis struct #struct_ident #impl_generics #where_clause {
        #named
    }
};

let item_impl = parse_quote! {
    impl #impl_generics #struct_ident #ty_generics #where_clause {
        #impl_item
    }
};
```

