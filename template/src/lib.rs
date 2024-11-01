use derive_more::derive::{Debug, Display, From};
use quote::{quote, ToTokens};
use std::iter::Peekable;

use proc_macro2::{token_stream, TokenStream, TokenTree};
use proc_macro_error::{abort, abort_call_site, proc_macro_error};

#[proc_macro_error]
#[proc_macro]
pub fn template(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream: TokenStream = stream.into();
    let mut tokens = stream.into_iter().peekable();
    let node = parse_element(&mut tokens);
    quote! { #node }.into()
}

type Tokens<'a> = &'a mut Peekable<token_stream::IntoIter>;

#[derive(Debug, From)]
enum Node {
    Text(TextNode),
    Element(ElementNode),
    Expr(syn::Expr),
}
impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::Text(x) => x.to_tokens(tokens),
            Node::Element(x) => x.to_tokens(tokens),
            Node::Expr(x) => x.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
struct TextNode(String);
impl ToTokens for TextNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.0;
        let node = quote! { ::rust_react::ReactNode::Text(#value.to_string()) };
        tokens.extend(node);
    }
}

#[derive(Debug)]
struct ElementNode {
    ty: ElementType,
    props: Vec<(String, Prop)>,
    children: Vec<Node>,
}
impl ToTokens for ElementNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children = &self.children;
        let children_nodes = quote! { vec![#(#children),*] };

        let node = match &self.ty {
            ElementType::Fragment => {
                quote! { ::rust_react::ReactNode::List(#children_nodes) }
            }
            ElementType::TagName(..) | ElementType::Component(..) => {
                let variant = match &self.ty {
                    ElementType::TagName(name) => {
                        quote! {
                            ::rust_react::ReactElementType::TagName(#name.to_string())
                        }
                    }
                    ElementType::Component(path) => {
                        quote! {
                            ::rust_react::ReactElementType::Component(#path)
                        }
                    }
                    ElementType::Fragment => unreachable!(),
                };
                quote! {
                    {
                        let ty = #variant;
                        ::rust_react::ReactNode::Element(
                            ::rust_react::ReactElement {
                                ty,
                                props: Box::new(()),
                                children: #children_nodes,
                            }
                        )
                    }
                }
            }
        };

        tokens.extend(node);
    }
}

#[derive(Debug, PartialEq, Eq, Display)]
enum ElementType {
    #[display("Fragment")]
    Fragment,
    #[display("{_0}")]
    TagName(String),
    #[display("{}", quote! { #_0 })]
    Component(syn::Path),
}

#[derive(Debug)]
enum Prop {
    Active,
    Text(String),
    Expr(syn::Expr),
}

fn parse_element(tokens: Tokens) -> Node {
    match tokens.peek() {
        Some(TokenTree::Punct(punct)) if punct.to_string() == "<" => {}
        Some(tt) => {
            abort!(tt.span(), "Expected template node");
        }
        None => abort_call_site!("Expected template node"),
    }

    tokens.next();

    parse_element_body(tokens)
}

/// Parse element without the leading trailing bracket. Use it it was already consumed by the
/// parser.
fn parse_element_body(tokens: Tokens) -> Node {
    let elem_ty = parse_element_type(tokens);

    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(punct)) if punct.to_string() == ">" => {
                tokens.next();
                break;
            }
            _ => todo!("attributes"),
        }
    }

    let children = parse_content(tokens);

    parse_end_tag_body(tokens, &elem_ty);

    ElementNode {
        ty: elem_ty,
        props: vec![],
        children,
    }
    .into()
}

fn parse_element_type(tokens: Tokens) -> ElementType {
    let mut path_buf = vec![];

    let mut angle_brackets = 0;
    let mut last_was_ident = false;

    loop {
        match tokens.peek() {
            Some(TokenTree::Punct(punct)) => match punct.to_string().as_str() {
                "<" => angle_brackets += 1,
                ">" if angle_brackets > 0 => angle_brackets -= 1,
                "::" | "." => {}
                _ => {
                    if angle_brackets == 0 {
                        break;
                    }
                }
            },
            Some(TokenTree::Group(..)) if angle_brackets > 0 => {}
            Some(TokenTree::Ident(..)) => {
                if angle_brackets == 0 && last_was_ident {
                    break;
                }
            }
            _ => break,
        }

        last_was_ident = matches!(tokens.peek(), Some(TokenTree::Ident(..)));

        path_buf.extend(tokens.next());
    }

    if path_buf.len() == 0 {
        return ElementType::Fragment;
    }

    let path = match syn::parse2::<syn::Path>(TokenStream::from_iter(path_buf)) {
        Ok(path) => path,
        Err(e) => abort!(e.span(), "{e}"),
    };
    let ident = path.get_ident().map(|x| x.to_string());

    match ident {
        Some(ident) if ident.chars().all(|c| c.is_lowercase()) => ElementType::TagName(ident),
        _ => ElementType::Component(path),
    }
}

/// Parse end tag without the leading trailing bracket. Use it it was already consumed by
/// the parser.
fn parse_end_tag_body(tokens: Tokens, elem_ty: &ElementType) {
    let Some(start_span) = tokens.peek().map(|tt| tt.span()) else {
        abort_call_site!("Invalid template");
    };

    let found = parse_element_type(tokens);
    if &found != elem_ty {
        abort!(start_span, "Expected '</{elem_ty}>'");
    }

    match tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.to_string() == ">" => {}
        Some(tt) => abort!(tt.span(), "Expected '>'"),
        _ => abort!(start_span, "Expected '</{elem_ty}>'"),
    }
}

fn parse_content(tokens: Tokens) -> Vec<Node> {
    let mut nodes = vec![];

    let mut text_buf = vec![];

    let flush_text_buf = |text_buf: &mut Vec<TokenTree>, nodes: &mut Vec<Node>| {
        let text_tts = text_buf.split_off(0);
        // FIXME: whitespace
        let text = TokenStream::from_iter(text_tts).to_string();
        nodes.push(TextNode(text).into());
    };

    loop {
        match tokens.peek().clone() {
            //nodes.push(parse_element(tokens));
            Some(TokenTree::Punct(punct)) if punct.to_string() == "<" => {
                flush_text_buf(&mut text_buf, &mut nodes);
                tokens.next();
                match tokens.peek().clone() {
                    Some(TokenTree::Punct(punct)) if punct.to_string() == "/" => {
                        tokens.next();
                        break;
                    }
                    _ => nodes.push(parse_element_body(tokens)),
                }
                continue;
            }
            Some(tt) => {
                text_buf.push(tt.clone());
            }
            None => break,
        }

        tokens.next();
    }

    flush_text_buf(&mut text_buf, &mut nodes);

    nodes
}
