use syn;
use proc_macro2::Span;

use diagnostic_shim::*;
use field::*;
use meta::*;

pub struct Model {
    pub name: syn::Ident,
    pub primary_key_names: Vec<syn::Ident>,
    table_name_from_attribute: Option<syn::Ident>,
    fields: Vec<Field>,
}

impl Model {
    pub fn from_item(item: &syn::DeriveInput) -> Result<Self, Diagnostic> {
        let table_name_from_attribute =
            MetaItem::with_name(&item.attrs, "table_name").map(|m| m.expect_ident_value());
        let primary_key_names = MetaItem::with_name(&item.attrs, "primary_key")
            .map(|m| Ok(m.nested()?.map(|m| m.expect_word()).collect()))
            .unwrap_or_else(|| Ok(vec!["id".into()]))?;
        let fields = fields_from_item_data(&item.data)?;
        Ok(Self {
            name: item.ident,
            table_name_from_attribute,
            primary_key_names,
            fields,
        })
    }

    pub fn table_name(&self) -> syn::Ident {
        self.table_name_from_attribute.unwrap_or_else(|| {
            syn::Ident::new(
                &infer_table_name(self.name.as_ref()),
                self.name.span.resolved_at(Span::call_site()),
            )
        })
    }

    pub fn dummy_mod_name(&self, trait_name: &str) -> syn::Ident {
        let name = self.name.as_ref().to_lowercase();
        format!("_impl_{}_for_{}", trait_name, name).into()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}

pub fn camel_to_snake(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    result.push_str(&name[..1].to_lowercase());
    for character in name[1..].chars() {
        if character.is_uppercase() {
            result.push('_');
            for lowercase in character.to_lowercase() {
                result.push(lowercase);
            }
        } else {
            result.push(character);
        }
    }
    result
}

fn infer_table_name(name: &str) -> String {
    let mut result = camel_to_snake(name);
    result.push('s');
    result
}

fn fields_from_item_data(data: &syn::Data) -> Result<Vec<Field>, Diagnostic> {
    use syn::Data::*;

    let struct_data = match *data {
        Struct(ref d) => d,
        _ => return Err(Span::call_site().error("This derive can only be used on structs")),
    };
    Ok(struct_data
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| Field::from_struct_field(f, i))
        .collect())
}