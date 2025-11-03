use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=views/*");
    println!("cargo:rerun-if-changed=wwwroot/*");

    let profile = env::var("PROFILE").unwrap();

    copy_to_output("src/views", &profile).expect("Could not copy resources");
    copy_to_output("src/wwwroot", &profile).expect("Could not copy resources");
}
