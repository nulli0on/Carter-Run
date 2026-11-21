fn main(){
    if std::env::var_os("CARGO_FEATURE_STEAM").is_some() {
        let root=std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
        let dir=if cfg!(target_os="windows"){root.join("steam/win64")}else{root.join("steam/linux64")};
        println!("cargo:rustc-link-search=native={}",dir.display());
        if cfg!(target_os="windows"){println!("cargo:rustc-link-lib=dylib=steam_api64");}
        else{println!("cargo:rustc-link-lib=dylib=steam_api");}
    }
}
