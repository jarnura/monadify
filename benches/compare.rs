use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use monadify::functor::Functor;
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};
#[cfg(feature = "legacy")]
use monadify::legacy::functor::Functor as LegacyFunctor; // Import Kind markers

// Benchmark for Option<T>
pub fn map_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("Functor_Option_map");
    let input: Option<i32> = Some(1);
    let input_ref = &input; // Criterion's bench_with_input expects a reference

    group.bench_with_input(BenchmarkId::new("hkt_map_option", 1), input_ref, |b, &s| {
        b.iter(|| OptionKind::map(s, |x: i32| x + 1)) // Call on marker type
    });

    group.bench_with_input(
        BenchmarkId::new("native_option_map", 1),
        input_ref,
        |b, &s| b.iter(|| Option::map(s, |x: i32| x + 1)),
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_map_option", 1),
        input_ref,
        |b, &s| {
            b.iter(|| <Option<i32> as LegacyFunctor<i32>>::map(s, |x: i32| x + 1))
            // Call as instance method
        },
    );

    group.finish();
}

// Benchmark for Result<T, E>
pub fn map_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("Functor_Result_map");
    let input: Result<i32, String> = Ok(1);
    let input_ref = &input; // input_ref is &Result<i32, String>

    group.bench_with_input(
        BenchmarkId::new("hkt_map_result", 1),
        input_ref,
        |b, s_ref: &Result<i32, String>| {
            b.iter(|| ResultKind::<String>::map(s_ref.clone(), |x: i32| x + 1))
            // Call on marker type
        },
    );

    group.bench_with_input(
        BenchmarkId::new("native_result_map", 1),
        input_ref,
        |b, s_ref: &Result<i32, String>| b.iter(|| Result::map(s_ref.clone(), |x: i32| x + 1)),
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_map_result", 1),
        input_ref,
        |b, s_ref: &Result<i32, String>| {
            b.iter(|| {
                <Result<i32, String> as LegacyFunctor<i32>>::map(s_ref.clone(), |x: i32| x + 1)
            }) // Call as instance method
        },
    );

    group.finish();
}

// Benchmark for Vec<T>
pub fn map_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("Functor_Vec_map");
    let input_vec: Vec<i32> = (0..100).collect(); // A slightly larger vector
    let input_vec_ref = &input_vec;

    group.bench_with_input(
        BenchmarkId::new("hkt_map_vec", input_vec.len()),
        input_vec_ref,
        |b, s_vec| {
            b.iter(|| VecKind::map(s_vec.clone(), |x: i32| x + 1)) // Call on marker type
        },
    );

    group.bench_with_input(
        BenchmarkId::new("native_vec_map", input_vec.len()),
        input_vec_ref,
        |b, s_vec| b.iter(|| s_vec.iter().map(|&x| x + 1).collect::<Vec<i32>>()),
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_map_vec", input_vec.len()),
        input_vec_ref,
        |b, s_vec| {
            b.iter(|| <Vec<i32> as LegacyFunctor<i32>>::map(s_vec.clone(), |x: i32| x + 1))
            // Call as instance method
        },
    );

    group.finish();
}

use monadify::apply::Apply;
use monadify::function::RcFn;
#[cfg(feature = "legacy")]
use monadify::legacy::apply::Apply as LegacyApply;
#[cfg(feature = "legacy")]
use monadify::legacy::monad::Bind as LegacyBind;
use monadify::monad::Bind; // Monad import removed // Import CFn

// Benchmark for Apply on Option<T>
// The `manual_*` arm intentionally hand-rolls the match (rather than `map`) —
// it is the manual baseline we compare HKT `apply` against, so the lint is
// suppressed.
#[allow(clippy::manual_map)]
pub fn apply_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("Apply_Option_ap");
    let static_val_opt: Option<i32> = Some(1);
    let val_opt_ref = &static_val_opt;

    group.bench_with_input(
        BenchmarkId::new("hkt_ap_option", 1),
        val_opt_ref,
        |b, &val_s| {
            b.iter(|| {
                let func_opt: Option<RcFn<i32, i32>> = Some(RcFn::new(|x: i32| x + 1));
                OptionKind::apply(val_s, func_opt) // Call on marker type
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("manual_ap_option", 1),
        val_opt_ref,
        |b, &val_s| {
            b.iter(|| {
                let f_option: Option<Box<dyn Fn(i32) -> i32>> = Some(Box::new(|x: i32| x + 1));
                match f_option {
                    Some(f) => match val_s {
                        Some(v) => Some(f(v)),
                        None => None,
                    },
                    None => None,
                }
            })
        },
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_ap_option", 1),
        val_opt_ref,
        |b, &val_s| {
            b.iter(|| {
                let func_opt: Option<RcFn<i32, i32>> = Some(RcFn::new(|x: i32| x + 1));
                <Option<i32> as LegacyApply<i32>>::apply(val_s, func_opt) // Call as instance method
            })
        },
    );

    group.finish();
}

// Benchmark for Monad on Option<T>
// The `native_*` arm intentionally uses `and_then` (not `map`) — it is the
// native baseline we compare HKT `bind` against, so the lint is suppressed.
#[allow(clippy::bind_instead_of_map)]
pub fn bind_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("Monad_Option_bind");
    let input: Option<i32> = Some(1);
    let input_ref = &input;

    group.bench_with_input(
        BenchmarkId::new("hkt_bind_option", 1),
        input_ref,
        |b, &s_opt| {
            b.iter(|| OptionKind::bind(s_opt, |x: i32| Some(x + 1))) // Call on marker type
        },
    );

    group.bench_with_input(
        BenchmarkId::new("native_option_and_then", 1),
        input_ref,
        |b, &s_opt| b.iter(|| s_opt.and_then(|x: i32| Some(x + 1))),
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_bind_option", 1),
        input_ref,
        |b, &s_opt| {
            b.iter(|| <Option<i32> as LegacyBind<i32>>::bind(s_opt, |x: i32| Some(x + 1)))
            // Call as instance method
        },
    );

    group.finish();
}

// Benchmark for Apply on Result<T, E>
pub fn apply_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("Apply_Result_ap");
    let static_val_res: Result<i32, String> = Ok(1);
    let val_res_ref = &static_val_res;

    group.bench_with_input(
        BenchmarkId::new("hkt_ap_result", 1),
        val_res_ref,
        |b, val_s_ref: &Result<i32, String>| {
            b.iter(|| {
                let func_res: Result<RcFn<i32, i32>, String> = Ok(RcFn::new(|x: i32| x + 1));
                ResultKind::<String>::apply(val_s_ref.clone(), func_res) // Call on marker type
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("manual_ap_result", 1),
        val_res_ref,
        |b, val_s_ref: &Result<i32, String>| {
            b.iter(|| {
                let f_res: Result<Box<dyn Fn(i32) -> i32>, String> = Ok(Box::new(|x: i32| x + 1));
                match f_res {
                    Ok(f) => match val_s_ref.clone() {
                        Ok(v) => Ok(f(v)),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(e),
                }
            })
        },
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_ap_result", 1),
        val_res_ref,
        |b, val_s_ref: &Result<i32, String>| {
            b.iter(|| {
                let func_res: Result<RcFn<i32, i32>, String> = Ok(RcFn::new(|x: i32| x + 1));
                <Result<i32, String> as LegacyApply<i32>>::apply(val_s_ref.clone(), func_res)
                // Call as instance method
            })
        },
    );

    group.finish();
}

// Benchmark for Bind on Result<T, E>
// The `native_*` arm intentionally uses `and_then` (not `map`) — it is the
// native baseline we compare HKT `bind` against, so the lint is suppressed.
#[allow(clippy::bind_instead_of_map)]
pub fn bind_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("Monad_Result_bind");
    let input: Result<i32, String> = Ok(1);
    let input_ref = &input;

    group.bench_with_input(
        BenchmarkId::new("hkt_bind_result", 1),
        input_ref,
        |b, s_res_ref: &Result<i32, String>| {
            b.iter(|| ResultKind::<String>::bind(s_res_ref.clone(), |x: i32| Ok(x + 1)))
            // Call on marker type
        },
    );

    group.bench_with_input(
        BenchmarkId::new("native_result_and_then", 1),
        input_ref,
        |b, s_res_ref: &Result<i32, String>| {
            b.iter(|| s_res_ref.clone().and_then(|x: i32| Ok(x + 1)))
        },
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_bind_result", 1),
        input_ref,
        |b, s_res_ref: &Result<i32, String>| {
            b.iter(|| {
                <Result<i32, String> as LegacyBind<i32>>::bind(
                    s_res_ref.clone(),
                    |x: i32| Ok(x + 1),
                )
            }) // Call as instance method
        },
    );

    group.finish();
}

// Benchmark for Apply on Vec<T>
pub fn apply_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("Apply_Vec_ap");
    let static_val_vec: Vec<i32> = (0..10).collect(); // Smaller vec for ap
    let val_vec_ref = &static_val_vec;

    group.bench_with_input(
        BenchmarkId::new("hkt_ap_vec", static_val_vec.len()),
        val_vec_ref,
        |b, val_s_vec_ref: &Vec<i32>| {
            b.iter(|| {
                let func_vec: Vec<RcFn<i32, i32>> = vec![RcFn::new(|x: i32| x + 1)];
                VecKind::apply(val_s_vec_ref.clone(), func_vec) // Call on marker type
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("manual_ap_vec", static_val_vec.len()),
        val_vec_ref,
        |b, val_s_vec_ref: &Vec<i32>| {
            b.iter(|| {
                let fs_vec: Vec<Box<dyn Fn(i32) -> i32>> = vec![Box::new(|x: i32| x + 1)];
                let mut result_vec = Vec::new();
                if val_s_vec_ref.is_empty() || fs_vec.is_empty() {
                    // return result_vec; // cannot return inside iter
                } else {
                    for f in &fs_vec {
                        for val_a in val_s_vec_ref.iter() {
                            result_vec.push(f(*val_a));
                        }
                    }
                }
                result_vec
            })
        },
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_ap_vec", static_val_vec.len()),
        val_vec_ref,
        |b, val_s_vec_ref: &Vec<i32>| {
            b.iter(|| {
                let func_vec: Vec<RcFn<i32, i32>> = vec![RcFn::new(|x: i32| x + 1)];
                <Vec<i32> as LegacyApply<i32>>::apply(val_s_vec_ref.clone(), func_vec)
                // Call as instance method
            })
        },
    );

    group.finish();
}

// Benchmark for Bind on Vec<T>
pub fn bind_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("Monad_Vec_bind");
    let input_vec: Vec<i32> = (0..10).collect(); // Smaller vec for bind
    let input_vec_ref = &input_vec;

    group.bench_with_input(
        BenchmarkId::new("hkt_bind_vec", input_vec.len()),
        input_vec_ref,
        |b, s_vec_ref: &Vec<i32>| {
            b.iter(|| VecKind::bind(s_vec_ref.clone(), |x: i32| vec![x + 1, x + 2]))
            // Call on marker type
        },
    );

    group.bench_with_input(
        BenchmarkId::new("native_vec_flat_map", input_vec.len()),
        input_vec_ref,
        |b, s_vec_ref: &Vec<i32>| {
            b.iter(|| {
                s_vec_ref
                    .iter()
                    .flat_map(|&x| vec![x + 1, x + 2].into_iter())
                    .collect::<Vec<i32>>()
            })
        },
    );

    #[cfg(feature = "legacy")]
    group.bench_with_input(
        BenchmarkId::new("legacy_bind_vec", input_vec.len()),
        input_vec_ref,
        |b, s_vec_ref: &Vec<i32>| {
            b.iter(|| {
                <Vec<i32> as LegacyBind<i32>>::bind(s_vec_ref.clone(), |x: i32| vec![x + 1, x + 2])
            }) // Call as instance method
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    map_option,
    map_result,
    map_vec,
    apply_option,
    apply_result,
    apply_vec,
    bind_option,
    bind_result,
    bind_vec
);
criterion_main!(benches);
