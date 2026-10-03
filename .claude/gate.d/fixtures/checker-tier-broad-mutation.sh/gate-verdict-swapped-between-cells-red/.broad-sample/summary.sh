#!/usr/bin/env bash
# 样本：--summary 跑的时候别的会话 --sync-tree 换了版本指针（tree.source 换成 ed83d6cc… 那一版），汇总判的是换上的那一版，本身全绿、退 0
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cat > "$here/home/tree.source" <<'EOF'
{"commit": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "revision": "HEAD", "fingerprint": "ed83d6cca1ecaf2b0000000000000000000000000000000000000000000000aa", "source_fingerprint": "34b17d47def01575b28c89a99595ff1ecb4d0206eed3e5add42ea4a1292f7481", "files": 6, "export_paths": ["crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo"]}
EOF
cat <<'EOF'
BROAD_MUTATION_SUMMARY fingerprint=ed83d6cca1ecaf2b commit=0123456789ab total=6 caught=5 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0
  抓到 5、没红 0、无效 1、超时 0、内存撞顶 0、只过了第一段 0、驱动错 0、这一版还没跑 0；清单 6 条
BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green
EOF
exit 0
