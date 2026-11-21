#!/usr/bin/env bash
# 正推腿的模型：造一个只有本地包、没有外部依赖的小工作区，十几个消费包各用一种写法去读 provider 包里的文件，
# 供 run-dep-info-model.sh 用真 cargo、真 rustc 编一遍，看 rustc 写的 dep-info 里有没有列出 provider 的文件。
# 用法：make-workspace.sh <工作区目录>（目录不能已存在；只写这个目录，不碰仓）
set -euo pipefail
(($# == 1)) || { echo "用法：make-workspace.sh <工作区目录>" >&2; exit 2; }
WS="$1"
mkdir "$WS"
WS="$(cd "$WS" && pwd)"

write_package() {   # write_package <包名>：Cargo.toml、空的 src/lib.rs
  local name="$1" dir="$WS/crates/$1"
  mkdir -p "$dir/src" "$dir/tests"
  cat > "$dir/Cargo.toml" <<MANIFEST
[package]
name = "$name"
version = "0.0.0"
edition = "2021"
publish = false
MANIFEST
  : > "$dir/src/lib.rs"
}

cat > "$WS/Cargo.toml" <<'ROOT'
[workspace]
resolver = "2"
members = [
  "crates/provider",
  "crates/c00", "crates/c01", "crates/c02", "crates/c03", "crates/c04", "crates/c05", "crates/c06", "crates/c07",
  "crates/c08", "crates/c09", "crates/c10", "crates/c11", "crates/c12", "crates/c13", "crates/c14", "crates/c15",
]
ROOT

# provider：被读的包。源码、着色器、中转用的 .rs、数据文件各一份
write_package provider
mkdir -p "$WS/crates/provider/tests/c"
echo 'pub fn provided() {}' > "$WS/crates/provider/src/lib.rs"
echo '// shader text' > "$WS/crates/provider/src/k.wgsl"
echo 'pub fn included() {}' > "$WS/crates/provider/src/inc.rs"
echo 'pub const C: u32 = 1;' > "$WS/crates/provider/tests/c/mod.rs"
echo '{}' > "$WS/crates/provider/data.json"
echo 'notes' > "$WS/crates/provider/notes.md"

# c00：对照，同包里的 #[path]，provider 不该出现在它的 dep-info 里
write_package c00
mkdir -p "$WS/crates/c00/tests/common"
echo 'pub const C: u32 = 1;' > "$WS/crates/c00/tests/common/mod.rs"
cat > "$WS/crates/c00/tests/cross.rs" <<'RS'
#[path = "common/mod.rs"]
mod c;
#[test]
fn same_package() { assert_eq!(c::C, 1); }
RS

# c01：#[path] 平写
write_package c01
cat > "$WS/crates/c01/tests/cross.rs" <<'RS'
#[path = "../../provider/tests/c/mod.rs"]
mod c;
#[test]
fn plain_path() { assert_eq!(c::C, 1); }
RS

# c02：cfg_attr 里的 path
write_package c02
cat > "$WS/crates/c02/tests/cross.rs" <<'RS'
#[cfg_attr(all(), path = "../../provider/tests/c/mod.rs")]
mod c;
#[test]
fn cfg_attr_path() { assert_eq!(c::C, 1); }
RS

# c03：改名导入的 include_str
write_package c03
cat > "$WS/crates/c03/tests/cross.rs" <<'RS'
use std::include_str as grab;
const SHADER: &str = grab!("../../provider/src/k.wgsl");
#[test]
fn renamed_include_macro() { assert!(!SHADER.is_empty()); }
RS

# c04：方括号定界的 include_str
write_package c04
cat > "$WS/crates/c04/tests/cross.rs" <<'RS'
const SHADER: &str = include_str!["../../provider/src/k.wgsl"];
#[test]
fn bracket_delimiters() { assert!(!SHADER.is_empty()); }
RS

# c05：原始字符串写的 path
write_package c05
cat > "$WS/crates/c05/tests/cross.rs" <<'RS'
#[path = r"../../provider/tests/c/mod.rs"]
mod c;
#[test]
fn raw_string_path() { assert_eq!(c::C, 1); }
RS

# c06：行内模块里的 #[path]，从 tests/common_shared/ 起算（目录要真在，.. 才走得通）
write_package c06
mkdir -p "$WS/crates/c06/tests/common_shared"
: > "$WS/crates/c06/tests/common_shared/.keep"
cat > "$WS/crates/c06/tests/cross.rs" <<'RS'
mod common_shared {
    #[path = "../../../provider/tests/c/mod.rs"]
    pub mod c;
}
#[test]
fn inline_module_base() { assert_eq!(common_shared::c::C, 1); }
RS

# c07：绝对路径（运行时写进工作区的真路径）
write_package c07
cat > "$WS/crates/c07/tests/cross.rs" <<RS
#[path = "$WS/crates/provider/tests/c/mod.rs"]
mod c;
#[test]
fn absolute_path() { assert_eq!(c::C, 1); }
RS

# c08：经 .inc 中转，中转文件里再 #[path] 过去
write_package c08
cat > "$WS/crates/c08/tests/cross.rs" <<'RS'
#[path = "shim.inc"]
mod shim;
#[test]
fn non_rs_intermediate() { assert_eq!(shim::c::C, 1); }
RS
cat > "$WS/crates/c08/tests/shim.inc" <<'RS'
#[path = "../../provider/tests/c/mod.rs"]
pub mod c;
RS

# c09：名叫 target 的子目录
write_package c09
mkdir -p "$WS/crates/c09/tests/target"
cat > "$WS/crates/c09/tests/cross.rs" <<'RS'
mod target;
#[test]
fn target_directory() { assert_eq!(target::c::C, 1); }
RS
cat > "$WS/crates/c09/tests/target/mod.rs" <<'RS'
#[path = "../../../provider/tests/c/mod.rs"]
pub mod c;
RS

# c10：点起头的子目录
write_package c10
mkdir -p "$WS/crates/c10/tests/.shared"
cat > "$WS/crates/c10/tests/cross.rs" <<'RS'
#[path = ".shared/mod.rs"]
mod shared;
#[test]
fn hidden_directory() { assert_eq!(shared::c::C, 1); }
RS
cat > "$WS/crates/c10/tests/.shared/mod.rs" <<'RS'
#[path = "../../../provider/tests/c/mod.rs"]
pub mod c;
RS

# c11：读 provider 的数据文件（.md 与 .json）
write_package c11
cat > "$WS/crates/c11/tests/cross.rs" <<'RS'
const NOTES: &str = include_str!("../../provider/notes.md");
const DATA: &[u8] = include_bytes!("../../provider/data.json");
#[test]
fn data_files() { assert!(!NOTES.is_empty() && !DATA.is_empty()); }
RS

# c12：构建脚本单元自己的源码里 #[path] 到 provider
write_package c12
cat > "$WS/crates/c12/build.rs" <<'RS'
#[path = "../provider/tests/c/mod.rs"]
mod c;
fn main() { assert_eq!(c::C, 1); }
RS

# c13：库里 include! 一份 provider 的 .rs
write_package c13
echo 'include!("../../provider/src/inc.rs");' > "$WS/crates/c13/src/lib.rs"

# c14：构建脚本运行时读 provider 的文件并报 rerun-if-changed（不在 dep-info 里，在构建脚本的 output 里）
write_package c14
cat > "$WS/crates/c14/build.rs" <<'RS'
fn main() {
    let text = std::fs::read_to_string("../provider/src/k.wgsl").unwrap();
    assert!(!text.is_empty());
    println!("cargo:rerun-if-changed=../provider/src/k.wgsl");
}
RS

# c15：编不过：读一份不存在的文件
write_package c15
cat > "$WS/crates/c15/tests/cross.rs" <<'RS'
const SHADER: &str = include_str!("../../provider/src/missing.wgsl");
#[test]
fn compile_failure() { assert!(!SHADER.is_empty()); }
RS
echo "工作区造好：$WS"
