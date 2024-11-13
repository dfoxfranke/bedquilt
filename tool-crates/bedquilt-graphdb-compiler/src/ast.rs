use proc_macro2::TokenStream;
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

use quote::{ToTokens, TokenStreamExt};

pub mod kw {
    syn::custom_keyword!(class);
    syn::custom_keyword!(entity);
    syn::custom_keyword!(exists);
    syn::custom_keyword!(relation);
    syn::custom_keyword!(table);
    syn::custom_keyword!(unique);
}

#[derive(Debug, Clone)]
pub struct BracedBody<T> {
    pub brace_token: Option<syn::token::Brace>,
    pub body: Punctuated<T, syn::token::Comma>,
    pub semi_token: Option<syn::token::Semi>,
}

impl<T> Parse for BracedBody<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            let braced;
            let brace_token = braced!(braced in input);
            let body = braced.parse_terminated(T::parse, Token![,])?;
            let semi_token = input.parse()?;
            Ok(BracedBody {
                brace_token: Some(brace_token),
                body,
                semi_token,
            })
        } else if lookahead.peek(Token![;]) {
            Ok(BracedBody {
                brace_token: None,
                body: Punctuated::new(),
                semi_token: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<T> ToTokens for BracedBody<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(brace_token) = self.brace_token {
            brace_token.surround(tokens, |tokens| self.body.to_tokens(tokens));
        }
        if let Some(semi_token) = self.semi_token {
            semi_token.to_tokens(tokens);
        }
    }
}

#[derive(Debug, Clone)]
pub enum SimplePath {
    Top(Token![*]),
    Path {
        leading_colon: Option<Token![::]>,
        segments: Punctuated<syn::Ident, Token![::]>,
    },
}

impl Parse for SimplePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse_segment(input: ParseStream) -> syn::Result<syn::Ident> {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Ident)
                || lookahead.peek(Token![super])
                || lookahead.peek(Token![self])
                || lookahead.peek(Token![crate])
                || input.peek(Token![try])
            {
                input.call(syn::Ident::parse_any)
            } else {
                Err(lookahead.error())
            }
        }

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![*]) {
            input.parse().map(SimplePath::Top)
        } else if lookahead.peek(Token![::])
            || lookahead.peek(syn::Ident)
            || lookahead.peek(Token![super])
            || lookahead.peek(Token![self])
            || lookahead.peek(Token![crate])
            || input.peek(Token![try])
        {
            Ok(SimplePath::Path {
                leading_colon: input.parse()?,
                segments: Punctuated::parse_separated_nonempty_with(input, parse_segment)?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for SimplePath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SimplePath::Top(star) => star.to_tokens(tokens),
            SimplePath::Path {
                leading_colon,
                segments,
            } => {
                if let Some(leading_colon) = leading_colon {
                    leading_colon.to_tokens(tokens);
                }
                segments.to_tokens(tokens);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub shebang: Option<String>,
    pub attrs: Vec<syn::Attribute>,
    pub items: Vec<Item>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_inner)?;
        let mut items = Vec::new();
        while !input.peek(syn::parse::End) {
            items.push(input.parse()?);
        }
        Ok(File {
            shebang: None,
            attrs,
            items,
        })
    }
}

impl ToTokens for File {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        tokens.append_all(&self.items);
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Use(syn::ItemUse),
    Super(ItemSuper),
    Class(ItemClass),
    Relation(ItemRelation),
    Entity(ItemEntity),
    Table(ItemTable),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let after_attrs = input.fork();
        after_attrs.call(syn::Attribute::parse_outer)?;
        let lookahead = after_attrs.lookahead1();

        if lookahead.peek(Token![use]) {
            input.parse().map(Item::Use)
        } else if lookahead.peek(Token![super]) {
            input.parse().map(Item::Super)
        } else if lookahead.peek(kw::class) {
            input.parse().map(Item::Class)
        } else if lookahead.peek(kw::relation) {
            input.parse().map(Item::Relation)
        } else if lookahead.peek(kw::entity) {
            input.parse().map(Item::Entity)
        } else if lookahead.peek(kw::table) {
            input.parse().map(Item::Table)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Item::Use(item_use) => item_use.to_tokens(tokens),
            Item::Super(item_super) => item_super.to_tokens(tokens),
            Item::Class(item_class) => item_class.to_tokens(tokens),
            Item::Relation(item_relation) => item_relation.to_tokens(tokens),
            Item::Entity(item_entity) => item_entity.to_tokens(tokens),
            Item::Table(item_table) => item_table.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemSuper {
    pub attrs: Vec<syn::Attribute>,
    pub super_token: Token![super],
    pub trees: Punctuated<SuperTreeRoot, Token![+]>,
    pub semi_token: Token![;],
}

impl Parse for ItemSuper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ItemSuper {
            attrs: input.call(syn::Attribute::parse_outer)?,
            super_token: input.parse()?,
            trees: input.call(Punctuated::parse_separated_nonempty)?,
            semi_token: input.parse()?,
        })
    }
}

impl ToTokens for ItemSuper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.super_token.to_tokens(tokens);
        self.trees.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct SuperTreeRoot {
    pub leading_colon: Option<Token![::]>,
    pub tree: SuperTree,
}

impl Parse for SuperTreeRoot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![::]) {
            Ok(SuperTreeRoot {
                leading_colon: Some(input.parse()?),
                tree: input.parse()?,
            })
        } else {
            Ok(SuperTreeRoot {
                leading_colon: None,
                tree: input.parse()?,
            })
        }
    }
}

impl ToTokens for SuperTreeRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.leading_colon.to_tokens(tokens);
        self.tree.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub enum SuperTree {
    Path(SuperPath),
    Name(syn::Ident),
    Group(SuperGroup),
}

impl Parse for SuperTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            input.parse().map(SuperTree::Group)
        } else if lookahead.peek(syn::Ident) {
            if input.peek2(Token![::]) {
                input.parse().map(SuperTree::Path)
            } else {
                input.parse().map(SuperTree::Name)
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for SuperTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SuperTree::Path(super_path) => super_path.to_tokens(tokens),
            SuperTree::Name(ident) => ident.to_tokens(tokens),
            SuperTree::Group(super_group) => super_group.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SuperPath {
    pub ident: syn::Ident,
    pub colon2_token: Token![::],
    pub tree: Box<SuperTree>,
}

impl Parse for SuperPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(SuperPath {
            ident: input.parse()?,
            colon2_token: input.parse()?,
            tree: input.parse()?,
        })
    }
}

impl ToTokens for SuperPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon2_token.to_tokens(tokens);
        self.tree.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct SuperGroup {
    pub brace_token: syn::token::Brace,
    pub items: Punctuated<SuperTree, Token![+]>,
}

impl Parse for SuperGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let braced;
        let brace_token = braced!(braced in input);
        Ok(SuperGroup {
            brace_token,
            items: braced.call(Punctuated::parse_terminated)?,
        })
    }
}

impl ToTokens for SuperGroup {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token
            .surround(tokens, |tokens| self.items.to_tokens(tokens));
    }
}

#[derive(Debug, Clone)]
pub struct ItemRelation {
    pub attrs: Vec<syn::Attribute>,
    pub relation_token: kw::relation,
    pub mutability: Option<Token![mut]>,
    pub name: syn::Ident,
    pub colon_token: Token![:],
    pub domain: Domain,
    pub arrow_token: Token![->],
    pub unique: Option<kw::unique>,
    pub range: Range,
    pub semi_token: Option<Token![;]>,
}

impl Parse for ItemRelation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let relation_token = input.parse()?;
        let mutability = if input.peek(Token![mut]) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let domain = input.parse()?;
        let arrow_token = input.parse()?;
        let unique = if input.peek(kw::unique) || !input.peek2(Token![;]) {
            Some(input.parse()?)
        } else {
            None
        };
        let range = input.parse()?;

        let semi_token = match range {
            Range::Enum { .. } => Some(input.parse()?),
            _ => input.parse()?,
        };

        Ok(ItemRelation {
            attrs,
            relation_token,
            mutability,
            name,
            colon_token,
            domain,
            arrow_token,
            unique,
            range,
            semi_token,
        })
    }
}

impl ToTokens for ItemRelation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.relation_token.to_tokens(tokens);
        self.mutability.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.domain.to_tokens(tokens);
        self.arrow_token.to_tokens(tokens);
        self.unique.to_tokens(tokens);
        self.range.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub enum Domain {
    Top(Token![*]),
    Bottom(Token![!]),
    Unit(syn::token::Paren),
    Path(SimplePath),
}

impl Parse for Domain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![!]) {
            input.parse().map(Domain::Bottom)
        } else if input.peek(syn::token::Paren) {
            let e = input.error("expected `!`, `()`, `*`, or a class");
            let parened;
            let paren_token = parenthesized!(parened in input);
            if parened.is_empty() {
                Ok(Domain::Unit(paren_token))
            } else {
                Err(e)
            }
        } else if input.peek(Token![*]) {
            input.parse().map(Domain::Top)
        } else {
            let mut path_error = input.error("expected `!`, `()`, `*`, or a class; next error assumes this was meant to be a class");
            match input.parse() {
                Ok(path) => Ok(Domain::Path(path)),
                Err(e) => {
                    path_error.combine(e);
                    Err(path_error)
                }
            }
        }
    }
}

impl ToTokens for Domain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Domain::Top(star) => star.to_tokens(tokens),
            Domain::Bottom(not) => not.to_tokens(tokens),
            Domain::Unit(paren) => paren.surround(tokens, |_| {}),
            Domain::Path(simple_path) => simple_path.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Range {
    Top(Token![*]),
    Type(syn::Type),
    Enum {
        enum_token: Token![enum],
        brace_token: syn::token::Brace,
        variants: Punctuated<RelationVariant, Token![,]>,
    },
}

impl Parse for Range {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) && !input.peek2(Token![const]) && !input.peek2(Token![mut]) {
            input.parse().map(Range::Top)
        } else if input.peek(Token![enum]) {
            let braced;
            Ok(Range::Enum {
                enum_token: input.parse()?,
                brace_token: braced!(braced in input),
                variants: braced.call(Punctuated::parse_terminated)?,
            })
        } else {
            let mut type_error = input.error(
                "expected `*`, `enum`, or a type; next error assumes this was meant to be a type",
            );
            match input.parse() {
                Ok(ty) => Ok(Range::Type(ty)),
                Err(e) => {
                    type_error.combine(e);
                    Err(type_error)
                }
            }
        }
    }
}

impl ToTokens for Range {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Range::Top(star) => star.to_tokens(tokens),
            Range::Type(t) => t.to_tokens(tokens),
            Range::Enum {
                enum_token,
                brace_token,
                variants,
            } => {
                enum_token.to_tokens(tokens);
                brace_token.surround(tokens, |tokens| variants.to_tokens(tokens));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelationVariant {
    pub attrs: Vec<syn::Attribute>,
    pub variant: syn::Ident,
    pub paren_token: syn::token::Paren,
    pub range: SimpleRange,
}

impl Parse for RelationVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let variant = input.parse()?;
        let parened;
        let paren_token = parenthesized!(parened in input);
        let range = parened.parse()?;

        Ok(RelationVariant {
            attrs,
            variant,
            paren_token,
            range,
        })
    }
}

impl ToTokens for RelationVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.variant.to_tokens(tokens);
        self.paren_token
            .surround(tokens, |tokens| self.range.to_tokens(tokens));
    }
}

#[derive(Debug, Clone)]
pub enum SimpleRange {
    Top(Token![*]),
    Class(SimplePath),
}

impl Parse for SimpleRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) && !input.peek2(Token![const]) && !input.peek2(Token![mut]) {
            input.parse().map(SimpleRange::Top)
        } else if input.peek(Token![enum]) {
            Err(input.error("enum ranges cannot be nested"))
        } else {
            let mut class_error = input
                .error("expected `*` or a class; next error assumes this was meant to be a class");
            match input.parse() {
                Ok(ty) => Ok(SimpleRange::Class(ty)),
                Err(e) => {
                    class_error.combine(e);
                    Err(class_error)
                }
            }
        }
    }
}

impl ToTokens for SimpleRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SimpleRange::Top(star) => star.to_tokens(tokens),
            SimpleRange::Class(t) => t.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemClass {
    pub attrs: Vec<syn::Attribute>,
    pub class_token: kw::class,
    pub name: syn::Ident,
    pub inheritance: Option<ClassInheritance>,
    pub body: BracedBody<ClassConstraint>,
}

impl Parse for ItemClass {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let class_token = input.parse()?;
        let name = input.parse()?;

        let lookahead = input.lookahead1();
        let inheritance = if lookahead.peek(Token![:]) {
            Some(input.parse()?)
        } else if lookahead.peek(syn::token::Brace) || lookahead.peek(Token![;]) {
            None
        } else {
            return Err(lookahead.error());
        };

        let body = input.parse()?;

        Ok(ItemClass {
            attrs,
            class_token,
            name,
            inheritance,
            body,
        })
    }
}

impl ToTokens for ItemClass {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.class_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.inheritance.to_tokens(tokens);
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ClassInheritance {
    pub colon_token: Token![:],
    pub supers: syn::punctuated::Punctuated<SimplePath, Token![+]>,
}

impl Parse for ClassInheritance {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ClassInheritance {
            colon_token: input.parse()?,
            supers: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

impl ToTokens for ClassInheritance {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.colon_token.to_tokens(tokens);
        self.supers.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ClassConstraint {
    pub attrs: Vec<syn::Attribute>,
    pub exists: Option<kw::exists>,
    pub unique: Option<kw::unique>,
    pub mutability: Option<Token![mut]>,
    pub relation: SimplePath,
    pub range: Option<RangeAnnotation>,
}

impl Parse for ClassConstraint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let exists = if !input.peek2(Token![:])
            && !input.peek2(Token![,])
            && !input.peek2(syn::parse::End)
        {
            Some(input.parse()?)
        } else {
            None
        };

        let unique = if !input.peek2(Token![:])
            && !input.peek2(Token![,])
            && !input.peek2(syn::parse::End)
        {
            Some(input.parse()?)
        } else {
            None
        };

        let mutability = input.parse()?;

        let lookahead = input.lookahead1();
        let relation = if lookahead.peek(Token![::])
            || lookahead.peek(Token![crate])
            || lookahead.peek(Token![super])
            || lookahead.peek(Token![self])
            || lookahead.peek(syn::Ident)
            || input.peek(Token![try])
        {
            input.parse()?
        } else {
            return Err(lookahead.error());
        };

        let range = if input.peek(Token![:]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(ClassConstraint {
            attrs,
            exists,
            unique,
            mutability,
            relation,
            range,
        })
    }
}

impl ToTokens for ClassConstraint {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.exists.to_tokens(tokens);
        self.unique.to_tokens(tokens);
        self.mutability.to_tokens(tokens);
        self.relation.to_tokens(tokens);
        self.range.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct RangeAnnotation {
    pub colon_token: Token![:],
    pub range: Range,
}

impl Parse for RangeAnnotation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RangeAnnotation {
            colon_token: input.parse()?,
            range: input.parse()?,
        })
    }
}

impl ToTokens for RangeAnnotation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.colon_token.to_tokens(tokens);
        self.range.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntity {
    pub attrs: Vec<syn::Attribute>,
    pub entity_token: kw::entity,
    pub name: SimplePath,
    pub class: Option<ClassAnnotation>,
    pub body: BracedBody<EntityField>,
}

impl Parse for ItemEntity {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let entity_token = input.parse()?;
        let name = input.parse()?;

        let lookahead = input.lookahead1();
        let class = if lookahead.peek(Token![:]) {
            Some(input.parse()?)
        } else if lookahead.peek(syn::token::Brace) || lookahead.peek(Token![;]) {
            None
        } else {
            return Err(lookahead.error());
        };

        let body = input.parse()?;

        Ok(ItemEntity {
            attrs,
            entity_token,
            name,
            class,
            body,
        })
    }
}

impl ToTokens for ItemEntity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.entity_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.class.to_tokens(tokens);
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ClassAnnotation {
    pub colon_token: Token![:],
    pub class: SimplePath,
}

impl Parse for ClassAnnotation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ClassAnnotation {
            colon_token: input.parse()?,
            class: input.parse()?,
        })
    }
}

impl ToTokens for ClassAnnotation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.colon_token.to_tokens(tokens);
        self.class.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct EntityField {
    pub attrs: Vec<syn::Attribute>,
    pub name: SimplePath,
    pub colon_token: Token![:],
    pub value: syn::Expr,
}

impl Parse for EntityField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EntityField {
            attrs: input.call(syn::Attribute::parse_outer)?,
            name: input.parse()?,
            colon_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for EntityField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.name.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct ItemTable {
    pub attrs: Vec<syn::Attribute>,
    pub table_token: kw::table,
    pub name: SimplePath,
    pub ty: Option<TypeAnnotation>,
    pub body: BracedBody<TableEntry>,
}

impl Parse for ItemTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ItemTable {
            attrs: input.call(syn::Attribute::parse_outer)?,
            table_token: input.parse()?,
            name: input.parse()?,
            ty: {
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![:]) {
                    Some(input.parse()?)
                } else if lookahead.peek(syn::token::Brace) || lookahead.peek(Token![;]) {
                    None
                } else {
                    return Err(lookahead.error());
                }
            },
            body: input.parse()?,
        })
    }
}

impl ToTokens for ItemTable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.table_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.body.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub colon_token: Token![:],
    pub ty: syn::Type,
}

impl Parse for TypeAnnotation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TypeAnnotation {
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl ToTokens for TypeAnnotation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[derive(Debug, Clone)]
pub struct TableEntry {
    pub attrs: Vec<syn::Attribute>,
    pub key: syn::Ident,
    pub colon_token: Token![:],
    pub value: syn::Expr,
}

impl Parse for TableEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(TableEntry {
            attrs: input.call(syn::Attribute::parse_outer)?,
            key: input.parse()?,
            colon_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ToTokens for TableEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(&self.attrs);
        self.key.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub fn parse_file(mut content: &str) -> syn::Result<File> {
    // Strip the BOM if it is present
    const BOM: &str = "\u{feff}";
    if content.starts_with(BOM) {
        content = &content[BOM.len()..];
    }

    let mut shebang = None;
    if content.starts_with("#!") {
        let rest = crate::whitespace::skip(&content[2..]);
        if !rest.starts_with('[') {
            if let Some(idx) = content.find('\n') {
                shebang = Some(content[..idx].to_string());
                content = &content[idx..];
            } else {
                shebang = Some(content.to_string());
                content = "";
            }
        }
    }

    let mut file: File = syn::parse_str(content)?;
    file.shebang = shebang;
    Ok(file)
}
