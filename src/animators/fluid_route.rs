use leptos::{logging::log, prelude::*};
use leptos_router::{
    components::{ParentRoute, Route, RouteChildren},
    ChooseView, NestedRoute, SsrMode,
};

#[component(transparent)]
pub fn FluidRoute<Segments, View>(
    /// The path fragment that this route should match. This can be created using the
    /// [`path`](crate::path) macro, or path segments ([`StaticSegment`](crate::StaticSegment),
    /// [`ParamSegment`](crate::ParamSegment), [`WildcardSegment`](crate::WildcardSegment), and
    /// [`OptionalParamSegment`](crate::OptionalParamSegment)).
    path: Segments,
    /// The view for this route.
    view: View,
    /// The mode that this route prefers during server-side rendering.
    /// Defaults to out-of-order streaming.
    #[prop(optional)]
    ssr: SsrMode,
) -> NestedRoute<Segments, (), (), View>
where
    View: ChooseView,
{
    log!("I'm in a route :)");
    Route(leptos_router::components::RouteProps { path, view, ssr })
}

#[component(transparent)]
pub fn FluidParentRoute<Segments, View, Children>(
    /// The path fragment that this route should match. This can be created using the
    /// [`path`](crate::path) macro, or path segments ([`StaticSegment`](crate::StaticSegment),
    /// [`ParamSegment`](crate::ParamSegment), [`WildcardSegment`](crate::WildcardSegment), and
    /// [`OptionalParamSegment`](crate::OptionalParamSegment)).
    path: Segments,
    /// The view for this route.
    view: View,
    /// Nested child routes.
    children: RouteChildren<Children>,
    /// The mode that this route prefers during server-side rendering.
    /// Defaults to out-of-order streaming.
    #[prop(optional)]
    ssr: SsrMode,
) -> NestedRoute<Segments, Children, (), View>
where
    View: ChooseView,
{
    log!("I am in a ParentRoute");
    ParentRoute(leptos_router::components::ParentRouteProps {
        path,
        view,
        children,
        ssr,
    })
}
