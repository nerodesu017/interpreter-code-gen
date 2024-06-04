use std::collections::HashMap;

use syn::{FieldsUnnamed, PathSegment, Variant};

fn main() {
    // keep track of all written file names
    // this is for the Visitor trait
    let mut all_expr_file_names_with_their_structs_names: Vec<(String, String)> = vec![];
    let mut all_files_names_with_their_structs_names: Vec<(String, String)> = vec![];

    let token_item_use = syn::parse_file("use crate::token::Token;").unwrap().items[0].clone();
    let literal_item_use = syn::parse_file("use crate::literal::Literal;")
        .unwrap()
        .items[0]
        .clone();
    let expr_item_use = syn::parse_file("use super::Expr;").unwrap().items[0].clone();

    let mut mappings: HashMap<&str, &syn::Item> = HashMap::new();
    mappings.insert("Expr", &expr_item_use);
    mappings.insert("Token", &token_item_use);
    mappings.insert("Literal", &literal_item_use);

    let expr_files_structures = vec![
        "Binary : left: Box<Expr>, operator: Box<Token>, right: Box<Expr>",
        // we need Box<Expr> here in Grouping cause otherwise we get a recursion problem
        "Grouping : expression: Box<Expr>",
        "Literal : value: Option<Literal>",
        "Unary : operator: Box<Token>, right: Box<Expr>",
        "Assign : name: Box<Token>, value: Box<Expr>",
        "Call : callee: Box<Expr>, paren: Box<Token>, arguments: Vec<Expr>",
        "Get : object: Box<Expr>, name: Box<Token>",
        "Logical : left: Box<Expr>, operator: Box<Token>, right: Box<Expr>",
        "Set : object: Box<Expr>, name: Box<Token>, value: Box<Expr>",
        "Super : keyword: Box<Token>, method: Box<Token>",
        "This : keyword: Box<Token>",
        "Variable : name: Box<Token>",
    ];

    std::fs::create_dir_all("expr").unwrap();
    for expr_file_structure in expr_files_structures {
        let (struct_name, props) = {
            let split = expr_file_structure.split(" : ").collect::<Vec<&str>>();
            (split[0], split[1])
        };

        let base_file_name = struct_name.to_lowercase() + "_expr";
        let file_name = base_file_name.clone() + ".rs";

        let struct_name = struct_name.to_owned() + "Expr";

        all_expr_file_names_with_their_structs_names
            .push((base_file_name.clone(), struct_name.clone()));
        all_files_names_with_their_structs_names
            .push((base_file_name.clone(), struct_name.clone()));

        let props = props
            .split(", ")
            .map(|prop| return "pub ".to_owned() + prop)
            .collect::<Vec<String>>()
            .join(", ");

        let sstruct = format!("pub struct {} {{ {} }}", struct_name, props);

        let mut ast = syn::parse_file(sstruct.as_str()).unwrap();

        for (key, val) in mappings.iter() {
            if props.contains(*key) {
                ast.items.insert(0, (*val).clone());
            }
        }

        std::fs::write(
            ("expr/".to_owned() + file_name.as_str()).as_str(),
            prettyplease::unparse(&ast),
        )
        .unwrap();
    }

    create_expr_file(&all_expr_file_names_with_their_structs_names);
    create_visitor_file(&all_files_names_with_their_structs_names);
}

fn create_expr_file(all_expr_file_names_with_their_structs_names: &Vec<(String, String)>) {
    let mut pub_mods = all_expr_file_names_with_their_structs_names
        .iter()
        .map(|entry| {
            syn::Item::Mod(syn::ItemMod {
                attrs: vec![],
                vis: syn::Visibility::Public(syn::token::Pub {
                    ..Default::default()
                }),
                unsafety: None,
                mod_token: syn::token::Mod {
                    ..Default::default()
                },
                ident: syn::Ident::new(entry.0.as_str(), proc_macro2::Span::call_site()),
                content: None,
                semi: Some(syn::token::Semi{..Default::default()}),
            })
        })
        .collect::<Vec<syn::Item>>();

    let mut expr_struct = syn::ItemEnum {
        attrs: vec![],
        vis: syn::Visibility::Public(syn::token::Pub{..Default::default()}),
        enum_token: Default::default(),
        ident: syn::Ident::new("Expr", proc_macro2::Span::call_site()),
        generics: Default::default(),
        brace_token: Default::default(),
        variants: syn::punctuated::Punctuated::new(),
    };

    for (file_name, struct_name) in all_expr_file_names_with_their_structs_names {
        let mut unnamed = syn::punctuated::Punctuated::new();
        unnamed.push(syn::Field {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            ident: None,
            colon_token: None,
            mutability: syn::FieldMutability::None,
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: {
                        let mut segments = syn::punctuated::Punctuated::new();

                        segments.push(PathSegment { ident: syn::Ident::new(file_name, proc_macro2::Span::call_site()), arguments: syn::PathArguments::None });
                        segments.push(PathSegment { ident: syn::Ident::new(struct_name, proc_macro2::Span::call_site()), arguments: syn::PathArguments::None });
                        segments
                    }
                }
            })
        });
        expr_struct.variants.push(
            Variant { attrs: Default::default(), ident: syn::Ident::new(&struct_name[0..&struct_name.len() - 4], proc_macro2::Span::call_site()), fields: 
                syn::Fields::Unnamed(FieldsUnnamed {
                    paren_token: Default::default(),
                    unnamed
                }), discriminant: None }
        );
    }

    let mut expr_ast = syn::parse_file("").unwrap();
    expr_ast.items.append(&mut pub_mods);
    expr_ast.items.push(syn::Item::Enum(expr_struct));

    std::fs::write("expr.rs", prettyplease::unparse(&expr_ast)).unwrap();


}

fn create_visitor_file(all_files_names_with_their_structs_names: &Vec<(String, String)>) {
    let visitor_imports = vec![syn::parse_file("use crate::expr::*;").unwrap().items[0].clone()];

    let mut visitor_ast = syn::parse_file(
        "pub trait Visitor<R> {fn visit_nero_expr(&mut self, expr: &nero_expr::NeroExpr) -> R;}",
    )
    .unwrap();
    let visitor_trait = visitor_ast.items[0].clone();
    visitor_ast.items = vec![];

    let mut visitor_trait = if let syn::Item::Trait(item_trait) = visitor_trait {
        item_trait
    } else {
        unreachable!()
    };

    let trait_item_method = visitor_trait.items[0].clone();
    let trait_item_method: syn::TraitItemFn =
        if let syn::TraitItem::Fn(trait_item_fn) = trait_item_method {
            trait_item_fn
        } else {
            unreachable!()
        };
    visitor_trait.items = vec![];

    for (file_name, struct_name) in all_files_names_with_their_structs_names {
        let mut item_met = trait_item_method.clone();
        item_met.sig.ident = syn::Ident::new(
            ("visit_".to_owned() + file_name.as_str()).as_str(),
            proc_macro2::Span::call_site(),
        );

        if let syn::FnArg::Typed(syn::PatType { ref mut ty, .. }) =
            item_met.sig.inputs.last_mut().unwrap()
        {
            if let syn::Type::Reference(ref mut ty) = **ty {
                if let syn::Type::Path(ref mut type_path) = *ty.elem {
                    type_path.path.segments[0] = PathSegment {
                        ident: syn::Ident::new(file_name.as_str(), proc_macro2::Span::call_site()),
                        arguments: syn::PathArguments::None,
                    };
                    type_path.path.segments[1] = PathSegment {
                        ident: syn::Ident::new(
                            struct_name.as_str(),
                            proc_macro2::Span::call_site(),
                        ),
                        arguments: syn::PathArguments::None,
                    };
                }
            }
        }

        visitor_trait.items.insert(0, syn::TraitItem::Fn(item_met));
    }

    for (index, import) in visitor_imports.iter().enumerate() {
        visitor_ast.items.insert(index, import.clone());
    }

    visitor_ast.items.push(syn::Item::Trait(visitor_trait));
    std::fs::write("visitor.rs", prettyplease::unparse(&visitor_ast)).unwrap();
}
