use std::str::FromStr;

use proc_macro2::TokenStream;
use syn::{
    braced, custom_keyword,
    parse::{Parse, ParseStream},
    parse2, Ident, Result, Token,
};

pub type RtlResult<T> = Result<T>;

pub fn parse(ts: &str) -> Result<Rattle> {
    let ts = TokenStream::from_str(ts)?;
    let rattle: Rattle = parse2(ts)?;
    Ok(rattle)
}

custom_keyword!(f);
custom_keyword!(import);
custom_keyword!(gen);
custom_keyword!(def);
custom_keyword!(var);

// The struct for a Rattle program
#[derive(Debug)]
pub struct Rattle {
    // Rattle top-level Declarations (variables, constants, functions, structs, defs)
    decls: Vec<RtlDecl>,
    // Rattle imports
    imports: Vec<RtlImport>,
    // Rattle exported public functions/structs/types/constants
    public: Vec<RtlPub>,
}

// The struct for Rattle declarations
#[derive(Debug)]
pub struct RtlDecl {
    value: RtlDeclValue,
}

// Enum for different types of Rattle declarations
#[derive(Debug)]
pub enum RtlDeclValue {
    // Rattle functions
    RtlFn(RtlFn),
    // Rattle constants
    RtlConst(RtlConstExpr),
    // Rattle variables
    RtlVar(RtlVarExpr),
    // Rattle statics
    RtlStatic(RtlStatic),
    // Rattle structs
    RtlStruct(RtlStruct),
    // Rattle definitions
    RtlDef(RtlDef),
    // Rattle generics
    RtlGen(RtlGen),
}

// The struct for a Rattle function
#[derive(Debug)]
pub struct RtlFn {
    name: Ident,
    args: Vec<RtlFnArg>,
    ret: Ident,
    body: RtlBody,
}

// The struct for a Rattle function argument
#[derive(Debug)]
pub struct RtlFnArg {
    ty: Ident,
    name: Ident,
}

// The struct for a Rattle constant expression
#[derive(Debug)]
pub struct RtlConstExpr {
    name: Ident,
    ty: Ident,
    data: RtlExpr,
}

// The struct for a Rattle variable expression
#[derive(Debug)]
pub struct RtlVarExpr {
    name: Ident,
    ty: Ident,
    is_mut: bool,
    data: RtlExpr,
}

// The struct for a Rattle static variable
#[derive(Debug)]
pub struct RtlStatic {
    name: Ident,
    ty: Ident,
    is_mut: bool,
    data: RtlExpr,
}

// The struct for a Rattle struct
#[derive(Debug)]
pub struct RtlStruct {
    name: Ident,
    fields: Vec<RtlStructField>,
}

// The struct for a field in a Rattle struct
#[derive(Debug)]
pub struct RtlStructField {
    ty: Ident,
    name: Ident,
}

// The struct for a Rattle definition
#[derive(Debug)]
pub struct RtlDef {
    struct_name: Ident,
    defs: Vec<RtlFn>,
    def_for: Option<Ident>,
}

// The struct for Rattle generics
#[derive(Debug)]
pub struct RtlGen {
    methods: Vec<RtlFn>,
}

// Dummy structs to make the code compile
#[derive(Debug)]
pub struct RtlExpr;
#[derive(Debug)]
pub struct RtlBody;
#[derive(Debug)]
pub struct RtlImport {
    path: Vec<Ident>,
    alias: Option<Ident>,
}
#[derive(Debug)]
pub struct RtlPub;

impl Parse for Rattle {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut imports = Vec::new(); // Implement parsing for imports if necessary
        while input.peek(import) {
            imports.push(input.parse::<RtlImport>()?);
        }

        let mut decls = Vec::new();
        while !input.is_empty() {
            let forked = input.fork();
            if forked.parse::<Token![;]>().is_ok() {
                input.parse::<Token![;]>()?;
            }
            decls.push(input.parse()?);
        }

        let public = Vec::new(); // Implement parsing for public if necessary

        Ok(Rattle {
            decls,
            imports,
            public,
        })
    }
}

impl Parse for RtlDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(f) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlFn(input.parse()?),
            })
        } else if lookahead.peek(Token![const]) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlConst(input.parse()?),
            })
        } else if lookahead.peek(var) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlVar(input.parse()?),
            })
        } else if lookahead.peek(Token![static]) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlStatic(input.parse()?),
            })
        } else if lookahead.peek(Token![struct]) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlStruct(input.parse()?),
            })
        } else if lookahead.peek(def) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlDef(input.parse()?),
            })
        } else if lookahead.peek(gen) {
            Ok(RtlDecl {
                value: RtlDeclValue::RtlGen(input.parse()?),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for RtlFn {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<f>()?;
        let name: Ident = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let mut args = Vec::new();
        while !content.is_empty() {
            args.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        let ret: Ident = input.parse()?;
        let body: RtlBody = RtlBody;
        let forked = input.fork();
        if forked.parse::<Token![;]>().is_ok() {
            input.parse::<Token![;]>()?;
        } else {
            let body: RtlBody = input.parse()?;
        }

        Ok(RtlFn {
            name,
            args,
            ret,
            body,
        })
    }
}

impl Parse for RtlFnArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Ident = input.parse()?;
        let name: Ident = input.parse()?;
        Ok(RtlFnArg { ty, name })
    }
}

impl Parse for RtlConstExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![const]>()?;
        let ty: Ident = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let data: RtlExpr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(RtlConstExpr { name, ty, data })
    }
}

impl Parse for RtlVarExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<var>()?;
        let ty: Ident = input.parse()?;
        let is_mut = input.peek(Token![mut]);
        if is_mut {
            input.parse::<Token![mut]>()?;
        }
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let data: RtlExpr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(RtlVarExpr {
            name,
            ty,
            is_mut,
            data,
        })
    }
}

impl Parse for RtlStatic {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![static]>()?;
        let ty: Ident = input.parse()?;
        let is_mut = input.peek(Token![mut]);
        if is_mut {
            input.parse::<Token![mut]>()?;
        }
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let data: RtlExpr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(RtlStatic {
            name,
            ty,
            is_mut,
            data,
        })
    }
}

impl Parse for RtlStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);
        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(RtlStruct { name, fields })
    }
}

impl Parse for RtlStructField {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Ident = input.parse()?;
        let name: Ident = input.parse()?;
        Ok(RtlStructField { ty, name })
    }
}

impl Parse for RtlDef {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<def>()?;
        let struct_name: Ident = input.parse()?;
        let mut defs = Vec::new();
        let content;
        braced!(content in input);
        while !content.is_empty() {
            defs.push(content.parse()?);
        }
        let def_for = if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }
        Ok(RtlDef {
            struct_name,
            defs,
            def_for,
        })
    }
}

impl Parse for RtlGen {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<gen>()?;
        let mut methods = Vec::new();
        while !input.is_empty() {
            methods.push(input.parse()?);
        }
        Ok(RtlGen { methods })
    }
}

// Dummy implementations for RtlExpr, RtlBody, RtlImport, RtlPub to make the code compile
impl Parse for RtlExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RtlExpr)
    }
}

impl Parse for RtlBody {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RtlBody)
    }
}

impl Parse for RtlImport {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<import>()?;
        let mut parts: Vec<Ident> = vec![];
        let mut alias: Option<Ident> = None;
        while !input.is_empty() {
            let forked = input.fork();
            if forked.parse::<Token![as]>().is_ok() {
                input.parse::<Token![as]>()?;
                alias = Some(input.parse()?);
            } else if forked.parse::<Token![;]>().is_ok() {
                input.parse::<Token![;]>()?;
                break;
            } else if forked.parse::<Token![::]>().is_ok() {
                input.parse::<Token![::]>()?;
                continue;
            } else {
                parts.push(input.parse()?);
            }
        }

        Ok(RtlImport { path: parts, alias })
    }
}

impl Parse for RtlPub {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![pub]>()?;
        Ok(RtlPub)
    }
}
