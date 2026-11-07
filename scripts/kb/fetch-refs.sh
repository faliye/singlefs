#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 带 --check 只查本机已有的几份、不用 curl；下载时缺 curl 在那一步逐份判红
# 把被当作证据引用的外部文献重新固定到本机，并留下取得方式与 sha256。
#
#   bash scripts/kb/fetch-refs.sh [--check]
#
# 为什么要有它：checks-owed.md C38（外部文献不可复核）——一批标着「本机 PDF 逐字核实」的文献
# 今天已不在本机，引用它们的结论却仍标着「已核实」。文献没了，那些引用就只是线索。
# --check 只查在不在、hash 对不对，不下载。
# 文献放哪：FS_REFS 设了照它（下面的 docs/），没设取共享数据根下的 fs-refs（.claude/hooks/lib_work_root.py shared fs-refs，两台同一个绝对路径；
#   不经 ~ 与 $HOME 取：第二台的家目录是另一个；scripts/kb/verify-citations.sh 与 scripts/kb/pdf-text.py 从同一处读），取不到退 2。
# 管的只有下面 LIST 里的七份 PDF（写进 docs/、每份旁边记 .sha256）。同一个根底下 verify-citations.sh 读的另几样不归它：
#   linux-6.17/ 与 zfs/ 两棵源码树、docs/rfc8439.txt 都不是它取的，它也不核；它们各自的取得方式、版本与校验边界写在 verify-citations.sh 的树登记表 TREE_LIST。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
references_root="${FS_REFS:-}"
if [[ -z "$references_root" ]]; then
  if ! references_root="$(python3 "$(dirname "${BASH_SOURCE[0]}")/../../.claude/hooks/lib_work_root.py" shared fs-refs)" || [[ "$references_root" != /* ]]; then
    echo '  ✗ 取不到共享数据根下的 fs-refs（.claude/hooks/lib_work_root.py shared fs-refs 没打出绝对路径，原因在上一行），一份都没查' >&2
    echo '  → 怎么办：照上一行的出路配好数据根（多机配置的 SHARED_DATA_ROOT，或 SINGLEFS_DATA_ROOT），或设 FS_REFS 指到文献树的上一层' >&2
    exit 2
  fi
fi
DEST="$references_root/docs"
if (( $# > 1 )) || [[ $# == 1 && "$1" != --check ]]; then
  echo '  ✗ 用法：fetch-refs.sh [--check]' >&2
  echo '     → 怎么办：不带参数是下载并固定副本，--check 只核已有副本的 sha256' >&2
  exit 2
fi
mode="${1:-fetch}"
if [[ "$mode" != --check ]]; then
  mkdir -p "$DEST" || exit 1
fi
LOG="$DEST/../fetch-docs.log"
partial=""
trap '[[ -z "$partial" ]] || rm -f -- "$partial" "$partial.sha256"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# 文件名 | URL | 引用它的决策/实验
#
# ⚠️ 不在这张表里的一份：**ZFS On-Disk Specification**。下得回来，但 pdf-text.py 抽出的是乱码
# （CID 双字节字体，抽取器只支持单字节），按它自己的规矩这算**抽取失败**，不算「原文没这句」。
# 引它的那几条（4 个 label × 128 KiB uberblock 环、槽号 = txg %% 槽数）改从 OpenZFS 源码树（verify-citations.sh 的 refs-zfs，
# 不是本脚本取的，取得方式与校验边界见那边的树登记表）核，断言在 verify-citations.sh 的 D20 三条。⇒ 那份 PDF 对本仓没有承重作用，不收。
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

ok=0; miss=0
while IFS='|' read -r name url who; do
  [[ -z "$name" ]] && continue
  path="$DEST/$name"
  if [[ "$mode" != "--check" && ( ! -s "$path" || ! -s "$path.sha256" ) ]]; then
    partial="$(mktemp "$DEST/.fetch.XXXXXX")" || exit 1
    if ! curl -fsSL --max-time 120 -o "$partial" "$url"; then
      echo "  ✗ $name  下载失败：$url" >&2; rm -f "$partial"; miss=$((miss+1)); continue  # gate-lint:detail
    fi
    # 下到一页 HTML 错误页也会是 200，所以认文件类型，不认退出码
    # 末段不许是 `grep -q`（门禁阶段 41）：先落到变量再判。
    ftype=$(file -b "$partial" || true)
    if ! grep -qi pdf <<<"$ftype"; then
      echo "  ✗ $name  下回来的不是 PDF（$ftype）：$url" >&2  # gate-lint:detail
      rm -f "$partial"; miss=$((miss+1)); continue
    fi
    checksum="$(sha256sum "$partial")" || exit 1
    checksum="${checksum%% *}"
    # PDF 丢失时仍须遵守已有摘要，不能用远端的新版本重置旧证据。
    if [[ -e "$path.sha256" ]]; then
      expected="$(cat -- "$path.sha256" 2>/dev/null)" || expected=""
      if [[ ! "$expected" =~ ^[0-9a-f]{64}$ || "$checksum" != "$expected" ]]; then
        echo "  ✗ $name  下载内容与已有 sha256 记录不符或记录损坏；保留原记录，未替换副本" >&2  # gate-lint:detail
        rm -f -- "$partial"; miss=$((miss+1)); continue
      fi
    fi
    # 旧缓存没有摘要时，远端内容可能已经变了；不能用新版本覆盖唯一的旧证据。
    if [[ -s "$path" ]] && ! cmp -s -- "$path" "$partial"; then
      echo "  ✗ $name  旧副本没有校验记录，且与这次下载不同；旧副本保留，未建立新记录" >&2
      echo '    → 先核实版本变化并保存旧副本，再移开目标文件重新下载；不能把新版本当成原有证据' >&2
      rm -f -- "$partial"; miss=$((miss+1)); continue
    fi
    printf '%s\n' "$checksum" >"$partial.sha256" || exit 1
    mv -- "$partial" "$path" && mv -- "$partial.sha256" "$path.sha256" || exit 1
    partial=""
    printf '%s\t%s\t%s\t%s\n' "$(date -Is)" "$name" "$url" "$checksum" >>"$LOG" || exit 1
  fi
  if [[ -s "$path" ]]; then
    expected="$(cat -- "$path.sha256" 2>/dev/null)"
    actual="$(sha256sum "$path")" || actual=""
    actual="${actual%% *}"
    if [[ ! "$expected" =~ ^[0-9a-f]{64}$ || "$actual" != "$expected" ]]; then
      echo "  ✗ $name  sha256 记录缺失或与取得时不同，不能确认固定副本" >&2
      echo '    → 没有记录的重新运行下载；校验不符的先保存旧副本，查明原因后再重新下载' >&2
      miss=$((miss+1)); continue
    fi
    printf '  ✓ %-38s %8s 字节  sha256=%s  引用方 %s\n' "$name" "$(stat -c%s "$path")" "$actual" "$who"
    ok=$((ok+1))
  else
    printf '  ✗ %-38s 本机没有，引用方 %s ⇒ 那些引用只能当线索\n' "$name" "$who"; miss=$((miss+1))  # gate-lint:detail
  fi
done <<<"$LIST"

echo "在本机 $ok 份／缺 $miss 份  目录 $DEST"
if [[ $miss -ne 0 ]]; then
  echo "  → 怎么办：对着上面逐条列出的 ✗，下载失败的重跑一遍（网络问题多试几次）；" \
       "下回来不是 PDF 的，去本脚本 LIST 里那一行对应的 URL 手动看一眼是不是页面搬家了，换成新地址；" \
       "本机没有的那几份先跑 'bash scripts/kb/fetch-refs.sh'（不带 --check）去下载。"
  exit 1
fi
echo "  ✓ 承重文献在本机 $ok 份，一份不缺"
