use std::rc::Rc;

use leptos::html::ElementType;
use leptos::prelude::{GetUntracked, NodeRef, document};
use leptos::wasm_bindgen::JsCast;
use web_sys::Element;

#[doc(hidden)]
#[derive(Clone)]
pub enum FlipTargetSource {
    Element(Element),
    Resolver(Rc<dyn Fn() -> Option<Element>>),
    IdSelector(String),
}

impl FlipTargetSource {
    pub(crate) fn resolve(&self) -> Option<Element> {
        match self {
            FlipTargetSource::Element(element) => Some(element.clone()),
            FlipTargetSource::Resolver(resolver) => resolver(),
            FlipTargetSource::IdSelector(id_selector) => document().get_element_by_id(id_selector),
        }
    }
}

/// Stable target that can back a single-element `Flip` animator.
pub trait FlipTarget {
    fn into_source(self) -> FlipTargetSource;
}

impl FlipTarget for Element {
    fn into_source(self) -> FlipTargetSource {
        FlipTargetSource::Element(self)
    }
}

impl<E> FlipTarget for NodeRef<E>
where
    E: ElementType,
    E::Output: JsCast + Clone + 'static,
{
    fn into_source(self) -> FlipTargetSource {
        FlipTargetSource::Resolver(Rc::new(move || {
            self.get_untracked().map(|node| node.unchecked_into())
        }))
    }
}
