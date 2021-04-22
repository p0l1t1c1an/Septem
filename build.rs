use cc;

fn main() {
    cc::Build::new()
        .file("src/application/proc_name.c")
        .compile("proc_name");
    println!(r"cargo:rustc-link-search=/usr/local/lib");
}
