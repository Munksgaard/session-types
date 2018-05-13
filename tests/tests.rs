#[cfg(feature = "chan_select")]
extern crate compiletest_rs as compiletest;

#[cfg(feature = "chan_select")]
use std::path::PathBuf;

#[cfg(feature = "chan_select")]
fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.target_rustcflags = Some("-L target/debug".to_string());

    compiletest::run_tests(&config);
}

#[cfg(feature = "chan_select")]
#[test]
fn compile_test() {
    run_mode("compile-fail");
}
