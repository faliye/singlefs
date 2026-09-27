#!/usr/bin/env bash
# N2 ③：同一份源码、同一个 `cargo -V && rustc -V`（准入模块 toolchain_line 算指纹的两行），编跑同一个测试二进制，看行为随哪些输入变。
# 用法：CFGPROBE_DIR=<草稿里的目录，下面有 repo3/ 与三个包装脚本> bash build-env-scenarios.sh
# 每个场景打一行 SCENARIO：工具链两行的 sha256 前 16 位、这一趟 cargo 编没编（Compiling 行数）、测试二进制打的 OPUS_R3_ENV 行。
set -u
S="${CFGPROBE_DIR:?设 CFGPROBE_DIR}"
cd "$S/repo3"
run() { # <场景名> [env 赋值…]
  local name="$1"; shift
  local tool; tool="$(env "$@" sh -c 'cargo -V && rustc -V' | tr '\n' ';')"
  local line; line="$(env "$@" cargo test --release -p singlefs-format --test opus_r3_build_env_probe -- --nocapture 2>"$S/$name.stderr" | grep '^OPUS_R3_ENV')"
  echo "SCENARIO $name toolchain=[$tool] toolchain_sha256=$(printf '%s' "$tool" | sha256sum | cut -c1-16) compiled=$(grep -c Compiling "$S/$name.stderr") $line"
}
run S0_baseline
# 仓根之上一层的 .cargo/config.toml（cargo 从工作目录往上逐层读；54 号 --full 的 worktree 在 mktemp -d 之下，往上是 /tmp、/）
mkdir -p "$S/.cargo"
printf '[profile.release]\noverflow-checks = false\n\n[env]\nOPUS_R3_BUILD_ENV = "from-ancestor-config"\nOPUS_R3_RUN_ENV = "from-ancestor-config"\n' > "$S/.cargo/config.toml"
run S1_ancestor_dot_cargo_config
rm "$S/.cargo/config.toml"; rmdir "$S/.cargo"
mkdir -p "$S/cargohome"
printf '[build]\nrustflags = ["--cfg", "opus_probe"]\n' > "$S/cargohome/config.toml"
run S2_CARGO_HOME_config CARGO_HOME="$S/cargohome"
run S3_CARGO_TARGET_triple_RUSTFLAGS CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="--cfg opus_probe"
run S6_CARGO_TARGET_triple_RUNNER CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="$S/runner.sh"
run S7_RUSTC_WORKSPACE_WRAPPER RUSTC_WORKSPACE_WRAPPER="$S/rustc-wrapper.sh"
# RUSTC_WRAPPER 与 RUSTC：旧 target/ 上 cargo 不重编；给空的 target 目录（--full 的新 worktree 里 target/ 本来就是空的）
rm -rf "$S/target-s0" "$S/target-s4" "$S/target-s5"
run S0b_baseline_fresh_target CARGO_TARGET_DIR="$S/target-s0"
run S4b_RUSTC_WRAPPER_fresh_target RUSTC_WRAPPER="$S/rustc-wrapper.sh" CARGO_TARGET_DIR="$S/target-s4"
run S5b_RUSTC_fresh_target RUSTC="$S/rustc-as-RUSTC.sh" CARGO_TARGET_DIR="$S/target-s5"
run S4c_RUSTC_WRAPPER_old_target RUSTC_WRAPPER="$S/rustc-wrapper.sh"
run S8_back_to_baseline
