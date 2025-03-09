use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Data, DeriveInput, Error, Field, Fields, Index, LitInt, LitStr, Member, Result, Type, Visibility};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let input = WDC1Struct::from_syn(node)?;
    input.validate()?;

    Ok(input.impl_derive())
}

// TODO: IMPLEMENT Derive macro to get Deserialize/Serialize order
// This should follow the field order
// toml-rs de example - https://github.com/toml-rs/toml/blob/main/crates/toml/src/de.rs
// proc macro example to iterate thru struct field names - https://github.com/dtolnay/syn/issues/516

/// WDC1TopLayerAttrs contains attributes at the derive level.
struct WDC1TopLayerAttrs {
    layout_hash:          Option<LitInt>,
    client_db2_file_name: Option<LitStr>,
    db2_db_table:         Option<LitStr>,
    db2_db_locale_table:  Option<Option<LitStr>>,
}

impl WDC1TopLayerAttrs {
    fn from_syn(input: &[Attribute]) -> Result<Self> {
        let mut a = WDC1TopLayerAttrs {
            layout_hash:          None,
            client_db2_file_name: None,
            db2_db_table:         None,
            db2_db_locale_table:  None,
        };

        for attr in input {
            if attr.path().is_ident("layout_hash") {
                let layout_hash = attr.parse_args_with(|input: ParseStream| {
                    let ret = input.parse::<LitInt>()?;
                    ret.base10_parse::<u32>()?;
                    Ok(ret)
                })?;
                if a.layout_hash.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[layout_hash(...)] attribute is allowed"));
                }
                a.layout_hash = Some(layout_hash);
            }
            if attr.path().is_ident("db2_db_table") {
                let s = attr.parse_args_with(|input: ParseStream| input.parse::<LitStr>())?;
                if a.db2_db_table.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[db2_db_table(...)] attribute is allowed"));
                }
                a.db2_db_table = Some(s);
            }
            if attr.path().is_ident("db2_db_locale_table") {
                let s = attr.parse_args_with(|input: ParseStream| input.parse::<LitStr>()).ok();
                if a.db2_db_locale_table.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[db2_db_locale_table(...)] attribute is allowed"));
                }
                a.db2_db_locale_table = Some(s);
            }
        }

        Ok(a)
    }

    fn validate(&self, original: &DeriveInput) -> Result<()> {
        if self.layout_hash.is_none() {
            return Err(Error::new_spanned(original, "#[layout_hash(...)] is required"));
        }
        if self.client_db2_file_name.is_none() {
            return Err(Error::new_spanned(original, "#[client_db2(...)] is required"));
        }
        if self.db2_db_table.is_none() {
            return Err(Error::new_spanned(original, "#[db2_db_table(...)] is required"));
        }
        Ok(())
    }
}

struct WDC1Struct {
    original:     DeriveInput,
    derive_attrs: WDC1TopLayerAttrs,
    fields:       Vec<WDC1Field>,
    struct_name:  Ident,
}

struct WDC1Field {
    member: Member,
    attrs:  WDC1FieldAttrs,
    idx:    usize,
    ty:     Type,
}

struct WDC1FieldAttrs {
    /// determines if an id field exists.
    /// if Some(true), id comes from attr, (i.e. is inlined)
    /// otherwise, id comes from ident name (i.e. not inlined)
    /// None denotes that an id field does not exist
    inlined_id:        Option<bool>,
    /// determines if a parent field exists. Is optional
    /// If Some(true), parent is inlined, otherwise its not inlined.
    inlined_parent_id: Option<bool>,
}

impl WDC1FieldAttrs {
    fn from_syn(field_idx: usize, num_fields: usize, field: &Field, processed_field_ty: WDC1FieldType) -> Result<Self> {
        syn::custom_keyword!(inline);

        let mut a = Self {
            inlined_parent_id: None,
            inlined_id:        None,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("id_inline") {
                if a.inlined_id.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[id_inline] attribute is allowed"));
                }
                a.inlined_id = Some(true);
            } else if attr.path().is_ident("parent") {
                if a.inlined_parent_id.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[parent] attribute is allowed"));
                }

                // checking for #[parent(inline)]. explicit inlining here
                let has_inlined_keyword: bool = match attr.parse_args_with(|input: ParseStream| Ok(input.parse::<Option<inline>>()?.is_some())) {
                    Err(e) => {
                        let err_string = e.to_compile_error().to_string();
                        if !err_string.contains("expected attribute arguments in parentheses") {
                            return Err(e);
                        }
                        false
                    },
                    Ok(r) => r,
                };
                // inline check - parent is inlined iff:
                // 1. if theres an explicit inline keyword
                // 2. if its not the last elem
                //
                // Otherwise #[parent] will be inlined.
                // The usual case is parent is not inlined b/c most of the time parent id is last
                let parent_inline = if has_inlined_keyword { true } else { field_idx != num_fields - 1 };
                a.inlined_parent_id = Some(parent_inline);
            }
        }
        if let Some(name) = &field.ident {
            if *name == "id" && a.inlined_id.is_none() {
                if !matches!(processed_field_ty, WDC1FieldType::Single(WDC1FieldSingleType::U32)) {
                    return Err(Error::new_spanned(field, format!("ID field must be of u32 type, got {:?}", processed_field_ty)));
                }
                a.inlined_id = Some(false)
            }
        }
        Ok(a)
    }
}

#[derive(Debug)]
enum WDC1FieldType {
    Single(WDC1FieldSingleType),
    Array { arity: usize, typ: WDC1FieldSingleType },
    Vector3 { typ: WDC1FieldSingleType },
    Vector4 { typ: WDC1FieldSingleType },
}

impl WDC1FieldType {
    fn token(&self) -> (usize, TokenStream) {
        match self {
            WDC1FieldType::Single(typ) => (1, typ.token()),
            WDC1FieldType::Array { arity, typ } => (*arity, typ.token()),
            WDC1FieldType::Vector3 { typ, .. } => (3, typ.token()),
            WDC1FieldType::Vector4 { typ, .. } => (4, typ.token()),
        }
    }

    /// Returns the token type for field values and whether or not the value is single or an array
    fn value_token(&self) -> TokenStream {
        match self {
            WDC1FieldType::Single(typ) => typ.value_token(),
            WDC1FieldType::Array { typ, .. } => typ.value_token(),
            WDC1FieldType::Vector3 { typ } => typ.value_token(),
            WDC1FieldType::Vector4 { typ } => typ.value_token(),
        }
    }
}

#[derive(Debug)]
enum WDC1FieldSingleType {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    F32,
    LocalisedString,
    String,
}

impl WDC1FieldSingleType {
    fn token(&self) -> TokenStream {
        match self {
            WDC1FieldSingleType::I64 => quote!(wow_db2::DB2FieldType::I64),
            WDC1FieldSingleType::I32 => quote!(wow_db2::DB2FieldType::I32),
            WDC1FieldSingleType::I16 => quote!(wow_db2::DB2FieldType::I16),
            WDC1FieldSingleType::I8 => quote!(wow_db2::DB2FieldType::I8),
            WDC1FieldSingleType::U64 => quote!(wow_db2::DB2FieldType::U64),
            WDC1FieldSingleType::U32 => quote!(wow_db2::DB2FieldType::U32),
            WDC1FieldSingleType::U16 => quote!(wow_db2::DB2FieldType::U16),
            WDC1FieldSingleType::U8 => quote!(wow_db2::DB2FieldType::U8),
            WDC1FieldSingleType::F32 => quote!(wow_db2::DB2FieldType::F32),
            WDC1FieldSingleType::LocalisedString => quote!(wow_db2::DB2FieldType::LocalisedString),
            WDC1FieldSingleType::String => quote!(wow_db2::DB2FieldType::String),
        }
    }

    fn value_token(&self) -> TokenStream {
        match self {
            WDC1FieldSingleType::I64 => quote!(wow_db2::DB2Field::I64),
            WDC1FieldSingleType::I32 => quote!(wow_db2::DB2Field::I32),
            WDC1FieldSingleType::I16 => quote!(wow_db2::DB2Field::I16),
            WDC1FieldSingleType::I8 => quote!(wow_db2::DB2Field::I8),
            WDC1FieldSingleType::U64 => quote!(wow_db2::DB2Field::U64),
            WDC1FieldSingleType::U32 => quote!(wow_db2::DB2Field::U32),
            WDC1FieldSingleType::U16 => quote!(wow_db2::DB2Field::U16),
            WDC1FieldSingleType::U8 => quote!(wow_db2::DB2Field::U8),
            WDC1FieldSingleType::F32 => quote!(wow_db2::DB2Field::F32),
            WDC1FieldSingleType::LocalisedString => quote!(wow_db2::DB2Field::LocalisedString),
            WDC1FieldSingleType::String => quote!(wow_db2::DB2Field::String),
        }
    }
}

impl WDC1Field {
    fn multiple_from_syn(fields: &Fields, span: Span) -> Result<Vec<Self>> {
        let num_fields = fields.len();
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let member = field
                    .ident
                    .clone()
                    .map(Member::Named)
                    .unwrap_or(Member::Unnamed(Index { index: i as u32, span }));

                let field_ty = WDC1Field::validate_ty(&member, &field.ty, false)?;
                let attrs = WDC1FieldAttrs::from_syn(i, num_fields, field, field_ty)?;
                Ok(Self {
                    member,
                    attrs,
                    idx: i,
                    ty: field.ty.clone(),
                })
            })
            .collect()
    }

    fn validate_ty(mem: &Member, t: &Type, nested: bool) -> Result<WDC1FieldType> {
        let res = match t {
            Type::Path(p) if p.path.is_ident("i64") => WDC1FieldType::Single(WDC1FieldSingleType::I64),
            Type::Path(p) if p.path.is_ident("i32") => WDC1FieldType::Single(WDC1FieldSingleType::I32),
            Type::Path(p) if p.path.is_ident("i16") => WDC1FieldType::Single(WDC1FieldSingleType::I16),
            Type::Path(p) if p.path.is_ident("i8") => WDC1FieldType::Single(WDC1FieldSingleType::I8),
            Type::Path(p) if p.path.is_ident("u64") => WDC1FieldType::Single(WDC1FieldSingleType::U64),
            Type::Path(p) if p.path.is_ident("u32") => WDC1FieldType::Single(WDC1FieldSingleType::U32),
            Type::Path(p) if p.path.is_ident("u16") => WDC1FieldType::Single(WDC1FieldSingleType::U16),
            Type::Path(p) if p.path.is_ident("u8") => WDC1FieldType::Single(WDC1FieldSingleType::U8),
            Type::Path(p) if p.path.is_ident("f32") => WDC1FieldType::Single(WDC1FieldSingleType::F32),
            Type::Path(p) if p.path.is_ident("String") => WDC1FieldType::Single(WDC1FieldSingleType::String),
            Type::Path(p) if p.path.is_ident("LocalisedString") => WDC1FieldType::Single(WDC1FieldSingleType::LocalisedString),
            Type::Path(p) if !p.path.segments.is_empty() && p.path.segments[0].ident == "Vector3" => {
                if nested {
                    return Err(Error::new_spanned(mem.clone(), "cannot have nested vectors"));
                }
                let seg = &p.path.segments[0];
                let args_typ = match &seg.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if args.args.is_empty() {
                            return Err(Error::new_spanned(mem.clone(), "Vector3 should at least one type defined in angled brackets"));
                        }
                        match &args.args[0] {
                            syn::GenericArgument::Type(t) => Self::validate_ty(mem, t, true)?,
                            ga => {
                                return Err(Error::new_spanned(mem.clone(), format!("Vector3 args should be types, got ga {ga:?}")));
                            },
                        }
                    },
                    p => {
                        return Err(Error::new_spanned(
                            mem.clone(),
                            format!("Vector3 should have type info in angled brackets, got p {p:?}"),
                        ));
                    },
                };

                let args_typ = if let WDC1FieldType::Single(r) = args_typ {
                    r
                } else {
                    return Err(Error::new_spanned(mem.clone(), format!("typ isnt a single type: typ {args_typ:?}")));
                };
                WDC1FieldType::Vector3 { typ: args_typ }
            },
            Type::Path(p) if !p.path.segments.is_empty() && p.path.segments[0].ident == "Vector4" => {
                if nested {
                    return Err(Error::new_spanned(mem.clone(), "cannot have nested vectors"));
                }
                let seg = &p.path.segments[0];
                let args_typ = match &seg.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if args.args.is_empty() {
                            return Err(Error::new_spanned(mem.clone(), "Vector4 should at least one type defined in angled brackets"));
                        }
                        match &args.args[0] {
                            syn::GenericArgument::Type(t) => Self::validate_ty(mem, t, true)?,
                            ga => {
                                return Err(Error::new_spanned(mem.clone(), format!("Vector4 args should be types, got ga {ga:?}")));
                            },
                        }
                    },
                    p => {
                        return Err(Error::new_spanned(
                            mem.clone(),
                            format!("Vector4 should have type info in angled brackets, got p {p:?}"),
                        ));
                    },
                };

                let args_typ = if let WDC1FieldType::Single(r) = args_typ {
                    r
                } else {
                    return Err(Error::new_spanned(mem.clone(), format!("typ isnt a single type: typ {args_typ:?}")));
                };
                WDC1FieldType::Vector4 { typ: args_typ }
            },
            Type::Array(t) => {
                if nested {
                    return Err(Error::new_spanned(mem.clone(), "cannot have nested tuple types"));
                }

                let typ = Self::validate_ty(mem, &t.elem, true)?;
                let arity = match &t.len {
                    syn::Expr::Lit(l) => match &l.lit {
                        syn::Lit::Int(l2) => l2.base10_parse()?,
                        _ => return Err(Error::new_spanned(mem.clone(), "Literal on array count is not a number")),
                    },
                    _ => return Err(Error::new_spanned(mem.clone(), "only support literal as Array count definition")),
                };

                let typ = if let WDC1FieldType::Single(r) = typ {
                    r
                } else {
                    return Err(Error::new_spanned(mem.clone(), format!("typ isnt a single type: typ {typ:?}")));
                };
                WDC1FieldType::Array { arity, typ }
            },
            p => return Err(Error::new_spanned(mem.clone(), format!("type {p:?} not supported for the following field"))),
        };
        Ok(res)
    }

    fn validate(&self) -> Result<WDC1FieldType> {
        let res = Self::validate_ty(&self.member, &self.ty, false)?;
        Ok(res)
    }

    fn get_concrete_type(&self) -> WDC1FieldType {
        self.validate().unwrap()
    }

    // fn get_concrete_field_value(&self) -> {
    //     DB2Field::LocalisedString
    //     DB2Field::U32
    //     DB2Field::U32
    //     DB2Field::U16
    //     DB2Field::U8
    //     DB2Field::U8
    //     DB2Field::U8
    //     DB2Field::U8
    //     DB2Field::U8
    //     DB2Field::I32
    // }
}

impl WDC1Struct {
    fn from_syn(node: &DeriveInput) -> Result<WDC1Struct> {
        let data = match &node.data {
            Data::Struct(data) => data,
            _ => return Err(Error::new_spanned(node, "only structs are supported")),
        };
        let mut derive_attrs = WDC1TopLayerAttrs::from_syn(&node.attrs)?;

        let span = Span::call_site();
        if derive_attrs.client_db2_file_name.is_none() {
            derive_attrs.client_db2_file_name = Some(LitStr::new(&format!("{}.db2", node.ident), span));
        }
        let db_table_name = node.ident.to_string().to_case(Case::Snake);
        if derive_attrs.db2_db_table.is_none() {
            derive_attrs.db2_db_table = Some(LitStr::new(&db_table_name, span));
        }

        let fields: Vec<WDC1Field> = WDC1Field::multiple_from_syn(&data.fields, span)?;
        if derive_attrs.db2_db_locale_table.is_none()
            && fields
                .iter()
                .any(|f| matches!(f.get_concrete_type(), WDC1FieldType::Single(WDC1FieldSingleType::LocalisedString)))
        {
            derive_attrs.db2_db_locale_table = Some(Some(LitStr::new(&format!("{db_table_name}_locale"), span)));
        }
        Ok(WDC1Struct {
            original: node.clone(),
            derive_attrs,
            fields,
            struct_name: node.ident.clone(),
        })
    }

    fn validate(&self) -> Result<()> {
        self.derive_attrs.validate(&self.original)?;

        if self.fields.is_empty() {
            return Err(Error::new_spanned(self.original.clone(), "Must define at least 1 field"));
        }
        let mut has_id = None;
        let mut has_parent = None;
        for f in self.fields.iter() {
            let ty = f.validate()?;

            if let Some(b) = f.attrs.inlined_id {
                if has_id.is_some() {
                    return Err(Error::new_spanned(self.original.clone(), "multiple fields with #[id_inline] found"));
                }
                if let WDC1FieldType::Single(WDC1FieldSingleType::U32) = ty {
                    has_id = Some((f.idx, b))
                } else {
                    return Err(Error::new_spanned(
                        self.original.clone(),
                        format!("ID field is not of the correct type, got: {:?}, want u32", ty),
                    ));
                }
            }
            if f.attrs.inlined_parent_id.is_some() {
                if f.attrs.inlined_id.is_some() {
                    return Err(Error::new_spanned(
                        self.original.clone(),
                        "can't specify both #[parent] and #[id_inline] at the same time",
                    ));
                }
                if has_parent.is_some() {
                    return Err(Error::new_spanned(self.original.clone(), "cannot define multiple #[parent] fields"));
                }
                has_parent = f.attrs.inlined_parent_id;
            }
        }
        let has_id_tup = if let Some(e) = has_id {
            e
        } else {
            return Err(Error::new_spanned(self.original.clone(), "must contain at least one field with #[id_inline]"));
        };
        let (idx, from_attr) = has_id_tup;
        if !from_attr && idx != 0 {
            return Err(Error::new_spanned(
                self.original.clone(),
                "ID field must be the first if its not from an #[id_inline] attribute field",
            ));
        }

        Ok(())
    }

    fn spanned_wdbc1_trait(&self) -> TokenStream {
        let vis_span = match &self.original.vis {
            Visibility::Public(vis) => Some(vis.span),
            Visibility::Restricted(vis) => Some(vis.pub_token.span),
            Visibility::Inherited => None,
        };
        let data_span = match &self.original.data {
            Data::Struct(data) => data.struct_token.span,
            Data::Enum(data) => data.enum_token.span,
            Data::Union(data) => data.union_token.span,
        };
        let first_span = vis_span.unwrap_or(data_span);
        let last_span = self.original.ident.span();
        let path = quote_spanned!(first_span=> wow_db2::);
        let wdc1 = quote_spanned!(last_span=> DB2);

        quote!(#path #wdc1)
    }

    fn layout_method(&self) -> TokenStream {
        let h = self.derive_attrs.layout_hash.to_owned().unwrap();
        quote! {
            fn layout_hash() -> u32 {
                #h
            }
        }
    }

    fn db2_file_method(&self) -> TokenStream {
        let h = self.derive_attrs.client_db2_file_name.to_owned().unwrap();
        quote! {
            fn db2_file() -> &'static str {
                #h
            }
        }
    }

    fn id_method(&self) -> TokenStream {
        let f = self.fields.iter().find(|f| f.attrs.inlined_id.is_some()).unwrap();
        let fmem = &f.member;
        // FOrce and assume ID is u32 - which should always hold through for wdc1.
        quote! { fn id(&self) -> u32 { self.#fmem } }
    }

    #[allow(clippy::type_complexity)]
    fn extract_db2_field_info(
        &self,
    ) -> (
        Option<(usize, Member)>,
        Option<TokenStream>,
        Option<(Member, TokenStream)>,
        Option<(usize, Member)>,
        Vec<(usize, &WDC1Field)>,
        usize,
    ) {
        let mut inlined_id_index = None;
        let mut non_inlined_parent_field_typ_token = None;
        let mut non_inlined_parent_field_val = None;
        let mut inlined_parent_id_index = None;
        let mut num_fields = self.fields.len();

        let mut field_idx = 0;
        // db2 fields are not the same as rust fields
        let db2_field_info = self
            .fields
            .iter()
            .filter_map(|f| {
                if let Some(has_inlined_id) = f.attrs.inlined_id {
                    if !has_inlined_id {
                        num_fields -= 1;
                        return None;
                    }
                    inlined_id_index = Some((field_idx, f.member.clone()));
                } else if let Some(has_inlined_parent) = f.attrs.inlined_parent_id {
                    if !has_inlined_parent {
                        let (_, t) = f.get_concrete_type().token();
                        let v = f.get_concrete_type().value_token();
                        num_fields -= 1;
                        non_inlined_parent_field_typ_token = Some(t);
                        non_inlined_parent_field_val = Some((f.member.clone(), v));
                        return None;
                    }
                    inlined_parent_id_index = Some((field_idx, f.member.clone()));
                };
                let res = (field_idx, f);
                field_idx += 1;
                Some(res)
            })
            .collect_vec();
        (
            inlined_id_index,
            non_inlined_parent_field_typ_token,
            non_inlined_parent_field_val,
            inlined_parent_id_index,
            db2_field_info,
            num_fields,
        )
    }

    fn db2_sql_stmt_method(&self) -> TokenStream {
        let db_table = self.derive_attrs.db2_db_table.as_ref().unwrap().value();
        let mut db2_stmt = String::from("SELECT ");
        for (i, f) in self.fields.iter().enumerate() {
            if i > 0 {
                db2_stmt += ", ";
            }
            let (struct_field_name, col_name) = db_query_col_name_from_wdc1_field(f);
            let (arity, typ) = match f.get_concrete_type() {
                WDC1FieldType::Array { arity, typ } => (arity, typ),
                WDC1FieldType::Vector3 { typ } => (3, typ),
                WDC1FieldType::Vector4 { typ } => (4, typ),
                WDC1FieldType::Single(typ) => (1, typ),
            };
            let arity_suffixes = 1..=arity;
            let selected_col = match (&typ, arity) {
                (WDC1FieldSingleType::LocalisedString, arity) => {
                    if arity <= 1 {
                        format!("JSON_OBJECT('enUS', `{db_table}`.`{col_name}`)")
                    } else {
                        let s = arity_suffixes.map(|i| format!("JSON_OBJECT('enUS', `{db_table}`.`{col_name}{i}`)")).join(",");
                        format!("JSON_ARRAY({s})")
                    }
                },
                (_, arity) => {
                    if arity <= 1 {
                        format!("`{db_table}`.`{col_name}`")
                    } else {
                        let s = arity_suffixes.map(|i| format!("`{db_table}`.`{col_name}{i}`")).join(",");
                        format!("JSON_ARRAY({s})")
                    }
                },
            };
            db2_stmt += &format!("{selected_col} as `{struct_field_name}`");
        }
        db2_stmt += &format!(" FROM `{db_table}`");
        let sql = LitStr::new(&db2_stmt, self.original.span());
        quote! {
            fn db2_sql_stmt() -> &'static str {
                #sql
            }
        }
    }

    fn db2_locale_sql_stmt_method(&self) -> TokenStream {
        let Some(locale_table) = self.derive_attrs.db2_db_locale_table.as_ref().map(|v| v.as_ref().unwrap().value()) else {
            return quote! {
                fn db2_sql_locale_stmt() -> Option<&'static str> {
                    None
                }
            };
        };

        let (id_field_struct_name, id_field_col_name) = self
            .fields
            .iter()
            .find(|f| f.attrs.inlined_id.is_some())
            .map(db_query_col_name_from_wdc1_field)
            .unwrap();

        let mut stmt = format!("SELECT {locale_table}.{id_field_col_name} as {id_field_struct_name} ");

        let localised_string_fields = self.fields.iter().filter_map(|f| {
            let a = match f.get_concrete_type() {
                WDC1FieldType::Array {
                    arity,
                    typ: WDC1FieldSingleType::LocalisedString,
                } => arity,
                WDC1FieldType::Vector3 {
                    typ: WDC1FieldSingleType::LocalisedString,
                } => 3,
                WDC1FieldType::Vector4 {
                    typ: WDC1FieldSingleType::LocalisedString,
                } => 4,
                WDC1FieldType::Single(WDC1FieldSingleType::LocalisedString) => 1,
                _ => return None,
            };
            let (sn, cn) = db_query_col_name_from_wdc1_field(f);
            Some((sn, cn, a))
        });
        for (struct_field_name, col_name, arity) in localised_string_fields {
            stmt += ", ";
            let arity_suffixes = 1..=arity;
            let selected_col = if arity <= 1 {
                format!("JSON_OBJECTAGG({locale_table}.locale, {locale_table}.{col_name}_lang)")
            } else {
                let s = arity_suffixes
                    .map(|i| format!("JSON_OBJECTAGG({locale_table}.locale, {locale_table}.{col_name}{i})_lang)"))
                    .join(",");
                format!("JSON_ARRAY({s})")
            };
            stmt += &format!("{selected_col} as {struct_field_name}");
        }

        stmt += &format!(" FROM {locale_table} GROUP BY {locale_table}.{id_field_col_name}");

        let sql = LitStr::new(&stmt, self.original.span());
        quote! {
            fn db2_sql_locale_stmt() -> Option<&'static str> {
                Some(#sql)
            }
        }
    }

    fn db2_fields_method(db2_field_info: &[(usize, &WDC1Field)]) -> TokenStream {
        let fs = db2_field_info
            .iter()
            .map(|(fi, f)| {
                let (sn, _) = db_query_col_name_from_wdc1_field(f);
                let struct_field_col_name = LitStr::new(&sn, f.member.span());
                let fi = LitInt::new(&fi.to_string(), f.member.span());
                let (arity, typ_token) = f.get_concrete_type().token();
                quote!((#fi, (#struct_field_col_name.to_string(), #typ_token, #arity)))
            })
            .collect_vec();

        quote! {
            fn db2_fields() -> std::collections::BTreeMap<usize, (String, wow_db2::DB2FieldType, usize)> {
                std::collections::BTreeMap::from([#(
                    #fs,
                )*])
            }
        }
    }

    fn id_index_method(inlined_id_index: &Option<(usize, Member)>) -> TokenStream {
        let res = if let Some((i, mem)) = inlined_id_index {
            let li = LitInt::new(&i.to_string(), mem.span());
            quote!( Some(#li) )
        } else {
            quote!(None)
        };
        quote! {
            fn inlined_id_index() -> std::option::Option<usize> {
                #res
            }
        }
    }

    fn num_fields_method(num_fields: usize, derive_input: &DeriveInput) -> TokenStream {
        let nf = LitInt::new(&num_fields.to_string(), derive_input.span());
        quote! {
            fn num_fields() -> usize {
                #nf
            }
        }
    }

    fn inline_parent_index_method(inlined_parent_id_index: &Option<(usize, Member)>) -> TokenStream {
        let inline_parent_index_body = if let Some((i, mem)) = inlined_parent_id_index {
            let li = LitInt::new(&i.to_string(), mem.span());
            quote!( Some(#li) )
        } else {
            quote!(None)
        };
        quote! {
            fn inline_parent_index() -> Option<usize> {
                #inline_parent_index_body
            }
        }
    }

    fn non_inline_parent_index_type_method(non_inlined_parent_field_typ_token: &Option<TokenStream>) -> TokenStream {
        let res = if let Some(t) = &non_inlined_parent_field_typ_token {
            quote!( Some(#t) )
        } else {
            quote!(None)
        };
        quote! {
            fn non_inline_parent_index_type() -> Option<wow_db2::DB2FieldType> {
                #res
            }
        }
    }

    fn from_raw_to_wdc1_type_impl(
        ty: &Ident,
        non_inlined_parent_field_val: &Option<(Member, TokenStream)>,
        db2_field_info: &[(usize, &WDC1Field)],
    ) -> TokenStream {
        let from_body_field_set = db2_field_info
            .iter()
            .map(|(fi, f)| {
                let fi = LitInt::new(&fi.to_string(), f.member.span());
                let fmem: &Member = &f.member;
                let concrete_typ = f.get_concrete_type();
                let val_token = concrete_typ.value_token();

                let assigned_value = match concrete_typ {
                    WDC1FieldType::Single(_) => quote!(v[0].clone()),
                    WDC1FieldType::Array { .. } => quote!(v.clone().try_into().unwrap()),
                    WDC1FieldType::Vector3 { .. } => {
                        quote!(nalgebra::Vector3::new(v[0].clone(), v[1].clone(), v[2].clone()))
                    },
                    WDC1FieldType::Vector4 { .. } => quote!(nalgebra::Vector4::new(v[0].clone(), v[1].clone(), v[2].clone(), v[3].clone())),
                };
                quote! {
                    if let Some((_, #val_token(v))) = value.fields.get(&#fi) {
                        s.#fmem = #assigned_value;
                    }
                }
            })
            .collect_vec();

        let non_inline_parent = if let Some((fmem, val_token)) = non_inlined_parent_field_val {
            quote! {
                if let Some(#val_token(v)) = value.parent {
                    s.#fmem = v[0].clone();
                }
            }
        } else {
            quote! {}
        };

        quote! {
            impl std::convert::From<wow_db2::DB2RawRecord> for #ty {
                fn from(value: wow_db2::DB2RawRecord) -> Self {
                    let mut s = Self::default();
                    s.id = value.id;
                    #(
                        #from_body_field_set
                    )*
                    #non_inline_parent
                    s
                }
            }
        }
    }

    fn to_raw_record_data_body_method(db2_field_info: &[(usize, &WDC1Field)]) -> TokenStream {
        let to_raw_record_fields_write = db2_field_info
            .iter()
            .map(|(_, f)| {
                let fmem: &Member = &f.member;
                let concrete_typ = f.get_concrete_type();

                match concrete_typ {
                    WDC1FieldType::Single(WDC1FieldSingleType::LocalisedString) => quote!(
                        res.extend_from_slice(self.#fmem.str(_locale).as_bytes());
                        res.push(0);
                    ),
                    WDC1FieldType::Single(WDC1FieldSingleType::String) => quote!(
                        res.extend_from_slice(self.#fmem.as_bytes());
                        res.push(0);
                    ),
                    WDC1FieldType::Single(..) => quote!(
                        res.extend_from_slice(&self.#fmem.to_be_bytes()[..]);
                    ),
                    WDC1FieldType::Array { typ, .. } | WDC1FieldType::Vector3 { typ, .. } | WDC1FieldType::Vector4 { typ, .. }
                        if matches!(typ, WDC1FieldSingleType::LocalisedString) =>
                    {
                        quote!(
                            for v in &self.#fmem {
                                res.extend_from_slice(v.str(_locale).as_bytes());
                                res.push(0);
                            }
                        )
                    },
                    WDC1FieldType::Array { typ, .. } | WDC1FieldType::Vector3 { typ, .. } | WDC1FieldType::Vector4 { typ, .. }
                        if matches!(typ, WDC1FieldSingleType::String) =>
                    {
                        quote!(
                            for v in &self.#fmem {
                                res.extend_from_slice(v.as_bytes());
                                res.push(0);
                            }
                        )
                    },
                    WDC1FieldType::Array { .. } | WDC1FieldType::Vector3 { .. } | WDC1FieldType::Vector4 { .. } => quote!(
                        for v in &self.#fmem {
                            res.extend_from_slice(&v.to_be_bytes()[..]);
                        }
                    ),
                }
            })
            .collect_vec();
        quote! {
            fn to_raw_record_data(&self, _locale: wow_db2::Locale) -> Vec<u8> {
                let mut res = vec![];
                #(
                    #to_raw_record_fields_write
                )*
                res
            }
        }
    }

    fn merge_strs_method(db2_field_info: &[(usize, &WDC1Field)]) -> TokenStream {
        let localised_str_field_tokens = db2_field_info
            .iter()
            .filter_map(|(fi, f)| {
                let fmem: &Member = &f.member;
                let concrete_typ = f.get_concrete_type();
                let arity = match concrete_typ {
                    WDC1FieldType::Single(WDC1FieldSingleType::LocalisedString) => 1,
                    WDC1FieldType::Array {
                        typ: WDC1FieldSingleType::LocalisedString,
                        arity,
                    } => arity,
                    WDC1FieldType::Vector3 {
                        typ: WDC1FieldSingleType::LocalisedString,
                    } => 3,
                    WDC1FieldType::Vector4 {
                        typ: WDC1FieldSingleType::LocalisedString,
                    } => 4,
                    _ => return None,
                };
                let field_body = if arity <= 1 {
                    quote!(self.#fmem.merge(&ss[0]);)
                } else {
                    let vs = (0..arity).map(|i| {
                        let i = LitInt::new(&i.to_string(), f.member.span());
                        quote!(self.#fmem[#i].merge(&ss[#i]);)
                    });
                    quote! {
                        #(
                            #vs
                        )*
                    }
                };

                let value_typ = concrete_typ.value_token();
                let fi = LitInt::new(&fi.to_string(), f.member.span());
                // ss is Vec<LocalisedString>
                let res = quote! {
                    if let Some((_, #value_typ(ss))) = _raw.fields.get(&#fi) {
                        #field_body
                    }
                };
                Some(res)
            })
            .collect_vec();
        quote! {
            fn merge_strs(&mut self, _raw: &wow_db2::DB2RawRecord) {
                #(
                    #localised_str_field_tokens
                )*
            }
        }
    }

    fn impl_derive(&self) -> TokenStream {
        let ty = &self.struct_name;
        let id_method = self.id_method();
        let layout_method = self.layout_method();
        let db2_file_method = self.db2_file_method();
        let (inlined_id_index, non_inlined_parent_field_typ_token, non_inlined_parent_field_val, inlined_parent_id_index, db2_field_info, num_fields) =
            self.extract_db2_field_info();
        let db2_sql_stmt_method = self.db2_sql_stmt_method();
        let db2_locale_sql_stmt_method = self.db2_locale_sql_stmt_method();
        let db2_fields_method = Self::db2_fields_method(&db2_field_info);
        let merge_strs_method = Self::merge_strs_method(&db2_field_info);
        let from_raw_to_ty_impl = Self::from_raw_to_wdc1_type_impl(ty, &non_inlined_parent_field_val, &db2_field_info);
        let to_raw_record_data_body_method = Self::to_raw_record_data_body_method(&db2_field_info);
        let id_index_method = Self::id_index_method(&inlined_id_index);
        let num_fields_method = Self::num_fields_method(num_fields, &self.original);
        let inline_parent_index_method = Self::inline_parent_index_method(&inlined_parent_id_index);
        let non_inline_parent_index_type_method = Self::non_inline_parent_index_type_method(&non_inlined_parent_field_typ_token);
        let wdc1_trait = self.spanned_wdbc1_trait();

        let res = quote! {
            impl #wdc1_trait for #ty {
                #id_method
                #db2_file_method
                #db2_sql_stmt_method
                #db2_locale_sql_stmt_method
                #layout_method
                #id_index_method
                #num_fields_method
                #db2_fields_method
                #non_inline_parent_index_type_method
                #to_raw_record_data_body_method
                #inline_parent_index_method
                #merge_strs_method
                // #provide_method
            }
            #from_raw_to_ty_impl
        };
        // println!("OUTPUT: {res}");
        res
    }
}

/// From the wdc1 field, the name of the struct field to be used and its respective DB column name (Pascal case with ID uppercase)
fn db_query_col_name_from_wdc1_field(f: &WDC1Field) -> (String, String) {
    let mut sn = f.member.to_token_stream().to_string();
    if let Some(s) = sn.strip_prefix("r#") {
        // Remove raw identifier prefix from rust end, if any
        sn = s.to_string();
    }
    let cn = {
        let mut sn = sn.to_case(Case::Pascal);
        if sn.ends_with("Id") {
            let id_part = sn.split_off(sn.len() - 2);
            sn += &id_part.to_uppercase();
        }
        sn
    };
    (sn, cn)
}
