#!/usr/bin/env bash
# 样本现场（情形 literal-global-key）：暂存区与别的 library-version-is-staged-tree 样本相同；库目录里的那一版不经 prepare.py、不调驱动的任何函数，
# tree.source、manifest.tsv、toolchain.txt、global-key.txt 与版本指纹都照写死的字面值写（十六进制在驱动与 research/scripts/admission.py 之外现算：
# 照它们文件头写的口径用 sha256 逐行算，sha256 的输入就是下面 printf 写的内容；全局键一行一行的写法见驱动文件头「指纹」）。
# 驱动的 global_key_lines（丢一类行、换次序、换写法）、fingerprint_of_manifest（不算路径或内容）、build_environment_of、version_key 改了，这一份就红；
# admission.py 的 build_environment_lines 逐类写错也红：构建环境里放齐了这几类——从库目录的 tree/ 往上每一层的 .cargo/config.toml（tree/、库目录那一层、样本根）、
# 库目录那一层的 rust-toolchain.toml、tree/.cargo/config.toml 里 build.rustc-wrapper 指的程序、CARGO_HOME 下的 config.toml，编译变量 CARGO_ENCODED_RUSTFLAGS、
# CARGO_PROFILE_DEV_OVERFLOW_CHECKS、CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER（连同它指的程序）、RUSTC_WRAPPER（连同它指的程序）、RUSTFLAGS、RUSTC（值与它的 -V），
# 文档测试的 RUSTDOCFLAGS。全局键的构建环境按 .broad-sample/environment 算（PATH 前面是样本的 .broad-sample/bin，那四个程序在那里），不读跑样本那个会话的环境；
# 样本拷进临时目录跑，上层目录里不放 .cargo/config 与 rust-toolchain。
set -euo pipefail
mkdir -p crates/sample/src litmus .cargo .claude/gate.d .broad-sample/cargo-home .broad-sample/bin .broad-sample/home/tree/.cargo .broad-sample/home/.cargo
printf '[workspace]\nmembers = ["crates/sample"]\nresolver = "2"\n' > Cargo.toml
printf '# 样本的锁文件\nversion = 4\n' > Cargo.lock
printf '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n' > crates/sample/Cargo.toml
printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left + right\n}\n' > crates/sample/src/lib.rs
printf 'C sample\n{}\nP0(int *x) { WRITE_ONCE(*x, 1); }\nexists (x=1)\n' > litmus/sample.litmus
printf '[build]\njobs = 1\n' > .cargo/config.toml
printf 'checker-tier-crates-mutation-replay.sh\tcrates/ Cargo.toml Cargo.lock litmus/\t# 样本：广谱变异的导出范围取这一行\n' > .claude/gate.d/stage-inputs.tsv
printf 'rustc 1.0.0-sample (0000000 样本)\nhost: x86_64-unknown-linux-gnu\n' > .broad-sample/rustc-vv.txt
printf 'cargo-nextest 0.9.0-sample\ncargo-mutants 25.0.0-sample\nLLVM version 19.0.0-sample\n' > .broad-sample/tool-versions.txt
printf '# 样本的忽略用例排除表（一行都没登记）\n' > .broad-sample/exclusions.tsv
printf '[profile.dev]\noverflow-checks = false\n' > .broad-sample/cargo-home/config.toml
printf '[build]\nrustc-wrapper = "sample-config-wrapper"\n' > .broad-sample/home/tree/.cargo/config.toml
printf '[profile.dev]\ndebug = 0\n' > .broad-sample/home/.cargo/config.toml
printf '[toolchain]\nchannel = "1.0.0-sample"\n' > .broad-sample/home/rust-toolchain.toml
printf '#!/bin/sh\n# 样本：cargo 配置里 build.rustc-wrapper 指的程序\nexec "$@"\n' > .broad-sample/bin/sample-config-wrapper
printf '#!/bin/sh\n# 样本：RUSTC_WRAPPER 指的程序\nexec "$@"\n' > .broad-sample/bin/sample-wrapper
printf '#!/bin/sh\n# 样本：CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER 指的程序\nexec "$@"\n' > .broad-sample/bin/sample-runner
printf '#!/bin/sh\n# 样本：RUSTC 指的编译器，只答 -V\necho "rustc 1.0.0-sample-compiler (样本的编译器)"\n' > .broad-sample/bin/sample-rustc
chmod 755 .broad-sample/bin/sample-config-wrapper .broad-sample/bin/sample-wrapper .broad-sample/bin/sample-runner .broad-sample/bin/sample-rustc
cat > .broad-sample/environment <<'ENVIRONMENT'
CARGO_HOME={root}/.broad-sample/cargo-home
HOME={root}/.broad-sample
PATH={root}/.broad-sample/bin:/usr/bin:/bin
RUSTFLAGS=-C sample-literal
RUSTDOCFLAGS=--cfg sample
CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false
CARGO_ENCODED_RUSTFLAGS=-Coverflow-checks=off
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=sample-runner
RUSTC_WRAPPER=sample-wrapper
RUSTC=sample-rustc
ENVIRONMENT
version=54cb89f92c3f87f55e736923c14005fc5c8a254eb6e2fe1094a1ce240770af52
mkdir -p ".broad-sample/home/versions/$version"
cat > ".broad-sample/home/versions/$version/manifest.tsv" <<'MANIFEST'
641596c9b874de076e06ae9142a5befa2c157204597bbc07ff02c3071ac41a86	.cargo/config.toml
f730865d5114f5012e0dcd7b7de541320318b672fc6b96c4d5b540b58dd162e1	Cargo.lock
217bce2894842630d986b1927b4e549ad0032e13af18c555d80c0b3754a1db80	Cargo.toml
7fbea37912385b30bfde397c2556445d5759e104f804111ef949aaee202685df	crates/sample/Cargo.toml
94270aa3000bd9c667c5c5f5a35d3563845e1bc210ad20580a5f209bb92bd47c	crates/sample/src/lib.rs
083f9d2c8de413b6943cfde4885e13c01a7fa8c0b5c89f68752ff1f67c93f666	litmus/sample.litmus
MANIFEST
printf 'rustc 1.0.0-sample (0000000 样本)\nhost: x86_64-unknown-linux-gnu\n' > ".broad-sample/home/versions/$version/toolchain.txt"
cat > ".broad-sample/home/versions/$version/global-key.txt" <<'GLOBALKEY'
rules prune-r3-e1-e6-fresh-build-mac
toolchain rustc 1.0.0-sample (0000000 样本) | host: x86_64-unknown-linux-gnu
file .cargo/config.toml 641596c9b874de076e06ae9142a5befa2c157204597bbc07ff02c3071ac41a86
file Cargo.lock f730865d5114f5012e0dcd7b7de541320318b672fc6b96c4d5b540b58dd162e1
file Cargo.toml 217bce2894842630d986b1927b4e549ad0032e13af18c555d80c0b3754a1db80
file crates/sample/Cargo.toml 7fbea37912385b30bfde397c2556445d5759e104f804111ef949aaee202685df
build 58a1b295a2a6b57c12ceede2c292fae634ab3e31062375524a15265dc63851f9 <构建环境：目录层级里的 .cargo/config.toml>
build d8f7085d44ab3404bc6e43211c5dec8b5522634acd44e6d234ca9d4944e30894 <构建环境：目录层级里的 .cargo/config.toml 里 build.rustc-wrapper 指的程序>
build f9c467c1c65aa42f01eb115f573b4272a0834c81f8bbb084ab6e7a4d97b94d59 <构建环境：目录层级里的 .cargo/config.toml>
build dbc7de1b59bca420b488f11a2d16e7ed128c1a3bd7a2c89414203ed9bc3e2d9d <构建环境：目录层级里的 rust-toolchain.toml>
build 641596c9b874de076e06ae9142a5befa2c157204597bbc07ff02c3071ac41a86 <构建环境：目录层级里的 .cargo/config.toml>
build 5b0bcd10f1c0b69ebd6c2373caec83b88dfe232120f690e3b40af5b34b48d99f <构建环境：CARGO_HOME 下的 config.toml>
build 1dd9b8949b528d9ada724e77297a3249af9eb6222b22aff6a234bad072d9cc8d <构建环境：环境变量 CARGO_ENCODED_RUSTFLAGS>
build fcbcf165908dd18a9e49f7ff27810176db8e9f63b4352213741664245224f8aa <构建环境：环境变量 CARGO_PROFILE_DEV_OVERFLOW_CHECKS>
build d4639d7df3b2c5248436d0de900151633e48d18cf2f79598a5e79852e8dd46e8 <构建环境：环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER>
build befd2896aaab9f3ea248364f56a685075b7785d29e69dff19c334584449aaed6 <构建环境：环境变量 RUSTC_WRAPPER>
build b748c8187184ddbca586b62cafa999f158a9cf0a64ac0b8011e4ef9de43bf294 <构建环境：环境变量 RUSTFLAGS>
build 220efbc5da10f29d42283d4bd6aae5f1f5f5ff126ed9f219b07fcf81e9321ec8 <构建环境：环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER 指的程序>
build 3721a72b6518c297a97cae975aa0858375805f446b598cadfbd95eb381de2e1c <构建环境：环境变量 RUSTC_WRAPPER 指的程序>
build 7abea85936553c214b756d536589fda455e69b644efa4f035fb503971c4415d2 <构建环境：RUSTC 的值与它的 -V>
build 9882cb5697c916cceec67e8a668d3bb06cc6b57adbe620a9a3548e5952e4ccdc <文档测试：环境变量 RUSTDOCFLAGS>
tool cargo-nextest 0.9.0-sample
tool cargo-mutants 25.0.0-sample
tool LLVM version 19.0.0-sample
exclusions c42149c4713b8628fdce3f8ed49768dc9b33d4819e4aa134f9be42fb2aaf7ec7
limits cpu=3.0x+30.0 wall=10.0x+300.0
escalation time=4x,20xbaseline memory=1.5x,2xbaseline
GLOBALKEY
cat > .broad-sample/home/tree.source <<'POINTER'
{"commit": "0123456789abcdef0123456789abcdef01234567", "revision": "HEAD", "fingerprint": "54cb89f92c3f87f55e736923c14005fc5c8a254eb6e2fe1094a1ce240770af52", "source_fingerprint": "6a5df13f755b3e9357e2bc072aeaf3e08086f3687779fe3df5855b2b2beda865", "global_key": "1ae540837dacf0cb9b156af7d21908e17b1569ba66778954f060e13cf8bee174", "files": 6, "export_paths": ["crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo"]}
POINTER
git init -q .
git add -A
