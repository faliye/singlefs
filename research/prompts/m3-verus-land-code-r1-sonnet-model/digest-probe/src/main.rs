// 草稿探针：用 crash_identity.rs 里非测试的 judging_code_of_the_judges 在几份仓树上算判法摘要。不是 checker 档的测试。
mod layer0_progress {
    pub struct Layer0ToolchainIdentity {
        pub rustc_version_lines: String,
        pub cargo_version: String,
        pub target_triple: String,
    }
}
#[allow(dead_code)]
#[path = "../../tree/crates/singlefs-checker-tier/src/crash_identity.rs"]
mod crash_identity;

use layer0_progress::Layer0ToolchainIdentity;

fn hex(d: &[u8]) -> String { d.iter().map(|b| format!("{b:02x}")).collect() }

fn main() {
    let roots: Vec<String> = std::env::args().skip(1).collect();
    let toolchain = Layer0ToolchainIdentity { rustc_version_lines: "probe-fixed".into(), cargo_version: "probe-fixed".into(), target_triple: "x86_64".into() };
    for root in roots {
        let code = crash_identity::judging_code_of_the_judges(std::path::Path::new(&root), &toolchain).expect("摘要");
        println!("ROOT {root}\n digest {}\n files {}", hex(&code.digest), code.files.len());
        for f in &code.files {
            if f.contains("replay-prefix") || f.contains("recovery.rs") { println!("  FILE {f}  reason={}", code.reason_of_each_file.get(f).cloned().unwrap_or_default()); }
        }
        println!(" stopped_at {:?}", code.stopped_at);
    }
}
