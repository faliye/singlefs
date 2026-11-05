#!/usr/bin/env bash
# 建沙箱：cargo 垫片（只记调用，不编译）与样本文件。参数：工作目录
set -eu
W="$1"; S="$W/sandbox"
mkdir -p "$S/bin" "$S/pathdir"
printf '#!/bin/sh\necho "CARGO $*" >> "${SHIM_LOG:-/dev/null}"\n' > "$S/bin/cargo"; chmod +x "$S/bin/cargo"
printf 'cargo test --all\n' > "$S/bashenv.sh"
printf 'cargo test --all\n' > "$S/heavy.sh"
printf 'cargo test --all\n' > "$S/g1.sh"
printf 'cargo test --all\n' > "$S/f.txt"
printf 'echo hi\n' > "$S/benign.sh"
printf '#!/bin/bash\ncargo test --all\n' > "$S/pathdir/evil.sh"; chmod +x "$S/pathdir/evil.sh"
printf '#!/bin/bash\ncargo test --all\n' > "$S/execd.sh"; chmod +x "$S/execd.sh"
printf '#!/bin/sh\ncargo test --all\n' > "$S/execsh.sh"; chmod +x "$S/execsh.sh"
printf 'echo hi\n' > "$S/src_ok.sh"
printf 'cargo test --all\n' > "$S/.bashrc"
