use std::marker::PhantomData;
use std::rc::Rc;

use web_sys::Element;

use super::{Flip, FlipGroup, FlipOptions, FlipTarget, FlipTargetSource};

#[doc(hidden)]
pub struct FlipBuilderNeedsTarget;

#[doc(hidden)]
pub struct FlipBuilderReady;

pub struct FlipBuilder<State = FlipBuilderNeedsTarget> {
    source: Option<FlipTargetSource>,
    options: FlipOptions,
    _marker: PhantomData<State>,
}

pub type ReadyFlipBuilder = FlipBuilder<FlipBuilderReady>;

#[doc(hidden)]
pub struct FlipGroupBuilderNeedsSelector;

#[doc(hidden)]
pub struct FlipGroupBuilderReady;

pub struct FlipGroupBuilder<State = FlipGroupBuilderNeedsSelector> {
    selector: Option<String>,
    options: FlipOptions,
    _marker: PhantomData<State>,
}

pub type ReadyFlipGroupBuilder = FlipGroupBuilder<FlipGroupBuilderReady>;

impl FlipBuilder {
    pub(crate) fn new() -> Self {
        Self {
            source: None,
            options: FlipOptions::new(),
            _marker: PhantomData,
        }
    }
}

impl<State> FlipBuilder<State> {
    pub fn options(mut self, options: FlipOptions) -> Self {
        self.options = options;
        self
    }
}

impl FlipBuilder<FlipBuilderNeedsTarget> {
    pub fn id(mut self, id_selector: impl Into<String>) -> ReadyFlipBuilder {
        self.source = Some(FlipTargetSource::IdSelector(id_selector.into()));
        FlipBuilder {
            source: self.source,
            options: self.options,
            _marker: PhantomData,
        }
    }

    pub fn target<T: FlipTarget>(mut self, target: T) -> ReadyFlipBuilder {
        self.source = Some(target.into_source());
        FlipBuilder {
            source: self.source,
            options: self.options,
            _marker: PhantomData,
        }
    }

    pub fn resolver<F>(mut self, resolver: F) -> ReadyFlipBuilder
    where
        F: Fn() -> Option<Element> + 'static,
    {
        self.source = Some(FlipTargetSource::Resolver(Rc::new(resolver)));
        FlipBuilder {
            source: self.source,
            options: self.options,
            _marker: PhantomData,
        }
    }
}

impl ReadyFlipBuilder {
    pub fn install(self) -> Flip {
        let source = self
            .source
            .expect("FlipBuilder requires a target, resolver, or id selector");
        Flip::from_source(source, self.options)
    }
}

impl FlipGroupBuilder {
    pub(crate) fn new() -> Self {
        Self {
            selector: None,
            options: FlipOptions::new(),
            _marker: PhantomData,
        }
    }
}

impl<State> FlipGroupBuilder<State> {
    pub fn options(mut self, options: FlipOptions) -> Self {
        self.options = options;
        self
    }
}

impl FlipGroupBuilder<FlipGroupBuilderNeedsSelector> {
    pub fn selector(mut self, selector: impl Into<String>) -> ReadyFlipGroupBuilder {
        self.selector = Some(selector.into());
        FlipGroupBuilder {
            selector: self.selector,
            options: self.options,
            _marker: PhantomData,
        }
    }
}

impl ReadyFlipGroupBuilder {
    pub fn install(self) -> FlipGroup {
        let selector = self
            .selector
            .expect("FlipGroupBuilder requires a selector before install()");
        FlipGroup::from_selector(selector, self.options)
    }
}
