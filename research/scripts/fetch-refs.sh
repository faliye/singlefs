#!/usr/bin/env bash
# 把被当作证据引用的外部文献重新固定到本机，并留下取得方式与 sha256。
#
#   bash research/scripts/fetch-refs.sh [--check]
#
# 为什么要有它：checks-owed.md C38（外部文献不可复核）——一批标着「本机 PDF 逐字核实」的文献
# 今天已不在本机，引用它们的结论却仍标着「已核实」。文献没了，那些引用就只是线索。
# --check 只查在不在、hash 对不对，不下载。
set -uo pipefail
DEST="${FS_REFS:-/home/fy5090/code/fs-refs}/docs"
mkdir -p "$DEST"
LOG="$DEST/../fetch-docs.log"

# 文件名 | URL | 引用它的决策/实验
#
# ⚠️ 不在这张表里的一份：**ZFS On-Disk Specification**。下得回来，但 pdf-text.py 抽出的是乱码
# （CID 双字节字体，抽取器只支持单字节），按它自己的规矩这算**抽取失败**，不算「原文没这句」。
# 引它的那几条（4 个 label × 128 KiB uberblock 环、槽号 = txg %% 槽数）改从已固定的 OpenZFS
# 源码树核，断言在 verify-citations.sh 的 D20 三条。⇒ 那份 PDF 对本仓没有承重作用，不收。
LIST=$(cat <<'TSV'
ostep-45-file-integrity.pdf|https://pages.cs.wisc.edu/~remzi/OSTEP/file-integrity.pdf|D18
naclcrypto.pdf|https://cr.yp.to/highspeed/naclcrypto-20090310.pdf|D9
nist-sp800-38b.pdf|https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-38B.pdf|D9
nist-sp800-38d.pdf|https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf|D9
betrfs-fast17-senescence.pdf|https://www.usenix.org/system/files/conference/fast17/fast17-conway.pdf|D10,E10
betrfs-fast18-fullpath.pdf|https://www.usenix.org/system/files/conference/fast18/fast18-zhan.pdf|D8,D10,D11,E7
netapp-fast20-ssd-reliability.pdf|https://www.usenix.org/system/files/fast20-maneas.pdf|D18
TSV
)

mode="${1:-fetch}"
ok=0; miss=0
while IFS='|' read -r name url who; do
  [[ -z "$name" ]] && continue
  path="$DEST/$name"
  if [[ "$mode" != "--check" && ! -s "$path" ]]; then
    if ! curl -sSL --max-time 120 -o "$path.part" "$url"; then
      echo "  ✗ $name  下载失败：$url" >&2; rm -f "$path.part"; miss=$((miss+1)); continue
    fi
    # 下到一页 HTML 错误页也会是 200，所以认文件类型，不认退出码
    if ! file -b "$path.part" | grep -qi pdf; then
      echo "  ✗ $name  下回来的不是 PDF（$(file -b "$path.part" | cut -c1-40)）：$url" >&2
      rm -f "$path.part"; miss=$((miss+1)); continue
    fi
    mv "$path.part" "$path"
    printf '%s\t%s\t%s\n' "$(date -Is)" "$name" "$url" >>"$LOG"
  fi
  if [[ -s "$path" ]]; then
    printf '  ✓ %-38s %8s 字节  sha256=%s  引用方 %s\n' "$name" "$(stat -c%s "$path")" "$(sha256sum "$path" | cut -c1-16)" "$who"
    ok=$((ok+1))
  else
    printf '  ✗ %-38s 本机没有，引用方 %s ⇒ 那些引用只能当线索\n' "$name" "$who"; miss=$((miss+1))
  fi
done <<<"$LIST"

echo "在本机 $ok 份／缺 $miss 份  目录 $DEST"
[[ $miss -eq 0 ]] || exit 1
