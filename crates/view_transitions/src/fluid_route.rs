use leptos::prelude::*;
use leptos_router::{
    MatchNestedRoutes,
    components::{RouteChildren, Routes},
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
                    // Dynamic/optional/wildcard segments are normalized so route-order
                    // comparison can still infer forward/backward navigation direction.
                    _ => String::from(":"),
                })
                .map(|seg| seg.replace("/", ""))
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
