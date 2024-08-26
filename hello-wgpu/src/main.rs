use hello_wgpu::run;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    run();
}
