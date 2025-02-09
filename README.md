## Leptos Fluid
Leptos Fluid allows you to animate nested outlets in Leptos
You really only need to worry about 3 parts:
  - The Manager
  - FluidRoutes
  - FluidOutlet

Following the example, we setup the manager context before the routes are declared.
We wrap all of our routes in a FluidRoutes component which is just a wrapper around the Routes component.
Then we just use FluidOutlet where we would use a normal Outlet and provide and intro and outro CSS class. The crate will handle the rest!
