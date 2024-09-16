fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=templates");

    // dotenv().ok();

    // let sefaz_mode: String = env::var("SEFAZ_MODE")
    //     .unwrap_or_else(|_| "unset".to_string())
    //     .to_lowercase();
    // if sefaz_mode == "homologation" {
    //     println!("cargo:rustc-cfg=sefaz_service_homologation");
    // }
}
