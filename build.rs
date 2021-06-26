fn main() {
    if cfg!(target_os = "freebsd") {
        cc::Build::new()
            .file("src/server/recorder/proc_name.c")
            .compile("proc_name");
        println!(r"cargo:rustc-link-search=/usr/local/lib");
    }
}
