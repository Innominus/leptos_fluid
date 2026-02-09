use leptos::prelude::*;
use leptos_router::{
    components::{RouteChildren, Routes},
    MatchNestedRoutes,
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
