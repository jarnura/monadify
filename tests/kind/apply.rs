use monadify::apply::kind::{apply_first, apply_second, lift2, lift3};
use monadify::fn2;
use monadify::fn3;
use monadify::function::RcFn;
use monadify::kind_based::kind::{OptionKind, ResultKind, VecKind};

type TestError = String;

// ── lift2 ──────────────────────────────────────────────────────────────────

#[test]
fn lift2_option_some() {
    let add = fn2!(|x: i32| move |y: i32| x + y);
    let result = lift2::<OptionKind, _, _, _, _>(add, Some(3), Some(7));
    assert_eq!(result, Some(10));
}

#[test]
fn lift2_option_first_none() {
    let add = fn2!(|x: i32| move |y: i32| x + y);
    let result = lift2::<OptionKind, _, _, _, _>(add, None, Some(7));
    assert_eq!(result, None);
}

#[test]
fn lift2_option_second_none() {
    let add = fn2!(|x: i32| move |y: i32| x + y);
    let result = lift2::<OptionKind, _, _, _, _>(add, Some(3), None);
    assert_eq!(result, None);
}

#[test]
fn lift2_result_ok() {
    let concat = fn2!(|x: i32| move |y: i32| format!("{x}+{y}"));
    let result = lift2::<ResultKind<TestError>, _, _, _, _>(concat, Ok(1), Ok(2));
    assert_eq!(result, Ok("1+2".to_string()));
}

#[test]
fn lift2_result_err_first() {
    let concat = fn2!(|x: i32| move |y: i32| format!("{x}+{y}"));
    let result = lift2::<ResultKind<TestError>, _, _, _, _>(concat, Err("bad".to_string()), Ok(2));
    assert!(result.is_err());
}

#[test]
fn lift2_vec_cartesian() {
    // Vec Apply is a cartesian product: [1,2] + [10,20] → [11,21,12,22]
    let add = fn2!(|x: i32| move |y: i32| x + y);
    let mut result = lift2::<VecKind, _, _, _, _>(add, vec![1, 2], vec![10, 20]);
    result.sort_unstable();
    assert_eq!(result, vec![11, 12, 21, 22]);
}

// ── lift3 ──────────────────────────────────────────────────────────────────

#[test]
fn lift3_option_all_some() {
    let add3 = fn3!(|x: i32| move |y: i32| move |z: i32| x + y + z);
    let result = lift3::<OptionKind, _, _, _, _, _>(add3, Some(1), Some(2), Some(3));
    assert_eq!(result, Some(6));
}

#[test]
fn lift3_option_middle_none() {
    let add3 = fn3!(|x: i32| move |y: i32| move |z: i32| x + y + z);
    let result = lift3::<OptionKind, _, _, _, _, _>(add3, Some(1), None, Some(3));
    assert_eq!(result, None);
}

#[test]
fn lift3_option_last_none() {
    let add3 = fn3!(|x: i32| move |y: i32| move |z: i32| x + y + z);
    let result = lift3::<OptionKind, _, _, _, _, _>(add3, Some(1), Some(2), None);
    assert_eq!(result, None);
}

// ── apply_first ────────────────────────────────────────────────────────────

#[test]
fn apply_first_option_both_some() {
    let result = apply_first::<OptionKind, i32, &str>(Some(42), Some("ignored"));
    assert_eq!(result, Some(42));
}

#[test]
fn apply_first_option_first_none() {
    let result = apply_first::<OptionKind, i32, &str>(None, Some("ignored"));
    assert_eq!(result, None);
}

#[test]
fn apply_first_option_second_none() {
    // If the second container is None the result is None (sequencing is strict)
    let result = apply_first::<OptionKind, i32, &str>(Some(42), None);
    assert_eq!(result, None);
}

#[test]
fn apply_first_result_both_ok() {
    let result = apply_first::<ResultKind<TestError>, i32, &str>(Ok(7), Ok("right"));
    assert_eq!(result, Ok(7));
}

#[test]
fn apply_first_result_err() {
    let result =
        apply_first::<ResultKind<TestError>, i32, &str>(Err("oops".to_string()), Ok("right"));
    assert!(result.is_err());
}

// ── apply_second ───────────────────────────────────────────────────────────

#[test]
fn apply_second_option_both_some() {
    let result = apply_second::<OptionKind, &str, i32>(Some("ignored"), Some(99));
    assert_eq!(result, Some(99));
}

#[test]
fn apply_second_option_first_none() {
    let result = apply_second::<OptionKind, &str, i32>(None, Some(99));
    assert_eq!(result, None);
}

#[test]
fn apply_second_option_second_none() {
    let result = apply_second::<OptionKind, &str, i32>(Some("ignored"), None);
    assert_eq!(result, None);
}

#[test]
fn apply_second_result_both_ok() {
    let result = apply_second::<ResultKind<TestError>, &str, i32>(Ok("left"), Ok(55));
    assert_eq!(result, Ok(55));
}

// ── CFn composition operators (>> and <<) ─────────────────────────────────

#[test]
fn cfn_forward_compose_shr() {
    // f: i32 -> String, g: String -> usize
    // f >> g  should be  i32 -> usize
    let f = RcFn::new(|x: i32| x.to_string());
    let g = RcFn::new(|s: String| s.len());
    let fg = f >> g;
    assert_eq!(fg.call(42), 2); // "42".len() == 2
    assert_eq!(fg.call(100), 3); // "100".len() == 3
}

#[test]
fn cfn_backward_compose_shl() {
    // g: String -> usize, f: i32 -> String
    // g << f  should be  i32 -> usize
    let f = RcFn::new(|x: i32| x.to_string());
    let g = RcFn::new(|s: String| s.len());
    let fg = g << f;
    assert_eq!(fg.call(42), 2);
    assert_eq!(fg.call(1000), 4); // "1000".len() == 4
}

#[test]
fn cfn_compose_identity_left() {
    // id >> f == f
    let f = RcFn::new(|x: i32| x * 2);
    let id = RcFn::new(|x: i32| x);
    let composed = id >> f;
    assert_eq!(composed.call(5), 10);
}

#[test]
fn cfn_compose_identity_right() {
    // f >> id == f
    let f = RcFn::new(|x: i32| x * 2);
    let id = RcFn::new(|x: i32| x);
    let composed = f >> id;
    assert_eq!(composed.call(5), 10);
}

#[test]
fn cfn_compose_three_functions() {
    let f = RcFn::new(|x: i32| x + 1);
    let g = RcFn::new(|x: i32| x * 2);
    let h = RcFn::new(|x: i32| x.to_string());
    // (f >> g) >> h
    let fgh = (f >> g) >> h;
    // (3 + 1) * 2 = 8 → "8"
    assert_eq!(fgh.call(3), "8");
}
