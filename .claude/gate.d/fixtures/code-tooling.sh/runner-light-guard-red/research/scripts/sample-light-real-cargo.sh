#!/usr/bin/env bash
# 样本：登记成 light 的自证，却经 PATH 起 cargo build（清单不在：守卫没拦住时真 cargo 也当场报错退出、不编任何东西），自己照常退 0
[[ "${1:-}" == --selftest ]] || exit 2
cargo build --manifest-path "$PWD/no-such-crate/Cargo.toml" >/dev/null 2>&1
echo "  ✓ 登记成 light 的样本自证 1 条都判对"
exit 0
