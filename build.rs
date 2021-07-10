fn main() {
    if cfg!(any(target_os = "freebsd", target_os = "openbsd")) {
        cc::Build::new()
            .file("src/server/recorder/proc_name.c")
            .compile("proc_name");
        if cfg!(target_os = "freebsd") {
            println!(r"cargo:rustc-link-search=/usr/local/lib");
        } else if cfg!(target_os = "openbsd") {
            println!(r"cargo:rustc-link-search=/usr/X11R6/lib"); 
        }
    }
}
