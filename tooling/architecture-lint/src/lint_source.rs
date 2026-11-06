use std::path::Path;
use syn::{Item, ReturnType, Type};

fn standalone_types(items: &[Item], names: &mut Vec<String>) {
    for item in items {
        match item {
            Item::Struct(item) => names.push(item.ident.to_string()),
            Item::Enum(item) => names.push(item.ident.to_string()),
            Item::Trait(item) => names.push(item.ident.to_string()),
            // A namespace or inline test module does not make standalone types nested classes.
            Item::Mod(item) => {
                if let Some((_, items)) = &item.content {
                    standalone_types(items, names);
                }
            }
            _ => {}
        }
    }
}
fn path_ends_with(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == name),
        Type::Reference(reference) => path_ends_with(&reference.elem, name),
        _ => false,
    }
}
fn is_universal_signature(signature: &syn::Signature) -> bool {
    signature.ident == "execute"
        && (signature
            .inputs
            .iter()
            .any(|arg| matches!(arg,syn::FnArg::Typed(arg) if path_ends_with(&arg.ty,"Command")))
            || matches!(&signature.output,ReturnType::Type(_,ty) if path_ends_with(ty,"Output")))
}
fn is_universal_dispatcher(item: &Item) -> bool {
    match item {
        Item::Trait(item) => item.items.iter().any(
            |item| matches!(item,syn::TraitItem::Fn(method) if is_universal_signature(&method.sig)),
        ),
        Item::Impl(item) => item.items.iter().any(
            |item| matches!(item,syn::ImplItem::Fn(method) if is_universal_signature(&method.sig)),
        ),
        Item::Fn(item) => is_universal_signature(&item.sig),
        Item::Mod(item) => item
            .content
            .as_ref()
            .is_some_and(|(_, items)| items.iter().any(is_universal_dispatcher)),
        _ => false,
    }
}
pub fn lint_source(path: &Path, source: &str) -> Result<Vec<String>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut names = Vec::new();
    standalone_types(&syntax.items, &mut names);
    let mut errors = Vec::new();
    if names.len() > 1 {
        errors.push(format!(
            "{}: one standalone struct/enum/trait per file; found {}",
            path.display(),
            names.join(", ")
        ));
    }
    if !names.is_empty()
        && path
            .file_name()
            .is_some_and(|name| name == "lib.rs" || name == "mod.rs")
    {
        errors.push(format!(
            "{}: entry point files only declare modules and explicit exports",
            path.display()
        ));
    }
    if syntax.items.iter().any(is_universal_dispatcher) {
        errors.push(format!(
            "{}: replace universal execute(Command)/Output with focused typed use-case contracts",
            path.display()
        ));
    }
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_multiple_standalone_types_and_inline_module_bypass() {
        for source in [
            "struct First; struct Second;",
            "struct Owner; mod namespace { struct Other; }",
            "trait Port {} enum Status { Ready }",
        ] {
            assert!(
                !lint_source(Path::new("sample.rs"), source)
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn permits_owner_implementations_and_function_local_helpers() {
        assert!(
            lint_source(
                Path::new("owner.rs"),
                "struct Owner; impl Owner { fn run() { struct Local; } }"
            )
            .unwrap()
            .is_empty()
        );
    }
    #[test]
    fn rejects_universal_dispatcher_and_types_in_entry_points() {
        assert!(
            !lint_source(
                Path::new("queue.rs"),
                "trait Queue { fn execute(&mut self, command: Command) -> Output; }"
            )
            .unwrap()
            .is_empty()
        );
        assert!(
            !lint_source(Path::new("lib.rs"), "struct Owner;")
                .unwrap()
                .is_empty()
        );
        assert!(
            lint_source(
                Path::new("publish.rs"),
                "trait PublishMessage { fn publish(&mut self, request: Publish) -> Message; }"
            )
            .unwrap()
            .is_empty()
        );
    }
}
