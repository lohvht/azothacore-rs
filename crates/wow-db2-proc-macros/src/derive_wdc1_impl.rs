use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Data, DeriveInput, Error, Field, Fields, Index, LitInt, Member, Result, Type, Visibility};

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
    layout_hash: Option<LitInt>,
}

impl WDC1TopLayerAttrs {
    fn from_syn(input: &[Attribute]) -> Result<Self> {
        let mut a = WDC1TopLayerAttrs { layout_hash: None };

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
        }

        Ok(a)
    }

    fn validate(&self, original: &DeriveInput) -> Result<()> {
        if self.layout_hash.is_none() {
            return Err(Error::new_spanned(original, "#[layout_hash(...)] is required"));
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
    id:     Option<bool>,
    /// determines if a parent field exists. Is optional
    /// If Some(true), parent is inlined, otherwise its not inlined.
    parent: Option<bool>,
}

impl WDC1FieldAttrs {
    fn from_syn(field_idx: usize, num_fields: usize, field: &Field, processed_field_ty: WDC1FieldType) -> Result<Self> {
        syn::custom_keyword!(inline);

        let mut a = Self { parent: None, id: None };

        for attr in &field.attrs {
            if attr.path().is_ident("id") {
                if a.id.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[id] attribute is allowed"));
                }
                a.id = Some(true);
            } else if attr.path().is_ident("parent") {
                if a.parent.is_some() {
                    return Err(Error::new_spanned(attr, "only one #[parent] attribute is allowed"));
                }

                // checking for #[parent(inline)]. explicit inlining here
                let has_inlined_keyword: Result<bool> = match attr.parse_args_with(|input: ParseStream| Ok(input.parse::<Option<inline>>()?.is_some())) {
                    Err(e) => {
                        let err_string = e.to_compile_error().to_string();
                        if !err_string.contains("expected attribute arguments in parentheses") {
                            return Err(e);
                        }
                        Ok(false)
                    },
                    Ok(r) => Ok(r),
                };
                // i.e. inline if theres an explicit inline keyword, otherwise, if its not the last elem
                let parent_inline = if has_inlined_keyword? { true } else { field_idx != num_fields - 1 };
                a.parent = Some(parent_inline);
            }
        }
        if let Some(name) = &field.ident {
            if *name == "id" && a.id.is_none() {
                if !matches!(processed_field_ty, WDC1FieldType::Single(WDC1FieldSingleType::U32)) {
                    return Err(Error::new_spanned(field, format!("ID field must be of u32 type, got {:?}", processed_field_ty)));
                }
                a.id = Some(false)
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
            Type::Path(p) if p.path.is_ident("LocalisedString") => WDC1FieldType::Single(WDC1FieldSingleType::String),
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
    //     DB2Field::String
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
        let derive_attrs = WDC1TopLayerAttrs::from_syn(&node.attrs)?;

        let span = Span::call_site();
        let fields: Vec<WDC1Field> = WDC1Field::multiple_from_syn(&data.fields, span)?;

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

            if let Some(b) = f.attrs.id {
                if has_id.is_some() {
                    return Err(Error::new_spanned(self.original.clone(), "multiple fields with #[id] found"));
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
            if f.attrs.parent.is_some() {
                if f.attrs.id.is_some() {
                    return Err(Error::new_spanned(
                        self.original.clone(),
                        "can't specify both #[parent] and #[id] at the same time",
                    ));
                }
                if has_parent.is_some() {
                    return Err(Error::new_spanned(self.original.clone(), "cannot define multiple #[parent] fields"));
                }
                has_parent = f.attrs.parent;
            }
        }
        let has_id_tup = if let Some(e) = has_id {
            e
        } else {
            return Err(Error::new_spanned(self.original.clone(), "must contain at least one field with #[id]"));
        };
        let (idx, from_attr) = has_id_tup;
        if !from_attr && idx != 0 {
            return Err(Error::new_spanned(
                self.original.clone(),
                "ID field must be the first if its not from an #[id] attribute field",
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
        let path = quote_spanned!(first_span=> wow_db2::wdc1::);
        let wdc1 = quote_spanned!(last_span=> WDC1);

        quote!(#path #wdc1)
    }

    fn impl_derive(&self) -> TokenStream {
        let ty = &self.struct_name;

        let layout_hash = self.derive_attrs.layout_hash.to_owned().unwrap();
        let layout_method = quote! {
            fn layout_hash() -> u32 {
                #layout_hash
            }
        };

        let id_field = self.fields.iter().find(|e| e.attrs.id.is_some()).unwrap();
        let has_inlined_id = id_field.attrs.id.unwrap();

        let id_field_index = LitInt::new(&id_field.idx.to_string(), id_field.member.span());
        let id_index_res = if has_inlined_id { quote!( Some(#id_field_index) ) } else { quote!(None) };
        let id_index_method = quote! {
            fn id_index() -> std::option::Option<usize> {
                #id_index_res
            }
        };
        let mut num_fields = self.fields.len();
        if !has_inlined_id {
            num_fields -= 1;
        };

        let mut parent_field_typ_token = None;
        let mut parent_field_val = None;
        let parent_field = self.fields.iter().find(|e| e.attrs.parent.is_some());
        if let Some(pf) = parent_field {
            let has_inlined_parent = pf.attrs.parent.unwrap();
            if !has_inlined_parent {
                let (_, t) = pf.get_concrete_type().token();
                let v = pf.get_concrete_type().value_token();

                parent_field_typ_token = Some(t);
                parent_field_val = Some((&pf.member, v));
                num_fields -= 1
            }
        }

        let non_inline_parent_index_type_res = if let Some(t) = &parent_field_typ_token {
            quote!( Some(#t) )
        } else {
            quote!(None)
        };
        let non_inline_parent_index_type_method = quote! {
            fn non_inline_parent_index_type() -> Option<wow_db2::DB2FieldType> {
                #non_inline_parent_index_type_res
            }
        };

        let num_fields = LitInt::new(&num_fields.to_string(), self.original.span());
        let num_fields_method = quote! {
            fn num_fields() -> usize {
                #num_fields
            }
        };

        // Some(field_index), is_id, is_parent
        //      => has_inlined_id, Some(field_index)
        let mut field_idx = 0;
        // db2 fields are not the same as rust fields
        let db2_field_info = self
            .fields
            .iter()
            .filter_map(|f| {
                if has_inlined_id && parent_field_typ_token.is_none() {
                    // this means that every field is inlined, use the idx from the field itself
                    return Some((f.idx, f));
                }
                if let Some(has_inlined_id) = f.attrs.id {
                    if !has_inlined_id {
                        return None;
                    }
                } else if let Some(has_inlined_parent) = f.attrs.parent {
                    if !has_inlined_parent {
                        return None;
                    }
                };
                let res = (field_idx, f);
                field_idx += 1;
                Some(res)
            })
            .collect_vec();

        let db2_fields = db2_field_info
            .iter()
            .map(|(fi, f)| {
                let fi = LitInt::new(&fi.to_string(), f.member.span());
                let (arity, typ_token) = f.get_concrete_type().token();
                quote!((#fi, (#typ_token, #arity)))
            })
            .collect_vec();

        let db2_fields_method = quote! {
            fn db2_fields() -> std::collections::BTreeMap<usize, (wow_db2::DB2FieldType, usize)> {
                std::collections::BTreeMap::from([#(
                    #db2_fields,
                )*])
            }
        };

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
                    if let Some(#val_token(v)) = value.fields.get(&#fi) {
                        s.#fmem = #assigned_value;
                    }
                }
            })
            .collect_vec();

        let parent = if let Some((fmem, val_token)) = parent_field_val {
            quote! {
                if let Some(#val_token(v)) = value.parent {
                    s.#fmem = v[0].clone();
                }
            }
        } else {
            quote! {}
        };

        let from_raw_to_ty_impl = quote! {
            impl std::convert::From<wow_db2::DB2RawRecord> for #ty {
                fn from(value: wow_db2::DB2RawRecord) -> Self {
                    let mut s = Self::default();
                    s.id = value.id;
                    #(
                        #from_body_field_set
                    )*
                    #parent
                    s
                }
            }
        };

        let wdc1_trait = self.spanned_wdbc1_trait();

        let res = quote! {
            impl #wdc1_trait for #ty {
                #layout_method
                #id_index_method
                #num_fields_method
                #db2_fields_method
                #non_inline_parent_index_type_method
                // #provide_method
            }
            #from_raw_to_ty_impl
        };
        // println!("OUTPUT: {res}");
        res
    }
}
