#!/usr/bin/env bash
# 样本用的桩装置：e165_replay_bounded_enumeration --pinned-cases <表> 的约定照 pinned-red-examples-replay-red 那一份；这一份打完第一行结果就 kill -9 自己（外面把装置停了），
# 阶段判红、✗ 写「装置被信号停了」、这一次的输出不算判定，汇总的出路照包装文件头「退出码」一段办、不指修实现。
[[ "${1:-}" == --pinned-cases && -f "${2:-}" ]] || { echo "桩只认 --pinned-cases <表>" >&2; exit 2; }
grep -v '^#' "$2" | grep . | head -n 1 | cut -f7
kill -9 "$$"
