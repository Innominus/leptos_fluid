#[cfg(not(feature = "bench"))]
compile_error!("Enable the bench feature: cargo bench -p leptos_fluid_motion --features bench");

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use leptos_fluid_motion::{FluidStyle, Spring, Transition};

#[cfg(feature = "bench")]
use leptos_fluid_motion::spring_step;

fn bench_spring_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("spring_step");
    let configs = [(400, 0.2), (600, 0.5), (800, 0.8)];

    for (duration, bounce) in configs {
        let spring = Spring::new(duration, bounce);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}ms-{}", duration, bounce)),
            &spring,
            |b, spring| {
                b.iter(|| {
                    let mut value = 0.0;
                    let mut velocity = 0.0;
                    let target = 1.0;
                    let dt = 1.0 / 60.0;
                    for _ in 0..120 {
                        #[cfg(feature = "bench")]
                        {
                            let (next_value, next_velocity) =
                                spring_step(value, velocity, target, *spring, dt);
                            value = next_value;
                            velocity = next_velocity;
                        }
                    }
                    black_box((value, velocity));
                })
            },
        );
    }

    group.finish();
}

fn bench_fluid_style_to_props(c: &mut Criterion) {
    c.bench_function("fluid_style_to_props", |b| {
        let style = FluidStyle::new()
            .opacity(0.8)
            .x(12.0)
            .y(-6.0)
            .scale(1.05)
            .rotate(8.0)
            .with("filter", "blur(6px)")
            .with("background", "linear-gradient(120deg, #0b0d18, #111827)")
            .with("border-radius", "24px")
            .with("box-shadow", "0 18px 50px rgba(6, 7, 18, 0.45)");
        b.iter(|| {
            let props = style.to_props();
            black_box(props);
        })
    });
}

fn bench_transition_css(c: &mut Criterion) {
    c.bench_function("transition_css", |b| {
        let transition = Transition::spring_with(620, 0.45)
            .exclude_properties(["width", "height", "filter"])
            .duration_ms(520)
            .delay_ms(30);
        b.iter(|| {
            #[cfg(feature = "bench")]
            {
                let css = transition.transition_css_public();
                black_box(css);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_spring_steps,
    bench_fluid_style_to_props,
    bench_transition_css
);
criterion_main!(benches);
