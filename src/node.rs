use std::any::Any;
use std::ops::Deref;

use derive_more::derive::{Debug, From};

use crate::Component;

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
    TagName(&'static str),
    Component(#[debug("{:?}", Component::type_id(_0.deref()))] Box<dyn Component>),
}
