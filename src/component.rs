use std::any::{Any, TypeId};

use crate::ReactNode;

pub trait Component: 'static {
    fn render_untyped(&self, untyped_props: &dyn Any) -> ReactNode;
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub trait ComponentType {
    type Props: Sized;
    fn render(&self, props: &Self::Props) -> ReactNode;
}

impl<'a, T> Component for dyn ComponentType<Props = T>
where
    Self: 'static,
{
    fn render_untyped(&self, untyped_props: &dyn Any) -> ReactNode {
        let props: &T = *untyped_props
            .downcast_ref()
            .expect("props should be of correct type");
        self.render(props)
    }
}

impl<'a, F, T> ComponentType for F
where
    F: FnMut(&T) -> ReactNode,
{
    type Props = T;
    fn render(&self, props: &T) -> ReactNode {
        self(props)
    }
}
