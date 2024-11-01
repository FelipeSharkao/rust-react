use std::any::Any;

use derive_more::derive::From;

#[derive(From, Debug)]
pub enum ReactNode {
    Text(String),
    Element(ReactElement),
    List(Vec<ReactNode>),
}

#[derive(From, Debug)]
pub struct ReactElement {
    pub ty: ReactElementType,
    pub props: Box<dyn Any>,
    pub children: Vec<ReactNode>,
}

#[derive(From, Debug)]
pub enum ReactElementType {
    TagName(String),
    Component(()),
}
