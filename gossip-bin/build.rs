use std::process::Command;

//mod graph;
//pub use graph::{info_graph::InfoGraph, info_edge::InfoEdge};
//mod triple;
//pub use triple::{info_table::InfoTable, info_triple::InfoTriple};
//
////mod builder;
////pub use builder::{buffers::Buffers};
//
//mod builder;
//pub use crate::builder::buffers_facade;
//pub use crate::builder::triple_facade;
//pub use crate::builder::*;
//
//pub use crate::graph::descriptor::Descriptor;

fn main() {
    // link to bundled libraries
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
