use std::collections::HashMap;

fn main() {
    let token_item_use_str = "use crate::token::Token;";
    let literal_item_use = "use crate::literal::Literal;";
    let expr_item_use_str = "use super::Expr;";

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
        "Variable : name: Box<Token>"
    ];

    std::fs::create_dir_all("expr").unwrap();
    for expr_file_structure in expr_files_structures {
        let (struct_name, props) = {
            let split = expr_file_structure.split(" : ").collect::<Vec<&str>>();
            (split[0], split[1])
        };

        let file_name = struct_name.to_lowercase() + "_expr.rs";

        let sstruct = format!("pub struct {} {{ {} }}", (struct_name.to_owned() + "Expr"), props);

        let mut ast = syn::parse_file(sstruct.as_str()).unwrap();

        for (key, val) in mappings.iter() {
            if props.contains(*key) {
                ast.items.insert(0, (*val).clone());
            }
        }

        std::fs::write(("expr/".to_owned() + file_name.as_str()).as_str() , prettyplease::unparse(&ast)).unwrap();
    }
}
