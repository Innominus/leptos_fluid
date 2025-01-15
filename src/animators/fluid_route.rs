use leptos::{logging::log, prelude::*};
use leptos_router::{
    components::{ParentRoute, Route, RouteChildren, Routes},
    ChooseView, MatchNestedRoutes, NestedRoute, PossibleRouteMatch, SsrMode,
};

use super::fluid_manager::FluidManager;

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
    Segments: PossibleRouteMatch,
    View: ChooseView,
{
    let mut router_vec = Vec::new();
    path.generate_path(&mut router_vec);
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
    Segments: PossibleRouteMatch,
    Children: MatchNestedRoutes + Clone + Send + 'static,
    View: ChooseView,
{
    let new_children = children
        .clone()
        .into_inner()
        .generate_routes()
        .into_iter()
        .collect::<Vec<_>>();

    let mut router_vec = Vec::new();
    path.generate_path(&mut router_vec);
    ParentRoute(leptos_router::components::ParentRouteProps {
        path,
        view,
        children,
        ssr,
    })
}

#[component(transparent)]
pub fn FluidRoutes<Defs, FallbackFn, Fallback>(
    /// A function that returns the view that should be shown if no route is matched.
    fallback: FallbackFn,
    /// Whether to use the View Transition API during navigation.
    #[prop(optional)]
    transition: bool,
    /// The route definitions. This should consist of one or more [`ParentRoute`] or [`Route`]
    /// components.
    children: RouteChildren<Defs>,
) -> impl IntoView
where
    Defs: MatchNestedRoutes + Clone + Send + 'static,
    FallbackFn: FnOnce() -> Fallback + Clone + Send + 'static,
    Fallback: IntoView + 'static,
{
    let inner = children.clone().into_inner();
    let mut routes = inner
        .generate_routes()
        .into_iter()
        .map(|data| {
            data.segments
                .iter()
                .map(|seg| match seg {
                    leptos_router::PathSegment::Static(_) | leptos_router::PathSegment::Unit => {
                        seg.as_raw_str().to_string()
                    }
                    _ => String::from(":"),
                })
                .map(|seg| seg.replace("/", ""))
                // .filter(|s| !s.is_empty() && s != "/")
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    FluidManager::get_manager()
        .generated_routes
        .update_value(|vals| vals.append(&mut routes));

    Routes(leptos_router::components::RoutesProps {
        fallback,
        transition,
        children,
    })
}

#[component]
pub fn IndexProvider(index: usize, children: Children) -> impl IntoView {
    let manager = FluidManager::get_manager();
    view! { {children()} }
}
