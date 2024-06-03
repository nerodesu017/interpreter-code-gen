use std::collections::HashMap;

use syn::PathSegment;

fn main() {
    // keep track of all written file names
    // this is for the Visitor trait
    let mut all_files_names_with_their_structs_names: Vec<(String, String)> = vec![];

    let token_item_use_str = "use crate::token::Token;";
    let literal_item_use = "use crate::literal::Literal;";
    let expr_item_use_str = "use super::Expr;";
    let f = syn::parse_file("").unwrap();
    let token_item_use = syn::parse_file(&token_item_use_str).unwrap().items[0].clone();
    let literal_item_use = syn::parse_file(&literal_item_use).unwrap().items[0].clone();
    let expr_item_use = syn::parse_file(&expr_item_use_str).unwrap().items[0].clone();

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

        all_files_names_with_their_structs_names
            .push((base_file_name.clone(), struct_name.clone()));

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

    // visitor
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
