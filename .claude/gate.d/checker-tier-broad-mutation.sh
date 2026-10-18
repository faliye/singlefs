#!/usr/bin/env bash
# admission: always 判的是此刻被判的那棵工作树上每条变异的单元键与广谱变异判定库，判定库不在 git 树里，上一次的结论不替这一次作保
# run-condition: command python3
# gate-stage: 广谱变异（提交门禁：此刻源码上每一条变异都有单元键相同的结论、没有一条没红）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 手写的几十行逐条改坏 crates/、自己起 cargo、要点名的测试红；这里不起 cargo，读 cargo-mutants 全量清单在广谱变异判定库里的结论，两张表、两个判据、两个判定库都不同
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判变异表成形、锚点唯一；这里不读变异表，读驱动 --summary 的判定行与判定库
# gate-similar: checker-tier-formal-verification.sh 它起 Verus 判证明，每条判定按单元键在它自己的库里复用；这里只读广谱变异驱动的判定，并进去会让两件不相干的事共用一份判法
# gate-similar: checker-tier-crash-point-replay.sh 它的快档逐条核崩溃枚举用例的崩溃状态块、缺块报本次未跑；这里有没结论的判红不报本次未跑（用户定提交前跑完才能提交）
#
# gate-cell: gate-verdict 驱动 --summary 的 BROAD_MUTATION_GATE 判绿：此刻源码上每一条变异都有单元键相同的结论、没有一条没红、没有单元缺清单，判定行、汇总行、明细与退出码互相对得上，升档登记行与判定行的校验列逐行复核
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判什么：此刻源码上每一条变异都有单元键相同的结论、没有一条没红（gate-verdict 格）；结论键是被变异的函数单元与它调到的单元各自的指纹、原文、替换文、
#   抓它的测试的单元闭包键（没红的是能碰到它的测试各自的闭包键）、工具链、cargo-mutants / nextest / llvm 版本、构建环境的生效值、解析出的依赖版本、判法单元指纹
#   （规则在 scripts/mutation/mutation-prune.py 文件头，判定库在 scripts/mutation/broad-mutation.py 文件头「库目录」「键」）。
#   判定库里没有「一版」：不比暂存树、不比提交，没有 library-version-is-staged-tree 格，没有 --marker-check；也没有整道标记，不问 stage-must-run.sh，每趟照判（只读库目录，几秒）。
#   驱动由主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起、剪枝跑（每条按单元键沿用、沿用不了的现判），这一道只读结论；
#   被判的那棵工作树就是第一个参数（主工作区，或 gate.sh --staged 的临时 worktree：内容是 HEAD 加暂存区）。
# gate-verdict：驱动 --summary --source <被判的根>（--home 指到同一个判定库）在此刻的源码上照清单按单元存的那张表复原清单、现算每条的单元键、查判定库，
#   打恰好一行 BROAD_MUTATION_SUMMARY、一行 BROAD_MUTATION_GATE；verdict=green、missed=0、without_conclusion=0、unlisted_units=0、concluded=total、total>0，退出码 0；
#   verdict=red 而退 8 是驱动判红，照抄它的 NO_CONCLUSION（带作废时变了的单元）/ MISSED / UNLISTED 明细；
#   判定行与退出码对不上（verdict 不是 green / red、写 red 而退 0）、计数自相矛盾（判绿而 missed、without_conclusion 或 unlisted_units 不是 0，
#   NO_CONCLUSION、MISSED、REFUSED、UNLISTED 明细行数与计数不同、清单 0 条）、汇总行与判定行各数对不上（两行的 total、refused、unlisted_units，
#   汇总行抓到加没红加无效对判定行的 concluded、汇总行超时加内存撞顶加只过了第一段加驱动错加没跑对 without_conclusion、汇总行八栏之和对它的 total），都判红。
#   拒算的条（refused=：单元键里有认不出的形态、或抓到它的测试认不出）照报条数、逐条 REFUSED，有键相同的现判结论就算有结论（驱动每趟都现判它们）。
#   超时、内存撞顶照驱动算没结论（用户 2026-10-02 定）；驱动对超时、撞顶的那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到（用户 2026-10-03 定，驱动文件头「升档」）：
#   --summary 逐条打 ESCALATED <TAB> 序号 <TAB> 标签 <TAB> 变异名 <TAB> 键、判定行带 escalated=<条数>（条数与 ESCALATED 行数不同判红；有 ESCALATED 而判定行没有 escalated= 判红）。
#   这一格逐行复核库目录登记表 escalated-verdicts.tsv（列名、标签、升档常量取此刻驱动里的 ESCALATION_COLUMNS 与四个常量）：只认键是某一行 ESCALATED 的键的登记行
#   （别的键的行不看，连段数也不核）；每一行要：段数对、标签认得；基线按登记行的 baseline_row（行号）找 baselines.tsv 的那一行：成形、基线键、步名、结局 ok、线程数都与登记的相同，
#   秒数（超时）或峰值字节（撞顶）与登记的 baseline 相同；升档那趟的限时或上限比原来的大、不低于基线的 K 倍；两份日志在库目录的 escalation/ 下、sha256 与登记的相同、
#   末尾有驱动写的 BROAD_MUTATION_EVIDENCE 那一行（键、序号、步名、哪一趟、限时或上限、退出码 253 / 250）、有包装打的「超过限时 <限时> 秒」或「撞了内存上限 <上限>」；
#   日志不在的是没有驱动写的证据（手写的行）。一行复核不过判红；每一条 ESCALATED 都要有一行复核过的登记行（键、标签相同），没有判红。
#   判定行的校验列（驱动文件头「判定行的校验列」，用户 2026-10-03 定）：这一格另用同一把密钥逐行核判定库 verdicts.tsv（密钥位置取驱动的 result_key_path，
#   ${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key；样本里是 .broad-sample/result.key）：每一行十七段、末段是前十六段的 HMAC-SHA256。
#   少一段的（没有校验列）、校验列对不上的、段数不对的、密钥不在或权限不是 600 时的每一行都判红；判定库不在就没有行可核。这一段的算法在这里另写一份，不调驱动的函数。
#
# 样本（fixtures/checker-tier-broad-mutation.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）：
#   样本根放了标记 .broad-mutation-sample（只供样本的开关：被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认），里面一行 scenario=<族>:<情形>；
#   开格之前起 lib/broad-mutation-samples.py <驱动> <样本根> <情形> 造现场（三族的造法与各情形写在那份文件头）：summary 族与 escalated 族写 .broad-sample/summary.sh，
#   格起它（打录好的输出、退录好的码）；units 族在样本根写一棵小工作区、用此刻驱动的函数把清单与判定记进 .broad-sample/home/，再改源码，
#   格起真的驱动 --summary（--home .broad-sample/home、--result-key .broad-sample/result.key、--toolchain-from .broad-sample/rustc-vv.txt、--tools-from .broad-sample/tool-versions.txt）。
#   判定库、密钥换成 .broad-sample/home 与 .broad-sample/result.key。
#   gate-verdict-units-* 那几份是新键的样本（controls 路第 5 节、words 路 N27）：-green 照记的那一份源码判绿；-callee-changed-red 被调的 helper 改了函数体，
#   只有调它的那一条要现判（NO_CONCLUSION 带「变了：改了 helper」）、helper 自己那个单元没记清单（UNLISTED），别的条照样有结论；
#   -comment-only-green 只加注释，零条要现判；-unlisted-red 新加一个函数，那个单元没记清单。
#   gate-verdict-* 录好输出的那几份：-green 全有结论；-missed-red 一条没红、退 8；-no-conclusion-red 三条没结论、退 8；-silent-driver-red 判定行写 green、退 0，而 missed=1；
#   -line-missing-red 只有汇总行、退 0；-red-exit-zero-red 判定行写 red、计数与明细一致、退 0；-detail-count-red 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION；
#   -empty-list-red 清单 0 条、判定行 green；-unknown-verdict-red verdict=pending、退 8；-totals-mismatch-red 汇总行 total=5091、not_run=5085，判定行 total=6 写 green；
#   -refused-count-red 两行都写 refused=1、一行 REFUSED 都没列；-unlisted-lie-red 判定行写 green 而 unlisted_units=2。
#   gate-verdict-escalated-* 与 gate-verdict-result-mac-* 那几份（escalated 族，各情形写在 lib/broad-mutation-samples.py 的 build_escalated）。
# 弄坏开关 CHECKER_TIER_BROAD_MUTATION_BREAK=<项>（逗号分隔；只给样本证红用），打开后判错的样本：
#   summary-red-passes            驱动退 8、判定行写 red 也算绿，判定行与退出码对不上不判
#                                                                             gate-verdict-missed-red、-no-conclusion-red、-red-exit-zero-red、-units-callee-changed-red、-units-unlisted-red（判绿）
#   verdict-exit-unchecked        判定行写 red 而退 0 不判                    gate-verdict-red-exit-zero-red（判绿）
#   verdict-value-unchecked       verdict 不是 green / red 不判               gate-verdict-unknown-verdict-red（判绿）
#   counts-unchecked              不核判定行的计数与明细                      gate-verdict-silent-driver-red、-detail-count-red、-empty-list-red、-refused-count-red、-unlisted-lie-red（判绿）
#   detail-count-unchecked        不核明细行数与计数                          gate-verdict-detail-count-red、-refused-count-red（判绿）
#   empty-list-passes             清单 0 条不判                               gate-verdict-empty-list-red（判绿）
#   unlisted-passes               判绿时不看 unlisted_units                   gate-verdict-unlisted-lie-red（判绿）
#   summary-gate-totals-unchecked 不核汇总行与判定行的各数                    gate-verdict-totals-mismatch-red（判绿）
#   gate-line-optional            没有判定行而退 0 算绿                       gate-verdict-line-missing-red（判绿）
#   escalated-multiple-unchecked  登记行不核倍数门槛（连同「比原来的大」）    gate-verdict-escalated-multiple-red、-memory-multiple-red、-not-larger-red（判绿）
#   escalated-log-digest-unchecked 登记行不核日志的 sha256                    gate-verdict-escalated-log-digest-red（判绿）
#   escalated-key-unchecked       登记行不按键挑，每一行都核、ESCALATED 对登记行只比序号与标签
#                                                                             gate-verdict-escalated-old-key-red（判绿）、-green（那一行别的键的坏行被核，判红）
#   escalated-evidence-optional   登记行指的日志不在（或不在 escalation/ 下）也认
#                                                                             gate-verdict-escalated-handwritten-red、-logs-elsewhere-red（判绿）
#   escalated-count-unchecked     不核判定行 escalated= 与 ESCALATED 行数（连同有 ESCALATED 而没有 escalated=）
#                                                                             gate-verdict-escalated-count-red、-no-escalated-field-red（判绿）
#   escalated-baseline-last-row   基线照改前取：这一步最后一行绿记录，不看登记的行号、基线键与线程数
#                                                                             gate-verdict-escalated-threads-rerun-green（判红：旧登记行对上了 16 线程那一行）、
#                                                                             -baseline-row-changed-red（判绿：不看那一行的线程数）
#   escalated-baseline-row-identity-unchecked 不核登记行指的那一行的基线键、步名、结局与线程数
#                                                                             gate-verdict-escalated-baseline-row-changed-red、-baseline-key-differs-red（判绿）
#   escalated-baseline-unchecked  不比登记的基线与它指的那一行的值            gate-verdict-escalated-baseline-lie-red（判绿）
#   escalated-not-larger-unchecked 不核升档那趟比原来的大                     gate-verdict-escalated-not-larger-red（判绿）
#   escalated-memory-multiple-ignored 撞顶那一边的倍数门槛按 0 算             gate-verdict-escalated-memory-multiple-red（判绿）
#   escalated-trailer-unchecked   不核日志末尾驱动写的那一行                  gate-verdict-escalated-no-trailer-red、-trailer-wrong-red（判绿）
#   escalated-trailer-key-only    驱动那一行只核键（不核序号、步名、哪一趟、限时、退出码）
#                                                                             gate-verdict-escalated-trailer-wrong-red（判绿）、-no-trailer-red（照红，缺打全那一行的 want）
#   escalated-wrapper-unchecked   不核日志里包装打的那一句                    gate-verdict-escalated-no-wrapper-red（判绿）
#   escalated-columns-loose       登记行只要不少于 10 段就认                  gate-verdict-escalated-columns-red（判绿）
#   escalated-field-optional      有 ESCALATED 而判定行没有 escalated= 不判   gate-verdict-escalated-no-escalated-field-red（判绿）
#   escalated-label-unchecked     ESCALATED 对登记行只比键（不比标签）        gate-verdict-escalated-label-differs-red（判绿）
#   escalated-logs-anywhere       日志不在 escalation/ 下也认                 gate-verdict-escalated-logs-elsewhere-red（判绿）
#   result-mac-unchecked          不核判定库的校验列                          gate-verdict-result-mac-forged-red、-result-mac-key-missing-red（判绿）、
#                                                                             gate-verdict-escalated-green、-escalated-threads-rerun-green（照绿，缺「判定库 N 行逐行核过」的 want）
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本的格按绿记、整道退 0）。
#
#   bash .claude/gate.d/checker-tier-broad-mutation.sh [项目根]
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --list              逐行打格名与判什么，不起驱动、不起 rustc
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --check <格名>      只跑点名的格；格名写错退 2
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --tool-versions     打 rustc -Vv、cargo nextest、cargo mutants 与 llvm-tools 的 llvm-profdata 版本（只问版本，不跑变异）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
DRIVER="$STAGE_REPOSITORY/scripts/mutation/broad-mutation.py"
SAMPLE_BUILDER="$STAGE_REPOSITORY/.claude/gate.d/lib/broad-mutation-samples.py"
SAMPLE_MARKER=".broad-mutation-sample"
SAMPLE_DIRECTORY=".broad-sample"
RUN_DRIVER_HOWTO="主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起驱动剪枝跑，再跑这一道"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell gate-verdict cell_gate_verdict "驱动 --summary 的 BROAD_MUTATION_GATE 判绿：此刻源码上每一条变异都有单元键相同的结论、没有一条没红、没有单元缺清单，判定行、汇总行、明细与退出码互相对得上，升档登记行与判定行的校验列逐行复核" \
  "没有结论的、有单元没记清单的：$RUN_DRIVER_HOWTO（剪枝跑也重跑超时、内存撞顶的条，那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到；证据不齐、仍是这两种的加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun）；没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判）；升档登记行复核不过的照它点的那一样修，或 --item <序号> --rerun 重判那一条；--summary 退 1（判定库或清单表对不上）照驱动打的 → 修"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--tool-versions 与一个项目根（--marker-check 已删：没有整道标记，这一道每趟照判、只读判定库）。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"

if ((tool_versions_requested)); then
  if command -v rustc >/dev/null 2>&1; then
    rustc -Vv 2>&1 || echo "rustc -Vv 退出码非 0"
    # llvm-tools：与驱动不给 --llvm-bin 时同一处（rustc 的 sysroot 下 lib/rustlib/<host>/bin）
    llvm_sysroot="$(rustc --print sysroot 2>/dev/null)"
    llvm_host="$(rustc -Vv 2>/dev/null | sed -n 's/^host: //p')"
    if [[ -x "$llvm_sysroot/lib/rustlib/$llvm_host/bin/llvm-profdata" ]]; then
      "$llvm_sysroot/lib/rustlib/$llvm_host/bin/llvm-profdata" --version 2>&1 | head -n 3
    else
      echo "llvm-profdata：$llvm_sysroot/lib/rustlib/$llvm_host/bin 下没有（rustup component add llvm-tools）"
    fi
  else
    echo "rustc：PATH 上没有"
  fi
  if command -v cargo >/dev/null 2>&1; then
    cargo nextest --version 2>&1 | head -n 1
    cargo mutants --version 2>&1 | head -n 1
  else
    echo "cargo：PATH 上没有"
  fi
  exit 0
fi

sample_mode=0
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" && -f "$SAMPLE_MARKER" ]]; then sample_mode=1; fi
LIBRARY_HOME="${SINGLEFS_BROAD_MUTATION_HOME:-$HOME/.local/share/singlefs/broad-mutation-units}"
summary_command=(python3 "$DRIVER" --summary --home "$LIBRARY_HOME" --source "$ROOT")
sample_result_key=""
if ((sample_mode)); then
  LIBRARY_HOME="$ROOT/$SAMPLE_DIRECTORY/home"
  sample_result_key="$ROOT/$SAMPLE_DIRECTORY/result.key"
  scenario="$(sed -n 's/^scenario=//p' "$SAMPLE_MARKER" | head -n 1)"
  if [[ -n "$scenario" ]] && ! env -u BROAD_MUTATION_BREAK python3 "$SAMPLE_BUILDER" "$DRIVER" "$ROOT" "$scenario"; then
    echo "  ✗ 样本现场没造成：lib/broad-mutation-samples.py 情形 $scenario 退非 0"
    echo "     → 怎么办：看上面的报错；units 族载入 $DRIVER 调 Library、Judgement、canonical_items、record_list_cache，驱动里这几样改了名或参数就照着改那份造法。"
    exit 1
  fi
  if [[ -f "$SAMPLE_DIRECTORY/summary.sh" ]]; then
    summary_command=(bash "$ROOT/$SAMPLE_DIRECTORY/summary.sh")
  else
    summary_command=(python3 "$DRIVER" --summary --home "$LIBRARY_HOME" --source "$ROOT" --result-key "$sample_result_key"
                     --toolchain-from "$ROOT/$SAMPLE_DIRECTORY/rustc-vv.txt" --tools-from "$ROOT/$SAMPLE_DIRECTORY/tool-versions.txt")
  fi
fi

# ── 格 gate-verdict ──
cell_gate_verdict() {
local output
output="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
if env -u BROAD_MUTATION_BREAK "${summary_command[@]}" > "$output" 2>&1; then summary_exit=0; else summary_exit=$?; fi
env -u BROAD_MUTATION_BREAK python3 - "$output" "$summary_exit" "$LIBRARY_HOME" "${CHECKER_TIER_BROAD_MUTATION_BREAK:-}" "$DRIVER" "$sample_result_key" "$RUN_DRIVER_HOWTO" <<'PY'
import hashlib, hmac, importlib.util, os, re, sys
sys.dont_write_bytecode = True
output_path, exit_text, home, broken_text, driver_path, sample_result_key, howto_run = sys.argv[1:8]
broken = set(filter(None, broken_text.split(",")))
summary_exit = int(exit_text)
with open(output_path, encoding="utf-8", errors="replace") as handle:
    lines = handle.read().splitlines()
def fields_of(prefix):
    found = [line for line in lines if line.startswith(prefix + " ")]
    return [dict(token.split("=", 1) for token in line.split()[1:] if "=" in token) for line in found]
def tail():
    return "\n".join("       | " + line for line in lines[-12:])
def detail(prefix):
    return [line for line in lines if line.startswith(prefix + "\t")]
summaries, gates = fields_of("BROAD_MUTATION_SUMMARY"), fields_of("BROAD_MUTATION_GATE")
no_conclusion, missed_lines, refused_lines, unlisted_lines = detail("NO_CONCLUSION"), detail("MISSED"), detail("REFUSED"), detail("UNLISTED")
if summary_exit not in (0, 8):
    print(f"  ✗ 驱动 --summary 退 {summary_exit}（不是判绿的 0、也不是判红的 8）：判定库或清单表对不上、或驱动没跑成；它的输出末几行：")
    print(tail())
    print("     → 怎么办：照上面驱动打的 → 办（退 1 是判定库或清单表对不上，退 2 是用法或库目录不对）；修好再跑这一格。")
    sys.exit(1)
if len(summaries) != 1 or (len(gates) != 1 and not ("gate-line-optional" in broken and not gates and summary_exit == 0)):
    print(f"  ✗ 驱动 --summary 打了 {len(summaries)} 行 BROAD_MUTATION_SUMMARY、{len(gates)} 行 BROAD_MUTATION_GATE，各该恰好一行：判不了提交门禁；输出末几行：")
    print(tail())
    print("     → 怎么办：没有判定行是驱动还是旧的那一份：scripts/mutation/broad-mutation.py 从 git 取今天的那一份；行数多了是输出被拼进了别的东西。")
    sys.exit(1)
summary = summaries[0]
gate = gates[0] if gates else {"verdict": "green", "total": summary.get("total", "0"), "concluded": summary.get("total", "0"), "missed": "0",
                                "without_conclusion": "0", "refused": summary.get("refused", "0"), "unlisted_units": summary.get("unlisted_units", "0")}
def number(key, source=gate):
    try:
        return int(source.get(key, ""))
    except ValueError:
        return None
total, concluded, missed, without = number("total"), number("concluded"), number("missed"), number("without_conclusion")
refused, unlisted = number("refused"), number("unlisted_units")
if None in (total, concluded, missed, without, refused, unlisted):
    print(f"  ✗ 判定行的计数读不出：{gate}")
    print("     → 怎么办：驱动打的判定行坏了（要 total、concluded、missed、without_conclusion、refused、unlisted_units 各一个整数），scripts/mutation/broad-mutation.py 从 git 取今天的那一份再跑。")
    sys.exit(1)
verdict = gate.get("verdict")
if verdict == "red" and summary_exit == 8 and "summary-red-passes" not in broken:
    print(f"  ✗ 广谱变异提交门禁判红：此刻源码上 {total} 条里 {without} 条沿用不了也没现判、{missed} 条没红、{unlisted} 个单元没记清单")  # gate-lint:summary
    for line in (missed_lines + no_conclusion + unlisted_lines)[:30]:
        print("     " + line.replace("\t", " "))  # gate-lint:detail
    if len(missed_lines) + len(no_conclusion) + len(unlisted_lines) > 30:
        print(f"     ……另有 {len(missed_lines) + len(no_conclusion) + len(unlisted_lines) - 30} 行，全文在驱动 --summary 的输出里")  # gate-lint:detail
    print(f"     → 怎么办：没有结论的、有单元没记清单的：{howto_run}（剪枝跑也重跑超时、内存撞顶的条，那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到；"
          "证据不齐、仍是这两种的加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun）；"
          "没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（只那几条不再判）。")
    sys.exit(1)
expected_exit = 0 if verdict == "green" else 8
inconsistent = []
if verdict not in ("green", "red") and "verdict-value-unchecked" not in broken:
    inconsistent.append(f"verdict={verdict}（只认 green / red）")
elif summary_exit != expected_exit and "summary-red-passes" not in broken and "verdict-exit-unchecked" not in broken:
    inconsistent.append(f"判定行写 verdict={verdict}，驱动却退 {summary_exit}")
if "counts-unchecked" not in broken:
    if verdict == "green" and (missed or without or concluded != total):
        inconsistent.append(f"判定行写 green，计数却是 total={total} concluded={concluded} missed={missed} without_conclusion={without}")
    if verdict == "green" and unlisted and "unlisted-passes" not in broken:
        inconsistent.append(f"判定行写 green，却有 unlisted_units={unlisted} 个单元没记清单（清单不全，判不了「每一条都有结论」）")
    if concluded + without != total:
        inconsistent.append(f"concluded={concluded} 加 without_conclusion={without} 不等于 total={total}")
    if "detail-count-unchecked" not in broken and (len(no_conclusion) != without or len(missed_lines) != missed or number("missed", summary) != missed
                                                  or len(refused_lines) != refused or len(unlisted_lines) != unlisted):
        inconsistent.append(f"明细与计数不同：NO_CONCLUSION {len(no_conclusion)} 行、MISSED {len(missed_lines)} 行、REFUSED {len(refused_lines)} 行、UNLISTED {len(unlisted_lines)} 行，"
                            f"判定行 without_conclusion={without}、missed={missed}、refused={refused}、unlisted_units={unlisted}，汇总行 missed={summary.get('missed')}")
    if total <= 0 and "empty-list-passes" not in broken:
        inconsistent.append("此刻清单 0 条：没有可判的变异，不是通过")
if "summary-gate-totals-unchecked" not in broken:
    columns = ("total", "caught", "missed", "unviable", "timeout", "memory_cap", "stage_one_only", "driver_error", "not_run", "refused", "unlisted_units")
    counts = {key: number(key, summary) for key in columns}
    if None in counts.values():
        inconsistent.append(f"汇总行的计数读不出（要 {'、'.join(columns)} 各一个整数）：{summary}")
    else:
        decided = counts["caught"] + counts["missed"] + counts["unviable"]
        undecided = counts["timeout"] + counts["memory_cap"] + counts["stage_one_only"] + counts["driver_error"] + counts["not_run"]
        if counts["total"] != total:
            inconsistent.append(f"汇总行 total={counts['total']}，判定行 total={total}")
        if decided != concluded:
            inconsistent.append(f"汇总行抓到 {counts['caught']} 加没红 {counts['missed']} 加无效 {counts['unviable']} 是 {decided}，判定行 concluded={concluded}")
        if undecided != without:
            inconsistent.append(f"汇总行超时、内存撞顶、只过了第一段、驱动错、没跑加起来是 {undecided}，判定行 without_conclusion={without}")
        if decided + undecided != counts["total"]:
            inconsistent.append(f"汇总行八栏加起来 {decided + undecided}，它的 total={counts['total']}")
        if counts["refused"] != refused or counts["unlisted_units"] != unlisted:
            inconsistent.append(f"汇总行 refused={counts['refused']}、unlisted_units={counts['unlisted_units']}，判定行 refused={refused}、unlisted_units={unlisted}")
escalated_lines = [line.split("\t") for line in detail("ESCALATED")]
if "escalated-count-unchecked" not in broken:
    if "escalated" in gate and number("escalated") != len(escalated_lines):
        inconsistent.append(f"判定行 escalated={gate.get('escalated')}，ESCALATED 明细 {len(escalated_lines)} 行")
    elif "escalated" not in gate and escalated_lines and "escalated-field-optional" not in broken:
        inconsistent.append(f"有 {len(escalated_lines)} 行 ESCALATED 明细，判定行却没有 escalated=")
if inconsistent:
    print("  ✗ 驱动 --summary 的判定行、汇总行、计数、明细与退出码对不上（驱动的提交门禁判法失灵）：")  # gate-lint:summary
    for entry in inconsistent:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：这是驱动的毛病，不是此刻源码的结论：交工具实现员查 scripts/mutation/broad-mutation.py 的 summarise（它的 --selftest 有 summary-gate-silent 那一格）；修好前不提交。")
    sys.exit(1)
# 升档判的抓到（驱动文件头「升档」）：逐行复核登记表里 ESCALATED 那几把键的行，每一条 ESCALATED 都要有一行复核过的登记行
specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_gate_verdict", driver_path)
driver = importlib.util.module_from_spec(specification)
try:
    specification.loader.exec_module(driver)
    columns = tuple(driver.ESCALATION_COLUMNS)
    baseline_columns = tuple(driver.BASELINE_COLUMNS)
    timeout_label, memory_label = driver.ESCALATED_TIMEOUT_LABEL, driver.ESCALATED_MEMORY_LABEL
    multiples = {timeout_label: driver.ESCALATION_TIME_BASELINE_MULTIPLE,
                 memory_label: 0 if "escalated-memory-multiple-ignored" in broken else driver.ESCALATION_MEMORY_BASELINE_MULTIPLE}
    registry_path, evidence_directory = os.path.join(home, driver.ESCALATION_REGISTRY), driver.ESCALATION_DIRECTORY
    verdict_columns, signed_columns = len(driver.VERDICT_COLUMNS), driver.RESULT_SIGNED_COLUMNS
    result_key_path = sample_result_key or driver.result_key_path()
except Exception as error:  # 驱动被删、改坏、少了升档那几个常量
    print(f"  ✗ 读不进广谱变异的驱动 {driver_path} 里升档与判定库的常量：{error}")
    print("     → 怎么办：它随仓走，被删了或改坏了从 git 找回来；登记表与基线表的列、标签、倍数门槛、判定库的列与密钥的位置只认它一份。")
    sys.exit(1)
# 判定行的校验列（驱动文件头「判定行的校验列」）：判定库的每一行都要带用这台机器的密钥算的 HMAC-SHA256，这里另算一遍、逐行比
mac_problems, mac_rows = [], 0
verdicts_path = os.path.join(home, "verdicts.tsv")
if "result-mac-unchecked" not in broken and os.path.exists(verdicts_path):
    key, key_problem = None, ""
    try:
        key_status = os.stat(result_key_path)
        if key_status.st_uid != os.getuid() or key_status.st_mode & 0o077:
            key_problem = f"密钥 {result_key_path} 的权限是 {key_status.st_mode & 0o777:o}：要属主是自己、权限 600"
        else:
            with open(result_key_path, encoding="ascii", errors="replace") as handle:
                key_text = handle.read().strip()
            if re.fullmatch(r"[0-9a-f]{64}", key_text):
                key = bytes.fromhex(key_text)
            else:
                key_problem = f"密钥 {result_key_path} 的内容不是 64 位小写十六进制"
    except FileNotFoundError:
        key_problem = f"密钥 {result_key_path} 不在"
    except OSError as error:
        key_problem = f"读不了密钥 {result_key_path}：{error}"
    with open(verdicts_path, "rb") as handle:
        verdict_text = handle.read().decode("utf-8", errors="surrogateescape").split("\n")
    for line_number, line in enumerate(verdict_text[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        where = f"判定库第 {line_number} 行（第 {fields[1] if len(fields) > 1 else '?'} 条，结局 {fields[4] if len(fields) > 4 else '?'}）"
        if len(fields) == signed_columns:
            mac_problems.append(f"{where}没有校验列（{signed_columns} 段）")
        elif len(fields) != verdict_columns:
            mac_problems.append(f"{where}段数是 {len(fields)}，不是 {verdict_columns}")
        elif key is None:
            mac_problems.append(f"{where}校验不了：{key_problem}")
        elif not hmac.compare_digest(hmac.new(key, "\t".join(fields[:signed_columns]).encode("utf-8", "surrogateescape"), hashlib.sha256).hexdigest(), fields[-1]):
            mac_problems.append(f"{where}的校验列对不上")
        else:
            mac_rows += 1
if mac_problems:
    print(f"  ✗ 广谱变异判定库里有 {len(mac_problems)} 行不是驱动用这台机器的密钥写的（{verdicts_path}）：")  # gate-lint:summary
    for entry in mac_problems[:20]:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：没有校验列或校验列对不上的行是手写或拷来的：照驱动 --summary 打的出路把判定库拷一份留底、删掉点到的那几行，再让驱动点名重判那几条"
          "（--item <序号> --rerun，前缀与起法照 .claude/skills/crash-test/commit-stages.md 第 2b 步），跑完再跑这一道；"
          "密钥不在、权限不对，或每一行都校验不过（密钥丢了或换了），另起一个空库目录照第 2b 步重判")
    sys.exit(1)
cap_form, cap_units = re.compile(r"^([1-9][0-9]*)([KMGT])$"), {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}

def amount_of(label, text):
    """登记行里的限时（整数秒）或上限（run-with-memory-cap.sh 的写法）→ 数；读不出交 None。"""
    if label == timeout_label:
        return int(text) if re.fullmatch(r"[1-9][0-9]*", text or "") else None
    found = cap_form.match(text or "")
    return int(found.group(1)) * cap_units[found.group(2)] if found else None

def referenced_baseline(row):
    """登记行 baseline_row 指的那一行基线（驱动文件头「升档」①）：交 (那一行的值, 不对的地方)。值是秒数（超时）或峰值字节（撞顶）；
    那一行要是 baselines.tsv 里成形的一行，基线键、步名、结局（ok）、线程数都与登记的相同。没有那一行交 (None, [为什么])。"""
    value_column = "seconds" if row["label"] == timeout_label else "peak_bytes"
    path = os.path.join(home, "baselines.tsv")
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            baseline_lines = handle.read().split("\n")[:-1]
    except OSError:
        return None, [f"库目录里没有 baselines.tsv（{row['step']} 的基线）"]
    if "escalated-baseline-last-row" in broken:   # 改前的取法：这一步最后一行绿记录，不看行号、基线键与线程数
        found = None
        for line in baseline_lines:
            fields = line.split("\t")
            if not line.startswith("#") and len(fields) == len(baseline_columns) and fields[1] == row["step"] and fields[2] == "ok":
                found = dict(zip(baseline_columns, fields))[value_column]
        return found, ([] if found is not None else [f"没记 {row['step']} 的基线"])
    number = row.get("baseline_row", "")
    if not number.isdigit() or not 1 <= int(number) <= len(baseline_lines):
        return None, [f"登记的基线行号「{number}」在 baselines.tsv 里没有（它有 {len(baseline_lines)} 行）"]
    line = baseline_lines[int(number) - 1]
    fields = line.split("\t")
    if line.startswith("#") or len(fields) != len(baseline_columns):
        return None, [f"baselines.tsv 第 {number} 行不是一行基线（{len(fields)} 段）"]
    entry = dict(zip(baseline_columns, fields))
    identity = []
    if "escalated-baseline-row-identity-unchecked" not in broken:
        for column, wanted in (("baseline_key", row["baseline_key"]), ("step", row["step"]), ("outcome", "ok"), ("threads", row["threads"])):
            if entry[column] != wanted:
                identity.append(f"baselines.tsv 第 {number} 行的 {column} 是「{entry[column][:20]}」，登记的是「{wanted[:20]}」")
    return entry[value_column], identity

def row_problems(row):
    """一行登记的证据缺哪几样（空表是复核过了）。"""
    found, label = [], row["label"]
    if label not in multiples:
        return [f"标签「{label}」认不出（只认 {timeout_label}、{memory_label}）"]
    baseline_text, baseline_identity = referenced_baseline(row)
    found += baseline_identity    # 没有那一行时为什么也在里面
    if baseline_text is not None and baseline_text != row["baseline"] and "escalated-baseline-unchecked" not in broken:
        found.append(f"登记的基线 {row['baseline']} 与 baselines.tsv 第 {row.get('baseline_row', '?')} 行记的 {baseline_text or '空'} 不同")
    first, escalated = amount_of(label, row["first_limit"]), amount_of(label, row["escalated_limit"])
    if "escalated-multiple-unchecked" not in broken:
        try:
            baseline = float(row["baseline"])
        except ValueError:
            baseline = None
        if first is None or escalated is None:
            found.append(f"限时或上限读不出（{row['first_limit']} → {row['escalated_limit']}）")
        elif escalated <= first and "escalated-not-larger-unchecked" not in broken:
            found.append(f"升档那趟的 {row['escalated_limit']} 没比原来的 {row['first_limit']} 大")
        elif baseline is None or escalated < multiples[label] * baseline:
            found.append(f"升档那趟的 {row['escalated_limit']} 不到基线 {row['baseline']} 的 {multiples[label]} 倍（倍数门槛是驱动里的常量）")
    hit_exit = 253 if label == timeout_label else 250
    for attempt in ("first", "escalated"):
        relative, limit = row[f"{attempt}_log"], row[f"{attempt}_limit"]
        path = os.path.normpath(os.path.join(home, relative))
        outside = not path.startswith(os.path.join(home, evidence_directory) + os.sep) and "escalated-logs-anywhere" not in broken
        if outside or not os.path.isfile(path):
            if "escalated-evidence-optional" not in broken:
                found.append(f"{attempt} 那趟的日志 {relative} 不在库目录的 {evidence_directory}/ 下：没有驱动写的证据（手写的行）")
            continue
        with open(path, "rb") as handle:
            content = handle.read()
        if "escalated-log-digest-unchecked" not in broken and hashlib.sha256(content).hexdigest() != row[f"{attempt}_log_sha256"]:
            found.append(f"{attempt} 那趟的日志 {relative} 的 sha256 与登记的不同（登记之后改过）")
        text = content.decode("utf-8", errors="replace")
        trailer = f"BROAD_MUTATION_EVIDENCE key={row['key']} sequence={row['sequence']} step={row['step']} attempt={attempt} limit={limit} exit={hit_exit}"
        if "escalated-trailer-key-only" in broken:
            trailer = f"BROAD_MUTATION_EVIDENCE key={row['key']}"
        if trailer not in text and "escalated-trailer-unchecked" not in broken:
            found.append(f"{attempt} 那趟的日志末尾没有驱动写的那一行（{trailer}）")
        wrapper = f"超过限时 {limit} 秒" if label == timeout_label else f"撞了内存上限 {limit}"
        if wrapper not in text and "escalated-wrapper-unchecked" not in broken:
            found.append(f"{attempt} 那趟的日志里没有包装打的「{wrapper}」")
    return found

wanted_keys = {fields[4] for fields in escalated_lines if len(fields) == 5}
registry_problems, verified = [], set()
if os.path.exists(registry_path):
    with open(registry_path, encoding="utf-8", errors="replace") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip() or line.startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t")
            if fields[0] not in wanted_keys and "escalated-key-unchecked" not in broken:
                continue   # 只认 ESCALATED 那几把键的行（别的键的行不看，连段数也不核：它们可能是改列之前写的）
            if (len(fields) < 10) if "escalated-columns-loose" in broken else (len(fields) != len(columns)):
                registry_problems.append(f"登记表第 {line_number} 行段数是 {len(fields)}，不是 {len(columns)}")
                continue
            row = dict(zip(columns, fields))
            found = row_problems(row)
            if found:
                registry_problems.append(f"登记表第 {line_number} 行（第 {row['sequence']} 条 {row['label']}，键 {row['key'][:16]}）：" + "；".join(found))
            else:
                verified.add((row["key"], row["label"], row["sequence"]))
for fields in escalated_lines:
    if len(fields) != 5:
        registry_problems.append(f"ESCALATED 明细「{' '.join(fields)[:120]}」不是五段（序号、标签、变异名、键）")
        continue
    if "escalated-key-unchecked" in broken:
        matched = any(entry[2] == fields[1] and entry[1] == fields[2] for entry in verified)
    elif "escalated-label-unchecked" in broken:
        matched = any(entry[0] == fields[4] for entry in verified)
    else:
        matched = any(entry[0] == fields[4] and entry[1] == fields[2] for entry in verified)
    if not matched:
        registry_problems.append(f"--summary 说第 {fields[1]} 条（键 {fields[4][:16]}）是升档判的 {fields[2]}，登记表里没有这把键、这个标签复核得过的那一行")
if registry_problems:
    print(f"  ✗ 升档判的抓到复核不过（{len(registry_problems)} 处；登记表 {registry_path}）：")  # gate-lint:summary
    for entry in registry_problems[:20]:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：登记行与日志只由驱动写，手改、拷来的都不算；照点的那一样查（日志被删或改了、基线对不上），"
          "再让驱动点名重判那几条（--item <序号> --rerun，前缀与起法照 .claude/skills/crash-test/commit-stages.md 第 2b 步），跑完再跑这一道；"
          "倍数不够的加大 --timeout-floor / --memory-cap 再 --rerun")
    sys.exit(1)
print(f"  ✓ 广谱变异提交门禁判绿：此刻源码上 {total} 条每一条都有单元键相同的结论，没红 0 条，没记清单的单元 0 个，拒算 {refused} 条（有键相同的现判结论）；升档判的抓到 {len(escalated_lines)} 条、登记行逐行复核过；判定库 {mac_rows} 行逐行核过带密钥的校验列")
PY
python_exit=$?
rm -f "${output:?}"
exit "$python_exit"
}

if ( stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
exit "$stage_exit"
