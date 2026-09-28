#!/usr/bin/env bash
# gate-stage: 样本：排在最后，只在注释与出路文字里提到 cargo test，不算跑测试、不该被点名
# 要看测试：cargo test --release -p sample-crate
ROOT="${1:-.}"
cd "$ROOT" || exit 2
echo "  → 怎么办：单跑 cargo test --release -p sample-crate 看它报什么"
exit 0
