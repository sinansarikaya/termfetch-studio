fn main() {
    println!("cargo:rerun-if-changed=assets/brand/termfetch.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/brand/termfetch.ico")
            .set("ProductName", "TermFetch Studio")
            .set("FileDescription", "TermFetch Studio")
            .compile()
            .expect("Windows SDK resource compiler is required to embed the TermFetch icon");
    }
}
