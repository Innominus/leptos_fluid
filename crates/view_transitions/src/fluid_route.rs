use leptos::prelude::*;
use leptos_router::{
    MatchNestedRoutes,
    components::{FlatRoutes, RouteChildren, Routes},
};

use crate::fluid_manager::FluidManager;

/// Thin wrapper over `leptos_router::Routes` that also records route patterns
/// for transition direction detection.
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
    let routes = inner
        .generate_routes()
        .into_iter()
        .map(|data| {
            data.segments
                .iter()
                .map(|seg| match seg {
                    leptos_router::PathSegment::Static(_) => {
                        seg.as_raw_str().trim_matches('/').to_string()
                    }
                    leptos_router::PathSegment::Unit => String::new(),
                    // Dynamic/optional/wildcard segments are normalized so route-order
                    // comparison can still infer forward/backward navigation direction.
                    _ => String::from(":"),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    FluidManager::get_manager()
        .generated_routes
        .update_value(|vals| vals.extend(routes));

    Routes(leptos_router::components::RoutesProps {
        fallback,
        transition,
        children,
    })
}

/// Thin wrapper over `leptos_router::Routes` that also records route patterns
/// for transition direction detection.
/// This wrapper in particular is for flat routes
#[component(transparent)]
pub fn FluidFlatRoutes<Defs, FallbackFn, Fallback>(
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
    let routes = inner
        .generate_routes()
        .into_iter()
        .map(|data| {
            // Code was originally written with nested routes in mind, so we need to massage the data to be compatible with the current implementation.
            // Nested routes put an initial empty string into the route stack from the segments, so we mimic this.
            // This is important for the reversal detection currently, and can be changed in future.
            let mut route_stack = vec!["".to_string()];
            route_stack.append(
                &mut data
                    .segments
                    .iter()
                    .map(|seg| match seg {
                        leptos_router::PathSegment::Static(_) => {
                            seg.as_raw_str().trim_matches('/').to_string()
                        }
                        leptos_router::PathSegment::Unit => String::new(),
                        // Dynamic/optional/wildcard segments are normalized so route-order
                        // comparison can still infer forward/backward navigation direction.
                        _ => String::from(":"),
                    })
                    .collect::<Vec<_>>(),
            );

            route_stack
        })
        .collect::<Vec<_>>();

    FluidManager::get_manager()
        .generated_routes
        .update_value(|vals| vals.extend(routes));

    FlatRoutes(leptos_router::components::FlatRoutesProps {
        fallback,
        transition,
        children,
    })
}
