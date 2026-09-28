#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录、跑研究脚本与共用库自己的自证、起各道门禁的 --list；research-gate-lint 格调的上游 gate-lint.sh、shell-lint.sh 要 gawk，缺了由它们自己拒绝（退 78），那一格按红记，不交给 gate.sh 预判
# gate-stage: 工具层自检（十一格：研究脚本的自证都过、有阶段在跑；项目规则清单；阶段归属表按格登记；钩子注册与写范围表与定义头；改动范围只许一份取法；研究脚本与钩子的拒绝出路与 shell 纪律；阶段声称的样本与编造的日期；kb 登记表；门禁结构；环境检查排在跑测试的阶段之后）
# gate-category: 代码类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 里的两项（崩溃点重放、模型对拍）判的是被测代码的行为，这一道只判工具层自己，一项都不覆盖。
# gate-similar: checker-tier-research-build-and-replay.sh 它编 research/ 的装置、跑它们的单测，也先赋变量再 "$变量" --selftest 跑几份脚本的自证；这一道的 research-script-selftests 格跑的是 runner 表里研究脚本的自证、research-script-selftest-coverage 格核每份实现了 --selftest 的都有阶段在跑，不编译
# gate-similar: harness-model-differential-and-scenarios.sh 它在真仓上判 harness 用例一条一个场景，crash-case-check.py 的判别力由这一道 research-script-selftests 格跑它的 --selftest；判的对象一个是用例源码，一个是脚本的自证
# gate-similar: doc-text.sh 也扫全仓文件，但判的是文本里登记过的禁用写法与旧术语；这一道 fixture-claims 格判文件名与样本目录里编出来的日期、阶段头部声称的样本在不在
# gate-similar: code-source-discipline.sh 它判实验二进制的变异表、绝对值断言与读数纪律，对象是实验源码；crates-mutation-rows.py 与 check-segment-registry.py 这类脚本的自证由这一道 research-script-selftests 格替它们跑
# gate-similar: doc-process-records.sh 它是改动范围共用取法的使用者之一，判三方与同步留下的记录；这一道 change-range-single-source 格判每个阶段只经那一份取法、列路径带 quotepath，对象是门禁阶段的源码
# gate-similar: doc-experiments.sh 它也是改动范围共用取法的使用者，判实验页与产物；这一道只判工具层自己（脚本、钩子、登记表、定义头、门禁结构与次序），不读实验页
# gate-similar: doc-registries.sh 它判欠账表与里程碑收口表这类 kb 登记表；这一道判工具层自己的登记（阶段归属表、写范围表、kb 登记表对目录）与自证
# gate-similar: harness-test-environment.sh 它判这台机器此刻有没有残留设备、挂载、盘满；这一道 stage-order 格只判它的文件名排在跑测试的阶段之后，不读机器，放进它里面就成了它判自己
# gate-similar: hooks-registered.sh 上游这一道判 settings.json 里注册的钩子在、hook-events 挂全、带 --selftest 的自检通过；这一道 agent-write-scope 格判的是项目点名的那几个钩子挂在要的 matcher 上、写范围表与定义双向一致、定义头，钩子的自证只归它跑
# gate-similar: manifest.sh 上游这一道判规范包自己的 CLAUDE.md 与 rules/ 对得上；这一道 rules-manifest 格判项目本地 .claude/rules/ 与项目 CLAUDE.md 的 @ 引用，射程不重叠
# gate-similar: stage-selftest.sh 上游这一道拿 fixtures 下的红绿样本证明本地阶段会红；这一道 fixture-claims 格只判阶段头部的声称与样本目录对不对得上，gate-structure 格只判结构（格名表、--list、--check），都判不了样本本身有没有判别力
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十一格按下面的次序全跑，格名写错退 2）：
# gate-cell: research-script-selftests runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
# gate-cell: research-script-selftest-coverage research/scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
# gate-cell: rules-manifest 项目规则文件与 CLAUDE.md 的 @ 引用逐项相等
# gate-cell: stage-owners 阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根
# gate-cell: agent-write-scope 项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份
# gate-cell: change-range-single-source 门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码
# gate-cell: research-gate-lint 研究脚本、钩子与 .claude/scripts 的拒绝出路、shell 纪律、执行位，终止进程只许点名一个
# gate-cell: fixture-claims 阶段头部声称的样本真的在；文件名与样本正文里的日期不许是编的
# gate-cell: kb-registry kb 目录里的每一份都在 CLAUDE.md「项目本地事实」表里有一行
# gate-cell: gate-structure 每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2
# gate-cell: stage-order 环境检查那一道排在门禁目录里全部跑测试的阶段之后
#
# 前九格原是八道阶段（研究脚本自证、项目规则清单、阶段归属、写范围、改动范围取法、研究脚本门禁纪律、样本声称、kb 登记），2026-09-28 按
# .claude/singlefs-ai-sop/rules/sop-first.md「加门禁或钩子之前，先找已有的」第 2 条并成一道：都判工具层自己
# （研究脚本、钩子、门禁阶段、登记表、agent 定义头），其中四格共用同一份目录与清单表的双向比对（写成下面的 MANIFEST_LIBRARY）。
# 同日按判决 research/prompts/gate-shrink-r1-main-verification.md「采纳的改法」改成共用库的结构：研究脚本自证那一格拆成跑 runner 表与核覆盖两格
# （原来样本靠 .selftest-coverage-only 标记只判覆盖那一半，拆开以后样本用 .gate-cells 点名覆盖那一格），另加 gate-structure、stage-order 两格。
# 这一轮改了判法的只有这几处：agent-write-scope 不再跑钩子的 --selftest（3-B2）；research-script-selftest-coverage 认「有 --selftest」只认
# .sh / .py 代码里的（3-B5）；research-script-selftests 分批并行（3-B7）；两处认阶段的 glob 从「两位数开头」改成门禁目录顶层全部 *.sh（第 4 问）；
# stage-owners 改认按格登记的表（采纳的改法第 1 条）。别的格的判据、出路、成功行里报的数、射程与「管不到的」照原样留着。
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析自己的项目根。
#
# 判别力：fixtures/code-tooling.sh/ 下每一格至少一红一绿，每份样本根放 .gate-cells 点名它判的那一格（只跑那一格，别的格不掺进来）：
#   research-script-selftests          runner-red、runner-green
#   research-script-selftest-coverage  red、green、coverage-claims-red、coverage-claims-green
#   rules-manifest                     rules-manifest-red、rules-manifest-green
#   stage-owners                       stage-owners-red、stage-owners-green
#   agent-write-scope                  agent-write-scope-red、agent-write-scope-green
#   change-range-single-source         change-range-single-source-red、change-range-single-source-green
#   research-gate-lint                 research-gate-lint-red、research-gate-lint-green
#   fixture-claims                     fixture-claims-red、fixture-claims-unnumbered-red、fixture-claims-cells-red、fixture-claims-green
#   kb-registry                        kb-registry-red、kb-registry-green
#   gate-structure                     gate-structure-red、gate-structure-green
#   stage-order                        stage-order-red、stage-order-green
# 每一格的样本放了什么，写在那一格的注释里。原来八格的样本搬进来时每份八格全跑，绿样本为此补了同一副架子
# （research/scripts/vm-bench.sh、stage-owners.tsv、定义、settings.json、写范围表这些）；现在有 .gate-cells，那副架子留着不碍事，want 不变。
# 汇总与 --check 的弄坏开关归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0）。
# 格的判法各有弄坏开关 CODE_TOOLING_BREAK=<项>，每一项都让某一格的样本判错：
#   runner-exit-swallowed    research-script-selftests 回读时把每条的退出码都当 0：runner-red 判绿
#   claims-any-file          research-script-selftest-coverage 退回「全文出现 --selftest 就算声称有」：coverage-claims-green 判红
#   numbered-stages-only     research-script-selftest-coverage 与 fixture-claims 只认两位数开头的阶段：coverage-claims-green 判红、fixture-claims-unnumbered-red 退 77
#   bare-red-green-only      fixture-claims 退回只认裸 red / green 样本目录、不按格数红绿：fixture-claims-green 判红（cells-covered.sh 成了空壳）、fixture-claims-cells-red 里那条「有格缺红或缺绿」的 want 找不到
#   owner-rows-ignored       stage-owners 不判「--list 出的每一格都有一行」：stage-owners-red 里那几条 want 找不到
#   structure-exit-ignored   gate-structure 不看 gate-structure-check.py 的退出码：gate-structure-red 判绿
#   stage-order-ignored      stage-order 不比次序：stage-order-red 判绿
# agent-write-scope 格另有它原来的开关 AGENT_WRITE_SCOPE_BREAK=any-model（关掉定义 model 取值那一条），名字没改。
#
#   bash .claude/gate.d/code-tooling.sh [项目根]                                十一格都跑
#   bash .claude/gate.d/code-tooling.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-tooling.sh --check <格名>[,<格名>…] [项目根]       只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd -P)"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"

stage_cell research-script-selftests cell_research_script_selftests "runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）" \
  "按红的那条自证给的下一步修被测脚本，再单独跑那一条命令看它转绿；runner 表在这一道的 SELFTEST_RUNNERS"
stage_cell research-script-selftest-coverage cell_research_script_selftest_coverage "research/scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑" \
  "把没人跑的自证加进这一道的 SELFTEST_RUNNERS；确实不该每轮跑的登记进 NOT_RUN_HERE 并写明为什么；过期的豁免删掉"
stage_cell rules-manifest cell_rules_manifest "项目规则文件与 CLAUDE.md 的 @ 引用逐项相等" \
  "补上 CLAUDE.md 里缺的 @.claude/rules/<文件> 引用（写在代码围栏与行内代码之外），或删掉悬空引用；两侧逐项相等"
stage_cell stage-owners cell_stage_owners "阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根" \
  "照各道门禁的 --list 在 .claude/gate.d/stage-owners.tsv 逐格补行、改名、删行（四列：门禁文件名、格名、判红时派给谁修、为什么）；列不出格的先修 gate-structure 格报的结构；不认项目根的阶段加 ROOT=\"\${1:-…}\" 与 cd \"\$ROOT\""
stage_cell agent-write-scope cell_agent_write_scope "项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份" \
  "照上面逐条的出路改 .claude/settings.json 的 hooks、.claude/hooks/agent-write-scope.tsv 或定义的 frontmatter；共用模块的函数从模块导入，不另写一份"
stage_cell change-range-single-source cell_change_range_single_source "门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码" \
  "基准与改动范围只经 research/scripts/changed-paths.sh 的 gate_diff_base / gate_changed_paths / gate_added_lines 取，列路径的 git 带 -c core.quotepath=false，调共用取法判退出码"
stage_cell research-gate-lint cell_research_gate_lint "研究脚本、钩子与 .claude/scripts 的拒绝出路、shell 纪律、执行位，终止进程只许点名一个" \
  "拒绝补出路（sop-first.md「每一条拒绝都必须给出下一步」），shell 纪律照 command-safety.md，终止进程照 .claude/hooks/bash-command-detector.sh 文件头 ⑥ 只点名一个自己起的"
stage_cell fixture-claims cell_fixture_claims "阶段头部声称的样本真的在；文件名与样本正文里的日期不许是编的" \
  "建起声称的样本目录或改掉那句声称，删掉孤儿样本目录、补齐 expect；日期写真实发生的那一天，红样本要编造日期就在 setup.sh 里现造"
stage_cell kb-registry cell_kb_registry "kb 目录里的每一份都在 CLAUDE.md「项目本地事实」表里有一行" \
  "往 CLAUDE.md「## 项目本地事实」表加一行（第一格写反引号包起来的路径），文件搬家或删掉就同步改或撤那一行"
stage_cell gate-structure cell_gate_structure "每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2" \
  "改用 .claude/gate.d/lib/stage-cells.sh（写法在它的文件头）：文件头逐格写 # gate-cell: <格名> <判什么>，每格一个函数用 stage_cell 登记；规则在 .claude/rules/verification.md「门禁的结构」"
stage_cell stage-order cell_stage_order "环境检查那一道排在门禁目录里全部跑测试的阶段之后" \
  "给排在环境检查后面的那几道跑测试的阶段改名，让它们按 gate.sh 的 find … | sort 排到 harness-test-environment.sh 前面；环境检查那一道改了名就同步改这一格的 ENVIRONMENT_STAGE"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 --list 看。"
      exit 2
      ;;
    *)
      if [[ -n "$root_argument" ]]; then
        echo "  ✗ 参数里有两个项目根：$root_argument 与 $1"
        echo "     → 怎么办：只给一个项目根，外加 --check <格名>[,<格名>…]；项目根不给就取这一道脚本往上两级。"
        exit 2
      fi
      root_argument="$1"
      shift
      ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# 被判的根是样本（不是这一道所在的仓、也不在 gate.sh --staged 的临时树里）：与共用库认 .gate-cells 同一种判法。
# research-script-selftests 格只在样本里认样本根的 .selftest-runners（换掉 runner 表），真仓与 --staged 下放了它也不认。
judged_root_is_sample() {
  [[ "$(pwd -P)" != "$STAGE_CELLS_STAGE_REPOSITORY" && -z "${GATE_STAGED_FROM:-}" ]]
}

# 目录 ↔ 清单表的双向比对（rules-manifest、stage-owners、agent-write-scope、kb-registry 四格共用；原是门禁目录里一份单独的 lib-manifest.py，只有这四格调它）。
# 四格判的是同一种形状：盘上有一批东西（规则文件、门禁阶段、agent 定义、kb 文件），
# 一张清单表登记它们（CLAUDE.md 的 @ 引用、stage-owners.tsv、agent-write-scope.tsv、「项目本地事实」表）。
# 两个方向都要判：盘上有、清单没登记的，读的人不知道它是什么；清单登记了、盘上没有的，
# 清单指向空处，让人以为那一格有人管着。只判一个方向的比对，另一个方向永远不红。
# 各格自己决定「盘上有什么」「清单登记了什么」「怎么算登记上了」，这里只管比对与读表：
#   two_way(on_disk, registered, is_registered=None, exists=None) -> (unregistered, dangling)
#   table_rows(path) -> [(行号, 去掉换行的整行, 按制表符切开的各格)]
# 各格的 python 段把它当第一个参数收进来，装成一个模块：
#   manifest = types.ModuleType("lib_manifest"); exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
MANIFEST_LIBRARY="$(cat <<'PY_MANIFEST'
def two_way(on_disk, registered, is_registered=None, exists=None):
    """返回 (unregistered, dangling)，两份都保持输入的次序（要排序由调用方先排好再传）。

    unregistered：on_disk 里清单没登记的；is_registered(item) 缺省是「item 在 registered 里」。
    dangling：registered 里指向盘上不存在的东西的；exists(entry) 缺省是「entry 在 on_disk 里」。
    两个判定各给各的：盘上那一侧与清单那一侧的集合可以不同
    （例：agent-write-scope 格只要求有 Write / Edit 的定义登记，却要求表里每个名字都有定义）。
    """
    on_disk = list(on_disk)
    registered = list(registered)
    registered_set = set(registered)
    on_disk_set = set(on_disk)
    if is_registered is None:
        is_registered = registered_set.__contains__
    if exists is None:
        exists = on_disk_set.__contains__
    unregistered = [item for item in on_disk if not is_registered(item)]
    dangling = [entry for entry in registered if not exists(entry)]
    return unregistered, dangling


def table_rows(path):
    """制表符分隔的清单表：空行与 # 开头的行是注释，不返回；其余每行给 (行号, 去掉换行的整行, 各格)。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if not line.strip() or line.startswith("#"):
                continue
            rows.append((line_number, line, line.split("\t")))
    return rows
PY_MANIFEST
)"

# 去掉注释与出路文字之后剩下的代码（research-script-selftest-coverage、stage-order 两格共用）：认「脚本实现了 --selftest」
# 「阶段在跑某份脚本的自证」「阶段在跑测试」都只认代码，注释行不算，出路文字里提到的也不算。
# 各格的 python 段把它当一个参数收进来，照 MANIFEST_LIBRARY 那样装成模块：
#   code_text = types.ModuleType("lib_code_without_text"); exec(compile(sys.argv[i], "code-without-text-library", "exec"), code_text.__dict__)
CODE_WITHOUT_TEXT_LIBRARY="$(cat <<'PY_CODE_WITHOUT_TEXT'
import re

# 出路文字：echo / printf / howto / bad / die / say / print( 后面带引号的参数（可以连着几段）。里面提到的「x.py --selftest」不是在跑它
OUTPUT_TEXT = re.compile(r'''(?:\b(?:echo|printf|howto|bad|die|say)\b(?:\s+-\w+)*|\bprint\()(?:\s*f?(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'))+''')


def code_of(text):
    """去掉 # 开头的行（注释与 #!）与出路文字之后剩下的代码。"""
    code = "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))
    return OUTPUT_TEXT.sub(" ", code)
PY_CODE_WITHOUT_TEXT
)"

# ── 格 research-script-selftests：runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
#
# 判据：SELFTEST_RUNNERS 里的每一条自证都要退 0。表里有哪几份、哪几份不在表里要登记为什么，归下一格 research-script-selftest-coverage。
# 为什么：这几份自证此前都写着，却没有任何门禁阶段在跑（2026-09-12 现查 gate.d 与 .claude/scripts 零处调用）——自证只在写它的那天被跑过一次，
# 之后脚本改坏了也没人知道。2026-09-12 实测的两个坑都住在这里：
# ask-local.sh 判红时正文照样打到 stdout（一份作废输出顶着 -output-s1.md 落盘），
# 以及取法用不加引号的 $SPECS 传过 shell 被拆词（checklist-specs.py 就是为它写的）。
#
# check-segment-registry.py 不是三方论证脚本，但同一个道理成立：doc-registries.sh 的 segment-registry 那一格（段序列登记表与 E142 产物逐字比对）的
# 判别力样本只放三样合成输入，钉活代码的那几句与真产物的解析靠它自己的 --selftest 拿真文件测；
# 那份 --selftest 同样要有人在门禁里替它复跑，不然只在写它的那天跑过一次。
# 共用库 lib/stage-cells.sh 住在门禁目录的 lib/ 下，gate.sh 不把它当阶段跑，它的自证只在这张表里跑这一次。
#
# 分批并行（判决 research/prompts/gate-shrink-r1-main-verification.md 3-B7；攻方量过，research/prompts/gate-shrink-r1-opus-output.md 第七节：
#   45 条顺序跑合计约 400 秒，layer0-shard-run.sh 与 agent-watch.py 两条占 68%；按耗时分批、每批 8 条并行 137–178 秒，只两次观测，不算稳定）：
#   SELFTEST_RUNNERS_STARTED_ALONE 里那几条先各自单独起、一直跑到完；其余按表次序每批 N 条并行、一批全收回再起下一批，
#   N = 线程上限（SINGLEFS_THREAD_CAP，research/scripts/capped.sh 设的那个；不设或不是正整数就取 nproc）减去单独起的条数，至少 1。
#   每条的输出写自己的文件、退出码写自己的文件；收束时逐个 wait 那一批的进程号，全部跑完按表次序逐条回读，
#   数一遍退出码文件，份数与派出去的条数对不上整格红（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
#   没过的那几条按表次序逐条列出原样输出。单独起的那几条都要在 runner 表里：表里改了名忘了改它，就判红，不悄悄少跑。
#   几份自证互相抢同一份状态（同一个临时路径、同一个端口）的，并行下才会红：那一条挪进 SELFTEST_RUNNERS_STARTED_ALONE 单独起，或修它的自证各用各的临时目录。
#
# 样本：被判的根是样本时（judged_root_is_sample，与共用库认 .gate-cells 同一种判法），样本根的 .selftest-runners
#   （一行一条命令，空行与 # 开头的行不算）换掉 runner 表：表里那几十份自证要真仓里的脚本，装不进样本；真仓与 --staged 下放了它也不认。
#   fixtures/code-tooling.sh/runner-red 的表里四条、第二条必红（退 1 并打一句原因），必须判红、点名那一条并列出它的输出，
#   另外三条照样跑完、报 4 条里 1 条没过；runner-green 三条都退 0，判绿并报跑了 3 条。
#   样本里没有单独起的那一路（样本的表不认 SELFTEST_RUNNERS_STARTED_ALONE），那一路只在真仓里走。
SELFTEST_RUNNERS=(
  "bash research/scripts/ask-local-selftest.sh" "python3 research/scripts/checklist-specs.py --selftest"
  "python3 research/scripts/quote-kb.py --selftest" "python3 research/scripts/kb-sections.py --selftest"
  "python3 research/scripts/check-segment-registry.py --selftest" "python3 research/scripts/replace-once.py --selftest"
  "python3 research/scripts/replace-batch.py --selftest" "python3 research/scripts/e152-tables.py --selftest"
  "python3 research/scripts/agent-watch.py --selftest" "python3 research/scripts/quote-rust-items.py --selftest"
  "python3 research/scripts/sweep-term.py --selftest" "python3 research/scripts/archive-past-rounds.py --selftest"
  "python3 research/scripts/insert-row.py --selftest" "bash research/scripts/cache-keepalive.sh --selftest"
  "bash research/scripts/watch.sh --selftest" "python3 research/scripts/stale-candidates.py --selftest"
  "python3 research/scripts/stale-candidates.py --benchmark" "python3 research/scripts/test-environment-check.py --selftest"
  "bash research/scripts/change-touches-crates.sh --selftest" "bash research/scripts/verify-citations.sh --selftest"
  "python3 research/scripts/agent-handover.py --selftest" "python3 research/scripts/decision-slim-check.py --selftest"
  "python3 research/scripts/pdf-text.py --selftest" "python3 research/scripts/rules-sweep-audit.py --selftest"
  "bash research/scripts/stage-must-run.sh --selftest" "bash research/scripts/changed-paths.sh --selftest"
  "bash research/scripts/gate-staged.sh --selftest" "python3 research/scripts/admission.py --selftest"
  "bash research/scripts/capped.sh --selftest" "bash research/scripts/stage-run-or-skip.sh --selftest"
  "bash research/scripts/layer0-shard-run.sh --selftest" "bash research/scripts/mutation-shard-run.sh --selftest"
  "bash research/scripts/multi-host-run.sh --selftest" "bash research/scripts/run-with-memory-cap.sh --selftest"
  "bash research/scripts/mutate.sh --selftest" "python3 research/scripts/cite-check.py --selftest"
  "python3 research/scripts/kb-spec-check.py --selftest" "python3 research/scripts/crash-case-check.py --selftest"
  "bash research/scripts/prove-red.sh --selftest" "python3 research/scripts/apply-writer-patch.py --selftest"
  "python3 research/scripts/closeout-status.py --selftest" "python3 research/scripts/compile-then-swap.py --selftest"
  "python3 research/scripts/corruption-check.py --selftest" "python3 research/scripts/crates-mutation-rows.py --selftest"
  "bash .claude/gate.d/lib/stage-cells.sh --selftest" "python3 research/scripts/gate-structure-check.py --selftest"
  "python3 research/scripts/migrate-changelog-format.py --selftest"
)
SELFTEST_RUNNERS_STARTED_ALONE=("bash research/scripts/layer0-shard-run.sh --selftest" "python3 research/scripts/agent-watch.py --selftest")

cell_research_script_selftests() {
runners=("${SELFTEST_RUNNERS[@]}")
table_source="这一道的 SELFTEST_RUNNERS"
started_alone=("${SELFTEST_RUNNERS_STARTED_ALONE[@]}")
if judged_root_is_sample && [[ -f .selftest-runners ]]; then
  runners=()
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    runners+=("$line")
  done < .selftest-runners
  table_source="样本根的 .selftest-runners（$ROOT 是样本，不是这一道所在的仓）"
  started_alone=()
fi
if ((${#runners[@]} == 0)); then
  echo "  ✗ runner 表一条都没有（取自 $table_source）"
  echo "     → 怎么办：跑到 0 条自证而报绿，与判过了一模一样；表写在这一道的 SELFTEST_RUNNERS，样本写在样本根的 .selftest-runners。"
  exit 1
fi
declare -A in_table=() alone_set=()
for runner in "${runners[@]}"; do in_table[$runner]=1; done
for runner in ${started_alone[@]+"${started_alone[@]}"}; do
  if [[ -z "${in_table[$runner]+set}" ]]; then
    echo "  ✗ SELFTEST_RUNNERS_STARTED_ALONE 里的「$runner」不在 runner 表里"
    echo "     → 怎么办：表里改了名或删了那一条，就同步改 SELFTEST_RUNNERS_STARTED_ALONE；对不上的那一条会被悄悄少跑。"
    exit 1
  fi
  alone_set[$runner]=1
done

thread_cap="${SINGLEFS_THREAD_CAP:-}"
if [[ ! "$thread_cap" =~ ^[1-9][0-9]*$ ]]; then
  [[ -n "$thread_cap" ]] && echo "  ! SINGLEFS_THREAD_CAP=$thread_cap 不是正整数，并行度按 nproc 取"
  thread_cap="$(nproc)"
fi
batch_size=$((thread_cap - ${#alone_set[@]}))
((batch_size >= 1)) || batch_size=1

scratch="$(mktemp -d)" || { echo "  ✗ 建不了放逐条输出与退出码的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
started_pids=()
# start_selftest <表里的序号> <命令>：后台起一条，输出写 <序号>.out、退出码写 <序号>.exit，进程号记进 started_pids
start_selftest() {
  # runner 表每条是「解释器 脚本 参数」，按空白切词起，与原来逐条顺序跑时同一个起法
  { $2 > "$scratch/$1.out" 2>&1; printf '%s\n' "$?" > "$scratch/$1.exit"; } &
  started_pids+=("$!")
}
for index in "${!runners[@]}"; do
  [[ -n "${alone_set[${runners[$index]}]+set}" ]] && start_selftest "$index" "${runners[$index]}"
done
alone_pids=(${started_pids[@]+"${started_pids[@]}"})
batch=(); batches=0
run_selftest_batch() {
  local index pid
  started_pids=()
  for index in "${batch[@]}"; do start_selftest "$index" "${runners[$index]}"; done
  # 退出码在各自的 .exit 文件里，全部跑完按表次序逐条回读；这里只等这一批收齐
  for pid in "${started_pids[@]}"; do wait "$pid"; done
  batches=$((batches + 1))
  batch=()
}
for index in "${!runners[@]}"; do
  [[ -n "${alone_set[${runners[$index]}]+set}" ]] && continue
  batch+=("$index")
  ((${#batch[@]} == batch_size)) && run_selftest_batch
done
((${#batch[@]})) && run_selftest_batch
for pid in ${alone_pids[@]+"${alone_pids[@]}"}; do wait "$pid"; done

collected=0; failed_count=0; uncollected=()
for index in "${!runners[@]}"; do
  runner="${runners[$index]}"
  if [[ ! -f "$scratch/$index.exit" ]]; then uncollected+=("$runner"); continue; fi
  collected=$((collected + 1))
  rc="$(<"$scratch/$index.exit")"
  [[ "${CODE_TOOLING_BREAK:-}" == runner-exit-swallowed ]] && rc=0
  if [[ "$rc" != 0 ]]; then
    failed_count=$((failed_count + 1))
    echo "  ✗ $runner 没过（退出码 $rc）："
    sed 's/^/    /' "$scratch/$index.out"   # gate-lint:detail
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
  fi
done
rm -rf "${scratch:?}"
if ((collected != ${#runners[@]})); then
  echo "  ✗ runner 表派出去 ${#runners[@]} 条，只收回 $collected 份退出码；没收回的：${uncollected[*]}"
  echo "     → 怎么办：临时目录在跑的过程中被删了或写不进去；看 \$TMPDIR 再重跑这一格，收不全的一轮不算判过。"
  exit 1
fi
if ((failed_count)); then
  echo "  ✗ runner 表 ${#runners[@]} 条里 $failed_count 条没过（上面按表次序逐条列着）"
  echo "     → 怎么办：逐条按那份自证给的下一步修；只在并行下红、单跑绿的，是几份自证抢同一份状态，照这一格文件头那一段办。"
  exit 1
fi
echo "  ✓ research 脚本的自证都通过（这一格跑了 ${#runners[@]} 条，表取自 $table_source；单独起 ${#alone_pids[@]} 条，其余 $((${#runners[@]} - ${#alone_pids[@]})) 条按每批至多 $batch_size 条并行、共 $batches 批）"
}

# ── 格 research-script-selftest-coverage：research/scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
#
# 判据：research/scripts/ 里代码实现了 `--selftest` 的脚本，要么有阶段在调用它（上一格的 runner 表，
# 或 checker-tier-research-build-and-replay.sh 那种先赋给变量再调的写法），要么登记进 NOT_RUN_HERE 并写明为什么；
# NOT_RUN_HERE 里的每一行都要还指着一份实现了 --selftest、又没人跑的脚本，不然是过期的豁免。份数不在注释里写死，成功行现算。
# 为什么：同上一格——没人跑的自证只在写它的那天跑过一次。
#
# 「实现了 --selftest」（判决 3-B5，攻方 research/prompts/gate-shrink-r1-opus-output.md 第六节 M2 的 R3）：只认 .sh / .py，
# 去掉 # 注释行与出路文字之后，代码里还有 `--selftest` 才算。去掉的是同一套（下面的 code_of），与认「有阶段在跑它」一样。
# 原来全文出现 `--selftest` 就算，多认了只在注释里提别人的脚本与数据文件（2026-09-28 现算：54 份认 51 份，去掉的三份是
# layer0-shard-configuration-check.sh、layer0-shard-run-selftest.sh 两份只在注释里提、memory-peaks.tsv 一份数据文件）；
# 不列写法（== "--selftest"、case 分支、match 语句这些都认），新写一种实现自证的写法照样被认进来。
# 「有阶段在跑它」只认代码：注释行不算，出路文字也不算——echo / printf / howto / bad / die / say / print( 后面带引号的那几段
# 先去掉再认，一句「单跑 python3 research/scripts/foo.py --selftest 看它报什么」不是在跑它。
# 阶段是门禁目录顶层全部 *.sh（与 gate.sh 起的是同一批；原来只认两位数开头的，去编号之后每份都会被报成没人跑，只改一部分时又悄悄漏掉改了名的那几道）。
#
# 样本：fixtures/code-tooling.sh/red 放一份只在某个阶段的 echo 出路里被提到的 --selftest 脚本，必须报它没人跑；
# green 放一份真被阶段调用的与一份登记在 NOT_RUN_HERE 的，判绿并报对数。
# coverage-claims-green 放一份只在注释与出路文字里提 --selftest 的 .py、一份提到它的 .tsv（两份都不算实现了），
# 与一份由不带编号的阶段 tooling-runs.sh 在跑的 .sh，判绿并报 2 份中 1 份（另一份是 NOT_RUN_HERE 那一份）；
# coverage-claims-red 放一份用 match 语句实现 --selftest、没人跑的 .py，必须报它没人跑。
NOT_RUN_HERE=(
  "research/scripts/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁"
)
cell_research_script_selftest_coverage() {
# 被扫集合现算，不手抄份数（show-me-test.md「跳过清单要与被扫集合出自同一份数据、现算」）。
# 认两种调用写法：脚本名后面直接跟 --selftest，或先赋给变量再 "$变量" --selftest（checker-tier-research-build-and-replay.sh 那样）；注释行不算调用。
coverage="$(python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "${CODE_TOOLING_BREAK:-}" "${NOT_RUN_HERE[@]}" <<'PY_COVERAGE'
import glob, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
code_of = code_text.code_of
broken = sys.argv[2]
exempt = dict(row.split("\t", 1) for row in sys.argv[3:])
def implements_selftest(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    if broken == "claims-any-file":
        return "--selftest" in text
    return path.endswith((".sh", ".py")) and "--selftest" in code_of(text)
claimed = sorted(path for path in glob.glob("research/scripts/*")
                 if os.path.isfile(path) and implements_selftest(path))
run = set()
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
for stage in sorted(glob.glob(os.path.join(".claude/gate.d", stage_pattern))):
    code = code_of(open(stage, encoding="utf-8").read())
    variables = dict(re.findall(r'^\s*(\w+)="[^"\n]*?([\w.-]+\.(?:py|sh))"', code, re.M))
    for script in claimed:
        base = os.path.basename(script)
        if re.search(re.escape(base) + r'["\']?\s+--selftest', code):
            run.add(script)
        for variable, target in variables.items():
            if target == base and re.search(r'"\$' + variable + r'"\s+--selftest', code):
                run.add(script)
print(f"CLAIMED\t{len(claimed)}\tRUN\t{len(run)}")
for script in claimed:
    if script not in run and script not in exempt:
        print(f"MISSING\t{script}")
for path, why in exempt.items():
    if path not in claimed or path in run:
        print(f"STALE\t{path}\t{'已经有阶段在跑它' if path in run else '它的代码里已经没有 --selftest，或文件不在了'}")
    else:
        print(f"EXEMPT\t{path}\t{why}")
PY_COVERAGE
)" || { echo "  ✗ 自证覆盖的对账脚本没跑成"; echo "  → 怎么办：看上面 python 的报错修这一段；对账没跑等于这一格没验。"; exit 1; }
if grep -q '^MISSING' <<<"$coverage"; then
  echo "  ✗ 这些 research 脚本声称有 --selftest，却没有任何门禁阶段在跑它："
  grep '^MISSING' <<<"$coverage" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "  → 怎么办：把它加进 research-script-selftests 格的 SELFTEST_RUNNERS；确实不该每轮跑的，登记进 NOT_RUN_HERE 并写明为什么——没人跑的自证只在写它的那天跑过一次。"
  exit 1
fi
if grep -q '^STALE' <<<"$coverage"; then
  echo "  ✗ NOT_RUN_HERE 里有过期的豁免："
  grep '^STALE' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'   # gate-lint:detail
  echo "  → 怎么办：从 NOT_RUN_HERE 里删掉这一行；留着它会让人以为那份脚本还被绕开着。"
  exit 1
fi
read -r _ claimed_count _ run_count < <(grep '^CLAIMED' <<<"$coverage")
echo "  ✓ research/scripts/ 里声称有 --selftest 的 $claimed_count 份中 $run_count 份有门禁阶段在跑（只认 .sh / .py 代码里的 --selftest）"
echo "    没跑的 $(grep -c '^EXEMPT' <<<"$coverage") 份（登记在这一格的 NOT_RUN_HERE）："
grep '^EXEMPT' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'
}

# ── 格 rules-manifest：项目规则清单一致
#
# 还 checks-owed.md C4（项目规则清单不一致）。
# 判据：`.claude/rules/` 下的文件集合，必须与 CLAUDE.md 里 `@.claude/rules/` 的引用集合逐项相等。
# 任一侧多出一项即判红——多出的那一份规则**不会被读进上下文**，等于没写。
# 代码围栏（``` 或 ~~~ 起止）与行内代码（反引号括起的一段）里的 @ 不算引用：Claude Code 不在代码里展开 @ 导入，
# 只在那里提到一次的规则文件照样读不进上下文。
#
# ⚠️ 上游 manifest.sh 只覆盖 SOP 包自己的 CLAUDE.md + rules/，
# 项目本地那一份此前没有任何检查在看（C4 的原文）。
# 双向比对用这一道文件头的 MANIFEST_LIBRARY，与 stage-owners、agent-write-scope、kb-registry 三格同一份代码。没有 .claude/rules 或没有 CLAUDE.md ⇒ 无对象可判，退 77（不记通过）。
# 样本：fixtures/code-tooling.sh/rules-manifest-red 放一份没被引用的规则、一份只在代码围栏里与一份只在行内代码里被 @ 的规则，必须逐个报出；rules-manifest-green 两侧逐项相等，判绿。
cell_rules_manifest() {
python3 - "$MANIFEST_LIBRARY" <<'PY'
import fnmatch, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)

RULES = ".claude/rules"
MD = "CLAUDE.md"
if not os.path.isdir(RULES):
    print(f"  ! 没有 {RULES}，这一格无对象可判")
    sys.exit(77)
if not os.path.isfile(MD):
    print(f"  ! 找不到 {MD}，这一格跳过")
    sys.exit(77)

on_disk = sorted(name for name in os.listdir(RULES) if fnmatch.fnmatchcase(name, "*.md"))
def outside_code(text):
    """去掉代码围栏里的行与行内代码：那里的 @ 只是在说这个写法，Claude Code 不展开它。"""
    kept, fence = [], None
    for line in text.split("\n"):
        opening = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence is None and opening:
            fence = opening.group(1)
            continue
        if fence is not None:
            if re.match(r"^ {0,3}" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}\s*$", line):
                fence = None
            continue
        kept.append(re.sub(r"(`+)(?:(?!\1).)+?\1", " ", line))
    return "\n".join(kept)

with open(MD, encoding="utf-8", errors="replace") as handle:
    referenced = sorted({os.path.basename(match) for match in re.findall(r"@\.claude/rules/[A-Za-z0-9._-]+\.md", outside_code(handle.read()))})
missing, extra = manifest.two_way(on_disk, referenced)

if missing:
    print(f"  ✗ 这些规则文件存在，但 {MD} 没有 @ 引用它们——不会被读进上下文，等于没写：")   # gate-lint:detail
    for name in missing:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if extra:
    print(f"  ✗ {MD} 引用了这些规则，但文件不存在——引用悬空：")
    for name in extra:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if missing or extra:
    print("     → 怎么办：补上缺的 @ 引用，或删掉悬空引用；两侧必须逐项相等。")
    sys.exit(1)
print(f"  ✓ 项目规则清单一致（{len(on_disk)} 条，与 {MD} 的引用逐项相符）")
PY
}

# ── 格 stage-owners：阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根
#
# 判据：`.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁文件名、格名、判红时派给谁修（agent 名，逗号分隔）、为什么
# （.claude/rules/verification.md「门禁的结构」）。提交前的整轮门禁跑全部门禁，哪一格红就按这张表派给谁修。任一条不成立判红：
#   ① 门禁目录顶层每道门禁（`*.sh` 普通文件，共享 gate.sh 当本地阶段跑的就是这一批）`--list` 出的每一格，在表里恰好一行；
#      `--list` 的起法与 research/scripts/gate-structure-check.py 同一份（导入它的 header_cells、make_shims、start_stage：
#      项目根、--list、--force，清掉门禁的握手变量，cargo、qemu 这类重活换成替身，限时）；文件头没有格名表的门禁不起它
#      （起它就是全跑），与 --list 没退 0、起了重活、一格都没打的一起报「列不出格」，这几道的格登没登记判不了；
#   ② 表里每一行的门禁文件都在、那一格在那道门禁的 --list 里；
#   ③ 第三列每个 agent 名都有 `.claude/agents/<名字>.md`；
#   ④ 每一行四列齐全、没有空列；
#   ⑤ 每个阶段都认第一个参数当项目根：`ROOT="${1…`（或先解析选项再 `ROOT="${root_argument…`）、或直接 `cd "${1…`。
#      只看当前目录的阶段，被人从别的目录带着根目录参数调用时安静地判错地方，而 stage-selftest 既 cd 又传参，看不出来。
# 不再要求每道门禁登记给某个 agent 在干完活之后先跑：扫仓的门禁只在提交前由整轮门禁跑，这张表只回答红了派给谁。
#
# 为什么：归谁的清单原来在 CLAUDE.md 与 kb-scribe 定义里各手抄一份，没有任何东西盯着它们与目录同步——
# CLAUDE.md 那份 52 行清单里实验复跑那一道的说明已经与脚本头部对不上（records/2026-09-17-CLAUDE.md去冗余.md）。
# 按道登记时，一道里一格红把别的格的派修一起带走（判决 research/prompts/gate-shrink-r1-main-verification.md 第 2 问），所以按格登记。
# 表只写一份；新加一格忘了登记、删了一格或一道没改表、删了一个定义没改表，这一格当场红。
# ① ② 的双向比对与读表用这一道文件头的 MANIFEST_LIBRARY，与 rules-manifest、agent-write-scope、kb-registry 三格同一份代码。
# 样本：fixtures/code-tooling.sh/stage-owners-red 的表里有三列的行、同一格登记两行、没有定义的 agent、指向门禁里没有的格与指向不在的门禁的行，
# 目录里有一格没登记、一道没有格名表、一道 --list 退 1、一道不认第一个参数，必须逐个报出；
# stage-owners-green 两道门禁三格逐格登记，判绿并报门禁数、格数与 agent 数。
cell_stage_owners() {
python3 - "$MANIFEST_LIBRARY" "$STAGE_REPOSITORY/research/scripts/gate-structure-check.py" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, importlib.util, os, re, shutil, sys, tempfile, types
sys.dont_write_bytecode = True
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken = sys.argv[3]
table_path = ".claude/gate.d/stage-owners.tsv"
if not os.path.isfile(table_path):
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表，四列用制表符分隔：门禁文件名、格名、判红时派给谁修（agent 名，逗号分隔）、为什么；# 开头的行是注释。格名用 bash .claude/gate.d/<门禁> --list 看。")
    sys.exit(1)
try:
    specification = importlib.util.spec_from_file_location("gate_structure_check", sys.argv[2])
    structure = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(structure)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 导入不了 {sys.argv[2]}：{error}")
    print("     → 怎么办：各道门禁的 --list 经它起（替身拦重活、限时）；它随仓走，被删了或改坏了就从 git 找回来，别在这一格另写一份起法。")
    sys.exit(1)
ROOT_ARGUMENT_PATTERN = re.compile(r'\b(?:ROOT|root|REPOSITORY_ROOT|repository_root|PROJECT_ROOT)="?\$\{(?:1|root_argument)(?::-|\})|\bcd +"?\$\{?1\b')
# 与共享 gate.sh 认的本地阶段同一批：.claude/gate.d/ 顶层每个 *.sh 普通文件（它用 find -maxdepth 1 -name '*.sh'）
stage_files = sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh") if os.path.isfile(path))
root = os.getcwd()
cells_by_stage, unlisted = {}, []
scratch = tempfile.mkdtemp(prefix="stage-owners-list-")
try:
    shim_directory = structure.make_shims(scratch)
    trace = os.path.join(scratch, "shim-trace")
    for stage in stage_files:
        stage_path = os.path.join(root, ".claude", "gate.d", stage)
        if not structure.header_cells(stage_path):
            unlisted.append(f"{stage}：文件头没有格名表（# gate-cell: 行），不起它的 --list（起它就是全跑）")
            continue
        exit_code, stdout, stderr, called = structure.start_stage(stage_path, root, ["--list", "--force"], shim_directory, trace)
        if called:
            unlisted.append(f"{stage}：--list 起了重活（{'、'.join(called)}）")
        elif exit_code is None:
            unlisted.append(f"{stage}：--list 跑了 {structure.LIST_TIMEOUT_SECONDS} 秒还没退出")
        elif exit_code != 0:
            unlisted.append(f"{stage}：--list 退 {exit_code}；输出末几行：{structure.tail(stdout + stderr)}")
        else:
            cells = [line.split("\t", 1)[0].strip() for line in stdout.splitlines() if line.strip()]
            if cells:
                cells_by_stage[stage] = cells
            else:
                unlisted.append(f"{stage}：--list 一格都没打")
finally:
    shutil.rmtree(scratch, ignore_errors=True)

rows_by_cell = {}
malformed_rows, unknown_owner_rows = [], []
table = manifest.table_rows(table_path)
for line_number, line, fields in table:
    if len(fields) != 4 or not all(field.strip() for field in fields):
        malformed_rows.append(f"第 {line_number} 行：{line}")
        continue
    stage, cell, owners, _reason = (field.strip() for field in fields)
    rows_by_cell.setdefault((stage, cell), []).append(line_number)
    for owner in owners.split(","):
        if not os.path.isfile(f".claude/agents/{owner.strip()}.md"):
            unknown_owner_rows.append(f"第 {line_number} 行 {stage} {cell}：{owner.strip()}")
listed_cells = [(stage, cell) for stage in stage_files for cell in cells_by_stage.get(stage, [])]
def cell_exists(key):
    stage, cell = key
    if stage not in stage_files:
        return False
    if stage not in cells_by_stage:
        return True   # 列不出格的那几道另报，它们的行在不在判不了，不在这里重复报
    return cell in cells_by_stage[stage]
unregistered, dangling = manifest.two_way(listed_cells, list(rows_by_cell), exists=cell_exists)
if broken == "owner-rows-ignored":
    unregistered = []
duplicated = [f"{stage} {cell}（第 {', '.join(map(str, numbers))} 行）" for (stage, cell), numbers in rows_by_cell.items() if len(numbers) > 1]
stages_ignoring_the_root_argument = [stage for stage in stage_files if not ROOT_ARGUMENT_PATTERN.search(open(os.path.join('.claude/gate.d', stage), encoding='utf-8', errors='replace').read())]
failed = False
if malformed_rows:
    failed = True
    print("  ✗ 这些行不是四列、或有一列是空的：")  # gate-lint:summary
    for entry in malformed_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：门禁文件名、格名、判红时派给谁修（agent 名，逗号分隔）、为什么派给它；四列都要有内容。")
if unlisted:
    failed = True
    print("  ✗ 这些门禁列不出自己的格，它们的格登没登记判不了：")  # gate-lint:summary
    for entry in unlisted:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：先照 gate-structure 格的出路修结构（bash .claude/gate.d/code-tooling.sh --check gate-structure），--list 列得出格了再跑这一格。")
if unregistered:
    failed = True
    print("  ✗ 这些格在表里没有登记——判红时不知道派给谁修：")  # gate-lint:summary
    for stage, cell in unregistered:
        print(f"     {stage} 的 {cell}")  # gate-lint:detail
    print(f"     → 怎么办：在 {table_path} 给每一格加一行：门禁文件名、格名、判红时派给谁修、为什么；没有合适的 agent 就写 gate-triage。格名照 bash .claude/gate.d/<门禁> --list 抄。")
if duplicated:
    failed = True
    print("  ✗ 这些格在表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，几个 agent 名写在第三列、用逗号分隔。")
if dangling:
    failed = True
    print("  ✗ 表里登记了这些格，门禁目录里没有那道门禁、或那道门禁的 --list 里没有这一格——多半改了名或删了：")  # gate-lint:summary
    for stage, cell in dangling:
        where = "门禁目录里没有这道门禁" if stage not in stage_files else "那道门禁没有这一格"
        print(f"     {stage} {cell}（第 {', '.join(map(str, rows_by_cell[(stage, cell)]))} 行，{where}）")  # gate-lint:detail
    print("     → 怎么办：改成现在的门禁文件名与格名（bash .claude/gate.d/<门禁> --list 看），或删掉这一行。")
if unknown_owner_rows:
    failed = True
    print("  ✗ 这些 agent 名在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for entry in unknown_owner_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：改成已有定义的名字（文件名去掉 .md），或先建那个定义。")
if stages_ignoring_the_root_argument:
    failed = True
    print("  ✗ 这些阶段不认第一个参数当项目根（只看当前目录，从别的目录带着根目录参数调用时判的是错的地方）：")  # gate-lint:summary
    for stage in stages_ignoring_the_root_argument:
        print(f"     {stage}")  # gate-lint:detail
    print('     → 怎么办：在 preflight 那一行之后加 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}" 与 cd "$ROOT"（或者给每个路径加 "$ROOT/" 前缀），写法照 code-tooling.sh 自己。')
if failed:
    sys.exit(1)
owners = sorted({owner.strip() for _line_number, _line, fields in table for owner in fields[2].split(",")})
print(f"  ✓ 阶段归属表与各道门禁的格逐格一致（{len(stage_files)} 道门禁、{len(listed_cells)} 格，派给 {len(owners)} 个 agent；{len(stage_files)} 道都认第一个参数当项目根）")
PY
}

# ── 格 agent-write-scope：钩子注册在要的 matcher 上、写范围表与定义双向一致、每个定义带 omitClaudeMd 与「开工先读：」、共用切词与重型测试判定模块不另写一份
#
# 判据，任一条不成立判红：
#   ① `.claude/settings.json` 的 PreToolUse 里有一条 matcher 同时覆盖 Write 与 Edit、命令指向 `write-guard.sh` 的 hook（写范围与整份覆盖未跟踪文件两道合在里面）；
#   ② （空号：原来在这里跑 write-guard.sh 的 --selftest，见下面「钩子的自证」）；
#   ③ `.claude/agents/` 里 tools 含 Write 或 Edit 的每个定义，在 `.claude/hooks/agent-write-scope.tsv` 里至少有一条路径模式；
#   ④ 表里每个 agent 名都有定义，每行至少两列；
#   ⑤ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `bash-command-detector.sh` 的 hook（没超时的等待循环、run_in_background 里又自己放后台记进检出记录、放行：在跑的命令结不结束由主 agent 判断；起看门狗的错误写法——不用 run_in_background、带 `&` / nohup / setsid / disown、输出丢进 /dev/null——执行前拒绝）；
#   ⑥ PreToolUse 里有一条 matcher 覆盖 Agent、命令指向 `runner-dispatch-guard.sh` 的 hook（派执行员没点名岔路、续做没写还差的行会被拒；派 crash-verifier、gate-triage 之外的类型没写「重型测试：不跑」一行、缺定义要的输入、写范围之外的路径、实现员撞文件、opus 满上限会被拒；派 kb-scribe、implementation-writer 时要改的文件落在还没判完的三方轮开工快照里会被拒）；
#   ⑦ PreToolUse 里有一条 matcher 覆盖 SendMessage、命令指向 `continuation-guard.sh` 的 hook（给最近一次任务通知是 failed 或 killed 的、或已经交回过的子 agent 续做会被拒，上下文多大不拦）；
#   ⑧ `.claude/agents/` 里每个定义的 frontmatter 有 `omitClaudeMd: true`，正文有一行以「开工先读：」开头（不继承 CLAUDE.md 之后，要读的规则全靠这一行点名）；
#   ⑨ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向上游 SOP 的 `claude-hooks/pattern-process-guard.sh` 的 hook，且那个文件在
#     （按模式找进程在执行前拒绝；它的判别力由上游 selftest 的样本管，门禁「门禁自检」阶段跑它，这里只查注册着、文件在）；
#   ⑩ SessionStart 里有一条 matcher 覆盖 compact、一条 matcher 同时覆盖 startup 与 resume（可以是同一条），命令都指向 `session-start.sh`
#     （压缩上下文之后补回分支、未提交数、最近的记录与看门狗命令；会话启动或恢复时报近几小时内核因为内存不够杀过的进程，读不到内核日志报「没查成」）。
#   ⑫ PreToolUse 里有一条 matcher 覆盖 Bash、命令指向 `heavy-test-guard.sh` 的 hook（重型测试——层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置——子 agent 越出自己那一份就拒，主 agent 与 crash-verifier、gate-triage 不带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request` 也拒）。
#   ⑬ 两份共用模块顶层定义的每个函数名，只许在它自己那一份里定义：`.claude/hooks/` 下别的文件里再 `def <名字>(` 一份就判红——
#     切词模块 `.claude/hooks/lib_shell_words.py`（判 Bash 命令的 hook 手抄一份切词，一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出），
#     重型测试判定模块 `.claude/hooks/lib_heavy_tests.py`（heavy-test-guard.sh 与看门狗 research/scripts/agent-watch.py 共用 classify、classify_process、
#     judged_by_name 这些判定，手抄一份就是执行前的闸与进程这一层判得不一样）；名字从模块现读，读不到或一个函数都没有也判红。
#     不算的名字登记在 NOT_SHARED_JUDGMENT（每个 hook 都有自己的 selftest 入口，导入样板 load_sibling_module），登记了而模块里没有这个函数也判红。
#   ⑮ `.claude/agents/*.md` 的 frontmatter 里 `model:` 只许 opus、sonnet、haiku、inherit（行尾可带 YAML 注释）；AGENT_WRITE_SCOPE_BREAK=any-model 关掉这一条，判别力样本必须转红为绿。
#   ⑯ PreToolUse 里有一条 matcher 覆盖 SubagentHandback、命令指向 `handback-guard.sh` 的 hook（项目子 agent 交回正文超长、三方腿与核查员引文对不上、
#     书记员与执行员绿行没报数都拒）。
#   ⑭ PreToolUse 里有一条 matcher 覆盖 AskUserQuestion、命令指向 `ask-user-claim-guard.sh` 的 hook（弹窗问句与选项说明里
#     带断言词——不可能、造不出、从来不、一定、恒为这类——的句子，同一句里既没有出处也没写「推的，没量过」就拒）。
# 「命令指向某个 hook」按 shell 切词认：命令的第一个词（或 bash / sh / python3 / env 后面那个词）就是那个脚本，路径按结尾认；
#   只在注释、echo 的参数或别的词里提到它（`true # write-guard.sh`）不算注册。
# 「tools 含 Write 或 Edit」：frontmatter 没写 tools 的定义继承全部工具，算；`tools: [A, B]` 与 YAML 列表（下面几行 `- A`）照样认。
# ③ ④ 的双向比对与读表用这一道文件头的 MANIFEST_LIBRARY，与 rules-manifest、stage-owners、kb-registry 三格同一份代码。
#
# 钩子的自证（判决 research/prompts/gate-shrink-r1-main-verification.md 3-B2）：这一格原来逐个跑上面八个项目钩子的 --selftest，
# 与上游 hooks-registered.sh 重复——它对 settings.json 里注册的、文件里带 --selftest 字样的每个钩子都跑自证，射程还更宽，提交时由 gate.sh 跑。
# 所以这一格不再跑钩子的自证，只留上游判不到的：点名的钩子挂在要的 matcher 上（hooks-registered 只核有一条注册指向它，
# 与文件头 hook-events 写了工具名的那几个 matcher；攻方量过：write-guard 收成只挂 Write、heavy-test-guard 挪到 Edit 上、
# session-start 收成只挂 startup，它照样绿，research/prompts/gate-shrink-r1-opus-output.md「B2」）、写范围表与定义双向一致、定义头、共用模块不另写一份。
#
# 判别力：fixtures/code-tooling.sh/agent-write-scope-red 的 .claude/hooks/ 里放一份手抄 shell_tokens 的 hook 与一份手抄 classify_process 的 hook，必须各报出文件与行号；
# agent-write-scope-green 的 .claude/hooks/ 里放两份从共用模块导入、只写自己判定的 hook（连同被判仓自己那两份模块），必须判绿，成功行数进去的别的文件数要对。
# 共用模块的函数名取自这一道脚本旁边 ../hooks/ 下那两份（取自 $0 的目录，不取项目根）：样本仓里放坏的模块碰不到它。
#
# 为什么：执行类 agent 越界写，靠定义里一句「只写写范围」拦不住；hook 被删、注册挂错了 matcher、新加一个能写文件的定义忘了登记，
# 这道闸都会静默消失或静默放行——只有门禁会在它消失时说话。
# 2026-09-17 写 hook 时实测撞过一次静默放行：程序从标准输入读，hook 的 JSON 读不到，判定一律放行，而直接调判定函数的自证全绿。
cell_agent_write_scope() {
HOOKS_DIRECTORY="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)"
table_output="$(python3 - "$MANIFEST_LIBRARY" "$HOOKS_DIRECTORY/lib_shell_words.py" "$HOOKS_DIRECTORY/lib_heavy_tests.py" <<'PY'
import ast, glob, json, os, re, shlex, sys, types
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
failed = False
settings_path = ".claude/settings.json"
def command_invokes(command, script):
    """这条 hook 命令真的去跑 script：按 shell 切词（# 起的注释去掉），第一个词是它，或第一个词是 bash / sh / python3 / env、
    第二个词是它；路径按结尾认（…/.claude/hooks/write-guard.sh）。只在注释、echo 的参数、别的词里提到它的不算。"""
    try:
        words = shlex.split(command or "", comments=True)
    except ValueError:
        return False
    if len(words) > 1 and os.path.basename(words[0]) in ("bash", "sh", "python3", "env"):
        words = words[1:]
    return bool(words) and (words[0] == script or words[0].endswith("/" + script))
def mention_only_commands(entries, script):
    """提到了 script、却没有跑它的命令：出路里点名，免得读的人以为那一条已经注册着。"""
    return [hook.get("command") or "" for entry in entries for hook in entry.get("hooks") or []
            if script in (hook.get("command") or "") and not command_invokes(hook.get("command"), script)]
def matcher_covers(matcher, tool):
    if matcher in ("*", ""):
        return True
    if re.fullmatch(r"[A-Za-z0-9_|]+", matcher):
        return tool in matcher.split("|")
    try:
        return re.search(matcher, tool) is not None
    except re.error:
        return False
try:
    entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("PreToolUse") or []
except Exception as error:
    entries = None
    failed = True
    print(f"  ✗ 读不了 {settings_path}：{error}")
    print("     → 怎么办：修好这份 JSON，再在 hooks.PreToolUse 里注册写范围闸。")
if entries is not None:
    registered = [entry for entry in entries
                  if matcher_covers(entry.get("matcher", ""), "Write") and matcher_covers(entry.get("matcher", ""), "Edit")
                  and any(command_invokes(hook.get("command"), "write-guard.sh") for hook in entry.get("hooks") or [])]
    if not registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册写范围闸：PreToolUse 里没有 matcher 同时覆盖 Write 与 Edit、命令指向 write-guard.sh 的一条")
        for command in mention_only_commands(entries, "write-guard.sh"):
            print(f"     这一条只是提到 write-guard.sh、没有跑它，不算注册：{command}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Write|Edit"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/write-guard.sh"。')
    guard_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any(command_invokes(hook.get("command"), "bash-command-detector.sh") for hook in entry.get("hooks") or [])]
    if not guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册 Bash 检出 hook：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 bash-command-detector.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/bash-command-detector.sh"。')
    heavy_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any(command_invokes(hook.get("command"), "heavy-test-guard.sh") for hook in entry.get("hooks") or [])]
    if not heavy_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册重型测试闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 heavy-test-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/heavy-test-guard.sh"。')
    dispatch_registered = [entry for entry in entries
                           if matcher_covers(entry.get("matcher", ""), "Agent")
                           and any(command_invokes(hook.get("command"), "runner-dispatch-guard.sh") for hook in entry.get("hooks") or [])]
    if not dispatch_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续派闸：PreToolUse 里没有 matcher 覆盖 Agent、命令指向 runner-dispatch-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Agent|Task"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/runner-dispatch-guard.sh"。')
    continuation_registered = [entry for entry in entries
                               if matcher_covers(entry.get("matcher", ""), "SendMessage")
                               and any(command_invokes(hook.get("command"), "continuation-guard.sh") for hook in entry.get("hooks") or [])]
    if not continuation_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册续做闸：PreToolUse 里没有 matcher 覆盖 SendMessage、命令指向 continuation-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "SendMessage"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/continuation-guard.sh"。')
    handback_guard_registered = [entry for entry in entries
                                 if matcher_covers(entry.get("matcher", ""), "SubagentHandback")
                                 and any(command_invokes(hook.get("command"), "handback-guard.sh") for hook in entry.get("hooks") or [])]
    if not handback_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册交回闸：PreToolUse 里没有 matcher 覆盖 SubagentHandback、命令指向 handback-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 SubagentHandback 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/handback-guard.sh"。')
    claim_guard_registered = [entry for entry in entries
                              if matcher_covers(entry.get("matcher", ""), "AskUserQuestion")
                              and any(command_invokes(hook.get("command"), "ask-user-claim-guard.sh") for hook in entry.get("hooks") or [])]
    if not claim_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册弹窗断言闸：PreToolUse 里没有 matcher 覆盖 AskUserQuestion、命令指向 ask-user-claim-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "AskUserQuestion"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/ask-user-claim-guard.sh"。')
    pattern_guard = "singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh"
    pattern_guard_registered = [entry for entry in entries
                                if matcher_covers(entry.get("matcher", ""), "Bash")
                                and any(command_invokes(hook.get("command"), pattern_guard) for hook in entry.get("hooks") or [])]
    if not pattern_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册按模式找进程的钩子：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 {pattern_guard} 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/' + pattern_guard + '"（写法见 .claude/singlefs-ai-sop/rules/command-safety.md「pkill -f / killall 一律禁用」）。')
    elif not os.path.isfile(os.path.join(".claude", pattern_guard)):
        failed = True
        print(f"  ✗ 注册了按模式找进程的钩子，文件 .claude/{pattern_guard} 却不在：会话里每条 Bash 命令都会报钩子错")
        print("     → 怎么办：规范副本旧于 0.0.52 或没装全，按 CLAUDE.md「规范从哪来」那一行重新同步副本、跑 install.sh。")
try:
    session_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("SessionStart") or []
except Exception:
    session_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if session_entries is not None:
    compact_reminder_registered = [entry for entry in session_entries
                                   if matcher_covers(entry.get("matcher", ""), "compact")
                                   and any(command_invokes(hook.get("command"), "session-start.sh") for hook in entry.get("hooks") or [])]
    if not compact_reminder_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册压缩后提示 hook：SessionStart 里没有 matcher 覆盖 compact、命令指向 session-start.sh 的一条")
        print('     → 怎么办：在 hooks.SessionStart 里加一条 matcher "startup|resume|compact"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/session-start.sh"。')
    oom_report_registered = [entry for entry in session_entries
                             if matcher_covers(entry.get("matcher", ""), "startup") and matcher_covers(entry.get("matcher", ""), "resume")
                             and any(command_invokes(hook.get("command"), "session-start.sh") for hook in entry.get("hooks") or [])]
    if not oom_report_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册启动时的 OOM 报告 hook：SessionStart 里没有 matcher 同时覆盖 startup 与 resume、命令指向 session-start.sh 的一条")
        print('     → 怎么办：在 hooks.SessionStart 里把 session-start.sh 那一条的 matcher 写成 "startup|resume|compact"（会话崩了再起是 startup 或 resume，这时要先看内核 OOM 杀过谁）。')
table_path = ".claude/hooks/agent-write-scope.tsv"
patterns_by_agent, malformed = {}, []
if os.path.isfile(table_path):
    for line_number, line, fields in manifest.table_rows(table_path):
        if len(fields) < 2 or not fields[0].strip() or not fields[1].strip():
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        patterns_by_agent.setdefault(fields[0].strip(), []).append(fields[1].strip())
else:
    failed = True
    print(f"  ✗ 没有 {table_path}")
    print("     → 怎么办：建这张表：agent 名、路径模式、出处三列，用制表符分隔，按每个能写文件的定义的「写范围」一节转写。")
writers = []
missing_omit, missing_basis, bad_models = [], [], []
inherits_all_tools = set()   # 没写 tools 行、继承全部工具的定义
# ⑮ frontmatter 的 model 只许这几个取值（行尾可带 YAML 注释）：写成别的，换账号或换环境之后派发报 model_not_found、那一类 agent 全部起不来
ALLOWED_MODELS = {"opus", "sonnet", "haiku", "inherit"}
for path in sorted(glob.glob(".claude/agents/*.md")):
    whole = open(path, encoding="utf-8").read()
    head = whole.split("\n---", 1)[0]
    model_line = next((line for line in head.split("\n") if line.startswith("model:")), None)
    if model_line is not None:
        model_value = model_line[len("model:"):].split("#", 1)[0].strip().strip("'\"")
        if model_value not in ALLOWED_MODELS and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "any-model":
            bad_models.append(f"{os.path.basename(path)[:-3]}：{model_line.strip()}")
    if not any(line.strip() == "omitClaudeMd: true" for line in head.split("\n")):
        missing_omit.append(os.path.basename(path)[:-3])
    if not any(line.startswith("开工先读：") for line in whole.split("\n")):
        missing_basis.append(os.path.basename(path)[:-3])
    # tools：没写这一行就继承全部工具（含 Write、Edit）；`tools: A, B`、`tools: [A, B]` 与 YAML 列表（下面几行 `- A`）都认
    head_lines = head.split("\n")
    tools_index = next((index for index, line in enumerate(head_lines) if line.startswith("tools:")), None)
    if tools_index is None:
        writers.append(os.path.basename(path)[:-3])
        inherits_all_tools.add(os.path.basename(path)[:-3])
        continue
    inline_value = head_lines[tools_index][len("tools:"):].strip().strip("[]")
    tools = {tool.strip().strip("'\"") for tool in inline_value.split(",") if tool.strip()}
    for line in head_lines[tools_index + 1:]:
        list_item = re.match(r"^\s*-\s*(\S+)", line)
        if not list_item:
            break
        tools.add(list_item.group(1).strip("'\""))
    if tools & {"Write", "Edit", "*"}:
        writers.append(os.path.basename(path)[:-3])
if malformed:
    failed = True
    print("  ✗ 写范围表里这些行不到两列、或 agent 名与路径模式有空的：")  # gate-lint:summary
    for entry in malformed:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行至少写 agent 名与路径模式两列，用制表符分隔。")
# ③ 有 Write 或 Edit 的定义要在表里登记；④ 表里的名字要有定义。两个方向的盘上集合不同，各给各的判定。
unscoped, ghosts = manifest.two_way(writers, list(patterns_by_agent),
                                    exists=lambda name: os.path.isfile(f".claude/agents/{name}.md"))
if unscoped:
    failed = True
    print("  ✗ 这些定义的 tools 里有 Write 或 Edit，写范围表里却没有登记——hook 会把它们的每次写都拒掉：")  # gate-lint:summary
    for name in unscoped:
        print(f"     {name}（没写 tools，继承全部工具）" if name in inherits_all_tools else f"     {name}")  # gate-lint:detail
    print(f"     → 怎么办：照它定义里「写范围」一节，在 {table_path} 给它加路径模式。")
if ghosts:
    failed = True
    print("  ✗ 写范围表里登记的这些 agent 在 .claude/agents/ 里没有定义：")  # gate-lint:summary
    for name in ghosts:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：改成现在的定义名，或删掉这几行。")
if bad_models:
    failed = True
    print("  ✗ 这些定义的 frontmatter 里 model 不是 opus、sonnet、haiku、inherit 之一——派发时报 model_not_found，这一类 agent 起不来：")  # gate-lint:summary
    for line in bad_models:
        print(f"     {line}")  # gate-lint:detail
    print("     → 怎么办：改成 opus、sonnet、haiku 或 inherit（行尾可以带 # 注释）；要用本地模型的腿照定义走 research/scripts/ask-local.sh，不写进 model。")
if missing_omit:
    failed = True
    print("  ✗ 这些定义的 frontmatter 没有 omitClaudeMd: true——每次派发都白带约 10 万 token 的 CLAUDE.md 与规则：")  # gate-lint:summary
    for name in missing_omit:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在它的 frontmatter 里加一行 omitClaudeMd: true，开工先读那一句指到 .claude/agent-common.md「规则怎么读」。")
if missing_basis:
    failed = True
    print("  ✗ 这些定义没有「开工先读：」一行——不继承 CLAUDE.md 之后，它要读的规则没有地方点名：")  # gate-lint:summary
    for name in missing_basis:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：在正文里加一行「开工先读：」，点名它干活要读的规则文件与小节。")
# ⑬ 两份共用模块顶层定义的函数名从模块现读（这一道脚本旁边 ../hooks/ 下那一份），再扫被判仓 .claude/hooks/ 下它之外的每个文件
# 不算的名字：不是共用的判定，别的 hook 里有同名的一份不算手抄；登记了而模块里没有的判红（不起作用的排除项）
NOT_SHARED_JUDGMENT = {
    "lib_heavy_tests.py": {
        "selftest": "每个 hook 都有自己的 --selftest 入口，同名不是抄的判定",
        "load_sibling_module": "按文件路径导入同目录模块的样板，不是判定",
    },
}
# (模块文件名, 叫法, 这一道脚本旁边那一份的路径, 手抄一份会怎样, 该怎么办, 模块里该有什么)
SHARED_MODULES = [
    ("lib_shell_words.py", "共用切词模块", sys.argv[2],
     "手抄的切词会分叉：同一条命令一个 hook 认得出、另一个认不出",
     "从 `.claude/hooks/lib_shell_words.py` 导入，不另写一份（照 bash-command-detector.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑两个 hook 的 --selftest。",
     "切词、切简单命令、认命令位置、剥前缀、跟 cd 这几个函数要定义在 lib_shell_words.py 里，两个 hook 从它导入。"),
    ("lib_heavy_tests.py", "共用重型测试判定模块", sys.argv[3],
     "手抄的判定会分叉：执行前的闸 heavy-test-guard.sh 与看门狗 agent-watch.py 对同一条命令判得不一样",
     "从 `.claude/hooks/lib_heavy_tests.py` 导入，不另写一份（照 heavy-test-guard.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑它与 heavy-test-guard.sh 的 --selftest。",
     "classify、classify_process、judged_by_name 这些判定要定义在 lib_heavy_tests.py 里，heavy-test-guard.sh 与看门狗从它导入。"),
]
hooks_directory = ".claude/hooks"
shared_counts = []   # [(模块文件名, 查的函数个数, 查了几个别的文件)]
for module_file, module_title, module_path, divergence, fix, required in SHARED_MODULES:
    try:
        module_tree = ast.parse(open(module_path, encoding="utf-8").read(), module_path)
    except (OSError, SyntaxError, ValueError) as error:
        failed = True
        print(f"  ✗ 读不了{module_title} {module_path}：{error}")
        print(f"     → 怎么办：恢复 .claude/hooks/{module_file}；{fix}")
        continue
    function_names = [node.name for node in module_tree.body if isinstance(node, ast.FunctionDef)]
    if not function_names:
        failed = True
        print(f"  ✗ {module_title} {module_path} 里一个顶层函数都没有：这一条没有名字可查")
        print(f"     → 怎么办：{required}")
        continue
    not_judgment = NOT_SHARED_JUDGMENT.get(module_file, {})
    stale_exclusions = sorted(set(not_judgment) - set(function_names))
    if stale_exclusions:
        failed = True
        print(f"  ✗ NOT_SHARED_JUDGMENT 给 {module_file} 登记了模块里没有的函数：{'、'.join(stale_exclusions)}（不起作用的排除项会让人以为那几个名字被绕开了）")
        print("     → 怎么办：从 code-tooling.sh 的 agent-write-scope 格的 NOT_SHARED_JUDGMENT 里删掉这几项，或者核一下是不是模块里的函数改了名。")
    checked_names = [name for name in function_names if name not in not_judgment]
    module_in_scanned_repo = os.path.normpath(os.path.join(hooks_directory, module_file))
    redefinition = re.compile(r"^[ \t]*def[ \t]+(" + "|".join(map(re.escape, checked_names)) + r")[ \t]*\(", re.M)
    redefinitions, scanned_hook_files = [], 0
    for directory, subdirectories, file_names in os.walk(hooks_directory):
        subdirectories[:] = sorted(name for name in subdirectories if name != "__pycache__")
        for file_name in sorted(file_names):
            path = os.path.normpath(os.path.join(directory, file_name))
            if path == module_in_scanned_repo:
                continue
            try:
                content = open(path, encoding="utf-8").read()
            except (OSError, UnicodeDecodeError):
                continue
            scanned_hook_files += 1
            for match in redefinition.finditer(content):
                redefinitions.append(f"{path}:{content.count(chr(10), 0, match.start()) + 1}  def {match.group(1)}")
    shared_counts.append((module_file, len(checked_names), scanned_hook_files))
    if redefinitions:
        failed = True
        print(f"  ✗ {module_title} .claude/hooks/{module_file} 里的函数，在 .claude/hooks/ 下别的文件里又定义了一份（{divergence}）：")  # gate-lint:summary
        for entry in redefinitions:
            print(f"     {entry}")  # gate-lint:detail
        print(f"     → 怎么办：{fix}")
if failed:
    sys.exit(1)
pattern_count = sum(len(patterns) for patterns in patterns_by_agent.values())
print("TABLE_OK", len(writers), pattern_count, *(count for _, function_count, file_count in shared_counts for count in (function_count, file_count)),
      "、".join(name for names in NOT_SHARED_JUDGMENT.values() for name in names))
PY
)"; table_rc=$?
printf '%s\n' "$table_output" | grep -v '^TABLE_OK '
if [[ $table_rc -ne 0 ]]; then exit 1; fi
read -r _ writer_count pattern_count shared_function_count scanned_hook_file_count heavy_function_count heavy_scanned_file_count not_shared_judgment_names <<<"$(printf '%s\n' "$table_output" | grep '^TABLE_OK ')"
echo "  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册在要的 matcher 上，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义、${pattern_count} 条路径模式），共用切词模块的 ${shared_function_count} 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 ${scanned_hook_file_count} 个文件），共用重型测试判定模块的 ${heavy_function_count} 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 ${heavy_scanned_file_count} 个文件；${not_shared_judgment_names} 不算，见 NOT_SHARED_JUDGMENT）；钩子的自证归上游 hooks-registered.sh 跑"
}

# ── 格 change-range-single-source：改动范围只许一份取法：阶段不自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
#
# 判据（射程是 `.claude/gate.d/` 顶层的 *.sh 与 *.py；这一道文件里只除掉这一格自己，从这一格的标题行到下一格的标题行之间按空行判，
# 别的格照样在射程里——它们原来是别的阶段文件，并进来之前一直被这一条扫着）。三条，任一条不成立判红：
#   ① 算 diff 基准只许在 research/scripts/changed-paths.sh 里：代码行里出现 merge-base（`--is-ancestor` 核祖先除外）、
#      @{upstream} 或 @{u}、gate-ok、读 GATE_BASE 或 GATE_DIFF_BASE（`$GATE_BASE`、`${GATE_BASE`、environ）的，
#      以及跑 git 的那一行上写死 HEAD~N、HEAD^（HEAD^{tree} 这类剥类型的不算）、用三点号（A...B 就是取 merge-base）的，判红。`unset GATE_BASE` 这类不读它的不算。
#      阶段要改动范围就 source 那份共用脚本、调 gate_diff_base。
#   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false（键不分大小写，值必须是 false）：
#      同一行写了它；或者这一行用的变量 / 列表在同一个文件里带着它定义（例 `GIT = ["git", "-c", "core.quotepath=false"]`）；
#      或者这一行调的是同一个文件里带着它的包装函数（python 写成「名字(」，bash 写在命令位置上；名字恰好是 git 时，
#      裸的 git 命令与 ['git', …] 列表不算调了它——例 doc-experiments.sh 的 decision-links 格的 `def git(*args)`）；或者 git 自己带 -z（-z 下不转义路径）。
#      参数列表换了行、这一行以引号开头的，往上看三行找调用前缀。
#      中文路径不带这个选项会被转成带引号的八进制串，与仓里的路径逐字对不上，而 git 不报错。
#   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 `||`，或放在 if / while 里。
#      git 失败时共用脚本退非 0、什么都不交，不判就会把「没取到」读成「这次什么都没改」。
#   「代码行」：去掉前导空白后不以 #、echo、printf、print(、bad、howto、die、引号开头的行，或者以这些开头、但这一行里
#   跑了 `$(git …)`、带引号括起的列路径参数、或带引号括起的取基准参数（"merge-base"、"@{upstream}"、"HEAD~1"、"HEAD^"）的——
#   出路文字与注释里提到这些词不算；参数列表换了行、续行以引号开头的，① 与 ② 一样判。
#
# 为什么：改动范围的取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath；doc-decisions.sh 的格「定了新东西没回看同文件未定项」的基准在默认分支上
# 就是 HEAD，定案分两次提交、第一次没推，它就退 77 不判）。收完之后没有东西拦下一份拷贝。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T5；判据按第二轮攻方打中的写法收严（gate-fix-forks-r2）。
#
# 没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下的脚本不是门禁阶段，成功行把其中碰到这两条的逐个列名，清单现算。
# 样本：fixtures/code-tooling.sh/change-range-single-source-red 把三条判据该抓的写法各放几种（自己取 merge-base、@{u}...、HEAD~1、HEAD^、
# 换了行的参数列表续行里的 "merge-base"、
# 不带或带 =true 的 quotepath、包装函数名叫 git 而这一行没调它、test 的 -z、printf "$(git …)"、换行的参数列表、
# 不判退出码的共用取法），逐条点名判红；change-range-single-source-green 放五种合法写法与 unset GATE_BASE、merge-base --is-ancestor、HEAD^{tree}、判了退出码的调用，判绿。
# 并进来时 change-range-single-source-green 的四个 .sh 各补一行 ROOT 注释、补了 stage-owners.tsv（stage-owners 格要）；注释行不在这一格的射程里，want 不变。
cell_change_range_single_source() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
python3 - "$STAGE_NAME" <<'PY'
import glob, os, re, sys

own_name = sys.argv[1]
# 这一道文件里这一格自己的那一段（正则与出路里全是它要抓的写法）：从这一格的标题行到下一格的标题行，按空行判，行号不变
OWN_CELL_START = re.compile(r'^# ── 格 change-range-single-source：')
NEXT_CELL_START = re.compile(r'^# ── (格 |汇总)')
BASE_PATTERN = re.compile(r'merge-base|@\{u(pstream)?\}|gate-ok|\$\{?GATE_(DIFF_)?BASE\b|environ[^\n]*GATE_(DIFF_)?BASE')
# 写死的基准（HEAD~N）与三点号（A...B 就是取 merge-base）只在跑 git 的那一行上算：文档字符串里的「...」不算
BASE_IN_GIT_PATTERN = re.compile(r'\bHEAD(~\d|\^(?!\{))|\S\.\.\.(\s|$|[^.])')
RUNS_GIT_ANYWHERE = re.compile(r'\bgit\b|\bGIT\b')
BASE_ALLOWED = re.compile(r'merge-base["\']?[\s,]+["\']?--is-ancestor')
LIST_PATTERN = re.compile(r'--name-only|--name-status|\bls-files\b')
QUOTED_LIST_FLAG = re.compile(r"""["'](--name-only|--name-status|ls-files)["']""")
# 以引号开头的续行里，取基准的参数单独成一个带引号的参数（python 参数列表换了行）：这种行是代码，不是出路文字
QUOTED_BASE_ARGUMENT = re.compile(r"""["'](merge-base|@\{u(pstream)?\}|HEAD(~\d+|\^(?!\{)[^"']*)|[^"'\s]+\.\.\.[^"'\s]*)["']""")
TEXT_PREFIX = re.compile(r"""^(#|echo\b|printf\b|print\(|bad\b|howto\b|die\b|["'])""")
RUNS_GIT = re.compile(r'\$\(\s*git\b')
QUOTEPATH_FALSE = re.compile(r'quotepath=false', re.I)
QUOTE_VARIABLE = re.compile(r'^\s*([A-Za-z_][A-Za-z_0-9]*)\s*\+?=.*quotepath=false', re.I)
WRAPPER_HEAD = re.compile(r'^\s*(?:def\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(|(?:function\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*\(\)\s*\{?)')
GIT_Z = re.compile(r"""\bgit\b[^|;&]*\s-z\b|["']-z["']""")
SHARED_CALL = re.compile(r'\b(gate_changed_paths|gate_added_lines)\b')
STATUS_HANDLED = re.compile(r'\|\||^\s*(if|elif|while|until|!)\b')
WRAPPER_REACH = 3
LOOKBACK = 3
CONTINUATION = re.compile(r"""^\s*["'\[]""")

def is_text(stripped):
    """出路文字与注释：以这些开头、而且这一行不跑 git、不带引号括起的列路径参数的。"""
    return (bool(TEXT_PREFIX.match(stripped)) and not RUNS_GIT.search(stripped) and not QUOTED_LIST_FLAG.search(stripped)
            and not QUOTED_BASE_ARGUMENT.search(stripped))

def quoting_names(all_lines):
    """同一个文件里带着 quotepath=false 定义的变量 / 列表，与带着它的包装函数：(变量名集合, 函数名集合)。"""
    variables, wrappers = set(), set()
    for index, line in enumerate(all_lines):
        variable = QUOTE_VARIABLE.match(line)
        if variable:
            variables.add(variable.group(1))
        wrapper = WRAPPER_HEAD.match(line)
        if wrapper and any(QUOTEPATH_FALSE.search(body) for body in all_lines[index:index + 1 + WRAPPER_REACH]):
            wrappers.add(wrapper.group(1) or wrapper.group(2))
    return variables, wrappers

def uses_quoting(line, variables, wrappers):
    if QUOTEPATH_FALSE.search(line) or GIT_Z.search(line):
        return True
    if any(re.search(r'\$\{?' + re.escape(name) + r'\b|\*' + re.escape(name) + r'\b|\b' + re.escape(name) + r'\s*(\+|\[)', line)
           for name in variables):
        return True
    # 包装函数认「这一行调的是它」：python 写成「名字(」，bash 写在命令位置上。名字恰好是 git 时，
    # 「git(」认，裸的 git 命令与 ['git', …] 列表不认——doc-experiments.sh 的 decision-links 格的 def git(*args) 就是这种。
    for name in wrappers:
        if re.search(r'(?<![\w\'"])' + re.escape(name) + r'\(', line):
            return True
        if name != 'git' and re.search(r'(^|[;&|(]|\$\()\s*' + re.escape(name) + r'\s', line.strip()):
            return True
    return False

def judge(path):
    """返回 (算基准的行, 不带 quotepath 的列路径行, 不判退出码的共用取法调用)。"""
    with open(path, encoding='utf-8', errors='replace') as handle:
        all_lines = handle.read().split('\n')
    if os.path.basename(path) == own_name:
        inside = False
        for index, line in enumerate(all_lines):
            if OWN_CELL_START.match(line):
                inside = True
            elif inside and NEXT_CELL_START.match(line):
                inside = False
            if inside:
                all_lines[index] = ''
    variables, wrappers = quoting_names(all_lines)
    base_hits, list_hits, status_hits = [], [], []
    for index, line in enumerate(all_lines):
        stripped = line.strip()
        if not stripped or is_text(stripped):
            continue
        number = index + 1
        runs_git_here = RUNS_GIT_ANYWHERE.search(line) or (CONTINUATION.match(line) and QUOTED_BASE_ARGUMENT.search(line))
        if (BASE_PATTERN.search(line) or BASE_IN_GIT_PATTERN.search(line) and runs_git_here) \
                and not BASE_ALLOWED.search(line):
            base_hits.append((number, stripped))
        if LIST_PATTERN.search(line) and not uses_quoting(line, variables, wrappers):
            # 参数列表换了行（这一行以引号或方括号开头）：往上找这条语句开头的那一行，看调用前缀带没带选项
            head = index
            while head > 0 and index - head < LOOKBACK and CONTINUATION.match(all_lines[head]):
                head -= 1
            if head == index or not uses_quoting(all_lines[head], variables, wrappers):
                list_hits.append((number, stripped))
        if SHARED_CALL.search(line) and not re.match(r'^\s*(gate_changed_paths|gate_added_lines)\s*\(\)', line) \
                and not STATUS_HANDLED.search(line):
            status_hits.append((number, stripped))
    return base_hits, list_hits, status_hits

scanned = sorted(path for pattern in ('.claude/gate.d/*.sh', '.claude/gate.d/*.py')
                 for path in glob.glob(pattern))
base_bad, list_bad, status_bad = [], [], []
for path in scanned:
    base_hits, list_hits, status_hits = judge(path)
    base_bad += [(path, number, line) for number, line in base_hits]
    list_bad += [(path, number, line) for number, line in list_hits]
    status_bad += [(path, number, line) for number, line in status_hits]

failed = False
if base_bad:
    failed = True
    print(f'  ✗ {len(base_bad)} 行在阶段里自己算 diff 基准（改动范围只许一份取法：research/scripts/changed-paths.sh）：')
    for path, number, line in base_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：阶段里 source research/scripts/changed-paths.sh，基准写 base="$(gate_diff_base gate)"（问「这一次提交带哪些」的写 head），')
    print('               路径写 gate_changed_paths、新增行写 gate_added_lines；共用脚本缺哪种取法就往它里面加，连同它的 --selftest 一起改。')
if list_bad:
    failed = True
    print(f'  ✗ {len(list_bad)} 行列路径的 git 调用没带 -c core.quotepath=false：')
    for path, number, line in list_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：在那一行的 git 后面加 -c core.quotepath=false（python 里是 ["git", "-c", "core.quotepath=false", …]），')
    print('               或者把带它的调用前缀收进一个变量 / 列表再用；不带它时中文路径变成带引号的八进制串，与仓里的路径逐字对不上。')
if status_bad:
    failed = True
    print(f'  ✗ {len(status_bad)} 处调共用取法（gate_changed_paths / gate_added_lines）没判退出码：')
    for path, number, line in status_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：写成 x="$(gate_changed_paths …)" || { echo 取不到改动范围…; echo 出路…; exit 1; }，或放进 if；')
    print('               git 失败时共用脚本退非 0、什么都不交，不判退出码就会把「没取到」读成「这次什么都没改」。')
if failed:
    sys.exit(1)

outside = []
for pattern in ('research/scripts/*.sh', 'research/scripts/*.py', '.claude/scripts/*', '.claude/hooks/*'):
    for path in sorted(glob.glob(pattern)):
        if os.path.isfile(path) and path != 'research/scripts/changed-paths.sh':
            base_hits, list_hits, status_hits = judge(path)
            if base_hits or list_hits or status_hits:
                outside.append(f'{path}（算基准 {len(base_hits)} 行、不带 quotepath 的列路径 {len(list_hits)} 行、不判退出码 {len(status_hits)} 处）')
print(f'  ✓ 查了 .claude/gate.d/ 下 {len(scanned)} 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false，调共用取法都判了退出码')
if outside:
    print(f'     没扫的 {len(outside)} 份（不是门禁阶段，射程之外）：' + '；'.join(outside))
else:
    print('     没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下没有碰到这两条的脚本')
PY
}

# ── 格 research-gate-lint：research/scripts/、.claude/hooks/ 与 .claude/scripts/ 里每一条拒绝都带出路、守 shell 纪律；前两个目录的执行位不丢；连同 .claude/gate.d/，终止进程只许点名一个
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint（.claude/singlefs-ai-sop/scripts/gate.sh 里 run_stage "门禁自检" 那一行与它前面给 LINT_EXTRA 赋值的那一行），
# 研究脚本与 hook 不在射程里。2026-09-18 单跑整仓 gate-lint 红 90 处，84 处在这两个目录，没有任何一笔账记着（C382（研究脚本与 hook 的拒绝不在门禁自检的射程里））。
# research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# shell-lint 同一批目录一起跑（上游 show-me-test.md「射程只到 .claude/gate.d/」：只接 gate-lint 等于只补了一半，
# 2026-09-19 单跑 shell-lint 报 14 处，全是它认不出一行写完的函数造成的假红，上游已修、同步副本之后才生效）。两个 lint 都跑完再判，不在第一个红就停。
# .claude/scripts/ 此前同样不在这两个 lint 的射程里（共享门禁只把它交给「脚本执行位」），而 lkmm.sh 一份就有二十多处拒绝，
# 所以一并交给 gate-lint 与 shell-lint；执行位仍归共享那一道，这里不重复扫。
# 判别力：fixtures/code-tooling.sh/research-gate-lint-red 放一个只带一句话的 die 和一个靠子 shell 赋值往外带值的函数，两样都必须报出来；
# 它的 .claude/scripts/ 里再放一个不带出路的拒绝和一处 pkill -f，只有这个目录进了射程才报得出来。
# research-gate-lint-green 带上第二个参数、不靠子 shell 带值，必须判绿；它的 .claude/scripts/ 里放一个干净的包装，成功行的脚本数要把它数进去。
# 进程安全（终止只许点名一个自己起的进程号或任务号）：判定只有一份，在 .claude/hooks/bash-command-detector.sh（拒绝 ⑥），这里经它的 --scan-scripts 扫脚本，
# 射程比上面两个 lint 多一个 .claude/gate.d/（它的样本目录不扫）。用户 2026-09-25 定的规矩与写法见那个 hook 的文件头 ⑥。
# 判别力：red 的 research/scripts/stop-processes.sh（自己任务的进程组、循环里 kill、systemctl 带通配）、research/scripts/stop.py（循环 os.kill、os.killpg）
# 与 .claude/gate.d/50-stop-group.sh（按 cgroup.procs 批量发、没核 scope 也没标；只有 .claude/gate.d/ 进了射程才报得出来）逐处按行号报；
# green 的 research/scripts/stop-own.sh 只停点名的一个、按 cgroup 批量发的那一段核过 singlefs-memory-cap-*.scope 又标了 own-scope，必须判绿。
# 并进来时 research-gate-lint-green 补了绿样本的架子（research/scripts/vm-bench.sh、.claude/hooks/agent-write-scope.tsv、.claude/gate.d/10-sample.sh），
# want 跟着改了数：gate-lint 数到 4 个脚本、进程安全查 5 个、扫了 3 个目录（原样本是 3、3、2）。
cell_research_gate_lint() {
LINT="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
SHELL_LINT="$(dirname "$LINT")/shell-lint.sh"
for script in "$LINT" "$SHELL_LINT"; do
  [[ -f "$script" ]] || { echo "  ✗ 找不到共享脚本 $script"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
done
TARGETS=()
MODE_TARGETS=()
ABSENT=()
for directory in "$ROOT/research/scripts" "$ROOT/.claude/hooks" "$ROOT/.claude/scripts"; do
  [[ -d "$directory" ]] || { ABSENT+=("${directory#"$ROOT"/}"); continue; }
  TARGETS+=("$directory")
  [[ "$directory" == "$ROOT/.claude/scripts" ]] || MODE_TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 research/scripts/、.claude/hooks/ 也没有 .claude/scripts/，这一格无对象可判"; exit 77; }
# GATE_LINT_DIR 指到第一个目标目录：不设它，共享脚本默认还会扫 SOP 自己的包，样本目录里红绿就靠那个包碰巧干净来分。
failed=0
if ! GATE_LINT_DIR="${TARGETS[0]}" bash "$LINT" "${TARGETS[@]:1}"; then
  echo "  ✗ research/scripts/、.claude/hooks/ 或 .claude/scripts/ 里有不带出路的拒绝（上面逐处列出）"
  echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，直接打印的拒绝之后跟一行以箭头开头的出路"
  failed=1
fi
# 执行位也一起：共享门禁的「脚本执行位」阶段只扫 .claude/gate.d 与 .claude/scripts，
# research/scripts 与 .claude/hooks 不在它的射程里（同 gate-lint / shell-lint 那两条的理由）；.claude/scripts 已在那一道里，不重复扫。
MODES="$(dirname "$LINT")/script-modes.sh"
modes_note=""
if [[ ! -f "$MODES" ]]; then
  echo "  ✗ 找不到共享脚本 $MODES，research/scripts/ 与 .claude/hooks/ 的执行位这一项没判"
  echo "     → 怎么办：规范副本缺了这一份，把 singlefs-ai-sop-<语言> 仓重新拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh；别让这一项静默不判"
  failed=1
  modes_note="执行位：没判（找不到 script-modes.sh，上面已判红）"
elif ((${#MODE_TARGETS[@]} == 0)); then
  modes_note="执行位：本次无对象可判（射程里没有 research/scripts/ 也没有 .claude/hooks/）"
else
  modes_rc=0
  bash "$MODES" "${MODE_TARGETS[@]}" || modes_rc=$?
  if (( modes_rc == 77 )); then
    modes_note="执行位：本次未判（script-modes.sh 退 77，原因见它上面那一句）"
  elif (( modes_rc != 0 )); then
    echo "  ✗ research/scripts/ 或 .claude/hooks/ 里的脚本执行位在暂存区里不对（上面逐处列出）"
    echo "     → 怎么办：用 git update-index --chmod=+x <路径>（或 -x）改暂存区里的模式，让它与工作区一致"
    failed=1
    modes_note="执行位：判红"
  else
    modes_note="执行位：判过（${#MODE_TARGETS[@]} 个目录）"
  fi
fi
# 进程安全：同一份判定（bash-command-detector.sh --scan-scripts），连 .claude/gate.d/ 一起扫；待改清单是被判仓的 .claude/process-safety-pending
DETECTOR="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)/bash-command-detector.sh"
SAFETY_TARGETS=("${TARGETS[@]}")
[[ -d "$ROOT/.claude/gate.d" ]] && SAFETY_TARGETS+=("$ROOT/.claude/gate.d")
if [[ ! -f "$DETECTOR" ]]; then
  echo "  ✗ 找不到 $DETECTOR，终止进程的写法这一项没扫"
  echo "     → 怎么办：恢复 .claude/hooks/bash-command-detector.sh（判定只有那一份，门禁与执行前的钩子共用），再跑这一格：bash .claude/gate.d/${STAGE_NAME} --check research-gate-lint"
  failed=1
elif ! bash "$DETECTOR" --scan-scripts "$ROOT" "${SAFETY_TARGETS[@]}"; then
  echo "  ✗ research/scripts/、.claude/hooks/、.claude/scripts/ 或 .claude/gate.d/ 里有打得到别人进程的终止写法（上面逐处列出）"
  echo "     → 怎么办：照上面那句「怎么办」改；规矩是「后面的脚本不能终止前面的脚本」「不能动 ssh」「终止要按照任务号终止 禁止终止所有」，写法见 .claude/hooks/bash-command-detector.sh 文件头 ⑥"
  failed=1
fi
# shell-lint 一次只扫一个目录（SHELL_LINT_DIR），逐个目录跑
for directory in "${TARGETS[@]}"; do
  if ! SHELL_LINT_DIR="$directory" bash "$SHELL_LINT"; then
    echo "  ✗ ${directory#"$ROOT"/} 里有违反 shell 纪律的写法（上面逐处列出）"
    echo "     → 怎么办：照每一处给的改法改，规则在 .claude/singlefs-ai-sop/rules/command-safety.md；是 lint 认错了的，到上游 singlefs-ai-sop 修 shell-lint 并加样本，不在本仓绕开"
    failed=1
  fi
done
if ((failed == 0)); then
  scanned_names=()
  for directory in "${SAFETY_TARGETS[@]}"; do scanned_names+=("${directory#"$ROOT"/}"); done
  echo "  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 ${#TARGETS[@]} 个目录（${scanned_names[*]:0:${#TARGETS[@]}}），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 ${#SAFETY_TARGETS[@]} 个目录；${modes_note}"
  echo "    没扫的目录 ${#ABSENT[@]} 个（不在）：${ABSENT[*]:-（没有）}"
fi
exit "$failed"
}

# ── 格 fixture-claims：阶段头部声称的判别力样本必须真的存在；全仓文件名与样本目录里的日期不许是编的
#
# 判据：四条，任一条不成立判红。
#   ① 一个阶段的头部注释里写了 `fixtures/<它自己的文件名>`，那个样本目录就必须存在；
#   ② `.claude/gate.d/fixtures/` 下的每个目录都要对应一个存在的阶段文件——孤儿样本目录没有任何东西会跑它，
#      而目录摆在那里看着就像验过了；
#   ③ 样本目录下每个子目录都是一份样本（与上游 stage-selftest.sh 同一个认法：带 expect 的都跑），每个子目录都要有 `expect`，
#      至少一份；再按那道门禁文件头的格名表（`# gate-cell:` 行，经 research/scripts/gate-structure-check.py 的 header_cells 读；
#      没有格名表的整道算一格）逐格数，每一格至少一份红（exit=1）一份绿（exit=0），缺的逐格列名。一份样本算给哪几格：
#      `.gate-cells` 点名的格，没有的算给整道全部格；红样本点名一格的算给那一格，点名几格的只算给 want 里写了
#      「<格名>：红」（共用库汇总那一行）的格；退 0 的样本不算给 want 里写了「<格名>：本次未跑」的格。
#      绿的一侧按设计只能退 77（或别的非 0、非 1 码）的格，那份样本的 expect 里写一行 `not-green-by-design=<格名> <为什么>`
#      （理由至少 8 个字）才算绿。空目录与只认裸 red / green 的样本目录既骗过第 ① 条，又会被判别力自检整个跳过；
#   ④ 日期不许是编的——全仓**文件名**里的日期，以及 `fixtures/` 下 `.md` **正文**里的日期，
#      都要落在可能的区间里。判据不自己写，source 上游 lib.sh 的 `date_out_of_range`
#      （早于本仓第一个提交宽限 7 天、或晚于最晚时区的今天，都不可能是真发生过的事）。
#
# 第 ④ 条与前三条是同一个病理：**摆在那里看着是真的，其实是造的**。
# 前三条管「声称验过了而其实没验」，第 ④ 条管「日期看着是真的而其实是编的」。
# 实测（2026-09-23）：门禁样本里 8 个文件名与 32 处正文写着 2026-01-01 / 2026-01-02，
# 比本仓第一个提交（2026-08-26）早大半年，而当天带一个 records/2026-01-01-临时实测.md
# 跑 64 道阶段加 doc-lint，0 道点名它。上游 doc-lint 的 M 段只认正文里的
# `### 日期` 与「实测（日期）」两种形态，够不着文件名，也够不着
# `已定（日期）`、`—— 已跑（日期）` 这类装成本仓事实的写法。
#
# ⚠️ 第 ④ 条的正文那一维**只扫 fixtures**，不扫真 kb 正文：`date_out_of_range` 的下界假设是
# 「本仓的事不可能早于本仓第一个提交」，这只对说本仓自己事的日期成立。实测扫全仓正文得 52 处越界，
# 其中 20 处在 prior-art.md（别家项目的真实日期）、decisions-history（RFC 演进史 2015–2017）与
# checks-owed.md（C510（编造的日期没人拦） 引 2026-01-01 当例子）里，全部合法——射程放到真 kb 正文就是误判。
# 文件名那一维没有这个反例（带日期的路径说的都是本仓自己的事；有几条由成功行现报），所以扫全仓。
#
# 为什么：上游的判别力自检（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh`）把没有样本的阶段
# 列成「未自检」，这很好——但它只看目录在不在，不看**阶段自己怎么说**。一个阶段的头部逐字写着
# 「判别力：fixtures/<自己>/red 是一个……必须判红；green ……必须判绿」，而那个目录压根不存在时，
# 读脚本的人以为验过了，自检的名单里那一行又只是一句轻描淡写的「未自检」，两边对不上没有人会发现。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：当时的实验归档那一道（后来并进 `doc-experiments.sh` 的 archive-past-rounds 格）的头部就是这么写的，
# 而它的样本目录（后来并进 `fixtures/doc-experiments.sh/archive-past-rounds-*`）不存在——它立项那天起就没验过判别力。
#
# ⚠️ 射程：判的是「声称与目录对不对得上」，判不了样本本身有没有判别力——那一层归
# `stage-selftest.sh`（红样本真判红、绿样本真判绿）。**没有声称、也没有样本**的阶段这一道不判，
# 它们由自检的「未自检」名单管着。
#
# 判别力：fixtures/code-tooling.sh/fixture-claims-red 摆一个假的 gate.d，前三条各犯一次，必须判红；fixture-claims-green 都干净。
# fixture-claims-unnumbered-red 的 gate.d 里一道不带编号的阶段 tooling-claims.sh 声称有样本而目录不在、另一道带编号的干净阶段，必须点名前者判红
# （阶段只认两位数开头时它一声不吭：判决 research/prompts/gate-shrink-r1-main-verification.md 第 4 问）。
# 第 ③ 条按格数：fixture-claims-cells-red 的 cells-uncovered.sh 三格，样本按 <格>-red / <格>-green 起名，alpha 缺绿、beta 只在点名两格而
# want 只写了 gamma 红的样本里（不算给 beta，缺红）、gamma 的绿样本退 77 而 not-green-by-design 的理由太短（缺绿），必须逐格列名判红；
# fixture-claims-green 的 cells-covered.sh 两格没有裸 red / green，alpha 一红一绿、beta 的红来自点名两格而 want 写了「beta：红」的样本、
# 绿样本退 77 并写了够长的 not-green-by-design，必须判绿（只认裸 red / green 时它是空壳）。
# 第 ④ 条的坏日期由 fixture-claims-red/setup.sh 在临时仓里现造，不摆进仓里——摆进来它会被这一条自己扫到
# （C441（全仓清扫工具会吃掉自己的判别力样本））。setup.sh 里要 git init 再提交一个真实日期的提交，
# 否则 project_start_date 取不到值、下界那一支根本走不到，红样本就只证了上界。
#
# 不是 git 仓时第 ④ 条列不出要扫的文件、整条没判：前三条都过也不报绿，这一格退 77（本次未跑）。
# 第 ④ 条正文那一维唯一的排除是这一格自己的样本（fixtures/<这一道>/fixture-claims-* 那几份，setup.sh 里现造的日期就是判别力），
# 这一道别的格的样本照旧在射程里（并进来之前它们住在各自的样本目录下，一直被这一条扫着）。
# 第 ④ 条的下界取本仓第一个提交；取不到时退回上游的 SOP_START_DATE，成功行写明下界取自哪一个。
# 读不了的样本正文（grep 出错）不计进「查了几份」，逐个列进没扫的清单；二进制样本按文本读，里面的日期照判。
# 并进来时 fixture-claims-green 的 50-good.sh 补一行 ROOT 注释、补了 stage-owners.tsv（stage-owners 格要），另补 CLAUDE.md 那张表的一行
# （setup.sh 建了 .claude/kb/decisions-history/，kb-registry 格要它有登记）；want 不变。
cell_fixture_claims() {
LIB="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/lib.sh"
python_rc=0
python3 - "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/research/scripts/gate-structure-check.py" <<'PY' || python_rc=$?
import glob, importlib.util, os, re, sys
sys.dont_write_bytecode = True

broken = sys.argv[1]
STAGE_DIR = ".claude/gate.d"
FIXTURE_DIR = os.path.join(STAGE_DIR, "fixtures")
try:
    # 格名表的读法只有一份：gate-structure-check.py 的 header_cells（文件头注释块里的 # gate-cell: 行），stage-owners 格也导入它
    specification = importlib.util.spec_from_file_location("gate_structure_check", sys.argv[2])
    structure = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(structure)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 导入不了 {sys.argv[2]}：{error}")
    print("     → 怎么办：格名表经它的 header_cells 读；它随仓走，被删了或改坏了就从 git 找回来，别在这一格另写一份读法。")
    sys.exit(1)

# 没有格名表的阶段整道算一格，名字用这个占位
IMPLICIT_CELL = "（整道）"
# 按设计绿的一侧只能退 77（或别的非 0 码）的格，在那份绿样本的 expect 里写一行「not-green-by-design=<格名> <为什么>」
BY_DESIGN_KEY = "not-green-by-design="
BY_DESIGN_MINIMUM_REASON = 8


def read_sample(path):
    """一份样本：(目录名, 退出码, want 列表, 点名的格或 None（没有 .gate-cells）, 按设计不绿的格 → 理由)。"""
    with open(os.path.join(path, "expect"), encoding="utf-8", errors="replace") as handle:
        lines = handle.read().splitlines()
    exit_code = next((line[len("exit="):].strip() for line in lines if line.startswith("exit=")), "")
    wants = [line[len("want="):] for line in lines if line.startswith("want=")]
    by_design = {}
    for line in lines:
        if line.startswith(BY_DESIGN_KEY):
            parts = line[len(BY_DESIGN_KEY):].split(None, 1)
            if parts:
                by_design[parts[0]] = parts[1].strip() if len(parts) > 1 else ""
    marked = None
    marker = os.path.join(path, ".gate-cells")
    if os.path.isfile(marker):
        with open(marker, encoding="utf-8", errors="replace") as handle:
            marked = [entry for entry in (line.split("#", 1)[0].strip() for line in handle) if entry]
    return os.path.basename(path), exit_code, wants, marked, by_design


def summary_says(wants, cell, verdict):
    """样本的 want 里有共用库汇总那一行「  <格名>：<红 / 本次未跑>」。"""
    pattern = re.compile(rf"^\s*{re.escape(cell)}：{verdict}")
    return any(pattern.match(want) for want in wants)


def cell_gaps(cells, samples):
    """每一格至少一份红（exit=1）一份绿（exit=0，或按设计不绿并写了理由）；返回缺的逐格说明。
    一份样本算给哪几格：.gate-cells 点名的格；没有 .gate-cells 的算给整道全部格。
    红样本只点名一格的算给那一格；点名几格的（或整道跑的）只算给 want 里写了「<格名>：红」的格。
    退 0 的样本算给它跑的每一格，want 里写了「<格名>：本次未跑」的那几格除外。"""
    red = {cell: [] for cell in cells}
    green = {cell: [] for cell in cells}
    short_reasons = []
    for kind, exit_code, wants, marked, by_design in samples:
        covered = [cell for cell in (marked if marked is not None else cells) if cell in red]
        for cell in covered:
            if exit_code == "1":
                if len(covered) == 1 or summary_says(wants, cell, "红"):
                    red[cell].append(kind)
            elif exit_code == "0" and not summary_says(wants, cell, "本次未跑"):
                green[cell].append(kind)
            elif cell in by_design and exit_code != "1":
                if len(by_design[cell]) >= BY_DESIGN_MINIMUM_REASON:
                    green[cell].append(kind)
                else:
                    short_reasons.append(f"{kind} 的 {BY_DESIGN_KEY}{cell} 理由不到 {BY_DESIGN_MINIMUM_REASON} 个字")
    gaps = []
    for cell in cells:
        missing = [side for side, found in (("红", red[cell]), ("绿", green[cell])) if not found]
        if missing:
            gaps.append(f"{cell} 缺{'、'.join(missing)}")
    return gaps + short_reasons


judged_cells = 0
uncovered = []
# 阶段是门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批（原来只认两位数开头的，去编号时改了名的那几道会被悄悄漏掉）
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
stages = sorted(path for path in glob.glob(os.path.join(STAGE_DIR, stage_pattern)) if os.path.isfile(path))
if not stages:
    print(f"  ✗ {STAGE_DIR} 下一个 *.sh 阶段都没有")
    print("     → 怎么办：门禁目录搬了位置就同步改这个阶段里的路径；扫到 0 个阶段而报绿，与判过了一模一样。")
    sys.exit(1)

claimed_without_directory, hollow = [], []
claiming = 0
for stage in stages:
    name = os.path.basename(stage)
    head = open(stage, encoding="utf-8").read()
    claims = re.search(rf"fixtures/{re.escape(name)}", head) is not None
    directory = os.path.join(FIXTURE_DIR, name)
    if claims:
        claiming += 1
        if not os.path.isdir(directory):
            claimed_without_directory.append(f"{name}：头部写着 fixtures/{name}/…，而 {directory} 不存在")
            continue
    if not os.path.isdir(directory):
        continue
    if broken == "bare-red-green-only":
        # 弄坏开关：退回只认裸 red / green 目录、不按格判（第二波起样本都叫 <格>-red 这种名字，这样判整批都成空壳）
        kinds = [kind for kind in ("red", "green") if os.path.isdir(os.path.join(directory, kind))]
        if not kinds:
            hollow.append(f"{name}：{directory} 在，里面一个 red / green 都没有")
            continue
        for kind in kinds:
            if not os.path.isfile(os.path.join(directory, kind, "expect")):
                hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")
        continue
    # 与上游 stage-selftest.sh 同一个认法：样本目录下每个子目录都是一份样本，带 expect 的都跑
    samples = []
    for kind in sorted(entry for entry in os.listdir(directory) if os.path.isdir(os.path.join(directory, entry))):
        expect_path = os.path.join(directory, kind, "expect")
        if not os.path.isfile(expect_path):
            hollow.append(f"{name}：{directory}/{kind}/ 里没有 expect，判别力自检会整个跳过它")
            continue
        samples.append(read_sample(os.path.join(directory, kind)))
    if not samples:
        hollow.append(f"{name}：{directory} 在，里面一份带 expect 的样本都没有")
        continue
    cells = [cell for _number, cell, _title in structure.header_cells(stage)] or [IMPLICIT_CELL]
    judged_cells += len(cells)
    gaps = cell_gaps(cells, samples)
    if gaps:
        uncovered.append(f"{name}：{'；'.join(gaps)}")

orphans = []
if os.path.isdir(FIXTURE_DIR):
    for entry in sorted(os.listdir(FIXTURE_DIR)):
        if os.path.isdir(os.path.join(FIXTURE_DIR, entry)) and not os.path.isfile(os.path.join(STAGE_DIR, entry)):
            orphans.append(f"{entry}：样本目录在，而 {STAGE_DIR}/{entry} 不存在")

failed = False
if claimed_without_directory:
    failed = True
    print(f"  ✗ {len(claimed_without_directory)} 个阶段的头部声称有判别力样本，而目录不存在：")  # gate-lint:summary
    for entry in claimed_without_directory:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路，别两边都不做。① 把样本建起来（fixtures/<阶段名>/{red,green}/ 各一份加 expect），")
    print("               让那句声称变成真的；② 这个阶段确实装不进沙箱（要真起虚机、要 cargo 真跑），就把头部那句改成")
    print("               「本阶段没有 fixtures 样本：<为什么装不进>，判别力靠 <别的什么> 」——说清楚，别留一句对不上的声称。")
if orphans:
    failed = True
    print(f"  ✗ {len(orphans)} 个孤儿样本目录，没有对应的阶段：")  # gate-lint:summary
    for entry in orphans:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：阶段退役时把它的样本目录一起删掉；阶段改名了就把目录改成新名字——")
    print("               留着的目录没有任何东西会跑它，而它摆在那里看着就像那个阶段验过了。")
if hollow:
    failed = True
    print(f"  ✗ {len(hollow)} 个样本目录是空壳：")  # gate-lint:summary
    for entry in hollow:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：red 至少要有一条 want=（失败信息里必须出现的片段），green 写 exit=0；")
    print("               少了 expect，判别力自检会跳过那一份，而目录在、看着像验过了。")
if uncovered:
    failed = True
    print(f"  ✗ {len(uncovered)} 道门禁有格缺红或缺绿样本（按格名表逐格数，样本算给哪几格看 .gate-cells 与 expect）：")  # gate-lint:summary
    for entry in uncovered:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：给缺的那一格补一份样本 fixtures/<门禁>/<格>-red（或 -green）/，放 .gate-cells 只点名那一格，")
    print("               expect 写 exit=1 与至少一条 want=（红）或 exit=0（绿）；一份红样本点名几格的，want 里逐格写「  <格名>：红（退出 1）」。")
    print("               绿的一侧按设计只能退 77 的格（例：探针全对上就记本次未跑），在那份绿样本的 expect 里写一行")
    print(f"               「{BY_DESIGN_KEY}<格名> <为什么，至少 {BY_DESIGN_MINIMUM_REASON} 个字>」，别为了凑绿把那一格改成退 0。")
if failed:
    sys.exit(1)

print(f"  ✓ 阶段头部声称的判别力样本都在（{len(stages)} 个阶段里 {claiming} 个声称有样本），"
      f"{FIXTURE_DIR} 下没有孤儿目录，也没有缺 expect 的空壳；有样本的阶段 {judged_cells} 格逐格都有一红一绿")
PY

# ── 第 ④ 条：日期不许是编的 ────────────────────────────
# 判据 source 上游 lib.sh，不另写一份：两套装置算同一个量，就要落到同一个数。
# lib.sh 自带 set -euo pipefail，所以整段关在子 shell 里，判定经文件带出来
# （command-safety.md：子 shell 里的赋值传不回父进程）。
date_report="$(mktemp)"
trap 'rm -f "$date_report"' EXIT
date_rc=0
date_skipped=0
(
  source "$LIB"
  require_date_arithmetic
  git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'SKIP\t不是 git 仓，列不出要扫的文件\n'; exit 0; }
  if start_date="$(project_start_date "$(pwd)")" && [[ -n "$start_date" ]]; then
    start_source="本仓第一个提交"
  else
    start_date="$SOP_START_DATE"; start_source="取不到本仓第一个提交，退回上游的 SOP_START_DATE"
  fi
  # 阳性对照：判据先对一个确知越界的日期跑一次，报不出越界就说明它已经失效。
  # 少了这一句，上游把 date_out_of_range 改名或删掉之后，这一条会**静默报绿**——
  # 调用落在 `if` 条件位置，command not found 的 127 被读成「没越界」，COUNT 照样打印。
  # 2026-09-23 实测：把 lib.sh 里的函数改个名，这一条报「✓ 日期都可能是真的」，数看着完全正常。
  if ! date_out_of_range "1970-01-01" "$start_date" >/dev/null 2>&1; then
    printf 'BROKEN\t判据对 1970-01-01 都报不出越界：date_out_of_range 被改名、删掉，或边界口径变了\n'
    exit 0
  fi
  date_re='[0-9]{4}-[0-9]{2}-[0-9]{2}'
  # 唯一的排除：这一条自己的红样本，里面的越界日期是它的判别力本身（C441（全仓清扫工具会吃掉自己的判别力样本）
  # 说的就是清扫工具会吃掉自己的样本）。**不静默挖空**——跳过几份现算着报进成功句，
  # 读的人看得见排除了什么（show-me-test.md：排除的每一份都要登记，跳过清单要与被扫集合出自同一份数据）。
  own_fixture=".claude/gate.d/fixtures/${STAGE_NAME}/fixture-claims-"
  scanned=0; dated=0; bodies=0; body_dates=0; skipped_own=0; unreadable_bodies=()
  # 路径走 NUL：git 默认把非 ASCII 路径整条引起来并转义成八进制，报出来的那一行既搜不到也对不上 want
  while IFS= read -r -d '' path; do
    # --cached 列的是**索引**不是磁盘：git mv 过又被别的会话 `git reset` 掉暂存时，
    # 索引里还留着旧名，而那个文件早已不在工作区。不滤掉就会判红一批根本不存在的文件
    # （2026-09-23 真仓实测：6 个 git mv 改过名的样本产物被判红，磁盘上一个都没有）。
    [[ -e "$path" ]] || continue
    scanned=$((scanned + 1))
    # 名字就是 <年-月>.md 的文件不含「日」，date_re（年-月-日）那把尺抓不到。这一支判的是全仓任何位置的这种名字，
    # 不专为哪一个目录留：kb 的按月变更史撤了之后照样留着，新建一份按月命名的文件、月份是编的，照样要红。
    # 判法不能拿 2026-08 当 2026-08-01 比——那会误判合法的 2026-08.md
    # （它的第一天早于下界，而这个月本身是合法的）。整月与允许区间没有交集才算越界，
    # 而「没有交集」等价于两个端点都越界，所以不必分辨越的是哪一端，措辞变了也不受影响。
    if [[ "$path" =~ (^|/)([0-9]{4}-(0[1-9]|1[0-2]))\.md$ ]]; then
      month="${BASH_REMATCH[2]}"
      month_end="$(date -d "$month-01 + 1 month - 1 day" +%F)"
      dated=$((dated + 1))
      if date_out_of_range "$month-01" "$start_date" >/dev/null \
         && why="$(date_out_of_range "$month_end" "$start_date")"; then
        printf 'BADNAME\t%s\t%s（整月）\t%s\n' "$path" "$month" "$why"
      fi
    fi
    [[ "$path" =~ $date_re ]] || continue
    dated=$((dated + 1))
    while IFS= read -r found; do
      [[ -n "$found" ]] || continue
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADNAME\t%s\t%s\t%s\n' "$path" "$found" "$why"
      fi
    done < <(grep -oE "$date_re" <<<"$path" | sort -u)
  done < <(git ls-files -z --cached --others --exclude-standard | sort -z -u)
  # 正文只扫 fixtures：真 kb 正文里的日期常常说的是外部的事（别家项目、RFC 历史、引编造日期当例子），
  # 而下界假设「本仓的事不可能早于本仓第一个提交」对它们不成立。
  while IFS= read -r -d '' body; do
    [[ -e "$body" ]] || continue
    if [[ "$body" == "$own_fixture"* ]]; then skipped_own=$((skipped_own + 1)); continue; fi
    # grep 出错（读不了）与「没有日期」不是一回事：读不了的不计进 bodies，逐个列进没扫的清单；-a 让二进制样本按文本读
    # lib.sh 开着 set -e：grep 没命中（退 1）不许把子 shell 带走，退出码在 if 里取
    if hits="$(grep -a -noE "$date_re" "$body" 2>/dev/null)"; then grep_rc=0; else grep_rc=$?; fi
    if ((grep_rc == 2)); then unreadable_bodies+=("$body"); continue; fi
    bodies=$((bodies + 1))
    while IFS= read -r hit; do
      [[ -n "$hit" ]] || continue
      line_number="${hit%%:*}"; found="${hit##*:}"
      body_dates=$((body_dates + 1))
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADBODY\t%s:%s\t%s\t%s\n' "$body" "$line_number" "$found" "$why"
      fi
      # 整行去重，不拿行号当唯一键：`sort -u -t: -k1,1n` 按行号去重，
      # 一行上的第二个日期会被丢掉（实测：`实测（2026-09-20）…已定（2026-01-01）` 判绿）
    done < <(sort -u <<<"$hits")
  done < <(git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/' | sort -z -u)
  printf 'COUNT\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$scanned" "$dated" "$bodies" "$body_dates" "$start_date" "$skipped_own" "$own_fixture" "$start_source" "${unreadable_bodies[*]:-（没有）}"
) > "$date_report" 2>&1 || date_rc=$?

if grep -qE '^BAD(NAME|BODY)' "$date_report"; then
  date_rc=1
  if grep -q '^BADNAME' "$date_report"; then
    echo "  ✗ $(grep -c '^BADNAME' "$date_report") 个文件名里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADNAME' "$date_report")
  fi
  if grep -q '^BADBODY' "$date_report"; then
    echo "  ✗ $(grep -c '^BADBODY' "$date_report") 处样本正文里的日期不可能是真的："   # gate-lint:summary
    while IFS=$'\t' read -r _ where found why; do
      printf '      %s：%s %s\n' "$where" "$found" "$why"   # gate-lint:detail
    done < <(grep '^BADBODY' "$date_report")
  fi
  echo "     → 怎么办：写真实发生的那一天——记录写它是哪天记下的，产物写它是哪天跑的，"
  echo "       样本写它所属 fixture 的创建日：git log --reverse --format=%ad --date=short -- <fixture 目录>；"
  echo "       样本里要表示先后两次事件的，就用创建日和它的次日，别用 2026-01-01 / 2026-01-02 这种占位。"
  echo "       查不出来也别填占位日期：占位日期读起来和真日期一模一样，后面每一个引用它的人都会当真。"
  echo "       红样本非要一个编造日期不可的，让 fixtures/<阶段>/red/setup.sh 在临时仓里现造，别摆进仓里——"
  echo "       摆进来它会被这一条自己扫到，红的是样本而不是被测的东西。"
elif grep -q '^BROKEN' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期判据失效了，这一条什么也没在判："
  grep '^BROKEN' "$date_report" | cut -f2 | sed 's/^/      /'   # gate-lint:detail
  echo "     → 怎么办：判据是上游 .claude/singlefs-ai-sop/scripts/lib.sh 的 date_out_of_range，本仓不另写一份。"
  echo "       先 grep 那个文件确认函数还在、名字没变；同步过副本就读一遍 CHANGELOG 看边界口径改没改。"
  echo "       上游真改了名，改这一条去跟上新名字，别在本仓另抄一份判据——两套装置算同一个量就会分叉。"
elif grep -q '^SKIP' "$date_report"; then
  printf '  ⊘ 日期这一条未跑：%s\n' "$(grep '^SKIP' "$date_report" | cut -f2)"
  date_skipped=1
elif ! grep -q '^COUNT' "$date_report"; then
  date_rc=1
  echo "  ✗ 日期这一条没跑完，报告里没有计数行："
  sed 's/^/      /' "$date_report"   # gate-lint:detail
  echo "     → 怎么办：上面是子 shell 的原样输出；多半是 lib.sh 取不到或 git 列不出文件，按它说的修。"
else
  IFS=$'\t' read -r _ scanned dated bodies body_dates start_date skipped_own own_fixture start_source unreadable_list < <(grep '^COUNT' "$date_report")
  # 计数缺一个就判红：成功句是给人看的唯一结论，它报着空数照样绿，比没有这一条更糟
  # （2026-09-23 变异实测：砍掉「没有 COUNT 就判红」那一支，屏幕上出现「%s 个文件里  条带日期」四个数全空）。
  if ! [[ "$scanned" =~ ^[0-9]+$ && "$dated" =~ ^[0-9]+$ && "$bodies" =~ ^[0-9]+$ && "$body_dates" =~ ^[0-9]+$ && "$skipped_own" =~ ^[0-9]+$ ]]; then
    date_rc=1
    echo "  ✗ 日期这一条的计数行残缺，报不出检查了多少项："
    grep '^COUNT' "$date_report" | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：子 shell 多半在数完之前就退出了，上面那行是它吐出的原样；"
    echo "       扫到 0 项也不是通过，报着空数的成功句与判过了一模一样（show-me-test.md）。"
  else
    printf '  ✓ 日期都可能是真的（文件名：%s 个文件里 %s 条带日期；样本正文：%s 份文件里 %s 个日期；下界 %s 往前宽 7 天，下界取自%s）\n' \
      "$scanned" "$dated" "$bodies" "$body_dates" "$start_date" "$start_source"
    printf '    没扫的 %s 份：%s* 下的文件——这一格自己的样本，里面现造的越界日期就是它的判别力（C441（全仓清扫工具会吃掉自己的判别力样本））\n' \
      "$skipped_own" "$own_fixture"
    printf '    读不了、没扫的样本正文：%s\n' "$unreadable_list"
  fi
fi
if [[ "$python_rc" -ne 0 || "$date_rc" -ne 0 ]]; then
  exit 1
fi
if ((date_skipped)); then
  echo "  ⊘ 本次无对象可判（第 ④ 条）：前三条判过了，第 ④ 条整条没跑，这一格不报通过"
  exit 77
fi
exit 0
}

# ── 格 kb-registry：kb 目录里的每一份都要在 CLAUDE.md 的「项目本地事实」表里有一行
#
# 判据：三条，任一条不成立判红。
#   ① `.claude/kb/` 根目录下每一份 `.md`，都要在 `CLAUDE.md` 的「## 项目本地事实」表里被按路径点名；
#   ② `.claude/kb/` 下每一个子目录也要被点名（形态是带斜杠的路径，例如 `.claude/kb/decisions/`）；
#   ③ 反过来，那张表里点到的每一个 `.claude/kb/…` 路径都要真的存在。
#
# 为什么：那张表是 kb 各文件职责的唯一登记位——一份文件是什么、归谁读、与别处什么关系，只写在那里一行。
# 新建一份 kb 文件而不登记，它对别的会话与派出去的 agent 就是个来历不明的东西；删掉一份而不撤行，
# 表就指向空处。stage-owners 格早就在盯 `stage-owners.tsv` 与 `.claude/gate.d/` 逐项一致，kb 这一侧一直没有对应的闸。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：建 `.claude/kb/feature-bits.md` 时漏了登记，一查那张表还同时漏着
# `INDEX.md`、`term-renames.md`、`tooling.md` 三份——四份文件没有任何东西说得出它们是什么。
#
# 「登记」只认那张表里一行的**第一格**写的路径（逐字相等）：表外散文里提到的反引号路径不算，
# 一行 `.claude/kb/` 也不替它下面的每一份登记。「指得到东西」那一条对那一节里点到的每个 `.claude/kb/…` 路径判，
# 带 `<…>` 占位的形态也是一个路径，指不到东西照样判红。
#
# ⚠️ 射程：判的是「登记了没有」，判不了那一行写得对不对——后者要人看。
# `decisions/`、`experiments/` 这类由别的阶段管形状的子目录同样要有一行，表里写它与谁同进退。
#
# 双向比对用这一道文件头的 MANIFEST_LIBRARY，与 rules-manifest、stage-owners、agent-write-scope 三格同一份代码。
#
# 判别力：fixtures/code-tooling.sh/kb-registry-red 放一份没登记的 kb 文件与一行指向空处的登记，必须判红；
# 它的表里另有一行只写 `.claude/kb/`、散文里另提一份文件，这两处都不许替那两份没登记的文件登记；kb-registry-green 两边对齐。
cell_kb_registry() {
python3 - "$MANIFEST_LIBRARY" <<'PY'
import glob, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)

KB = ".claude/kb"
MANIFEST = "CLAUDE.md"
SECTION = "## 项目本地事实"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

if not os.path.isdir(KB):
    print(f"  ! 没有 {KB}，这一格无对象可判")
    sys.exit(77)
if not os.path.isfile(MANIFEST):
    fail(f"找不到 {MANIFEST}", ["怎么办：项目说明改了名就同步改这个阶段里的路径。"])

text = open(MANIFEST, encoding="utf-8").read()
start = text.find(SECTION)
if start < 0:
    fail(f"{MANIFEST} 里没有「{SECTION}」这一节", [
        f"怎么办：这一节是 kb 各文件职责的登记位。改了标题就同步改这个阶段里的 SECTION，别让这道闸扫空。",
    ])
end = text.find("\n## ", start + len(SECTION))
section = text[start:end if end > 0 else len(text)]
registered = set(re.findall(r"`(\.claude/kb/[^`]*)`", section))
# 登记只认表里一行的第一格
table_lines = [line for line in section.split("\n") if line.strip().startswith("|")]
first_cells = set()
for line in table_lines:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    first_cells |= set(re.findall(r"`(\.claude/kb/[^`]*)`", cells[0])) if cells else set()
if not registered:
    fail(f"「{SECTION}」那一节里一个 `.claude/kb/…` 路径都没点到", [
        "怎么办：扫到 0 项而报绿，与判过了一模一样。那一节的表每行第一格写路径，用反引号包起来。",
    ])

on_disk = set()
for path in sorted(glob.glob(f"{KB}/*.md")):
    on_disk.add(path)
for entry in sorted(os.listdir(KB)):
    full = os.path.join(KB, entry)
    if os.path.isdir(full):
        on_disk.add(full + "/")

def is_registered(path):
    # 逐字等于表里某一行的第一格才算登记；前缀不算——一行 `.claude/kb/` 会让每一项都满足前缀
    return path in first_cells

def points_at_something(entry):
    return os.path.exists(entry) or os.path.exists(entry.rstrip("/"))

missing, dangling = manifest.two_way(sorted(on_disk), sorted(registered),
                                     is_registered=is_registered, exists=points_at_something)

failed = False
if missing:
    failed = True
    print(f"  ✗ kb 里 {len(missing)} 份没有在 {MANIFEST} 的「{SECTION}」表里登记：")  # gate-lint:summary
    for path in missing:
        print(f"      {path}")  # gate-lint:detail
    print(f"     → 怎么办：往那张表加一行，第一格写路径（反引号包起来）、第二格写它是什么、归谁读、与别处什么关系。")
    print("               写不出那一行，多半说明这份文件该并进已有的某一份，或者压根不该单开。")
if dangling:
    failed = True
    print(f"  ✗ 那张表里 {len(dangling)} 个路径指向不存在的东西：")  # gate-lint:summary
    for entry in dangling:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：文件搬家或改名了就同步改那一行（.claude/rules/path-moves.md）；删掉了就把那一行撤掉——")
    print("               指向空处的登记比没有登记更糟，它让人以为那一格有人管着。")
if failed:
    sys.exit(1)

print(f"  ✓ kb 里 {len(on_disk)} 项（{len([p for p in on_disk if p.endswith('.md')])} 份 .md、"
      f"{len([p for p in on_disk if p.endswith('/')])} 个子目录）都在 {MANIFEST} 的「{SECTION}」表里登记着，"
      f"表里 {len(registered)} 个路径也都指得到东西")
PY
}

# ── 格 gate-structure：每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2
#
# 判据与起法都在 research/scripts/gate-structure-check.py，这一格只调它、按它的退出码记：门禁目录顶层每道 *.sh
# （共用库住在 lib/ 下，不在顶层，不判）文件头有格名表；`bash <阶段> <根> --list --force` 退 0、输出与格名表逐格相同、没起 cargo、QEMU、herd7 这类重活；
# `--check <不存在的格>` 退 2。规则：.claude/rules/verification.md「门禁的结构」。
# 为什么：用户 2026-09-28 定「门禁中的每一条都可以单独出来跑 不要编程一个测试函数 要可以全跑 也可以分项跑」；
# 没有这一格，新加一道门禁照老样子写成一个大函数、不认 --list，没有东西会红（records/2026-09-28-门禁59号提速与双机分片.md「第二轮」4b 行）。
# 脚本取这一道所在的仓里那一份（取自 $0 的目录，不取项目根），判的是被判的根：样本仓里放坏的脚本碰不到它。
# 退出码照它：0 绿；77（门禁目录里一道都没有）本次未跑；别的（1 结构不对、2 参数错、78 它自己的准入与运行条件没满足）都按红记。
# 判不了：见那份脚本文件头「判不了的」（按绝对路径起的重活、格判得对不对、「判什么」写得准不准）。
# 样本：fixtures/code-tooling.sh/gate-structure-red 的 gate.d 里一道没有格名表、一道 --check 写错格名照跑退 0、一道 --list 打的与格名表不一致，
# 必须逐道点名判红；gate-structure-green 两道都照规矩（格名表、--list 与表一致、--check 写错退 2），判绿并报查了 2 道。
cell_gate_structure() {
structure_check="$STAGE_REPOSITORY/research/scripts/gate-structure-check.py"
[[ -f "$structure_check" ]] || { echo "  ✗ 找不到 $structure_check"; echo "     → 怎么办：它随仓走（research/scripts/gate-structure-check.py），被删了就从 git 找回来；别在这一格另写一份判法。"; exit 1; }
structure_exit=0
python3 "$structure_check" "$ROOT" || structure_exit=$?
[[ "${CODE_TOOLING_BREAK:-}" == structure-exit-ignored ]] && structure_exit=0
case "$structure_exit" in
  0|77) exit "$structure_exit" ;;
  *)
    echo "  ✗ 门禁结构不对：gate-structure-check.py 退 $structure_exit（逐道列在上面）"
    echo "     → 怎么办：照它上面那一句改；退 2 是参数错，退 78 是它自己的准入与运行条件没满足，按它打的出路办。"
    exit 1
    ;;
esac
}

# ── 格 stage-order：环境检查那一道排在门禁目录里全部跑测试的阶段之后
#
# 判据：共享 gate.sh 按文件名排序逐道起本地阶段（.claude/singlefs-ai-sop/scripts/gate.sh 里 find "$GATE_D" -maxdepth 1 -name '*.sh' … | sort 那一行）；
# 环境检查那一道（ENVIRONMENT_STAGE）要排在门禁目录里每一道跑测试的阶段后面。排序用同一条 find … | sort 现算，不另写一份比较。
# 「跑测试的阶段」：去掉 # 注释行与出路文字之后（CODE_WITHOUT_TEXT_LIBRARY，与 research-script-selftest-coverage 格同一套），
# 代码里起了 cargo test / build / run / nextest / bench，起了 replay.sh、mutate.sh、mutation-shard-run.sh、layer0-shard-run.sh、
# multi-host-run.sh、vm-bench.sh、lkmm.sh 之一，或点名 qemu-system、herd7（TEST_RUNNING）。
# 为什么（判决 research/prompts/gate-shrink-r1-main-verification.md 3-B6 与「采纳的改法」第 2 条；攻方 research/prompts/gate-shrink-r1-opus-output.md 第三节 1）：
# 环境检查里与时刻无关的那几类（残留 loop / dm 设备、挂载、带 singlefs 的 qemu 进程、宿主盘只读与剩余空间、ext4 错误计数、内核日志）
# 判的是它开跑那一刻的机器，排在它后面的阶段编的、建的、留下的，这一趟看不见；去编号以后次序只由名字定，改一个名就能悄悄挪到它前面。
# 环境检查那一道不在门禁目录里判红（改了名就同步改 ENVIRONMENT_STAGE；删了就连这一格一起删）；门禁目录里一道跑测试的阶段都没有退 77。
# 判不了：用别的写法起测试的阶段（命令拼在变量里、经这里没列的脚本间接起）认不出，要把那个脚本名加进 TEST_RUNNING；
# 环境检查自己判得对不对；排序用的是跑这一格时的 locale（整轮门禁下就是 gate.sh 的，两边一样）。
# 样本：fixtures/code-tooling.sh/stage-order-red 的 gate.d 里 harness-tests.sh 起 cargo test、排在 harness-test-environment.sh 后面，必须点名它判红；
# 同一份里排在后面、只在注释与 echo 里提到 cargo test 的 zz-docs.sh 不算，不许点名。stage-order-green 把跑测试的那一道改名成 harness-model-scenarios.sh，
# 排到环境检查前面，判绿并报 2 道跑测试的阶段。
ENVIRONMENT_STAGE="harness-test-environment.sh"
cell_stage_order() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
# 与共享 gate.sh 起本地阶段同一条取法、同一个排序
stage_order="$(find "$ROOT/.claude/gate.d" -maxdepth 1 -name '*.sh' \( -type f -o -type l \) | sort)"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$ENVIRONMENT_STAGE" "$stage_order" "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
environment_stage, broken = sys.argv[2], sys.argv[4]
ordered = [os.path.basename(path) for path in sys.argv[3].split("\n") if path.strip()]
TEST_RUNNING = re.compile(r'\bcargo\s+(?:\+\S+\s+)?(?:test|build|run|nextest|bench)\b'
                          r'|\b(?:replay|mutate|mutation-shard-run|layer0-shard-run|multi-host-run|vm-bench|lkmm)\.sh\b'
                          r'|\bqemu-system|\bherd7\b')
if not ordered:
    print("  ! .claude/gate.d/ 顶层一道阶段都没有，这一格无对象可判")
    sys.exit(77)
if environment_stage not in ordered:
    print(f"  ✗ 门禁目录里没有环境检查那一道 {environment_stage}（顶层 {len(ordered)} 道：{'、'.join(ordered)}）")
    print("     → 怎么办：它改了名就同步改这一格的 ENVIRONMENT_STAGE；整道删了就连这一格一起删——留着一个指向空处的名字，次序就没人判了。")
    sys.exit(1)
environment_position = ordered.index(environment_stage)
test_stages = []
for position, name in enumerate(ordered):
    if name == environment_stage:
        continue
    with open(os.path.join(".claude/gate.d", name), encoding="utf-8", errors="replace") as handle:
        found = TEST_RUNNING.search(code_text.code_of(handle.read()))
    if found:
        test_stages.append((position, name, found.group(0)))
if not test_stages:
    print(f"  ! 门禁目录顶层 {len(ordered)} 道里一道跑测试的阶段都没认出来，这一格无对象可判")
    sys.exit(77)
late = [] if broken == "stage-order-ignored" else [entry for entry in test_stages if entry[0] > environment_position]
if late:
    print(f"  ✗ {environment_stage} 排在第 {environment_position + 1}/{len(ordered)} 道，这些跑测试的阶段排在它后面（它们编的、建的、留下的，这一趟环境检查看不见）：")  # gate-lint:summary
    for position, name, evidence in late:
        print(f"     {name}（第 {position + 1} 道，代码里有「{evidence}」）")  # gate-lint:detail
    print(f"     → 怎么办：给这几道改名，让它们按 gate.sh 的 find … | sort 排到 {environment_stage} 前面（照五类起名：checker-tier-<内容>.sh、harness-<排在 test-environment 前面的内容>.sh）；")
    print("               环境检查那一道要改名的，名字仍要排在全部跑测试的阶段之后，并同步改这一格的 ENVIRONMENT_STAGE。")
    sys.exit(1)
print(f"  ✓ {environment_stage} 排在第 {environment_position + 1}/{len(ordered)} 道，门禁目录里 {len(test_stages)} 道跑测试的阶段都排在它前面"
      f"（{'、'.join(name for _position, name, _evidence in test_stages)}）；没认成跑测试的另外 {len(ordered) - 1 - len(test_stages)} 道")
PY
}

stage_cells_run "$ROOT"
