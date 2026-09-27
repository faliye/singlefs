#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 每个实验二进制都要有变异表
#
# 判据：`research/e7-index-bench/src/bin/` 下每个 `*.rs` 在 `research/mutations/`
# 下都要有**同名** `.tsv` 变异表，且表里至少一条成形的变异（三段制表符分隔）。
#
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现（C40（变异表没存）——
# 结论留下了而产生结论的装置没留下，正是 2026-08-29 审计实测过的失败形态；
# 2026-09-03 把最后 19 张欠表补齐后，用这条阶段拦住它再欠回去）。
#
# `crates/mutations.tsv`（门禁 59 号的表，六段）的锚点也在这里数：59 号开跑前的预扫查的是同一件事，
# 而 59 号整道要跑几个钟头，改 `crates/` 的实现员不跑它。2026-09-18 第二波修复改掉 4 行锚点、另有 2 行更早就坏了，
# 实现员只核了自己追加的 8 行，到崩溃验证员跑 59 号才整表退出（`records/2026-09-16-subagent拆分提案.md` 第三十二节）。
# 另判两样会让一行变异「跑了却没跑到点名的测试」的写法：点名的测试标了 #[ignore] 而参数里没有 --include-ignored；
# `--` 之后的筛选词一个都筛不到点名的测试（libtest 比的是完整测试名：模块路径由点名的函数在哪个文件、套在哪几层内联 mod 里算出）。
#
# ⚠️ **这条阶段不跑变异**（跑一遍全部表要逐条重编译，量级是小时）。
# 「表今天还会不会红」由每轮改动实验代码时手跑 `research/scripts/mutate.sh` 证明，
# 复跑记录见各实验正文的「口径与复跑」——本阶段只保证装置在、形状对，不冒充跑过。
set -uo pipefail
BIN_DIR=research/e7-index-bench/src/bin
MUT_DIR=research/mutations
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，本阶段跳过"; exit 77; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  stem="$(basename "$src" .rs)"
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$tsv 不存在（被测的是 $src）"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$tsv：成形 $ok_rows 条、坏 $bad_rows 行"); fi
done

# ── 锚点还对得上吗（C327（变异表的锚点腐化没有会红的检查））──
# 成形不等于替换得上：`mutate.sh` 要求每条「原文」在对应源码里**恰好命中一次**，
# 命中 0 次或多次就退出码 3、**后面的条目一条都不跑**，而那张表对上面那几项检查是绿的。
# 2026-09-14 现查有 4 条这样的锚点（e143 两条、e67 一条、e79 一条），而它们所在的实验页
# 都写着「N 条变异全抓」——那句话当时已经复跑不出来了。
# ⚠️ 这里仍然**不跑变异**，只做子串计数，代价是毫秒级。
anchor_report="$(BIN_DIR="$BIN_DIR" MUT_DIR="$MUT_DIR" python3 - <<'PY'
import os, glob, re
bin_dir = os.environ["BIN_DIR"]; mut_dir = os.environ["MUT_DIR"]
bad = []; stray = []; unreadable = []; checked = 0
def stray_escapes(segment_name, segment):
    """mutate.sh 与 59 号只把 \\n 还原成换行，别的反斜杠按字面写进源码（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    src_path = os.path.join(bin_dir, stem + ".rs")
    # 没有同名二进制的表（shell 探针的变异表）不在本检查射程：它们的被测对象不是 .rs；成功行逐个列名
    if not os.path.exists(src_path):
        print("NO_BINARY", tsv, sep="\t")
        continue
    # 读不了（不是 UTF-8、是个目录、没权限）的一份判红、接着判下一份：崩在这里会让整段报告一行不剩
    try:
        src = open(src_path, encoding="utf-8").read()
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        unreadable.append((stem, f"{type(error).__name__}: {error}"))
        continue
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, frm, to = parts
        checked += 1
        # 口径与 mutate.sh 一致：表里的 \n 先还原成换行，再数子串
        hits = src.count(frm.replace("\\n", "\n"))
        if hits != 1:
            bad.append((stem, name, lineno, hits))
        for segment_name, segment in (("原文", frm), ("替换文", to)):
            for found in stray_escapes(segment_name, segment):
                stray.append((stem, name, lineno) + found)
print("CHECKED", checked)
for b in bad:
    print("BAD", *b, sep="\t")
for s in stray:
    print("STRAY", *s, sep="\t")
for u in unreadable:
    print("UNREADABLE", *u, sep="\t")
PY
)"
anchor_exit_code=$?
# ── crates/mutations.tsv 的锚点（六段：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）──
# 口径与 59 号的预扫一致：原文里的 \n 还原成换行，在「文件」那一段指的源码里恰好命中一次；文件不在也算腐化。
crates_report=""
if [[ -f crates/mutations.tsv ]]; then
  crates_report="$(python3 - <<'PY'
import os
import re
checked = 0
first_line_of_change = {}
first_line_of_name = {}
def stray_escapes(segment_name, segment):
    """59 号只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
attributes_by_package = {}
def module_path_of_file(package, path):
    """crates/<包>/src 下一个 .rs 在 libtest 测试名里的模块前缀：src/lib.rs 与 src/bin/<名>.rs 是空，src/a.rs 与 src/a/mod.rs 是 a，src/a/b.rs 是 a::b；tests/ 下的是空。"""
    relative = os.path.relpath(path, os.path.join("crates", package))
    parts = relative.split(os.sep)
    if parts[0] != "src" or parts[1:2] == ["bin"] or parts[1:] == ["lib.rs"]:
        return []
    parts = parts[1:]
    parts[-1] = parts[-1][:-len(".rs")]
    if parts[-1] == "mod":
        parts = parts[:-1]
    return parts
def enclosing_inline_modules(text):
    """(偏移, 那一刻套着的内联 mod 名单) 的有序表：按 `mod 名 {` 与花括号配平；字符串与注释里的花括号不另判。"""
    events, stack, depth = [], [], 0
    for token in re.finditer(r"\bmod\s+(\w+)\s*\{|[{}]", text):
        if token.group(1):
            depth += 1
            stack.append((token.group(1), depth))
        elif token.group(0) == "{":
            depth += 1
        else:
            if stack and stack[-1][1] == depth:
                stack.pop()
            depth -= 1
        events.append((token.end(), [name for name, _ in stack]))
    return events
def test_attributes_of_package(package):
    """包 crates/<包> 的 tests/ 与 src/ 下每个 fn 名 → [(它上面那串属性与注释行, libtest 里的完整测试名)]；每个包只扫一次。"""
    if package not in attributes_by_package:
        table = {}
        for directory_name in ("tests", "src"):
            for root, _directories, files in os.walk(os.path.join("crates", package, directory_name)):
                for file_name in sorted(files):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(root, file_name)
                    try:
                        text = open(path, encoding="utf-8").read()
                    except (OSError, UnicodeDecodeError):
                        continue
                    prefix = module_path_of_file(package, path)
                    events = enclosing_inline_modules(text)
                    for found in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(", text):
                        modules = []
                        for offset, stack in events:
                            if offset > found.start(2):
                                break
                            modules = stack
                        table.setdefault(found.group(2), []).append((found.group(1), "::".join(prefix + modules + [found.group(2)])))
        attributes_by_package[package] = table
    return attributes_by_package[package]
for line_number, line in enumerate(open("crates/mutations.tsv", encoding="utf-8"), 1):
    line = line.rstrip("\n")
    if not line or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) != 6:
        print("CRATES_BAD", line_number, fields[0], "不是六段", sep="\t")
        continue
    checked += 1
    name, path, original = fields[0], fields[1], fields[2].replace("\\n", "\n")
    # 两行的（文件, 原文, 替换文, 点名的测试）逐字相同 ⇒ 表里少了一条真改法，而条数看着没少（2026-09-22 实测：
    # 第 24 行那批八条里第 253 与第 257 行逐字相同，② 格因此没有自己的守卫）。
    # 只按改法认会误拒：同一个改法让两条不同的测试变红是合法的（第 221 与 227 行：一条单测、一条 checker 的 I-8.7 判别力）。
    change = (path, fields[2], fields[3], fields[5])
    if change in first_line_of_change:
        print("CRATES_DUP", line_number, name, f"（文件, 原文, 替换文, 点名的测试）与第 {first_line_of_change[change]} 行逐字相同", sep="\t")
    else:
        first_line_of_change[change] = line_number
    # 变异名相同 ⇒ 按名字分格的报告把两条并成一格，其中一条的结论看不见。
    if name in first_line_of_name:
        print("CRATES_DUP", line_number, name, f"变异名与第 {first_line_of_name[name]} 行相同", sep="\t")
    else:
        first_line_of_name[name] = line_number
    for segment_name, segment in (("原文", fields[2]), ("替换文", fields[3])):
        for found in stray_escapes(segment_name, segment):
            print("CRATES_STRAY", line_number, name, *found, sep="\t")
    try:
        source = open(path, encoding="utf-8").read()
    except FileNotFoundError:
        print("CRATES_BAD", line_number, name, f"文件 {path} 不存在", sep="\t")
        continue
    except (OSError, UnicodeDecodeError) as error:
        # 指向一个目录、没权限、不是 UTF-8：只接 FileNotFoundError 时这里一崩，整张表没判就报绿
        print("CRATES_BAD", line_number, name, f"文件 {path} 读不了（{type(error).__name__}）", sep="\t")
        continue
    hits = source.count(original)
    if hits != 1:
        print("CRATES_BAD", line_number, name, f"原文在 {path} 里命中 {hits} 次", sep="\t")
    # 点名的测试标了 #[ignore]（harness 重档、checker 档的全量），cargo test 参数里却没有 --include-ignored / --ignored：
    # 59 号与 prove-red.sh 照这一行跑，点名的测试被 libtest 跳过，这条变异永远报「没红」，而它守的那一格其实没人看。
    arguments = fields[4].split()
    package = arguments[arguments.index("-p") + 1] if "-p" in arguments[:-1] else None
    test_name = fields[5].split("::")[-1].strip()
    # cargo test 参数里 `--` 之后的位置参数是 libtest 的筛选词：点名的测试一个都筛不到，那条测试根本不跑，变异永远报没红。
    libtest_arguments = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    filters = [word for word in libtest_arguments if not word.startswith("-")]
    exact = "--exact" in libtest_arguments
    # libtest 拿筛选词去比完整测试名（模块路径 + 函数名）：完整名从点名的函数在哪个文件、套在哪几层内联 mod 里算出来；找不到那个函数的不判。
    candidates = [full_name for _attributes, full_name in test_attributes_of_package(package).get(test_name, [])] if package and test_name else []
    if candidates and filters and not any((full_name == word) if exact else (word in full_name) for word in filters for full_name in candidates):
        print("CRATES_FILTER", line_number, name, f"筛选词 {' '.join(filters)}{'（--exact）' if exact else ''} 筛不到点名的 {' 或 '.join(candidates)}", sep="\t")
    if package and test_name and not ({"--include-ignored", "--ignored"} & set(arguments)):
        found_tests = test_attributes_of_package(package).get(test_name, [])
        if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):
            print("CRATES_IGNORED", line_number, name, f"点名的 {test_name} 标了 #[ignore]，cargo test 参数「{fields[4]}」里没有 --include-ignored", sep="\t")
print("CRATES_CHECKED", checked)
PY
)"
  crates_exit_code=$?
fi
crates_checked="$(sed -n 's/^CRATES_CHECKED //p' <<<"$crates_report")"
mapfile -t crates_bad < <(grep '^CRATES_BAD' <<<"$crates_report")
mapfile -t crates_dup < <(grep '^CRATES_DUP' <<<"$crates_report")
mapfile -t crates_ignored < <(grep '^CRATES_IGNORED' <<<"$crates_report")
mapfile -t crates_filter < <(grep '^CRATES_FILTER' <<<"$crates_report")
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")
mapfile -t stray_rows < <(grep '^STRAY' <<<"$anchor_report")
mapfile -t crates_stray < <(grep '^CRATES_STRAY' <<<"$crates_report")
mapfile -t anchor_unreadable < <(grep '^UNREADABLE' <<<"$anchor_report")
mapfile -t tables_without_binary < <(sed -n 's/^NO_BINARY\t//p' <<<"$anchor_report")
# 两段 python 自己崩了（退出码非 0、或没报出 CHECKED 那一行）：它们没判完，不许读成「没有 BAD 行」
script_failures=()
if [[ "${anchor_exit_code:-1}" -ne 0 || -z "$anchor_checked" ]]; then
  script_failures+=("research 变异表的锚点检查没跑完（python 退出码 ${anchor_exit_code:-?}，CHECKED 行「${anchor_checked}」）")
fi
if [[ -f crates/mutations.tsv ]] && [[ "${crates_exit_code:-1}" -ne 0 || -z "$crates_checked" ]]; then
  script_failures+=("crates/mutations.tsv 的锚点检查没跑完（python 退出码 ${crates_exit_code:-?}，CRATES_CHECKED 行「${crates_checked}」）")
fi

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]} + ${#crates_bad[@]} + ${#crates_dup[@]} + ${#crates_ignored[@]} + ${#crates_filter[@]} + ${#stray_rows[@]} + ${#crates_stray[@]} + ${#anchor_unreadable[@]} + ${#script_failures[@]})); then
  if ((${#script_failures[@]})); then
    echo "  ✗ 锚点检查的 python 没跑完，这一段等于没判："
    printf '      %s\n' "${script_failures[@]}"
    echo "    → 怎么办：看上面 python 打出的报错（多半是某份文件读不了），修好再跑；没跑完不许当成锚点都对得上。"
  fi
  if ((${#anchor_unreadable[@]})); then
    echo "  ✗ 这些变异表或它的被测源码读不了（不是 UTF-8、是目录、没权限），表里的条目一条都没判："
    while IFS=$'\t' read -r _ stem why; do
      printf '      %s 或 %s：%s\n' "$MUT_DIR/$stem.tsv" "$BIN_DIR/$stem.rs" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_unreadable[@]}")
    echo "    → 怎么办：转成 UTF-8 文本（或把放错的目录挪走），再跑；mutate.sh 读它时同样会失败。"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："   # gate-lint:detail
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
    echo "    → 每行写成「变异名<TAB>原文<TAB>替换文」三段，坏行补齐或删掉，至少留一条成形的。"
  fi
  if ((${#anchor_bad[@]})); then
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      printf '      %s:%s %s：原文在 %s 里命中 %s 次\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem.rs" "$hits"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
    echo "  → 怎么办：缺表的写 research/mutations/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
    echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
    echo "    改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  fi
  if ((${#stray_rows[@]})); then
    echo "  ✗ 这些变异条目的原文或替换文里有 \\n 以外的反斜杠转义（mutate.sh 只还原 \\n，别的按字面写进源码）："
    while IFS=$'\t' read -r _ stem name lineno segment escape; do
      printf '      %s:%s %s：%s里的 %s\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${stray_rows[@]}")
    echo "    → 怎么办：去掉那个反斜杠。替换文按字面写进源码，\\& \\\" 这类会原样落进去、那份源码编不过，"
    echo "      变异既不算被抓也不算没红（.claude/rules/mutation-sampling.md 第八类：无效变异），而跑变异的脚本报的是「点名的测试没跑到」。"
  fi
  if ((${#crates_stray[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的原文或替换文里有 \\n 以外的反斜杠转义（门禁 59 号只还原 \\n）："
    while IFS=$'\t' read -r _ lineno name segment escape; do
      printf '      crates/mutations.tsv:%s %s：%s里的 %s\n' "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_stray[@]}")
    echo "    → 怎么办：去掉那个反斜杠，改完单跑那一行证明点名的测试红（2026-09-20 第 180 行踩过：替换文写 \\&report.outcome，"
    echo "      写进源码是字面反斜杠、编不过，59 号把它报成「点名的测试没跑到」，出路指向「先造一条会红的用例」，方向是反的）。"
  fi
  if ((${#crates_dup[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行与前面某一行重复（表里的条数没少，真改法少了一条）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_dup[@]}")
    echo "    → 怎么办：这一行本来要守的那一格今天没有守卫。把它换成真正打在那一格上的改法（改完单跑它、证明点名的测试红），"
    echo "      或者确认它多余就整行删掉。⚠️ 同一个改法让**两条不同的测试**变红是合法的，这道检查按「改法 + 点名的测试」四项认，不会拦那一种。"
  fi
  if ((${#crates_filter[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里的筛选词筛不到点名的测试（59 号跑它时那条测试不跑，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_filter[@]}")
    echo "    → 怎么办：把 \`--\` 后面的筛选词改成点名测试名的一段（测试改过名、拆过的多半是这个），或者删掉筛选词跑整个测试目标。"
  fi
  if ((${#crates_ignored[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试标了 #[ignore]，cargo test 参数里却没有 --include-ignored（59 号跑它时那条测试被跳过，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_ignored[@]}")
    echo "    → 怎么办：在那一行 cargo test 参数的 \`--\` 后面加 --include-ignored（没有 \`--\` 就补 \`-- --include-ignored\`），改完单跑那一行证明点名的测试红。"
  fi
  if ((${#crates_bad[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_bad[@]}")
    echo "    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），改完单跑那几行证明点名的测试红。"
  fi
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | wc -l)
crates_summary="；没有 crates/mutations.tsv"
[[ -f crates/mutations.tsv ]] && crates_summary="；crates/mutations.tsv ${crates_checked} 条的原文各命中源码一次"
if ((${#tables_without_binary[@]})); then
  crates_summary+="；没有同名实验二进制、锚点不在本阶段射程的表 ${#tables_without_binary[@]} 张：$(printf '%s ' "${tables_without_binary[@]}")"
fi
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次${crates_summary}（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复、点名标了 #[ignore] 的测试都带 --include-ignored、筛选词筛得到点名的测试——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）"
