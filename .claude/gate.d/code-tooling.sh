#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录、跑研究脚本与共用库自己的自证、起各道门禁的 --list；research-gate-lint 格调的上游 gate-lint.sh、shell-lint.sh 要 gawk，缺了由它们自己拒绝（退 78），那一格按红记，不交给 gate.sh 预判；function-units-two-implementations-agree 格编 unit-graph 要的 cargo 缺了那一格自己判红
# gate-stage: 工具层自检（十八格：研究脚本的自证都过、有阶段在跑；带判定库的脚本自证里复用三问都过；项目规则清单；阶段归属表按格登记；钩子注册与写范围表与定义头；改动范围只许一份取法；研究脚本与钩子的拒绝出路与 shell 纪律；阶段声称的样本与编造的日期；带编号登记位的样本都排在 doc-lint 之外；kb 登记表；门禁结构；环境检查排在跑测试的阶段之后；目录形态；阶段在 cd 之前取好自己的位置；并行度默认值只经每台默认重活线程数的登记位取；长活声明按现算读的脚本豁免表指得到、真要靠现算才齐、只缩不涨；函数单元两份实现对拍）
# gate-fixture: yes
# gate-category: 代码类
# 不写 gate-covers：.claude/gate-not-implemented.tsv 里的两项判的是被测代码的行为，这一道只判工具层自己。
# gate-similar: checker-tier-research-build-and-replay.sh 它编 research/ 的装置、跑它们的单测，不跑研究脚本的自证（stage-mine.py、relabel-item.py、claim-experiment.sh 这几份的自证不要 research 工作区先编好，同别的研究脚本一样在这里的 runner 表里跑）；这一道的 research-script-selftests 格跑 runner 表里研究脚本的自证、research-script-selftest-coverage 格核每份实现了 --selftest 的都有阶段在跑，不编译
# gate-similar: harness-model-differential-and-scenarios.sh 它在真仓上判 harness 用例一条一个场景，crash-case-check.py 的判别力由这一道 research-script-selftests 格跑它的 --selftest；判的对象一个是用例源码，一个是脚本的自证
# gate-similar: doc-text.sh 也扫全仓文件，但判的是文本里登记过的禁用写法与旧术语；这一道 fixture-claims 格判文件名与样本目录里编出来的日期、阶段头部声称的样本在不在
# gate-similar: code-source-discipline.sh 它判实验二进制的变异表、绝对值断言与读数纪律，对象是实验源码；crates-mutation-rows.py 与 check-segment-registry.py 这类脚本的自证由这一道 research-script-selftests 格替它们跑
# gate-similar: doc-process-records.sh 它是改动范围共用取法的使用者之一，判三方与同步留下的记录；这一道 change-range-single-source 格判每个阶段只经那一份取法、列路径带 quotepath，对象是门禁阶段的源码
# gate-similar: doc-experiments.sh 它也是改动范围共用取法的使用者，判实验页与产物；这一道只判工具层自己（脚本、钩子、登记表、定义头、门禁结构与次序），不读实验页
# gate-similar: doc-registries.sh 它判欠账表与里程碑收口表这类 kb 登记表；这一道判工具层自己的登记（阶段归属表、写范围表、kb 登记表对目录）与自证
# gate-similar: harness-test-environment.sh 它判这台机器此刻有没有残留设备、挂载、盘满；这一道 stage-order 格只判它的文件名排在跑测试的阶段之后，不读机器，放进它里面就成了它判自己
# gate-similar: hooks-registered.sh 上游这一道判 settings.json 里注册的钩子在、hook-events 挂全、带 --selftest 的自检通过；这一道 agent-write-scope 格判的是项目点名的那几个钩子挂在要的 matcher 上、写范围表与定义双向一致、定义头，钩子的自证只归它跑
# gate-similar: manifest.sh 上游这一道判规范包自己的 CLAUDE.md 与 rules/ 对得上；这一道 rules-manifest 格判项目本地 .claude/rules/ 与项目 CLAUDE.md 的 @ 引用，射程不重叠
# gate-similar: doc-lint.sh 上游这一道的 J 段判排除表自己的形状（理由、目录在、下面有 .md）；这一道 fixture-registrations-excluded 格判样本目录里有登记位的阶段都在排除表里、排除表没有指向 fixtures 下不存在的阶段，对象是样本与表的对账，不并进 doc-lint
# gate-similar: stage-selftest.sh 上游这一道拿 fixtures 下的红绿样本证明本地阶段会红；这一道 fixture-claims 格只判阶段头部的声称与样本目录对不对得上，gate-structure 格只判结构（格名表、--list、--check），都判不了样本本身有没有判别力
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十八格按下面的次序全跑，格名写错退 2）：
# gate-cell: research-script-selftests runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
# gate-cell: research-script-selftest-coverage scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
# gate-cell: reuse-selftests 登记表 verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了
# gate-cell: rules-manifest 项目规则文件与 CLAUDE.md 的 @ 引用逐项相等
# gate-cell: stage-owners 阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根
# gate-cell: agent-write-scope 项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份
# gate-cell: change-range-single-source 门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码
# gate-cell: research-gate-lint 研究脚本、钩子与 .claude/scripts 的拒绝出路、shell 纪律、执行位，终止进程只许点名一个
# gate-cell: fixture-claims 阶段头部的显式标记声称的样本真的在（yes 要目录在，none 要目录不在且登记在豁免表）；文件名与样本正文里的日期不许是编的
# gate-cell: fixture-registrations-excluded 样本目录里有编号登记位（## D<n> / ## E<n> 标题、doc-lint:registry 标记）的阶段，都在 .claude/doc-lint-exclude 里有一行；排除表里指向 fixtures 下不存在的阶段的行判红
# gate-cell: kb-registry kb 目录里的每一份都在 CLAUDE.md「项目本地事实」表里有一行
# gate-cell: gate-structure 每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2
# gate-cell: stage-order 环境检查那一道排在门禁目录里全部跑测试的阶段之后
# gate-cell: directory-layout research/ 与 scripts/ 的顶层、说明文件、层级与不放产物合目录规矩
# gate-cell: stage-location-before-cd 门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname "$0" 找旁边的文件
# gate-cell: heavy-thread-defaults 脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨
# gate-cell: long-run-header-exempt 长活声明（# long-run-hosts:）按现算读的脚本豁免表指得到、脚本真要靠现算读法才齐、只缩不涨
# gate-cell: function-units-two-implementations-agree 函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）dump 逐单元相同，登记入口的闭包与键逐个相同
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头），这里只登记格、解析项目根。
#
# 判别力：fixtures/code-tooling.sh/ 下每一格至少一红一绿，每份样本根放 .gate-cells 只点名它判的那一格：
#   research-script-selftests          runner-red、runner-green、runner-light-guard-red、runner-light-guard-green、runner-unit-key、runner-plan-exit-red、runner-plan-sentinel-red
#   research-script-selftest-coverage  red、green、coverage-claims-red、coverage-claims-green
#   reuse-selftests                    reuse-selftests-red、reuse-selftests-green
#   rules-manifest                     rules-manifest-red、rules-manifest-green
#   stage-owners                       stage-owners-red、stage-owners-green
#   agent-write-scope                  agent-write-scope-red、agent-write-scope-multiedit-red、agent-write-scope-timer-red、agent-write-scope-run-scope-red、agent-write-scope-green
#   change-range-single-source         change-range-single-source-red、change-range-single-source-green
#   research-gate-lint                 research-gate-lint-red、research-gate-lint-green、research-gate-lint-arithmetic-shift-red
#   fixture-claims                     fixture-claims-red、fixture-claims-unnumbered-red、fixture-claims-cells-red、fixture-claims-unmarked-red、
#                                      fixture-claims-marker-malformed-red、fixture-claims-none-with-directory-red、fixture-claims-exempt-red、fixture-claims-green
#   fixture-registrations-excluded     fixture-registrations-excluded-red、fixture-registrations-excluded-stale-red、fixture-registrations-excluded-green
#   kb-registry                        kb-registry-red、kb-registry-green
#   gate-structure                     gate-structure-red、gate-structure-green
#   stage-order                        stage-order-red、stage-order-green
#   directory-layout                   directory-layout-red、directory-layout-green
#   stage-location-before-cd           stage-location-before-cd-red、stage-location-before-cd-green
#   heavy-thread-defaults              heavy-thread-defaults-red、heavy-thread-defaults-cfg-test-red、heavy-thread-defaults-capped-elsewhere-red、heavy-thread-defaults-quoted-name-red、heavy-thread-defaults-per-line-red、heavy-thread-defaults-wide-cost-red、heavy-thread-defaults-var-source-red、heavy-thread-defaults-var-source-forms-red、heavy-thread-defaults-per-line-green、heavy-thread-defaults-var-source-green、heavy-thread-defaults-green
#   long-run-header-exempt             long-run-header-exempt-red、long-run-header-exempt-green、long-run-header-exempt-sha256-red、long-run-header-exempt-repin-red、long-run-header-exempt-base-green
#   function-units-two-implementations-agree  function-units-two-implementations-agree-green、-dump-red、-closure-red、-key-red（Rust 那一侧是录好的输出，不编、不起 unit-graph）
# 每格的样本放了什么写在那一格的注释里。各份绿样本里 vm-bench.sh、stage-owners.tsv、定义、settings.json 这副架子是并道时为八格全跑补的，留着不碍事。
# 汇总与 --check 的弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让这一道每份 exit=1 的样本（red 与各格的 *-red）都判错；别的开关见 lib/stage-cells.sh 文件头。
# 格的判法各有弄坏开关 CODE_TOOLING_BREAK=<项>，逐项写开关名、弄坏哪一处、打开后哪几份样本判错（样本自检报那一份退出码不对或 want 找不到）：
#   runner-exit-swallowed    research-script-selftests 回读时把每条的退出码都当 0：runner-red 判绿
#   light-guard-skipped      research-script-selftests 每条不经 light-selftest-guard.py、直接起（判定照「盯了、没扫到」写）：runner-light-guard-red 判绿，
#                            runner-light-guard-green 里「本次未盯 1 条」那条 want 找不到
#   selftest-key-whole-runner-table  research-script-selftests 每一条自证的键另加表里每一条自证的脚本（整张表一份输入）：runner-unit-key 里
#                            「· 按条复用：表里 3 条自证，现判 1、复用 2、拒算 0」与「复用「bash scripts/gate/sample-pass-a.sh --selftest」」那几条 want 找不到
#                            （只改了 c 那一份，另两条也现判）
#   selftest-judge-whole-guard  research-script-selftests 的判法取整份 light-selftest-guard.py：runner-unit-key 三条都拒算，「现判 1、复用 2、拒算 0」那几条 want 找不到
#   按条复用失灵判红的弄坏开关归 admission.py：ADMISSION_BREAK=plan-failure-not-red 让 runner-plan-exit-red、runner-plan-sentinel-red 判绿（退回全部现判、不判红）
#   reuse-selftest-line-unchecked  reuse-selftests 不看规范输出那一行、自证退 0 就算过：reuse-selftests-red 里「输出里没有 REUSE_SELFTEST」那条 want 找不到
#   claims-any-file          research-script-selftest-coverage 退回「全文出现 --selftest 就算声称有」：coverage-claims-green 判红
#   numbered-stages-only     research-script-selftest-coverage 与 fixture-claims 只认两位数开头的阶段：coverage-claims-green 判红、fixture-claims-unnumbered-red 退 77，
#                            fixture-claims-green（cells-covered.sh 不扫，「有样本的阶段 3 格」那条 want 找不到）、fixture-claims-cells-red（cells-uncovered.sh 不扫，扫到 0 个阶段，「有格缺红或缺绿」那条 want 找不到）
#   bare-red-green-only      fixture-claims 退回只认裸 red / green 样本目录、不按格数红绿：fixture-claims-green 判红（cells-covered.sh 成了空壳）、fixture-claims-cells-red 里那条「有格缺红或缺绿」的 want 找不到
#   claims-by-prose          fixture-claims 退回按散文子串认声称（标记一概不看、不对豁免表）：fixture-claims-none-with-directory-red 判绿（头部没有 fixtures/<它> 的句子，目录在而红绿齐全，退 0）、
#                            fixture-claims-green 判红（55-negated.sh 的否定句被当成声称，而目录不在）、fixture-claims-unmarked-red 里「头部注释块里没有 # gate-fixture: 行」那条 want 找不到、fixture-claims-exempt-red 与 fixture-claims-marker-malformed-red 退 77（标记一概不看，这两份的阶段头部没有 fixtures/<它> 的句子，无对象可判）
#   unmarked-stage-passes    fixture-claims 对没写标记的阶段直接跳过、照记通过：fixture-claims-unmarked-red 判绿（60-prose-only.sh、61-late-marker.sh 都被跳过，退 0）
#   owner-rows-ignored       stage-owners 不判「--list 出的每一格都有一行」：stage-owners-red 里那几条 want 找不到
#   structure-exit-ignored   gate-structure 不看 gate-structure-check.py 的退出码：gate-structure-red 判绿
#   stage-order-ignored      stage-order 不比次序：stage-order-red 判绿
#   safety-top-level-only    research-gate-lint 的进程安全那一路只扫 scripts/ 顶层、不交 scripts/<类>/ 子目录：research-gate-lint-red 里 scripts/gate/stop-processes.sh、
#                            scripts/gate/stop.py 那五条 want 找不到；research-gate-lint-green 里「查了 5 个脚本」「own-scope 标注放行 1 处」两条 want 找不到
#   bare-cd-unchecked        research-gate-lint 不判裸 cd：research-gate-lint-red 里 scripts/gate/bare-cd.sh 那条 want 找不到；research-gate-lint-green 里「裸 cd：查了」那条 want 找不到
#   heredoc-in-arithmetic    裸 cd 那一项把算术里的 << 也当 heredoc 起始（改前的认法）：research-gate-lint-arithmetic-shift-red 判绿（后面的裸 cd 被当成 heredoc 正文，退 0）、
#                            那份里「裸 cd：scripts/gate/arithmetic-shift.sh:5」那条 want 找不到
#   syntax-unchecked         research-gate-lint 不判语法（不跑 bash -n 与 python 的 compile）：research-gate-lint-red 里 scripts/gate/broken-quote.sh、
#                            scripts/gate/broken-syntax.py 两条 want 找不到；research-gate-lint-green 里「语法：查了」那条 want 找不到
#   dirname-after-cd-ignored stage-location-before-cd 不判 cd 之后的 dirname：stage-location-before-cd-red 判绿
#   registrations-unscanned  fixture-registrations-excluded 把样本里的登记位一概认成没有：fixture-registrations-excluded-red 判绿（退 0），
#                            fixture-registrations-excluded-green 里「2 道有登记位」那条 want 找不到；fixture-registrations-excluded-stale-red 照红（指向不存在阶段的那一半不读登记位）
#   thread-default-strings-scanned、thread-default-half-unchecked、thread-default-exempt-unchecked、thread-default-exempt-growth-unchecked、
#   thread-default-timeout-unchecked、thread-default-own-variable-unchecked、thread-default-capped-list-ignored、thread-default-shell-local-unchecked、
#   thread-default-test-module-unskipped、thread-default-cfg-test-without-mod、thread-default-fallback-by-file、thread-default-statement-single-line、
#   thread-default-subscript-write-counted、thread-default-shell-inline-assignment-unseen、thread-default-quoted-name-needs-environment-read、
#   thread-default-conditional-assignment-unpinned、thread-default-external-mod-skipped、thread-default-exempt-by-file、
#   thread-default-conditional-assignment-literal-only、thread-default-printenv-unseen、thread-default-conditional-variable-source-ignored、
#   thread-default-registry-substring-accepted、thread-default-defaulted-variable-accepted、thread-default-other-assignment-forms-unseen、thread-default-sourcing-ignored
#                            heavy-thread-defaults 格的二十五个开关，各弄坏哪一处、哪份样本判错写在那一格的注释里
#   long-run-exempt-path-unchecked、long-run-exempt-needless-unchecked、long-run-exempt-growth-unchecked、long-run-exempt-repin-unchecked、long-run-exempt-sha256-unchecked
#                            long-run-header-exempt 格的五个开关（折成 lib_long_run.py 的 LIB_LONG_RUN_BREAK 同名去掉前缀 long-run- 的项），各弄坏哪一处、哪份样本判错写在那一格的注释里
# agent-write-scope 格的弄坏开关 AGENT_WRITE_SCOPE_BREAK=any-model：关掉定义 model 取值那一条，agent-write-scope-red 照样退 1，
#   「model 不是 opus、sonnet、haiku、inherit 之一」与「delta：model: local-model」两条 want 找不到；
#   AGENT_WRITE_SCOPE_BREAK=multiedit-ignored：写范围闸的注册只要求覆盖 Write 与 Edit、不看 MultiEdit，agent-write-scope-multiedit-red 照样退 1，
#   「写范围闸没挂全工具」与「没覆盖：MultiEdit」两条 want 找不到；
#   AGENT_WRITE_SCOPE_BREAK=timer-monitor-ignored：定时唯一办法闸的注册不要求覆盖 Monitor，agent-write-scope-timer-red 照样退 1（Stop 那一半仍红），
#   「定时唯一办法闸没挂全工具」与「没覆盖：Monitor」两条 want 找不到；
#   AGENT_WRITE_SCOPE_BREAK=run-scope-unchecked：不判范围声明闸注册在 Bash 的 matcher 上，agent-write-scope-run-scope-red 判绿（期望退出 1，实测 0）。
#
#   bash .claude/gate.d/code-tooling.sh [项目根]                                十八格都跑
#   bash .claude/gate.d/code-tooling.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-tooling.sh --check <格名>[,<格名>…] [项目根]       只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd -P)"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
# research-script-selftests 格分批的并行度取每台默认重活线程数（heavy_thread_default，用这一道所在仓里那一份）
source "$STAGE_REPOSITORY/scripts/hosts/heavy-threads.sh"

stage_cell research-script-selftests cell_research_script_selftests "runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）" \
  "按红的那条自证给的下一步修被测脚本，再单独跑那一条命令看它转绿；runner 表在这一道的 SELFTEST_RUNNERS"
stage_cell research-script-selftest-coverage cell_research_script_selftest_coverage "scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑" \
  "把没人跑的自证加进这一道的 SELFTEST_RUNNERS；确实不该每轮跑的登记进 NOT_RUN_HERE 并写明为什么；过期的豁免删掉"
stage_cell reuse-selftests cell_reuse_selftests "登记表 verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了" \
  "照逐条的出路改：自证里补 reuse-three-questions 那一格（scripts/gate/lib_reuse_checks.py 的 report_three_questions 打规范输出），或在 .claude/gate.d/verdict-library-scripts.tsv 补、改那一行；判据在 lib_reuse_checks.py 文件头「四」"
stage_cell rules-manifest cell_rules_manifest "项目规则文件与 CLAUDE.md 的 @ 引用逐项相等" \
  "补上 CLAUDE.md 里缺的 @.claude/rules/<文件> 引用（写在代码围栏与行内代码之外），或删掉悬空引用；两侧逐项相等"
stage_cell stage-owners cell_stage_owners "阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根" \
  "照各道门禁的 --list 在 .claude/gate.d/stage-owners.tsv 逐格补行、改名、删行（四列：门禁文件名、格名、判红时派给谁修、为什么）；列不出格的先修 gate-structure 格报的结构；不认项目根的阶段加 ROOT=\"\${1:-…}\" 与 cd \"\$ROOT\""
stage_cell agent-write-scope cell_agent_write_scope "项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份" \
  "照上面逐条的出路改 .claude/settings.json 的 hooks、.claude/hooks/agent-write-scope.tsv 或定义的 frontmatter；共用模块的函数从模块导入，不另写一份"
stage_cell change-range-single-source cell_change_range_single_source "门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码" \
  "基准与改动范围只经 scripts/gate/changed-paths.sh 的 gate_diff_base / gate_changed_paths / gate_added_lines 取，列路径的 git 带 -c core.quotepath=false，调共用取法判退出码"
stage_cell research-gate-lint cell_research_gate_lint "研究脚本、钩子与 .claude/scripts 的拒绝出路、shell 纪律、执行位，终止进程只许点名一个" \
  "拒绝补出路（sop-first.md「每一条拒绝都必须给出下一步」），shell 纪律照 command-safety.md，终止进程照 .claude/hooks/bash-command-detector.sh 文件头 ⑥ 只点名一个自己起的"
stage_cell fixture-claims cell_fixture_claims "阶段头部的显式标记声称的样本真的在（yes 要目录在，none 要目录不在且登记在豁免表）；文件名与样本正文里的日期不许是编的" \
  "给头部补一行 # gate-fixture: yes 或 # gate-fixture: none <为什么>（none 同时登记进 .claude/gate.d/fixture-none-exempt.tsv），建起 yes 声称的样本目录，删掉孤儿样本目录、补齐 expect；日期写真实发生的那一天，红样本要编造日期就在 setup.sh 里现造"
stage_cell fixture-registrations-excluded cell_fixture_registrations_excluded "样本目录里有编号登记位（## D<n> / ## E<n> 标题、doc-lint:registry 标记）的阶段，都在 .claude/doc-lint-exclude 里有一行；排除表里指向 fixtures 下不存在的阶段的行判红" \
  "在 .claude/doc-lint-exclude 加一行 .claude/gate.d/fixtures/<阶段>/  # 理由（样本里造的决策与实验页与真表同号不同名，doc-lint 把两份一起扫会报多处登记位）；指向不存在的阶段的行删掉或改成现在的阶段名"
stage_cell kb-registry cell_kb_registry "kb 目录里的每一份都在 CLAUDE.md「项目本地事实」表里有一行" \
  "往 CLAUDE.md「## 项目本地事实」表加一行（第一格写反引号包起来的路径），文件搬家或删掉就同步改或撤那一行"
stage_cell gate-structure cell_gate_structure "每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2" \
  "改用 .claude/gate.d/lib/stage-cells.sh（写法在它的文件头）：文件头逐格写 # gate-cell: <格名> <判什么>，每格一个函数用 stage_cell 登记；规则在 .claude/rules/verification.md「门禁的结构」"
stage_cell stage-order cell_stage_order "环境检查那一道排在门禁目录里全部跑测试的阶段之后" \
  "给排在环境检查后面的那几道跑测试的阶段改名，让它们按 gate.sh 的 find … | sort 排到 harness-test-environment.sh 前面；环境检查那一道改了名就同步改这一格的 ENVIRONMENT_STAGE"
stage_cell directory-layout cell_directory_layout "research/ 与 scripts/ 的顶层、说明文件、层级与不放产物合目录规矩" \
  "照 .claude/rules/directory-layout.md 挪文件、补说明文件；判法与弄坏开关在 scripts/gate/directory-layout-check.py 文件头"
stage_cell stage-location-before-cd cell_stage_location_before_cd "门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname \"\$0\" 找旁边的文件" \
  "在 cd 之前取阶段自己的位置存进变量，cd 之后用那个变量；点名的文件:行改完再单跑这一格"
stage_cell heavy-thread-defaults cell_heavy_thread_defaults "脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨" \
  "shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default、python 经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取，起命令写 capped.sh default；不是重活的登记进 .claude/heavy-thread-exempt（<路径>:<锚串>  # 理由），排除只缩不涨"
stage_cell long-run-header-exempt cell_long_run_header_exempt "长活声明（# long-run-hosts:）按现算读的脚本豁免表指得到、脚本真要靠现算读法才齐、只缩不涨" \
  "表 .claude/long-run-live-hosts-exempt 一行「<相对仓根的路径>  <脚本文件的 sha256>  # 理由」；路径搬了就改、脚本改过（sha256 对不上）就撤回改动、脚本文件头已改成登记值读法（多机一行用 lib_long_run.py 的规范句）就删掉那一行，新脚本不往表里加；判法与开关在 .claude/hooks/lib_long_run.py 文件头"
stage_cell function-units-two-implementations-agree cell_function_units_two_implementations_agree "函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）dump 逐单元相同，登记入口的闭包与键逐个相同" \
  "照点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份；判法与样本写在这一格的注释里"
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

# 被判的根是样本：不是这一道所在的仓，也不在 gate.sh --staged 的临时树里（与共用库认 .gate-cells 同一种判法）。
judged_root_is_sample() {
  [[ "$(pwd -P)" != "$STAGE_CELLS_STAGE_REPOSITORY" && -z "${GATE_STAGED_FROM:-}" ]]
}

# 目录与清单表的双向比对，rules-manifest、stage-owners、agent-write-scope、kb-registry 四格共用；两个方向都判，只判一个方向的另一个方向永远不红。
# 各格的 python 段把它当第一个参数收进来，装成模块：
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

# 去掉注释行与出路文字之后剩下的代码，research-script-selftest-coverage、stage-order 两格共用：认「实现了 --selftest」「在跑某份自证」「在跑测试」都只认代码。
# 装法照 MANIFEST_LIBRARY：code_text = types.ModuleType("lib_code_without_text"); exec(compile(sys.argv[i], "code-without-text-library", "exec"), code_text.__dict__)
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
# 判据：SELFTEST_RUNNERS 里每一条自证都退 0；表全不全归 research-script-selftest-coverage 格。lib/stage-cells.sh 不是阶段，它的自证只在这张表里跑。
#   每一条经 scripts/gate/light-selftest-guard.py --only-if-light 起（登记表取被判那棵树的 .claude/hooks/selftest-registry.tsv）：登记成 light 的，
#   跑的时候它整棵进程树里起了真 cargo / rustc（版本查询之外）、真清场（gpu-quiesce-run.sh 读真多机配置）或私有清场脚本，守卫退 3，这一条照没过记；
#   经 PATH 起的 cargo、rustc 被守卫的替身当场拒、真的那一份不起。判据、射程与本次未查写在守卫文件头。别的（登记成 heavy、heavy-if-set 而变量设了、
#   表外的、不是单独一个 --selftest 的）守卫原样 exec、本次未盯，成功句逐条列名。每条的判定写 <序号>.guard，按表次序回读、数份数，缺一份整格红。
#
# 分批并行：SELFTEST_RUNNERS_STARTED_ALONE 里最慢的那几条先各自单独起、跑到完；其余按表次序每批 N 条并行，一批收齐再起下一批，
#   N = 每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；
#   取不了这一格判红）减去单独起的条数，至少 1。
#
# 按条复用：表里每一条自证一条判定（scripts/gate/admission.py verdict-plan / verdict-record，种类 code-tooling.sh:research-script-selftests），
#   键是那份脚本整份的单元闭包（它 import / source 到的模块与函数、读的数据文件、解释器版本，算法在 admission.py 文件头「单元键」）、
#   这一条命令原文（line:command=）、selftest-registry.tsv 里那份脚本登记的档（line:light-guard-tier=）与判法的闭包（judge:，在这一道所在的仓里算）；不带路径与行号。
#   判法取 light-selftest-guard.py 里判轻重的那几个函数（SELFTEST_JUDGE_ENTRIES：认哪条命令要盯、真工具链的位置、替身、进程树与 execve 记录怎么判），
#   不取整份程序：它的 main / guard 起的是调用方给的那条命令（subprocess 的命令折不出），整份闭包必拒算、每一条都现判；被起的那条命令已由入口与 line:command= 进键。键相同、判过绿的不起，表全部复用就一条都不起；拒算的（闭包收不全）照跑、不记。
#   现判而判绿、守卫给了判定的照键记进 <git common-dir>/singlefs-unit-verdicts/code-tooling.sh:research-script-selftests/（盯了而 execve 跟踪没跑的不记）。
#   成功句报「表里 N 条：现判 J、复用 M、拒算 R、本次未跑 0」，现判与拒算逐条列为什么（键对不上时是变了的单元），没过的那一条另报为什么现判。
#   样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库：runner-unit-key 三条自证、上一版只有 c 那一份不同，
#   必须只现判 c、另两条复用。
#   按条复用失灵（verdict-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），这一格把那一趟的输出交给它；它判失灵，这一格照旧全部现判跑完、结论判红。
#   样本 runner-plan-exit-red（.unit-verdict-sample/plan-failure 写 exit-1：verdict-plan 退 1）与 runner-plan-sentinel-red（写 sentinel-reuse-failed：哨兵报复用失灵）
#   各三条自证都退 0，必须判红、点名失灵在哪，三条照样跑完。
#   每条的输出与退出码各写自己的文件，跑完按表次序回读、数份数（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
#   几份自证抢同一份状态（同一个临时路径、同一个端口）的，只在并行下红：那一条挪进 SELFTEST_RUNNERS_STARTED_ALONE 单独起，或让它的自证各用各的临时目录。
#
# 样本：被判的根是样本时（judged_root_is_sample），样本根的 .selftest-runners（一行一条命令，空行与 # 开头的行不算）换掉 runner 表，
#   单独起的那一路不走；真仓与 --staged 下放了它也不认。runner-red 四条里第二条必红，必须判红、点名它并列出输出，另三条照样跑完、
#   报 4 条里 1 条没过；runner-green 三条都退 0，判绿并报跑了 3 条。
#   runner-light-guard-red 与 runner-light-guard-green 各放一份自己的 .claude/hooks/selftest-registry.tsv，把一份样本自证登记成 light：
#   red 那一份经 PATH 起 cargo build（清单不在），必须被守卫的替身拒、这一条退 3 判红，报 2 条里 1 条没过；green 那一份只用样本自己的假 cargo，
#   判绿并报盯了 1 条、本次未盯 1 条（另一条没登记）。
SELFTEST_RUNNERS=(
  "python3 scripts/agents/checklist-specs.py --selftest"
  "python3 scripts/kb/quote-kb.py --selftest" "python3 scripts/kb/kb-sections.py --selftest"
  "python3 scripts/crash/check-segment-registry.py --selftest" "python3 scripts/kb/replace-once.py --selftest"
  "python3 scripts/kb/replace-batch.py --selftest" "python3 scripts/experiments/e152-tables.py --selftest"
  "python3 scripts/agents/agent-watch.py --selftest" "python3 scripts/kb/quote-rust-items.py --selftest"
  "python3 scripts/kb/sweep-term.py --selftest" "python3 scripts/kb/archive-past-rounds.py --selftest"
  "python3 scripts/kb/insert-row.py --selftest" "bash scripts/hosts/cache-keepalive.sh --selftest"
  "bash scripts/agents/watch.sh --selftest" "python3 scripts/kb/stale-candidates.py --selftest"
  "python3 scripts/kb/stale-candidates.py --benchmark" "python3 scripts/hosts/test-environment-check.py --selftest"
  "bash scripts/kb/verify-citations.sh --selftest"
  "python3 scripts/agents/agent-handover.py --selftest" "python3 scripts/kb/decision-slim-check.py --selftest"
  "python3 scripts/kb/pdf-text.py --selftest" "python3 scripts/kb/rules-sweep-audit.py --selftest"
  "python3 scripts/agents/dispatch-prompt.py --selftest" "python3 scripts/agents/three-way-materials.py --selftest"
  "python3 scripts/gate/lib_workspace_lock.py --selftest" "python3 scripts/gate/lib_rebuilt_packages.py --selftest"
  "python3 scripts/gate/lib_function_units.py --selftest"
  "bash scripts/agents/three-way-rerun-template.sh --selftest"
  "bash scripts/gate/merge-and-verify.sh --selftest"
  "python3 scripts/agents/agent-cost.py --selftest" "bash scripts/devices/snapshot-check.sh --selftest"
  "bash scripts/gate/changed-paths.sh --selftest"
  "bash scripts/gate/gate-staged.sh --selftest" "python3 scripts/gate/admission.py --selftest"
  "bash scripts/hosts/capped.sh --selftest"
  "bash scripts/crash/crash-point-replay-shard-run.sh --selftest" "bash scripts/mutation/mutation-shard-run.sh --selftest"
  "bash scripts/hosts/multi-host-run.sh --selftest" "bash scripts/hosts/run-with-memory-cap.sh --selftest"
  "bash scripts/hosts/detached-run.sh --selftest" "bash scripts/experiments/replay.sh --selftest"
  "bash scripts/mutation/mutate.sh --selftest" "python3 scripts/agents/cite-check.py --selftest"
  "python3 scripts/kb/kb-spec-check.py --selftest" "python3 scripts/crash/crash-case-check.py --selftest"
  "python3 scripts/crash/crash-flow-coverage.py --selftest"
  "bash scripts/mutation/prove-red.sh --selftest" "python3 scripts/gate/apply-writer-patch.py --selftest"
  "python3 scripts/gate/lib_libtest_result_lines.py --selftest" "python3 scripts/gate/rerun-failed-tests.py --selftest"
  "python3 scripts/gate/closeout-status.py --selftest" "python3 scripts/hosts/compile-then-swap.py --selftest"
  "python3 scripts/mutation/crates-mutation-rows.py --selftest"
  "bash .claude/gate.d/lib/stage-cells.sh --selftest" "python3 scripts/gate/gate-structure-check.py --selftest"
  "python3 scripts/kb/migrate-changelog-format.py --selftest"
  "bash scripts/crash/crash-amplification.sh --selftest"
  "bash scripts/crash/crash-amplification-item.sh --selftest"
  "bash scripts/crash/crash-amplification-two-hosts.sh --selftest"
  "bash scripts/crash/gpu-quiesce-run.sh --selftest"
  "bash scripts/crash/crash-point-replay.sh --selftest"
  "python3 scripts/mutation/crates-mutation-light-rows.py --selftest" "python3 scripts/crash/gpu-exemptions-check.py --selftest"
  "python3 scripts/crash/cpu-judging-sites-check.py --selftest" "python3 scripts/crash/lib_verdict_library.py --selftest"
  "python3 scripts/mutation/broad-mutation.py --selftest" "python3 scripts/mutation/mutation-prune.py --selftest"
  "python3 scripts/gate/verus-proof-check.py --selftest" "python3 scripts/experiments/e165-pin-red-examples.py --selftest"
  "bash scripts/experiments/e165-replay-mutation-kill-bounds.sh --selftest"
  "python3 scripts/experiments/e165-compare-kill-bounds.py --selftest"
  "python3 scripts/gate/hook-swap.py --selftest" "python3 scripts/hosts/host-dispatch.py --selftest"
  "python3 scripts/gate/light-selftest-guard.py --selftest"
  "python3 scripts/hosts/host-resources.py --selftest" "python3 scripts/hosts/source-stamp.py --selftest"
  "python3 scripts/hosts/minimal-copy.py --selftest" "bash scripts/hosts/work-directory-ownership.sh --selftest"
  "bash scripts/devices/qemu-blklogwrites-check.sh --selftest" "bash scripts/devices/build-patched-qemu.sh --selftest"
  "python3 scripts/gate/stage-mine.py --selftest"
  "python3 scripts/kb/relabel-item.py --selftest" "bash scripts/kb/claim-experiment.sh --selftest"
  "python3 scripts/hosts/target-prune.py --selftest" "python3 scripts/gate/crate-sources-reachable.py --selftest"
  "python3 scripts/gate/directory-layout-check.py --selftest"
  "python3 scripts/gate/data-name-lint.py --selftest" "python3 scripts/gate/wallclock-assert-lint.py --selftest"
  "bash scripts/agents/remind.sh --selftest"
  "python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest"
  "bash scripts/hosts/heavy-threads.sh --selftest" "python3 scripts/hosts/heavy_threads.py --selftest" "python3 scripts/hosts/core_usage.py --selftest"
  "bash scripts/crash/crash-amplification-lib.sh --selftest" "python3 scripts/kb/doc-consolidate.py --selftest" "python3 scripts/kb/history-consolidate.py --selftest"
  "python3 scripts/gate/lib_reuse_checks.py --selftest" "bash scripts/gate/lkmm-selftest.sh --selftest"
  "python3 scripts/experiments/device-code-key.py --selftest" "python3 scripts/gate/shared-index-stale-check.py --selftest"
  "bash scripts/experiments/e180-reading-shards.sh --selftest" "bash scripts/experiments/e182-single-device-long-run.sh --selftest"
  "bash scripts/experiments/e181-landing-check-long-run.sh --selftest" "bash scripts/experiments/e10-aging-compaction-run.sh --selftest"
  "bash scripts/experiments/e182-single-device-units-kept-long-run.sh --selftest"
  "bash scripts/experiments/e183-capacity-cells.sh --selftest" "bash scripts/experiments/e183-sweep-history-shards.sh --selftest"
  "bash scripts/experiments/e184-segment-one-long-run.sh --selftest"
)
SELFTEST_RUNNERS_STARTED_ALONE=("bash scripts/crash/crash-point-replay-shard-run.sh --selftest" "python3 scripts/agents/agent-watch.py --selftest")

# selftest_plan_runners <草稿目录>：table_runners 每一条一条判定请求（这一格文件头「按条复用」），经 scripts/gate/admission.py verdict-plan 算键、查库；
# 结论放进 selftest_decision[<命令>]（reuse、judge、refused）与 selftest_reason[<命令>]，计划文件是 <草稿目录>/plan；算不出返回 1、原因放进 selftest_plan_problem。
# 样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库。
# 弄坏开关 CODE_TOOLING_BREAK=selftest-key-whole-runner-table：每一条的键另加表里每一条自证的脚本（file:），一份脚本改了整张表都作废；
#   =selftest-judge-whole-guard：判法取整份 light-selftest-guard.py（改前的样子），每一条都拒算
SELFTEST_JUDGE_ENTRIES=(
  scripts/gate/light-selftest-guard.py::light_judgement scripts/gate/light-selftest-guard.py::default_settings
  scripts/gate/light-selftest-guard.py::write_shims scripts/gate/light-selftest-guard.py::Watch.scan scripts/gate/light-selftest-guard.py::ExecTrace.poll
)
selftest_plan_runners() {
  local scratch="$1" runner tier name decision key reason entry plan_exit
  local -a words=() table_files=() judge_tokens=()
  for entry in "${SELFTEST_JUDGE_ENTRIES[@]}"; do judge_tokens+=("judge:$entry"); done
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-judge-whole-guard ]] && judge_tokens=("judge:scripts/gate/light-selftest-guard.py")
  local admission="$STAGE_REPOSITORY/scripts/gate/admission.py" kind="$STAGE_NAME:research-script-selftests"
  selftest_plan_problem=""; selftest_plan_failure=()
  : >"$scratch/requests"
  if [[ "${CODE_TOOLING_BREAK:-}" == selftest-key-whole-runner-table ]]; then
    for runner in "${table_runners[@]}"; do read -r -a words <<<"$runner"; [[ -f "${words[1]:-}" ]] && table_files+=("file:${words[1]}"); done
  fi
  for runner in "${table_runners[@]}"; do
    read -r -a words <<<"$runner"
    tier="$(awk -F'\t' -v script="${words[1]:-}" '$1 == script { print $2; exit }' .claude/hooks/selftest-registry.tsv 2>/dev/null)"
    { printf '%s\t%s' "$runner" "${words[1]:-${words[0]:-}}"
      printf '\t%s' "${judge_tokens[@]}" "judge-root:$STAGE_REPOSITORY" "line:command=$runner" "line:light-guard-tier=${tier:-没登记}" \
        ${table_files[@]+"${table_files[@]}"}
      printf '\n'; } >>"$scratch/requests"
  done
  if judged_root_is_sample && [[ -d .unit-verdict-sample/prior ]]; then
    if python3 "$admission" verdict-plan "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/requests" "$scratch/seed.plan" --library-root "$ROOT" >"$scratch/seed.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版的树上算不出键（$(tail -n 2 "$scratch/seed.out" | tr '\n' ' ' | cut -c1-300)）"
      selftest_plan_failure=(verdict-plan "$plan_exit" "$scratch/seed.out"); return 1
    fi
    cut -f1 "$scratch/requests" >"$scratch/seed.names"
    if python3 "$admission" verdict-record "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/seed.plan" "$scratch/seed.names" --how "样本：上一版" --library-root "$ROOT" >"$scratch/seed.record" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版那几条没记进库（$(tail -n 2 "$scratch/seed.record" | tr '\n' ' ')）"
      selftest_plan_failure=(verdict-record "$plan_exit" "$scratch/seed.record"); return 1
    fi
    echo "  · 样本：上一版（.unit-verdict-sample/prior）的 $(grep -c . "$scratch/seed.names") 条自证照判绿记进这一版样本的判定库"
  fi
  if python3 "$admission" verdict-plan "$ROOT" "$kind" "$scratch/requests" "$scratch/plan" >"$scratch/plan.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
  if ((plan_exit != 0)); then
    selftest_plan_problem="admission.py verdict-plan 没成（$(tail -n 2 "$scratch/plan.out" | tr '\n' ' ' | cut -c1-300)）"
    selftest_plan_failure=(verdict-plan "$plan_exit" "$scratch/plan.out"); return 1
  fi
  grep -q '^PLAN ' "$scratch/plan.out" || { selftest_plan_problem="admission.py verdict-plan 没打 PLAN 那一行"; selftest_plan_failure=(verdict-plan 0 "$scratch/plan.out"); return 1; }
  while IFS=$'\t' read -r name decision key reason; do
    [[ -n "$name" && "$name" != PLAN* ]] || continue
    selftest_decision[$name]="$decision"
    selftest_reason[$name]="$reason"
  done <"$scratch/plan.out"
  return 0
}

# selftest_plan_failure_verdict <子命令> <退出码> <输出文件>：这一趟按条复用失灵没有，判法只在 scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路，这里记 selftest_plan_red=1，这一格照旧跑完、结论判红
selftest_plan_failure_verdict() {
  python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" plan-failure "$STAGE_NAME 的 research-script-selftests 格" "$@" || selftest_plan_red=1
}

# selftest_record_greens <草稿目录> <判绿的命令文件>：现判而判绿的那几条照键记进库；记不下只报、不判红，复用哨兵报红（强制现判那一条判定不稳）的经 plan-failure 判红
selftest_record_greens() {
  local record_exit
  [[ -s "$2" ]] || return 0
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-record "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/plan" "$2" \
       --how "经 scripts/gate/light-selftest-guard.py --only-if-light 起" >"$1/record.out" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    echo "  · 判绿记库：$(tail -n 1 "$1/record.out")"
  else
    echo "  ! 判绿记库：admission.py verdict-record 没成（$(tail -n 2 "$1/record.out" | tr '\n' ' ')），下一趟这几条照跑"
    selftest_plan_failure_verdict verdict-record "$record_exit" "$1/record.out"
  fi
}

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

# 按条复用（这一格文件头「按条复用」）：每一条自证一条判定，键相同的不起；表全部复用就一条都不起
selftest_plan_scratch="$(mktemp -d)" || { echo "  ✗ 建不了算键的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
trap 'rm -rf -- "${selftest_plan_scratch:?}"' EXIT
: >"$selftest_plan_scratch/green.names"
table_runners=("${runners[@]}")
declare -A selftest_decision=() selftest_reason=()
selftest_plan_ok=0; selftest_judged=0; selftest_reused=0; selftest_refused=0; selftest_plan_red=0; selftest_plan_failure=()
if selftest_plan_runners "$selftest_plan_scratch"; then
  selftest_plan_ok=1
  runners=()
  for runner in "${table_runners[@]}"; do
    case "${selftest_decision[$runner]:-judge}" in
      reuse) selftest_reused=$((selftest_reused + 1)) ;;
      refused) selftest_refused=$((selftest_refused + 1)); runners+=("$runner") ;;
      *) selftest_judged=$((selftest_judged + 1)); runners+=("$runner") ;;
    esac
  done
  echo "  · 按条复用：表里 ${#table_runners[@]} 条自证，现判 ${selftest_judged}、复用 ${selftest_reused}、拒算 ${selftest_refused}"
  # 现判与拒算的逐条列（至多 20 条，连同为什么：键对不上时是变了的单元）；复用的只在样本或不过 20 条时逐条列
  shown=0
  for runner in "${table_runners[@]}"; do
    case "${selftest_decision[$runner]:-judge}" in
      reuse) continue ;;
      refused) selftest_word="拒算" ;;
      *) selftest_word="现判" ;;
    esac
    shown=$((shown + 1))
    ((shown <= 20)) && echo "    ${selftest_word}「$runner」：${selftest_reason[$runner]:-}"
  done
  ((shown > 20)) && echo "    …现判与拒算另有 $((shown - 20)) 条没列"
  if ((selftest_reused <= 20)) || judged_root_is_sample; then
    for runner in "${table_runners[@]}"; do [[ "${selftest_decision[$runner]:-}" == reuse ]] && echo "    复用「$runner」：${selftest_reason[$runner]:-}"; done
  fi
else
  echo "  ! 按条复用算不出（$selftest_plan_problem）：这一趟表里每一条都现判、一条都不记库"
  ((${#selftest_plan_failure[@]})) && selftest_plan_failure_verdict "${selftest_plan_failure[@]}"
fi
if ((${#runners[@]} == 0)); then
  rm -rf -- "${selftest_plan_scratch:?}"
  echo "  ✓ research 脚本的自证都通过（表取自 $table_source；表里 ${#table_runners[@]} 条都有键相同的判定，全部复用，这一格一条都没起：现判 0、复用 ${selftest_reused}、拒算 0、本次未跑 0）"
  exit 0
fi

if ! thread_cap="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），自证分批的并行度定不了，一条自证都没跑"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default 或 SINGLEFS_THREAD_CAP，再跑这一格；不退回按核数算。"
  exit 1
fi
batch_size=$((thread_cap - ${#alone_set[@]}))
((batch_size >= 1)) || batch_size=1

scratch="$(mktemp -d)" || { echo "  ✗ 建不了放逐条输出与退出码的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
started_pids=()
light_selftest_guard="$STAGE_REPOSITORY/scripts/gate/light-selftest-guard.py"
# start_selftest <表里的序号> <命令>：后台起一条，输出写 <序号>.out、退出码写 <序号>.exit、守卫的判定写 <序号>.guard，进程号记进 started_pids
start_selftest() {
  # runner 表每条是「解释器 脚本 参数」，按空白切词起，与原来逐条顺序跑时同一个起法；经守卫起：登记成 light 的盯着它的进程树，别的守卫原样 exec
  if [[ "${CODE_TOOLING_BREAK:-}" == light-guard-skipped ]]; then
    { printf 'watched\tfindings=0\tscan=no\texamined=0\texit=-\n' > "$scratch/$1.guard"; $2 > "$scratch/$1.out" 2>&1; printf '%s\n' "$?" > "$scratch/$1.exit"; } &
  else
    { python3 "$light_selftest_guard" --only-if-light --registry-root "$ROOT" --verdict-file "$scratch/$1.guard" -- $2 > "$scratch/$1.out" 2>&1
      printf '%s\n' "$?" > "$scratch/$1.exit"; } &
  fi
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

collected=0; failed_count=0; uncollected=(); watched_count=0; unwatched=(); unjudged=(); untraced=()
for index in "${!runners[@]}"; do
  runner="${runners[$index]}"
  if [[ ! -f "$scratch/$index.exit" ]]; then uncollected+=("$runner"); continue; fi
  collected=$((collected + 1))
  rc="$(<"$scratch/$index.exit")"
  [[ "${CODE_TOOLING_BREAK:-}" == runner-exit-swallowed ]] && rc=0
  guard_verdict=""
  [[ -f "$scratch/$index.guard" ]] && guard_verdict="$(head -n 1 "$scratch/$index.guard")"
  case "$guard_verdict" in
    watched$'\t'*)
      watched_count=$((watched_count + 1))
      # 判定文件末尾的 trace=no：守卫没能经 strace 跟踪 execve（strace 不在或起不了），这一条只有采样与替身那一半，不当绿报
      [[ "$guard_verdict" == *$'\ttrace=no'* ]] && untraced+=("$runner") ;;
    not-watched$'\t'*) unwatched+=("$runner") ;;
    *) unjudged+=("$runner") ;;
  esac
  if [[ "$rc" != 0 ]]; then
    failed_count=$((failed_count + 1))
    echo "  ✗ $runner 没过（退出码 $rc）："
    sed 's/^/    /' "$scratch/$index.out"   # gate-lint:detail
    ((selftest_plan_ok)) && echo "    这一条这一趟为什么现判：${selftest_reason[$runner]:-（计划里没有它）}"
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
  elif [[ "${selftest_decision[$runner]:-}" == judge && ( "$guard_verdict" == not-watched$'\t'* || ( "$guard_verdict" == watched$'\t'* && "$guard_verdict" != *$'\ttrace=no'* ) ) ]]; then
    # 现判而判绿、守卫给了判定（盯了而 execve 跟踪没跑的不记：那一条只验了一半）：记进库
    printf '%s\n' "$runner" >>"$selftest_plan_scratch/green.names"
  fi
done
rm -rf "${scratch:?}"
((selftest_plan_ok)) && selftest_record_greens "$selftest_plan_scratch" "$selftest_plan_scratch/green.names"
if ((collected != ${#runners[@]})); then
  echo "  ✗ runner 表派出去 ${#runners[@]} 条，只收回 $collected 份退出码；没收回的：${uncollected[*]}"
  echo "     → 怎么办：临时目录在跑的过程中被删了或写不进去；看 \$TMPDIR 再重跑这一格，收不全的一轮不算判过。"
  exit 1
fi
if ((${#unjudged[@]})); then
  echo "  ✗ scripts/gate/light-selftest-guard.py 没给 ${#unjudged[@]} 条写判定（盯没盯说不清）：$(printf '「%s」' "${unjudged[@]}")"
  echo "     → 怎么办：单跑 python3 scripts/gate/light-selftest-guard.py --only-if-light --verdict-file <文件> -- <那条命令> 看它报什么；判定不全的一轮不算判过。"
  exit 1
fi
if ((failed_count)); then
  echo "  ✗ runner 表 ${#runners[@]} 条里 $failed_count 条没过（上面按表次序逐条列着）"
  echo "     → 怎么办：逐条按那份自证给的下一步修；只在并行下红、单跑绿的，是几份自证抢同一份状态，照这一格文件头那一段办。"
  exit 1
fi
if ((selftest_plan_red)); then
  echo "  ✗ research 脚本的自证这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；表里 ${#table_runners[@]} 条照旧全部现判跑完、都过了"
  echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests；不能复用就判红，不退回全部现判当绿"
  exit 1
fi
unwatched_text=""
((${#unwatched[@]})) && unwatched_text="：$(printf '「%s」' "${unwatched[@]}")"
untraced_text=""
((${#untraced[@]})) && untraced_text="；其中 ${#untraced[@]} 条的 execve 跟踪本次未跑（strace 不在或起不了，只有采样与替身那一半）：$(printf '「%s」' "${untraced[@]}")"
selftest_counts="表里 ${#table_runners[@]} 条：现判 ${selftest_judged}、复用 ${selftest_reused}、拒算 ${selftest_refused}、本次未跑 0"
((selftest_plan_ok)) || selftest_counts="表里 ${#table_runners[@]} 条，按条复用算不出、全部现判"
echo "  ✓ research 脚本的自证都通过（这一格跑了 ${#runners[@]} 条，表取自 $table_source；${selftest_counts}；单独起 ${#alone_pids[@]} 条，其余 $((${#runners[@]} - ${#alone_pids[@]})) 条按每批至多 $batch_size 条并行、共 $batches 批；登记成 light 的 $watched_count 条经 light-selftest-guard.py 盯着进程树、经 strace 记下每一次 execve，没起真 cargo / rustc / qemu / herd7 / verus（版本查询除外）与真清场，本次未盯 ${#unwatched[@]} 条$unwatched_text$untraced_text）"
}

# ── 格 research-script-selftest-coverage：scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
#
# 判据：scripts/ 里实现了 --selftest 的脚本，要么有阶段在调它（脚本名后直接跟 --selftest，或先赋给变量再 "$变量" --selftest），
#   要么登记进 NOT_RUN_HERE 写明为什么；NOT_RUN_HERE 里那份脚本已经有阶段在跑、或不再实现 --selftest 的，是过期豁免，也红。
# 「实现了」只认 .sh / .py 经 code_of 去掉注释与出路文字之后的代码，不列具体写法（==、case、match 都认）；「有阶段在调」同样只认 code_of 之后的代码。
# 阶段取门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批。
#
# 样本：red 放一份只在某个阶段的 echo 出路里被提到的 --selftest 脚本，必须报它没人跑；green 放一份真被阶段调用的与 NOT_RUN_HERE 每一行的那份脚本，判绿并报对数。
#   NOT_RUN_HERE 是这一道自己的表，绿样本里缺了哪一行的脚本，那一行就判过期：往表里加一行，同时往 green、coverage-claims-green 里各放一份那个路径的脚本、改两份 expect 的份数。
#   coverage-claims-green 放一份只在注释与出路里提 --selftest 的 .py、一份提到它的 .tsv（都不算实现了）与一份由不带编号的阶段 tooling-runs.sh 在跑的 .sh，
#   判绿并报 5 份中 1 份（另四份是 NOT_RUN_HERE 那四行）；coverage-claims-red 放一份用 match 语句实现 --selftest、没人跑的 .py，必须报它没人跑。
NOT_RUN_HERE=(
  "scripts/devices/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁"
  "scripts/crash/lib_crash_point_replay_items.py	它自己没有自证：用法里提到的 --selftest 指 scripts/crash/crash-case-check.py 的自证（那一条已在这张表里）"
  "scripts/kb/lib_notation_check.py	它自己没有自证：只被 import，调它的两个脚本各自的 --selftest 走一遍拒绝与放行"
  "scripts/mutation/lib_mutation_units.py	它自己没有自证：只被 import、没有命令行入口，判法由 crates-mutation-rows.py 与 mutate.sh 两边的 --selftest 覆盖"
)
cell_research_script_selftest_coverage() {
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
claimed = sorted(path for path in glob.glob("scripts/*/*")
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
echo "  ✓ scripts/ 里声称有 --selftest 的 $claimed_count 份中 $run_count 份有门禁阶段在跑（只认 .sh / .py 代码里的 --selftest）"
echo "    没跑的 $(grep -c '^EXEMPT' <<<"$coverage") 份（登记在这一格的 NOT_RUN_HERE）："
grep '^EXEMPT' <<<"$coverage" | cut -f2,3 | sed 's/\t/：/; s/^/      /'
}

# ── 格 rules-manifest：项目规则清单一致
#
# 判据：.claude/rules/ 下的文件集合与 CLAUDE.md 里 @.claude/rules/ 的引用集合逐项相等；没被引用的规则读不进上下文。
#   代码围栏（``` 或 ~~~ 起止）与行内代码里的 @ 不算引用：Claude Code 不在代码里展开 @ 导入。
# 样本：rules-manifest-red 放一份没被引用的规则、一份只在代码围栏里与一份只在行内代码里被 @ 的规则，必须逐个报出；rules-manifest-green 两侧逐项相等，判绿。
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
# 判据（.claude/rules/verification.md「门禁的结构」）：.claude/gate.d/stage-owners.tsv 一行一格，四列：门禁文件名、格名、判红时派给谁修、为什么。任一条不成立判红：
#   ① 门禁目录顶层每道 *.sh 的 --list 出的每一格在表里恰好一行。--list 经 gate-structure-check.py 的 header_cells、make_shims、start_stage 起
#      （替身拦重活、限时）；没有格名表的不起它（起它就是全跑），与 --list 没退 0、起了重活、一格都没打的一起报「列不出格」；
#   ② 表里每一行的门禁文件在、那一格在它的 --list 里；③ 第三列每个 agent 名都有 .claude/agents/<名字>.md；④ 每行四列、没有空列；
#   ⑤ 每道阶段认第一个参数当项目根（ROOT_ARGUMENT_PATTERN）：只看当前目录的阶段从别的目录带着根参数调用时安静地判错地方，
#      stage-selftest 既 cd 又传参，看不出来。
# 按格登记、不按道：按道时一格红会把同道别的格的派修一起带走。
# 样本：stage-owners-red 的表里有三列的行、同一格登记两行、没有定义的 agent、指向门禁里没有的格与指向不在的门禁的行，
#   目录里有一格没登记、一道没有格名表、一道 --list 退 1、一道不认第一个参数，必须逐个报出；
#   stage-owners-green 两道门禁三格逐格登记，判绿并报门禁数、格数与 agent 数。
cell_stage_owners() {
python3 - "$MANIFEST_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py" "${CODE_TOOLING_BREAK:-}" <<'PY'
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
        exit_code, stdout, stderr, called = structure.start_stage(root, stage, ["--list", "--force"], shim_directory, trace)
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
    print(f"     → 怎么办：在 {table_path} 给每一格加一行：门禁文件名、格名、判红时派给谁修、为什么；没有合适的 agent 就写 investigator。格名照 bash .claude/gate.d/<门禁> --list 抄。")
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
# 判据，任一条不成立判红（「注册」指 settings.json 里有一条 matcher 覆盖那个工具或事件、命令指向那个钩子）：
#   ① PreToolUse 里命令指向 write-guard.sh 的注册，matcher 合起来覆盖 Write、Edit 与 MultiEdit（WRITE_GUARD_TOOLS）；
#   ③ tools 含 Write 或 Edit 的每个定义，在 .claude/hooks/agent-write-scope.tsv 里至少一条路径模式；④ 表里每个 agent 名都有定义，每行至少两列；
#   ⑤ Bash 注册 bash-command-detector.sh；⑥ Agent 注册 runner-dispatch-guard.sh；⑦ SendMessage 注册 continuation-guard.sh；
#   ⑧ 每个定义的 frontmatter 有 omitClaudeMd: true，正文有一行以「开工先读：」开头；
#   ⑨ Bash 注册上游 claude-hooks/pattern-process-guard.sh，且那个文件在（它的判别力归上游 selftest）；
#   ⑩ SessionStart 有一条 matcher 覆盖 compact、一条同时覆盖 startup 与 resume（可以是同一条），命令都指向 session-start.sh；
#   ⑫ Bash 注册 heavy-test-guard.sh；⑭ AskUserQuestion 注册 ask-user-claim-guard.sh；⑯ SubagentHandback 注册 handback-guard.sh；
#   ⑲ Bash 注册 run-scope-guard.sh（范围声明闸；注在不覆盖 Bash 的 matcher 上的不算）；
#   ⑰ PreToolUse 里命令指向 timer-only-remind-guard.sh 的注册，matcher 合起来覆盖 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor 与 Bash（TIMER_GUARD_TOOLS）；
#   ⑱ Stop 里有命令指向 remind-rearm-stop-guard.sh 的注册（循环提醒到点之后自启动靠它）；
#   ⑬ 共用模块 lib_shell_words.py、lib_heavy_tests.py 顶层定义的每个函数名，.claude/hooks/ 下别的文件里不许再 def 一份；名字从模块现读，
#      读不到或一个函数都没有也红；不算的名字登记在 NOT_SHARED_JUDGMENT，登记了而模块里没有的也红；
#   ⑮ frontmatter 的 model 只许 ALLOWED_MODELS 里的取值。
# 钩子的自证不在这里跑，归上游 hooks-registered.sh；它只核有一条注册指向钩子、不核挂在要的 matcher 上，所以 matcher 留在这一格判。
#
# 样本：agent-write-scope-red 的 .claude/hooks/ 里放一份手抄 shell_tokens 的 hook 与一份手抄 classify_process 的 hook，必须各报出文件与行号；
#   agent-write-scope-multiedit-red 的 settings.json 只把 write-guard.sh 注册在 "Write|Edit" 上，必须报没覆盖 MultiEdit；
#   agent-write-scope-timer-red 的 settings.json 把 timer-only-remind-guard.sh 注册在不含 Monitor 的 matcher 上、Stop 里没有 remind-rearm-stop-guard.sh，必须报没挂全工具（没覆盖 Monitor）与没注册停止钩子；
#   agent-write-scope-run-scope-red 是 green 那一份，只把 run-scope-guard.sh 注册在 "Write|Edit|MultiEdit" 上（不覆盖 Bash），必须报没注册范围声明闸；
#   agent-write-scope-green 的 .claude/hooks/ 里放两份从共用模块导入、只写自己判定的 hook（连同被判仓自己那两份模块），必须判绿，成功行数进去的别的文件数要对。
# 共用模块的函数名取自这一道脚本旁边 ../hooks/ 下那两份（取自 $0 的目录，不取项目根）：样本仓里放坏的模块碰不到它。
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
WRITE_GUARD_TOOLS = ("Write", "Edit") if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "multiedit-ignored" else ("Write", "Edit", "MultiEdit")
if entries is not None:
    registered = [entry for entry in entries
                  if any(command_invokes(hook.get("command"), "write-guard.sh") for hook in entry.get("hooks") or [])]
    uncovered_tools = [tool for tool in WRITE_GUARD_TOOLS
                       if not any(matcher_covers(entry.get("matcher", ""), tool) for entry in registered)]
    if not registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册写范围闸：PreToolUse 里没有命令指向 write-guard.sh 的一条")
        for command in mention_only_commands(entries, "write-guard.sh"):
            print(f"     这一条只是提到 write-guard.sh、没有跑它，不算注册：{command}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "Write|Edit|MultiEdit"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/write-guard.sh"。')
    elif uncovered_tools:
        failed = True
        print(f"  ✗ {settings_path} 写范围闸没挂全工具：命令指向 write-guard.sh 的注册，matcher 合起来没覆盖：{'、'.join(uncovered_tools)}"
              "——这几个工具写文件时写范围、角标、钟点、行尾空白那几道判定一道都不跑")
        print('     → 怎么办：把 write-guard.sh 那一条的 matcher 写成 "Write|Edit|MultiEdit"。')
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
    run_scope_registered = [entry for entry in entries
                            if matcher_covers(entry.get("matcher", ""), "Bash")
                            and any(command_invokes(hook.get("command"), "run-scope-guard.sh") for hook in entry.get("hooks") or [])]
    if not run_scope_registered and os.environ.get("AGENT_WRITE_SCOPE_BREAK") != "run-scope-unchecked":
        failed = True
        print(f"  ✗ {settings_path} 没注册范围声明闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 run-scope-guard.sh 的一条——起验证脚本的命令不带范围声明也没人拒")
        for command in mention_only_commands([entry for entry in entries if matcher_covers(entry.get("matcher", ""), "Bash")], "run-scope-guard.sh"):
            print(f"     这一条只是提到 run-scope-guard.sh、没有跑它，不算注册：{command}")  # gate-lint:detail
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/run-scope-guard.sh"（注在别的 matcher 上的挪过来）。')
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
    TIMER_GUARD_TOOLS = ("CronCreate", "ScheduleWakeup", "RemoteTrigger", "Skill", "Bash") if os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "timer-monitor-ignored" else ("CronCreate", "ScheduleWakeup", "RemoteTrigger", "Skill", "Monitor", "Bash")
    timer_registered = [entry for entry in entries
                        if any(command_invokes(hook.get("command"), "timer-only-remind-guard.sh") for hook in entry.get("hooks") or [])]
    timer_uncovered = [tool for tool in TIMER_GUARD_TOOLS
                       if not any(matcher_covers(entry.get("matcher", ""), tool) for entry in timer_registered)]
    if not timer_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册定时唯一办法闸：PreToolUse 里没有命令指向 timer-only-remind-guard.sh 的一条——CronCreate、ScheduleWakeup 这些别的定时办法没人拒")
        print('     → 怎么办：在 hooks.PreToolUse 里加一条 matcher "CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash"、command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/timer-only-remind-guard.sh"。')
    elif timer_uncovered:
        failed = True
        print(f"  ✗ {settings_path} 定时唯一办法闸没挂全工具：命令指向 timer-only-remind-guard.sh 的注册，matcher 合起来没覆盖：{'、'.join(timer_uncovered)}——这几个工具里的别的定时办法没人拒")
        print('     → 怎么办：把 timer-only-remind-guard.sh 那一条的 matcher 写成 "CronCreate|ScheduleWakeup|RemoteTrigger|Skill|Monitor|Bash"。')
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
try:
    stop_entries = (json.load(open(settings_path, encoding="utf-8")).get("hooks") or {}).get("Stop") or []
except Exception:
    stop_entries = None  # 读不了这份 JSON 的那一条上面已经报过、已判红
if stop_entries is not None and not any(command_invokes(hook.get("command"), "remind-rearm-stop-guard.sh") for entry in stop_entries for hook in entry.get("hooks") or []):
    failed = True
    print(f"  ✗ {settings_path} 没注册提醒自启动的停止钩子：Stop 里没有命令指向 remind-rearm-stop-guard.sh 的一条——循环提醒到点退出之后没人让主 agent 重起")
    print('     → 怎么办：在 hooks.Stop 里加一条 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/remind-rearm-stop-guard.sh"。')
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
    print("     → 怎么办：改成 opus、sonnet、haiku 或 inherit（行尾可以带 # 注释）；本地模型不当腿派（.claude/rules/three-way-inference.md「各条腿必须互不重复」），不写进 model。")
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
echo "  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸、弹窗断言闸与定时唯一办法闸注册在要的 matcher 上，提醒自启动的停止钩子注册着，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义、${pattern_count} 条路径模式），共用切词模块的 ${shared_function_count} 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 ${scanned_hook_file_count} 个文件），共用重型测试判定模块的 ${heavy_function_count} 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 ${heavy_scanned_file_count} 个文件；${not_shared_judgment_names} 不算，见 NOT_SHARED_JUDGMENT）；钩子的自证归上游 hooks-registered.sh 跑"
}

# ── 格 change-range-single-source：改动范围只许一份取法：阶段不自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
#
# 判据（射程 .claude/gate.d/ 顶层的 *.sh 与 *.py；这一道文件里只除掉这一格自己：从这一格的标题行到下一格的标题行按空行判），任一条不成立判红：
#   ① 算 diff 基准只许在 scripts/gate/changed-paths.sh 里：代码行出现 merge-base（--is-ancestor 核祖先除外）、@{upstream} 或 @{u}、gate-ok、
#      读 GATE_BASE 或 GATE_DIFF_BASE，或者跑 git 的那一行写死 HEAD~N、HEAD^（HEAD^{tree} 不算）、用三点号（A...B 就是取 merge-base）的，红。
#   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false：同一行带，或经同一个文件里带着它的变量 / 列表、
#      包装函数调，或 git 带 -z；参数列表换了行、这一行以引号开头的，往上看三行找调用前缀。不带时中文路径被转成带引号的八进制串，git 不报错。
#   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 ||，或放在 if / while 里。
#   「代码行」的认法在 is_text：出路文字与注释里提到这些词不算。
# 为什么：取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath），收完之后要有东西拦下一份拷贝。
#
# 样本：change-range-single-source-red 把三条判据该抓的写法各放几种（自己取 merge-base、@{u}...、HEAD~1、HEAD^、换了行的参数列表续行里的 "merge-base"、
#   不带或带 =true 的 quotepath、包装函数名叫 git 而这一行没调它、test 的 -z、printf "$(git …)"、换行的参数列表、不判退出码的共用取法），逐条点名判红；
#   change-range-single-source-green 放五种合法写法与 unset GATE_BASE、merge-base --is-ancestor、HEAD^{tree}、判了退出码的调用，判绿。
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
    print(f'  ✗ {len(base_bad)} 行在阶段里自己算 diff 基准（改动范围只许一份取法：scripts/gate/changed-paths.sh）：')
    for path, number, line in base_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：阶段里 source scripts/gate/changed-paths.sh，基准写 base="$(gate_diff_base gate)"（问「这一次提交带哪些」的写 head），')
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
for pattern in ('scripts/*/*.sh', 'scripts/*/*.py', '.claude/scripts/*', '.claude/hooks/*'):
    for path in sorted(glob.glob(pattern)):
        if os.path.isfile(path) and path != 'scripts/gate/changed-paths.sh':
            base_hits, list_hits, status_hits = judge(path)
            if base_hits or list_hits or status_hits:
                outside.append(f'{path}（算基准 {len(base_hits)} 行、不带 quotepath 的列路径 {len(list_hits)} 行、不判退出码 {len(status_hits)} 处）')
print(f'  ✓ 查了 .claude/gate.d/ 下 {len(scanned)} 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false，调共用取法都判了退出码')
if outside:
    print(f'     没扫的 {len(outside)} 份（不是门禁阶段，射程之外）：' + '；'.join(outside))
else:
    print('     没扫的：scripts/、.claude/scripts/、.claude/hooks/ 下没有碰到这两条的脚本')
PY
}

# ── 格 research-gate-lint：scripts/、.claude/hooks/ 与 .claude/scripts/ 里每一条拒绝都带出路、守 shell 纪律；前两个目录的执行位不丢；连同 .claude/gate.d/，终止进程只许点名一个
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint 与 shell-lint，这三个目录不在它的射程里，这里补上
#   （.claude/singlefs-ai-sop/rules/show-me-test.md「射程只到 .claude/gate.d/」）；两个 lint 都跑完再判，不在第一个红就停。
#   .claude/scripts/ 的执行位仍归共享那一道，这里不重复扫。research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# 进程安全（终止只许点名一个自己起的进程号或任务号）：判定只有一份，在 .claude/hooks/bash-command-detector.sh（拒绝 ⑥），
#   这里经它的 --scan-scripts 扫，射程比两个 lint 多一个 .claude/gate.d/（它的样本目录不扫）。
# 样本：research-gate-lint-red 放一个只带一句话的 die 和一个靠子 shell 赋值往外带值的函数；它的 .claude/scripts/ 里再放一个不带出路的拒绝和一处 pkill -f
#   （这个目录进了射程才报得出来）；进程安全那一路有 scripts/gate/stop-processes.sh（自己任务的进程组、循环里 kill、systemctl 带通配）、
#   scripts/gate/stop.py（循环 os.kill、os.killpg）与 .claude/gate.d/50-stop-group.sh（按 cgroup.procs 批量发、没核 scope 也没标），都必须逐处报出。
#   research-gate-lint-green 的 die 带第二个参数、不靠子 shell 带值，.claude/scripts/ 里放一个干净的包装（成功行的脚本数要把它数进去），
#   scripts/gate/stop-own.sh 只停点名的一个、按 cgroup 批量发的那一段核过 singlefs-memory-cap-*.scope 又标了 own-scope，必须判绿。
#   research-gate-lint-arithmetic-shift-red 是 green 加一个 scripts/gate/arithmetic-shift.sh：先写 `(( shift_bits = 1 << width ))` 与 `mask=$(( 1 << bits ))`，后面一行裸 cd 没接 || 也没接 &&，
#   再跟一个真 heredoc（正文里有一行 cd，不判）；必须只报第 5 行那一处。
cell_research_gate_lint() {
LINT="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
SHELL_LINT="$(dirname "$LINT")/shell-lint.sh"
for script in "$LINT" "$SHELL_LINT"; do
  [[ -f "$script" ]] || { echo "  ✗ 找不到共享脚本 $script"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
done
TARGETS=()
MODE_TARGETS=()
ABSENT=()
for directory in "$ROOT/scripts" "$ROOT/.claude/hooks" "$ROOT/.claude/scripts"; do
  [[ -d "$directory" ]] || { ABSENT+=("${directory#"$ROOT"/}"); continue; }
  TARGETS+=("$directory")
  [[ "$directory" == "$ROOT/.claude/scripts" ]] || MODE_TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 scripts/、.claude/hooks/ 也没有 .claude/scripts/，这一格无对象可判"; exit 77; }
# GATE_LINT_DIR 指到第一个目标目录：不设它，共享脚本默认还会扫 SOP 自己的包，样本目录里红绿就靠那个包碰巧干净来分。
failed=0
if ! GATE_LINT_DIR="${TARGETS[0]}" bash "$LINT" "${TARGETS[@]:1}"; then
  echo "  ✗ scripts/、.claude/hooks/ 或 .claude/scripts/ 里有不带出路的拒绝（上面逐处列出）"
  echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，直接打印的拒绝之后跟一行以箭头开头的出路"
  failed=1
fi
# 执行位也一起：共享门禁的「脚本执行位」阶段只扫 .claude/gate.d 与 .claude/scripts，
# scripts 与 .claude/hooks 不在它的射程里（同 gate-lint / shell-lint 那两条的理由）；.claude/scripts 已在那一道里，不重复扫。
MODES="$(dirname "$LINT")/script-modes.sh"
modes_note=""
if [[ ! -f "$MODES" ]]; then
  echo "  ✗ 找不到共享脚本 $MODES，scripts/ 与 .claude/hooks/ 的执行位这一项没判"
  echo "     → 怎么办：规范副本缺了这一份，把 singlefs-ai-sop-<语言> 仓重新拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh；别让这一项静默不判"
  failed=1
  modes_note="执行位：没判（找不到 script-modes.sh，上面已判红）"
elif ((${#MODE_TARGETS[@]} == 0)); then
  modes_note="执行位：本次无对象可判（射程里没有 scripts/ 也没有 .claude/hooks/）"
else
  modes_rc=0
  bash "$MODES" "${MODE_TARGETS[@]}" || modes_rc=$?
  if (( modes_rc == 77 )); then
    modes_note="执行位：本次未判（script-modes.sh 退 77，原因见它上面那一句）"
  elif (( modes_rc != 0 )); then
    echo "  ✗ scripts/ 或 .claude/hooks/ 里的脚本执行位在暂存区里不对（上面逐处列出）"
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
# --scan-scripts 只扫给它的那一层（不进子目录）；scripts/ 的脚本放在分类子目录 scripts/<类>/ 里（scripts/README.md），逐个子目录交给它。
# 射程到 scripts/<类>/ 这一层为止：实验装置那一层（scripts/experiments/<装置>/）与 scripts/devices/qemu/、scripts/mutation/tables/ 不进（搬迁前它们不在 research/scripts/ 里，也不扫）。
# 弄坏开关 CODE_TOOLING_BREAK=safety-top-level-only 不交子目录（搬迁前的样子，scripts/ 顶层没有脚本，这一路扫空）。
if [[ -d "$ROOT/scripts" && "${CODE_TOOLING_BREAK:-}" != safety-top-level-only ]]; then
  for directory in "$ROOT"/scripts/*/; do
    [[ -d "$directory" ]] && SAFETY_TARGETS+=("${directory%/}")
  done
fi
[[ -d "$ROOT/.claude/gate.d" ]] && SAFETY_TARGETS+=("$ROOT/.claude/gate.d")
if [[ ! -f "$DETECTOR" ]]; then
  echo "  ✗ 找不到 $DETECTOR，终止进程的写法这一项没扫"
  echo "     → 怎么办：恢复 .claude/hooks/bash-command-detector.sh（判定只有那一份，门禁与执行前的钩子共用），再跑这一格：bash .claude/gate.d/${STAGE_NAME} --check research-gate-lint"
  failed=1
elif ! bash "$DETECTOR" --scan-scripts "$ROOT" "${SAFETY_TARGETS[@]}"; then
  echo "  ✗ scripts/、.claude/hooks/、.claude/scripts/ 或 .claude/gate.d/ 里有打得到别人进程的终止写法（上面逐处列出）"
  echo "     → 怎么办：照上面那句「怎么办」改；规矩是「后面的脚本不能终止前面的脚本」「不能动 ssh」「终止要按照任务号终止 禁止终止所有」，写法见 .claude/hooks/bash-command-detector.sh 文件头 ⑥"
  failed=1
fi
# 语法：射程里（连 .claude/gate.d/，不进 fixtures/ 与编译目录 target/）每一份 .sh 过 bash -n、每一份 .py 过 python3 的 compile（只编译、不执行）；
# 搬迁、改名把一份脚本改坏了，提交时红在这里，不等到有人去跑它。弄坏开关 CODE_TOOLING_BREAK=syntax-unchecked 不判这一项
syntax_note="语法：没判（弄坏开关 syntax-unchecked）"
syntax_directories=("${TARGETS[@]}")
[[ -d "$ROOT/.claude/gate.d" ]] && syntax_directories+=("$ROOT/.claude/gate.d")
if [[ "${CODE_TOOLING_BREAK:-}" != syntax-unchecked ]]; then
  syntax_shell_count=0; syntax_python_files=(); syntax_bad=()
  while IFS= read -r -d '' file; do
    case "$file" in
      *.sh)
        syntax_shell_count=$((syntax_shell_count + 1))
        if ! syntax_message="$(bash -n "$file" 2>&1)"; then syntax_bad+=("${file#"$ROOT"/}：${syntax_message%%$'\n'*}"); fi ;;
      *.py) syntax_python_files+=("$file") ;;
    esac
  done < <(find "${syntax_directories[@]}" \( -name fixtures -o -name target \) -prune -o -type f \( -name '*.sh' -o -name '*.py' \) -print0 | sort -z)
  if ((${#syntax_python_files[@]})); then
    while IFS= read -r line; do [[ -n "$line" ]] && syntax_bad+=("${line#"$ROOT"/}"); done < <(python3 - "${syntax_python_files[@]}" <<'PY'
import sys
for path in sys.argv[1:]:
    try:
        with open(path, encoding="utf-8") as source:
            compile(source.read(), path, "exec")
    except (SyntaxError, ValueError, UnicodeDecodeError) as error:
        line = getattr(error, "lineno", None)
        print(f"{path}：第 {line} 行 {type(error).__name__}：{getattr(error, 'msg', error)}")
PY
)
  fi
  if ((${#syntax_bad[@]})); then
    for entry in "${syntax_bad[@]}"; do echo "  ✗ 语法不对：$entry"; done  # gate-lint:detail
    echo "  ✗ 射程里有 ${#syntax_bad[@]} 份脚本过不了语法检查（bash -n 或 python 的 compile，上面逐份列出）"
    echo "     → 怎么办：照每一行报的那一行改（多半是搬迁、改名时替换吃掉了引号或括号），改完单跑这一格：bash .claude/gate.d/${STAGE_NAME} --check research-gate-lint"
    failed=1
  fi
  syntax_note="语法：查了 $((syntax_shell_count + ${#syntax_python_files[@]})) 份（sh ${syntax_shell_count}、py ${#syntax_python_files[@]}），坏 ${#syntax_bad[@]} 份"
fi
# 裸 cd：不开 set -e 的 .sh 里，命令位置上的 cd 后面要接 || 出路或 &&（搬迁之后 cd 进不去、接着在调用方的目录里跑，病因会报错地方）。
# 射程与语法那一项相同；heredoc 里的正文与注释行不判，反斜杠续行接起来判。heredoc 起始是 << 后接分隔词：算术段 $(( … )) 与 (( … )) 里的 << 是移位，<<< 是 here-string，都不算。
# 弄坏开关 CODE_TOOLING_BREAK=bare-cd-unchecked 不判这一项，heredoc-in-arithmetic 把算术里的 << 也当 heredoc 起始
bare_cd_note="裸 cd：没判（弄坏开关 bare-cd-unchecked）"
if [[ "${CODE_TOOLING_BREAK:-}" != bare-cd-unchecked ]]; then
  bare_cd_output="$(python3 - "$ROOT" "${syntax_directories[@]}" <<'PY'
import os, re, sys
def read_word(text, i):
    depth = 0; quote = None
    while i < len(text):
        c = text[i]
        if quote == "'":
            if c == "'": quote = None
        elif c == "\\": i += 1
        elif text.startswith("$(", i) or text.startswith("${", i): depth += 1; i += 1
        elif c in ")}" and depth > 0: depth -= 1
        elif c == '"': quote = None if quote == '"' else ('"' if quote is None else quote)
        elif c == "'" and quote is None: quote = "'"
        elif quote is None and depth == 0 and (c.isspace() or c in ";&|)}"): return i
        i += 1
    return i
def heredoc_delimiter(line):
    # 算术段 $(( … )) 与 (( … )) 里的 << 是移位，<<< 是 here-string，都不是 heredoc 起始；弄坏开关 heredoc-in-arithmetic 退回改前的认法
    if os.environ.get("CODE_TOOLING_BREAK") == "heredoc-in-arithmetic":
        opener = re.search(r"<<-?\s*['\"]?([A-Za-z_]+)['\"]?", line)
    else:
        blanked = re.sub(r"\$?\(\(.*?\)\)", lambda found: " " * len(found.group(0)), line)
        opener = re.search(r"(?<!<)<<(?!<)-?\s*['\"]?([A-Za-z_]+)['\"]?", blanked)
    return opener.group(1) if opener else None
def bare_cd_lines(text):
    if re.search(r'^\s*set\s+(-[a-zA-Z]*e|-o\s+errexit)', text, re.M): return []
    hits = []; joined = []; buffer = ""; first = None; heredoc = None
    for number, line in enumerate(text.split("\n"), 1):
        if heredoc:
            if line.strip() == heredoc: heredoc = None
            continue
        opener = heredoc_delimiter(line)
        if opener and not line.lstrip().startswith("#"): heredoc = opener
        if first is None: first = number
        if line.endswith("\\"): buffer += line[:-1] + " "; continue
        joined.append((first, buffer + line)); buffer = ""; first = None
    for number, line in joined:
        if line.lstrip().startswith("#"): continue
        for match in re.finditer(r'(?:^|[;&|({!`]|\b(?:then|do|else)\b)\s*cd\b', line):
            i = match.end()
            while i < len(line) and line[i] == " ": i += 1
            while i < len(line) and not line.startswith(("||", "&&", ";", "|", ")", "}", "#"), i):
                i = read_word(line, i)
                while i < len(line) and line[i] == " ": i += 1
            rest = line[i:]
            if rest.startswith(("||", "&&")) or re.match(r';\s*[})]\s*(\|\||&&)', rest): continue
            hits.append((number, line.strip()[:120]))
    return hits
root = sys.argv[1]; scanned = 0
for top in sys.argv[2:]:
    for directory, subdirectories, files in os.walk(top):
        subdirectories[:] = sorted(d for d in subdirectories if d not in ("fixtures", "target"))
        for name in sorted(files):
            if not name.endswith(".sh"): continue
            path = os.path.join(directory, name); scanned += 1
            with open(path, encoding="utf-8", errors="replace") as source:
                for number, line in bare_cd_lines(source.read()):
                    print(f"bad\t{os.path.relpath(path, root)}:{number}\t{line}")
print(f"scanned\t{scanned}")
PY
)"
  bare_cd_scanned="$(awk -F'\t' '$1 == "scanned" { print $2 }' <<<"$bare_cd_output")"
  bare_cd_bad=()
  while IFS=$'\t' read -r kind location text; do
    [[ "$kind" == bad ]] && bare_cd_bad+=("$location：$text")
  done <<<"$bare_cd_output"
  if ((${#bare_cd_bad[@]})); then
    for entry in "${bare_cd_bad[@]}"; do echo "  ✗ 裸 cd：$entry"; done  # gate-lint:detail
    echo "  ✗ 射程里有 ${#bare_cd_bad[@]} 处 cd 没接 || 也没接 &&（脚本没开 set -e，cd 失败会在调用方的目录里接着跑）"
    echo "     → 怎么办：写成 cd <目录> || { echo \"进不去 <目录>\"; echo \"→ 怎么办：…\"; exit 2; }，或 (cd <目录> && …)；规矩在 .claude/rules/directory-layout.md「scripts/：反复要用的脚本」"
    failed=1
  fi
  bare_cd_note="裸 cd：查了 ${bare_cd_scanned:-0} 份 .sh，坏 ${#bare_cd_bad[@]} 处"
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
  echo "  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 ${#TARGETS[@]} 个目录（${scanned_names[*]:0:${#TARGETS[@]}}），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 ${#SAFETY_TARGETS[@]} 个目录；${modes_note}；${syntax_note}；${bare_cd_note}"
  echo "    没扫的目录 ${#ABSENT[@]} 个（不在）：${ABSENT[*]:-（没有）}"
fi
exit "$failed"
}

# ── 格 fixture-claims：阶段头部声称的判别力样本必须真的存在；全仓文件名与样本目录里的日期不许是编的
#
# 判据，任一条不成立判红：
#   ① 声称只认显式标记：每道阶段（门禁目录顶层的 *.sh）头部注释块里（第一行代码之前）有且只有一行 `# gate-fixture: yes` 或
#      `# gate-fixture: none <为什么装不进沙箱，至少 8 个字>`，判法只读这一行，头部别的句子里写没写 fixtures/<它> 都不算声称。
#      yes：fixtures/<阶段文件名>/ 要在，再过下面第 ③ 条；none：那个目录要不在（在就红：写了 none 却有样本目录）；
#      没写标记、写了不止一行、写的不是这两种，都红。
#      none 还要登记进豁免表 .claude/gate.d/fixture-none-exempt.tsv（一行一道 `<阶段文件名>  # 理由`，形态照 .claude/preflight-exclude）：
#      写了 none 没登记、登记了而头部写的是 yes、登记指向不存在的阶段、同一道登记两行、理由不到 8 个字，都红。
#      豁免表只缩不涨：补出样本的阶段改成 yes 同时删它那一行，不往表里加新行去绕开这一格；门禁不判涨，靠 review；
#   ② .claude/gate.d/fixtures/ 下每个目录都对应一个在的阶段文件：孤儿样本目录没有东西会跑它，摆在那里看着却像验过了；
#   ③ 样本目录下每个子目录都是一份样本（与上游 stage-selftest.sh 同一个认法），都要有 expect，至少一份；按那道门禁的格名表
#      （gate-structure-check.py 的 header_cells；没有格名表的整道算一格）逐格数，每格至少一份红（exit=1）一份绿（exit=0）。
#      一份样本算给哪几格见 cell_gaps 的文档字符串；绿的一侧按设计只能退 77 的格，那份样本的 expect 里写一行
#      not-green-by-design=<格名> <为什么>（理由至少 8 个字）才算绿；
#   ④ 全仓文件名里的日期，与 fixtures/ 下正文里的日期，都要落在可能的区间里：判据 source 上游 lib.sh 的 date_out_of_range，
#      下界取本仓第一个提交（取不到退回上游的 SOP_START_DATE）。正文只扫 fixtures 的理由写在第 ④ 条那段代码旁。
# 射程：判的是声称与目录对不对得上，判不了样本本身有没有判别力（归 stage-selftest.sh）；不声称（none）的阶段只有豁免表里登记了的才过，
#   不再有「既不声称也没样本、照记通过」的阶段。
# 不是 git 仓时第 ④ 条整条没判，前三条都过也退 77。第 ④ 条正文唯一的排除是这一格自己的样本（fixture-claims-*），这一道别的格的样本照旧扫。
#
# 样本：fixture-claims-red 摆一个假的 gate.d，前三条各犯一次，必须判红；fixture-claims-green 都干净（含一道头部写着「装不进 fixtures/<它>/…」
#   的否定句、标了 none 也登记了的阶段：按散文子串认声称会把它当成声称、判红）。
#   fixture-claims-unnumbered-red 的 gate.d 里一道不带编号的 tooling-claims.sh 标了 yes 而目录不在、另一道带编号的标 none 并已登记，必须点名前者判红。
#   fixture-claims-unmarked-red 一道阶段头部只有「装不进 fixtures/<它>/…」的否定句、没写标记，一道标记写在第一行代码之后，必须都红在「没写标记」。
#   fixture-claims-marker-malformed-red 的三道阶段分别是 none 理由太短、写了两行标记、写的是 maybe，必须逐道点名判红。
#   fixture-claims-none-with-directory-red 的阶段标了 none 也登记了，而样本目录里红绿齐全，必须红在「写了 none 却有样本目录」。
#   fixture-claims-exempt-red 摆三处豁免表对不上：标了 none 没登记、登记了而头部写的是 yes、登记指向不存在的阶段，另有重复行与理由太短的行，必须逐处判红。
#   fixture-claims-cells-red 的 cells-uncovered.sh 三格，样本按 <格>-red / <格>-green 起名：alpha 缺绿；beta 只在点名两格而 want 只写了 gamma 红的样本里
#   （不算给 beta，缺红）；gamma 的绿样本退 77 而 not-green-by-design 的理由太短（缺绿），必须逐格列名判红。
#   fixture-claims-green 的 cells-covered.sh 两格没有裸 red / green：alpha 一红一绿，beta 的红来自点名两格而 want 写了「beta：红」的样本，
#   绿样本退 77 并写了够长的 not-green-by-design，必须判绿。
#   第 ④ 条的坏日期由 fixture-claims-red/setup.sh 在临时仓里现造，不摆进仓里（摆进来会被这一条自己扫到）；setup.sh 要 git init 再提交一个
#   真实日期的提交，否则 project_start_date 取不到值，下界那一支走不到。
cell_fixture_claims() {
LIB="$(cd "$STAGE_DIRECTORY/../.." && pwd)/.claude/singlefs-ai-sop/scripts/lib.sh"
python_rc=0
python3 - "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py" <<'PY' || python_rc=$?
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
# 声称只认头部注释块里的这一行标记；none 要登记进豁免表（只缩不涨，门禁不判涨）
MARKER_PATTERN = re.compile(r"^#\s*gate-fixture:\s*(.*?)\s*$")
MARKER_NAME = "# gate-fixture:"
NONE_MINIMUM_REASON = 8
EXEMPT_TABLE = os.path.join(STAGE_DIR, "fixture-none-exempt.tsv")
EXEMPT_MINIMUM_REASON = 8


def header_markers(path):
    """头部注释块（到第一行代码之前，与 header_cells 同一个认法）里的标记行：[(行号, 标记后面的文字)]。"""
    found = []
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if number == 1 and line.startswith("#!"):
                continue
            if line.strip() and not line.lstrip().startswith("#"):
                break
            match = MARKER_PATTERN.match(line.strip())
            if match:
                found.append((number, match.group(1)))
    return found


def read_exempt_table():
    """豁免表：({阶段文件名: 第一次出现的行号}, 表本身的问题列表)。不存在的表当空表。"""
    entries, problems = {}, []
    if not os.path.isfile(EXEMPT_TABLE):
        return entries, problems
    with open(EXEMPT_TABLE, encoding="utf-8", errors="replace") as handle:
        for number, raw in enumerate(handle, 1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            entry, hash_mark, reason = line.partition("#")
            entry, reason = entry.strip(), reason.strip()
            if not hash_mark or len(reason) < EXEMPT_MINIMUM_REASON:
                problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 的理由不到 {EXEMPT_MINIMUM_REASON} 个字（一行写成「<阶段文件名>  # <为什么装不进沙箱>」）")
            if entry in entries:
                problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 重复登记（第 {entries[entry]} 行已经有了）")
            else:
                entries[entry] = number
    return entries, problems


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
# 阶段是门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
stages = sorted(path for path in glob.glob(os.path.join(STAGE_DIR, stage_pattern)) if os.path.isfile(path))
if not stages:
    print(f"  ✗ {STAGE_DIR} 下一个 *.sh 阶段都没有")
    print("     → 怎么办：门禁目录搬了位置就同步改这个阶段里的路径；扫到 0 个阶段而报绿，与判过了一模一样。")
    sys.exit(1)

claimed_without_directory, hollow = [], []
unmarked, malformed, none_with_directory = [], [], []
yes_stages, none_stages = [], []
for stage in stages:
    name = os.path.basename(stage)
    directory = os.path.join(FIXTURE_DIR, name)
    markers = header_markers(stage)
    if broken == "claims-by-prose":
        # 弄坏开关：退回改前的认法，标记一概不看，整份文件里出现 fixtures/<它> 就算声称（否定句也算），没声称的看目录在不在
        prose = re.search(rf"fixtures/{re.escape(name)}", open(stage, encoding="utf-8").read()) is not None
        if not os.path.isdir(directory):
            if prose:
                claimed_without_directory.append(f"{name}：头部写着 fixtures/{name}/…，而 {directory} 不存在")
            continue
        yes_stages.append(name)
    else:
        if not markers:
            if broken != "unmarked-stage-passes":  # 弄坏开关：没写标记的阶段跳过，照记通过
                unmarked.append(f"{name}：头部注释块里没有 {MARKER_NAME} 行")
            continue
        if len(markers) > 1:
            malformed.append(f"{name}：头部有 {len(markers)} 行 {MARKER_NAME}（第 {'、'.join(str(number) for number, _ in markers)} 行），只许一行")
            continue
        number, value = markers[0]
        word = value.split(None, 1)[0] if value else ""
        if value == "yes":
            yes_stages.append(name)
            if not os.path.isdir(directory):
                claimed_without_directory.append(f"{name}：头部标了 yes（声称有样本），而 {directory} 不存在")
                continue
        elif word == "none":
            reason = value[len(word):].strip()
            if len(reason) < NONE_MINIMUM_REASON:
                malformed.append(f"{name}：第 {number} 行 none 后面的理由不到 {NONE_MINIMUM_REASON} 个字（写成「{MARKER_NAME} none <为什么装不进沙箱>」）")
                continue
            none_stages.append(name)
            if os.path.isdir(directory):
                none_with_directory.append(f"{name}：头部写了 none，而 {directory} 在")
            continue
        else:
            malformed.append(f"{name}：第 {number} 行写的是「{value}」，只认 yes 与 none <为什么>")
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

# 豁免表与头部的 none 逐项对上（claims-by-prose 开关退回的是改前的判法，没有这一段）
exempt_problems = []
if broken != "claims-by-prose":
    exempt_entries, exempt_table_problems = read_exempt_table()
    exempt_problems.extend(exempt_table_problems)
    stage_names = {os.path.basename(stage) for stage in stages}
    for name in none_stages:
        if name not in exempt_entries:
            exempt_problems.append(f"{name}：头部写了 none，而 {EXEMPT_TABLE} 里没有它这一行")
    for entry, number in exempt_entries.items():
        if entry not in stage_names:
            exempt_problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 不是 {STAGE_DIR} 下的阶段")
        elif entry in yes_stages:
            exempt_problems.append(f"{EXEMPT_TABLE}:{number}：{entry} 登记了，而它头部写的是 yes")

failed = False
if unmarked:
    failed = True
    print(f"  ✗ {len(unmarked)} 个阶段头部没写 {MARKER_NAME} 标记，判不出它声称不声称有判别力样本：")  # gate-lint:summary
    for entry in unmarked:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：在头部注释块里（第一行代码之前）加一行「{MARKER_NAME} yes」（样本目录 fixtures/<阶段名>/ 在，每格有红有绿），")
    print(f"               或「{MARKER_NAME} none <为什么装不进沙箱，至少 {NONE_MINIMUM_REASON} 个字>」并把这道阶段登记进 {EXEMPT_TABLE}。")
    print("               头部别的句子里写 fixtures/<它> 不算声称，写在第一行代码之后的标记也不认。")
if malformed:
    failed = True
    print(f"  ✗ {len(malformed)} 个阶段的 {MARKER_NAME} 标记写得不对：")  # gate-lint:summary
    for entry in malformed:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：每道阶段头部只留一行，写「{MARKER_NAME} yes」或「{MARKER_NAME} none <为什么装不进沙箱，至少 {NONE_MINIMUM_REASON} 个字>」。")
if claimed_without_directory:
    failed = True
    print(f"  ✗ {len(claimed_without_directory)} 个阶段的头部声称有判别力样本，而目录不存在：")  # gate-lint:summary
    for entry in claimed_without_directory:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：两条出路，别两边都不做。① 把样本建起来（fixtures/<阶段名>/ 下按格放 <格>-red / <格>-green 各一份加 expect），")
    print("               让那句声称变成真的；② 这个阶段确实装不进沙箱（要真起虚机、要 cargo 真跑），就把头部那行改成")
    print(f"               「{MARKER_NAME} none <为什么装不进>」，并把它登记进 {EXEMPT_TABLE}（一行 `<阶段文件名>  # 理由`）。")
if none_with_directory:
    failed = True
    print(f"  ✗ {len(none_with_directory)} 个阶段头部写了 none，却有样本目录：")  # gate-lint:summary
    for entry in none_with_directory:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：目录里有样本就是有判别力样本，把头部改成「{MARKER_NAME} yes」并删掉豁免表里那一行；")
    print("               样本是废的就连目录一起删，别让 none 和样本目录同时摆着。")
if exempt_problems:
    failed = True
    print(f"  ✗ {len(exempt_problems)} 处豁免表 {EXEMPT_TABLE} 与阶段头部对不上：")  # gate-lint:summary
    for entry in exempt_problems:
        print(f"      {entry}")  # gate-lint:detail
    print(f"     → 怎么办：头部写 none 的阶段在表里有且只有一行「<阶段文件名>  # <理由，至少 {EXEMPT_MINIMUM_REASON} 个字>」；表里每一行都要指向在的阶段、")
    print("               且那道阶段头部写的是 none；阶段补出样本改成 yes 的，同一次改动把表里它那一行删掉。")
    print("               豁免表只缩不涨：不往表里加新行去绕开这一格，装得进沙箱的阶段建样本（这条门禁不判涨，靠 review）。")
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

print(f"  ✓ 阶段头部声称的判别力样本都在（判了 {len(yes_stages) + len(none_stages)} 道：yes {len(yes_stages)} 道、none {len(none_stages)} 道（逐个列名见下）），"
      f"{FIXTURE_DIR} 下没有孤儿目录，也没有缺 expect 的空壳；有样本的阶段 {judged_cells} 格逐格都有一红一绿")
print(f"    yes {len(yes_stages)} 道：{' '.join(yes_stages) or '（没有）'}")
print(f"    none {len(none_stages)} 道（都登记在 {EXEMPT_TABLE}）：{' '.join(none_stages) or '（没有）'}")
PY

# ── 第 ④ 条：日期不许是编的 ────────────────────────────
# lib.sh 自带 set -euo pipefail，所以整段关在子 shell 里，判定经文件带出来（command-safety.md「子 shell 里的赋值传不回父进程」）。
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
  # 阳性对照：上游把 date_out_of_range 改名或删掉时，if 条件里 command not found 的 127 会被读成「没越界」而静默报绿；先拿确知越界的日期试一次。
  if ! date_out_of_range "1970-01-01" "$start_date" >/dev/null 2>&1; then
    printf 'BROKEN\t判据对 1970-01-01 都报不出越界：date_out_of_range 被改名、删掉，或边界口径变了\n'
    exit 0
  fi
  date_re='[0-9]{4}-[0-9]{2}-[0-9]{2}'
  # 唯一的排除：这一格自己的样本，里面的越界日期就是判别力；跳过几份现算着报进成功句。
  own_fixture=".claude/gate.d/fixtures/${STAGE_NAME}/fixture-claims-"
  scanned=0; dated=0; bodies=0; body_dates=0; skipped_own=0; unreadable_bodies=()
  # 路径走 NUL：git 默认把非 ASCII 路径整条引起来并转义成八进制，报出来的那一行既搜不到也对不上 want
  while IFS= read -r -d '' path; do
    # --cached 列的是索引：git mv 过又被 reset 掉暂存时索引里还留着旧名、文件已不在，不滤掉会判红不存在的文件。
    [[ -e "$path" ]] || continue
    scanned=$((scanned + 1))
    # 名字是 <年-月>.md 的文件 date_re 抓不到，单独判：整月与允许区间没有交集（两个端点都越界）才算越界；拿月初比会误判合法的当月。
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
      # 整行去重，不按行号去重：按行号会丢掉同一行上的第二个日期。
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
  # 计数缺一个就判红：报着空数的成功句照样绿，比没有这一条更糟。
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

# ── 格 fixture-registrations-excluded：样本目录里有编号登记位的阶段，都要在 .claude/doc-lint-exclude 里有一行
#
# 判据，任一条不成立判红：
#   ① 扫 .claude/gate.d/fixtures/<阶段>/ 下递归的每一份 .md（代码围栏里的不算，与 doc-lint 取登记位同一个认法）：行首是 `## D<n> ` 或 `## E<n> `（编号登记标题），
#      或有一行含 doc-lint:registry（登记表标记），就是这一道阶段的样本里有登记位。有登记位的阶段，.claude/doc-lint-exclude 里要有一行指到
#      .claude/gate.d/fixtures/<阶段>/（末尾斜杠可有可无；指到它上面的目录也算）。缺的判红，点名阶段与它第一份带登记位的样本文件:行。
#   ② 排除表里指到 .claude/gate.d/fixtures/<阶段>/… 而 fixtures 下没有这个阶段目录的行判红，点名表里的行号。
#      排除表自己的形状（理由、目录在不在、下面有没有 .md、路径合不合法）由 doc-lint 的 J 段判，这里不重判；只判「有登记位却没排除」与「排除了 fixtures 下不存在的阶段」。
# 为什么：各道门禁的判别力样本各自造带登记标题的决策页与实验页，编号互相撞（样本里的 D94、D99 在好几份里同时登记）。样本目录各是一个沙箱，
#   撞号只在 doc-lint 把两份一起扫时才伤人，所以不判「各样本里的编号两两不同」，只判「有登记位的阶段整道排在 doc-lint 之外」；
#   新加一道带登记位的样本而没进排除表，会在整轮门禁的 doc-lint 里炸成「多处登记位」，这一格把它提前到改样本的当场。
# 判不了：登记位写成别的形状（`### D<n>`、两位数以外的前缀字母、非 .md 文件里造的）；doc-lint 的登记标题要求带破折号，这里不要求，宁可多拦。
# 弄坏开关 CODE_TOOLING_BREAK=registrations-unscanned 把登记位一概认成没有：fixture-registrations-excluded-red 判绿，fixture-registrations-excluded-green 里「2 道有登记位」那条 want 找不到。
# 样本（登记标题都在 setup.sh 里现写，静态样本文件里不放真的登记标题，免得本仓的 doc-lint 扫到样本目录就多出登记位）：
#   fixture-registrations-excluded-red 三道阶段：doc-sample.sh 有 `## D5 ` 标题且表里没有它、table-sample.sh 有 doc-lint:registry 标记且表里没有它、
#     covered-sample.sh 有 `## E7 ` 标题而表里有它；必须点名前两道与各自的文件:行，不许点名第三道；
#   fixture-registrations-excluded-stale-red 表里一行指到 fixtures 下不存在的阶段 gone-sample.sh，必须点名表里的行号；
#   fixture-registrations-excluded-green 四道阶段：两道有登记位且都在表里（一行带斜杠、一行不带）、一道没有登记位且不在表里、一道只在围栏里写了登记标题且不在表里，判绿并报扫了 4 道、2 道有登记位。
cell_fixture_registrations_excluded() {
[[ -d .claude/gate.d/fixtures ]] || { echo "  ! 没有 .claude/gate.d/fixtures，这一格无对象可判"; exit 77; }
python3 - "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys
broken = sys.argv[1]
FIXTURES = ".claude/gate.d/fixtures"
EXCLUDE = ".claude/doc-lint-exclude"
HEADING = re.compile(r"^## [DE][0-9]+ ")
FENCE = re.compile(r"^[ \t]*```")
MARKER = "doc-lint:registry"

stages = sorted(name for name in os.listdir(FIXTURES) if os.path.isdir(os.path.join(FIXTURES, name)))
if not stages:
    print("  ! .claude/gate.d/fixtures/ 下一个阶段目录都没有，这一格无对象可判")
    sys.exit(77)


def first_registration(stage):
    """这一道阶段的样本里第一份带登记位的 .md：(相对路径, 行号, 是哪一种)；没有就 None。"""
    if broken == "registrations-unscanned":
        return None
    for directory, subdirectories, files in os.walk(os.path.join(FIXTURES, stage)):
        subdirectories[:] = sorted(name for name in subdirectories if name != ".git")
        for name in sorted(files):
            if not name.endswith(".md"):
                continue
            path = os.path.join(directory, name)
            fenced = False
            with open(path, encoding="utf-8", errors="replace") as handle:
                for number, line in enumerate(handle, 1):
                    if FENCE.match(line):
                        fenced = not fenced
                        continue
                    if fenced:
                        continue
                    if HEADING.match(line):
                        return path, number, "编号登记标题"
                    if MARKER in line:
                        return path, number, "登记表标记"
    return None


# 排除表：与 doc-lint 同一个读法（`<相对目录>  # 理由`，去掉末尾斜杠）；没有 # 的行、空路径是 doc-lint J 段的事，这里不收
entries = []
if os.path.isfile(EXCLUDE):
    with open(EXCLUDE, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            if not line.strip() or line.lstrip().startswith("#") or "#" not in line:
                continue
            path = line.split("#", 1)[0].strip()
            if path.startswith("./"):
                path = path[2:]
            path = path.rstrip("/")
            if path:
                entries.append((number, path))


def covered(stage):
    target = FIXTURES + "/" + stage
    return any(target == path or target.startswith(path + "/") for _number, path in entries)


registered, unregistered, uncovered = [], [], []
for stage in stages:
    found = first_registration(stage)
    if found is None:
        unregistered.append(stage)
        continue
    registered.append(stage)
    if not covered(stage):
        uncovered.append((stage, found))
stale = []
for number, path in entries:
    if path.startswith(FIXTURES + "/"):
        stage = path[len(FIXTURES) + 1:].split("/")[0]
        if stage not in stages:
            stale.append((number, path, stage))

if uncovered or stale:
    if uncovered:
        print(f"  ✗ {len(uncovered)} 道阶段的样本里有编号登记位，却不在 {EXCLUDE} 里（扫了 {len(stages)} 道，{len(registered)} 道有登记位）：")  # gate-lint:summary
        for stage, (path, number, kind) in uncovered:
            print(f"     {FIXTURES}/{stage}/（第一份：{path}:{number}，{kind}）")  # gate-lint:detail
        print(f"     → 怎么办：在 {EXCLUDE} 加一行 {FIXTURES}/<阶段>/  # 理由（理由写这一道样本造了带登记标题的页、与真表同号不同名，扫它只会报登记冲突）；")
        print("               不想排除，就让样本里的登记位在 setup.sh 里现写（静态样本文件里不放登记标题和登记表标记），或改成不带登记位的写法。")
    if stale:
        print(f"  ✗ {len(stale)} 行排除表指到 {FIXTURES}/ 下不存在的阶段（fixtures 下现有 {len(stages)} 道）：")  # gate-lint:summary
        for number, path, stage in stale:
            print(f"     {EXCLUDE}:{number}  {path}（没有阶段目录 {stage}）")  # gate-lint:detail
        print("     → 怎么办：阶段改了名就把这一行的路径改成现在的名字，样本目录删了就删掉这一行；留着的排除项会让人以为还有样本在被绕开。")
    sys.exit(1)
print(f"  ✓ 扫了 {len(stages)} 道阶段的样本，{len(registered)} 道有登记位、都在排除表里（{'、'.join(registered) if registered else '无'}）；"
      f"另 {len(unregistered)} 道没有登记位，在不在表里都不判；排除表里指到 fixtures 下的 {sum(1 for _n, p in entries if p.startswith(FIXTURES + '/'))} 行都指得到阶段")
PY
}

# ── 格 kb-registry：kb 目录里的每一份都要在 CLAUDE.md 的「项目本地事实」表里有一行
#
# 判据，任一条不成立判红：
#   ① .claude/kb/ 根目录下每一份 .md，都要在 CLAUDE.md「## 项目本地事实」表里被按路径点名；
#   ② .claude/kb/ 下每一个子目录也要被点名（带斜杠的路径，例如 .claude/kb/decisions/）；
#   ③ 那一节里点到的每一个 .claude/kb/… 路径都要真的存在，带 <…> 占位的也算一个路径。
# 「登记」只认表里一行的第一格写的路径、逐字相等：表外散文里提到的不算，一行 .claude/kb/ 也不替它下面的每一份登记。
# 判不了那一行写得对不对，要人看。
# 样本：kb-registry-red 放一份没登记的 kb 文件与一行指向空处的登记，必须判红；它的表里另有一行只写 .claude/kb/、散文里另提一份文件，
#   这两处都不许替那两份没登记的文件登记；kb-registry-green 两边对齐。
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
# 判据与起法都在 scripts/gate/gate-structure-check.py（规则：.claude/rules/verification.md「门禁的结构」），这一格只调它、按它的退出码记：
#   0 绿；77（门禁目录里一道都没有）本次未跑；别的（1 结构不对、2 参数错、78 它自己的准入与运行条件没满足）都按红记。
# 脚本取这一道所在的仓里那一份（取自 $0 的目录，不取项目根），判的是被判的根：样本仓里放坏的脚本碰不到它。判不了的见它文件头「判不了的」。
# 样本：gate-structure-red 的 gate.d 里一道没有格名表、一道 --check 写错格名照跑退 0、一道 --list 打的与格名表不一致，必须逐道点名判红；
#   gate-structure-green 两道都照规矩（格名表、--list 与表一致、--check 写错退 2），判绿并报查了 2 道。
cell_gate_structure() {
structure_check="$STAGE_REPOSITORY/scripts/gate/gate-structure-check.py"
[[ -f "$structure_check" ]] || { echo "  ✗ 找不到 $structure_check"; echo "     → 怎么办：它随仓走（scripts/gate/gate-structure-check.py），被删了就从 git 找回来；别在这一格另写一份判法。"; exit 1; }
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
# 判据：环境检查那一道（ENVIRONMENT_STAGE）要排在门禁目录里每一道跑测试的阶段后面；次序用共享 gate.sh 起本地阶段的同一条 find … | sort 现算。
#   「跑测试」：经 code_of 去掉注释与出路文字之后，代码里匹配 TEST_RUNNING（cargo test / build / run / nextest / bench、几份起测试的脚本、qemu-system、herd7）。
# 为什么：环境检查判的是它开跑那一刻的机器（残留 loop / dm 设备、挂载、qemu 进程、宿主盘），排在它后面的阶段编的、建的、留下的，这一趟看不见；
#   次序只由文件名定，改一个名就能悄悄挪到它前面。
# 判不了：用别的写法起测试的阶段（命令拼在变量里、经没列的脚本间接起），要把那个脚本名加进 TEST_RUNNING；排序跟跑这一格时的 locale。
# 样本：stage-order-red 的 gate.d 里 harness-tests.sh 起 cargo test、排在 harness-test-environment.sh 后面，必须点名它判红；同一份里排在后面、
#   只在注释与 echo 里提到 cargo test 的 zz-docs.sh 不算，不许点名。stage-order-green 把跑测试的那一道改名成 harness-model-scenarios.sh，
#   排到环境检查前面，判绿并报 2 道跑测试的阶段。
ENVIRONMENT_STAGE="harness-test-environment.sh"
# ── 格 directory-layout：research/ 与 scripts/ 的目录形态（.claude/rules/directory-layout.md）
# 判法只在 scripts/gate/directory-layout-check.py（用这一道所在仓里那一份），被判的根照传；它退 77（两个目录都不在）这一格就退 77。
# 样本：directory-layout-green 里 research/ 两个子目录与说明文件、scripts/ 一个子目录带说明文件；directory-layout-red 在同样的架子上多一份 scripts/kb/run.log。
cell_directory_layout() {
python3 "$STAGE_REPOSITORY/scripts/gate/directory-layout-check.py" "$ROOT"
}

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
                          r'|\b(?:replay|mutate|mutation-shard-run|crash-point-replay-shard-run|multi-host-run|vm-bench|lkmm)\.sh\b'
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

# ── 格 stage-location-before-cd：阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname "$0" 找旁边的文件
#
# 判据：.claude/gate.d/ 顶层每个 *.sh（lib/ 不判）里，找第一行 `cd "$ROOT"` 或 `cd "${1`；它之后的非注释行（出路文字 echo / printf / print( 之类不算）里
#   出现 `dirname "$0"` 或 `dirname "${BASH_SOURCE` 判红，点名文件:行。没有那一行 cd 的阶段不判，成功句报出有几道。
# 为什么：用相对路径调阶段、又另给一个项目根时，cd 之后 $0 的相对目录解析不到原来的位置，阶段找不到自己旁边的共用脚本，退 1 说「找不到」。
#   整轮门禁用绝对路径调阶段，碰不到，只在手工带相对路径起一道时红。
# 判不了：用别的写法取自己的位置（readlink -f "$0"、$(realpath …)）；cd 写成别的变量名（cd "$root"）；heredoc 里嵌的 shell 文本（按普通行判）。
# 弄坏开关 CODE_TOOLING_BREAK=dirname-after-cd-ignored 不判 cd 之后的 dirname：stage-location-before-cd-red 判绿。
# 样本：stage-location-before-cd-red 的 gate.d 里 broken-stage.sh 在 cd 之后用 dirname "$0" 找 stage-run-or-skip.sh，必须点名它的文件:行；
#   同一份里 careful-stage.sh 的 dirname 在 cd 之前、注释里与 echo 出路文字里提到 dirname "$0" 的不许点名，没有 cd 的 plain-stage.sh 不判。
#   stage-location-before-cd-green 把 broken-stage.sh 改成 cd 之前取位置存进变量，判绿并报扫了 2 道、判红 0 处、没有 cd 的 1 道不判。
cell_stage_location_before_cd() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
stage_files="$(find "$ROOT/.claude/gate.d" -maxdepth 1 -name '*.sh' \( -type f -o -type l \) | sort)"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$stage_files" "${CODE_TOOLING_BREAK:-}" <<'PY'
import os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
broken = sys.argv[3]
paths = [path for path in sys.argv[2].split("\n") if path.strip()]
if not paths:
    print("  ! .claude/gate.d/ 顶层一道阶段都没有，这一格无对象可判")
    sys.exit(77)
CD_TO_ROOT = re.compile(r'^\s*cd\s+"(?:\$ROOT|\$\{1)')
FINDS_OWN_LOCATION = re.compile(r'dirname\s+"(?:\$0"|\$\{BASH_SOURCE)')
scanned, without_cd, hits = 0, [], []
for path in paths:
    name = os.path.basename(path)
    with open(path, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().splitlines()
    cd_line = next((number for number, line in enumerate(lines, 1) if CD_TO_ROOT.match(line) and not line.lstrip().startswith("#")), None)
    if cd_line is None:
        without_cd.append(name)
        continue
    scanned += 1
    if broken == "dirname-after-cd-ignored":
        continue
    for number in range(cd_line + 1, len(lines) + 1):
        code = code_text.code_of(lines[number - 1])
        if FINDS_OWN_LOCATION.search(code):
            hits.append((name, number, cd_line, lines[number - 1].strip()))
if hits:
    print(f"  ✗ {len(hits)} 处在 cd 到项目根之后还用 dirname \"$0\" / dirname \"${{BASH_SOURCE\" 找自己的位置（扫了 {scanned} 道有 cd 的阶段，另 {len(without_cd)} 道没有那一行 cd、不判）：")  # gate-lint:summary
    for name, number, cd_line, text in hits:
        print(f"     .claude/gate.d/{name}:{number}（cd 在第 {cd_line} 行）：{text[:140]}")  # gate-lint:detail
    print("     → 怎么办：在 cd 之前取阶段自己的位置存进变量（STAGE_REPOSITORY=\"$(cd \"$(dirname \"$0\")/../..\" && pwd)\" 这一类），cd 之后用那个变量；")
    print("               用相对路径调阶段、又另给一个项目根时，cd 之后 $0 的目录就解析不到了。")
    sys.exit(1)
print(f"  ✓ .claude/gate.d/ 顶层 {len(paths)} 道阶段里扫了 {scanned} 道有 cd 到项目根的，cd 之后都没再用 dirname 找自己的位置（判红 0 处）；没有那一行 cd 的另外 {len(without_cd)} 道不判")
PY
}

# ── 格 heavy-thread-defaults：scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里自己定并行度默认值的写法，都经每台默认重活线程数的登记位取
#
# 登记位：.claude/heavy-thread-default（写法在它的文件头），判它在不在、写没写对经 scripts/hosts/heavy_threads.py 的 default_heavy_threads
#   （用这一道所在仓里那一份，登记文件取被判的根里那一份，不认调用方的 SINGLEFS_THREAD_CAP）。
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里的 *.sh、*.py、*.rs，外加 .claude/cargo-command-prefix；
#   跳过 target/、__pycache__/ 与 .claude/gate.d/fixtures/（判别力样本是造出来的输入）。
# 只看代码：shell 去掉整行注释、行尾不在引号里的 # 注释与出路文字（echo / printf / print( 的字面量，CODE_WITHOUT_TEXT_LIBRARY 的 OUTPUT_TEXT）；
#   python 经 tokenize 去掉注释、文档串与字符串字面量，只留内容整个是 nproc、以 capped.sh 结尾、纯数字、--threads 一类选项名或线程变量名的字面量；
#   rust 去掉 // 之后的部分与字符串字面量（留下的同 python）。
# 算「自己定并行度默认值」的写法（每处记一个或几个种类，判法在 THREAD_DEFAULT_FORMS 与下面几行）：
#   nproc、cpu_count(、sched_getaffinity、available_parallelism、available_threads（此刻空闲线程，按忙闲现算）、min(16 / .min(16)；
#   核数的一半：/ 2 或 // 2，同一行里有核数、线程、并行一类的词；capped.sh <数字>：capped.sh 不在引号里、或它所在的那个字面量就在 capped.sh 收尾
#   （python 的列表写法 "capped.sh", "16" 也算），引号里整句拿来当样本的不算；
#   写死的线程数默认：${<线程变量>:-<数>}、--threads / --jobs / --workers / --lanes 的 default=<数>、名字里同时有 DEFAULT / WHEN_UNSET / BY_PROCESSOR
#   与 THREADS / JOBS / WORKERS 的常量赋了数、"threads" / "jobs" / "workers" 键赋了数，以及 unwrap_or(<正整数>) / Err(_) => <正整数> /
#   or <正整数>：同一行或往上 5 行代码里点了线程变量名（大写、含 THREAD / JOBS / WORKERS 的那种字面量）。
#   回退值只认线程 / 工作进程 / 并行度的上下文：线程变量名在往上 5 行里、而回退值所在那一行本身写着 timeout / deadline / seconds / duration / 超时 一类的词
#   （原行里找，字符串和键名里的也算，注释不算）的，那个数是超时秒数一类的别的量，不判（例：`"timeout": int(os.environ.get("X_TIMEOUT_SECONDS") or 600)`）；
#   线程变量名就在回退值同一行的照判。
# 排除表 .claude/heavy-thread-exempt：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中（那一处已经改掉了，留着就是白放行）都判红；只缩不涨：与改动范围的基准
#   （scripts/gate/changed-paths.sh 的 gate_diff_base gate）里那一版比，基准里没有的「路径:锚串」判红；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明。
# 装置读自己的线程变量（G3）：python / rust 代码里点了名的线程变量字面量（含 THREAD / JOBS / WORKERS 的大写名字）出现就算读，不看它在不在读环境变量的写法里
#   （经包装函数读、`ENV = os.environ` 的别名与拷贝、`std::env::vars()`、`use std::env::var as …` 这些写法里名字只是个字面量，认不出读的是哪一个）；只有 ["X"] = 值 这种写给子进程的不算读。
#   `if "X_THREADS" in 某个表` 这种字符串比较、没人用的常量也因此被点名：确实不是读环境变量的，登记进排除表 .claude/heavy-thread-exempt（锚串取那一行里的一段原文，理由写明）。
#   shell 里 $变量 / ${变量}（文件自己赋了值的局部变量不算，行首或 ; && || 之后同一行接着赋的也认；有一处 X="${X:-4}" 这种右边读着自己的，仍算读）；
#   shell 里 `|| X=<数>`、`&& X=<数>`（X 是线程变量名；数是正整数，或 `$变量`、`$((算术))`，即默认值先放进变量、先算好再赋；`[[ -z "$X" ]] && X=8` 这类没设就给默认值的写法）
#   认成「写死的线程数默认（|| / && 赋值）」，点名那一行；`$(命令)` 与 `$SINGLEFS_THREAD_CAP`（取自上限）不算钉数；
#   `$变量` 那个变量算「登记位取的」、不算钉数，要三样同时满足（registry_sourced_variables）：
#     ① 同一份文件里它的每一处「名=值」，右边整条是登记位读法："$(heavy_thread_default)"（可带 2>/dev/null）、"$(default_heavy_threads)"、"${SINGLEFS_THREAD_CAP}"
#        （引号、花括号可省；右边夹了别的东西的，如除以一个变量、${SINGLEFS_THREAD_CAP:-$另一个变量}、$(( … ))，都不算）；
#     ② 同一份文件里它没出现在 ${名:-…}、${名:=…}（不带冒号的同算）里，也没有别的赋值写法：(( 名 = … ))（含 +=、++ 这类）、let、declare -i、read、printf -v、
#        for 名 in、名+=、eval 后面同一行点了它的名字（other_assignment_forms）；
#     ③ 文件没有 source（或 .）scripts/hosts/heavy-threads.sh 之外的别的文件。
#   有一处不满足就照算钉数。判不了的：变量在别的文件里怎么赋（③ 一概不算）、mapfile / readarray / getopts 这类读进变量的写法、eval 里拼出来的名字。
#   shell 里 `printenv NAME`（NAME 是线程变量名）从环境里读它，认成读，文件里赋没赋过值都一样。
#   rust 行首 #[cfg(test)] 之后第一个非空行（夹着的 #[…] 属性不算）是 mod 的，那个测试模块往后不算，只罩一条 use、fn、const 的不跳过后面；
#   `#[cfg(test)] mod tests;`（外置模块，分号结尾）只是声明，它后面的代码照扫。
#   这个名字不在 scripts/hosts/capped.sh 的 variable_names 清单里（被判的根里那一份，读不出判红），
#   读它的任何一处所在的那一条语句里也没出现 SINGLEFS_THREAD_CAP、heavy_thread_default、default_heavy_threads、heavy-threads.sh、heavy_threads、capped.sh 之一（注释与文档串里写的不算；
#   同文件别的语句里提到不算回退），capped.sh <上限> 到不了它、没设时也不回退到上限或登记值：点名的是读它的那一行（每个变量第一次被读的那一行，同一行的几个变量写在一起；
#   那一行本来就是别的种类的被判行，就并到那一行上，不是的单列一条）。排除表按被判行匹配，不按文件并组：锚串要落在读它的那一行原文里才放那一行，
#   同文件别的被判行上的锚串（核数、available_parallelism 那几行）放行不了它。
#   一条语句的算法在 statement_range：python 取 tokenize 的逻辑行，shell 取反斜杠续行，rust 往前到上一条语句收尾（; { }）之后、往后到括号配平且以 ; 或 } 收尾（或闭合外层的块）；
#   回退写在另一条语句里（先读进变量、下一句再回退）认不出，要么写进同一条表达式，要么把变量名加进 capped.sh 的 variable_names。
#   crates/ 里的（crash_amplification.rs、两份 GPU 对拍测试）与 scripts/mutation/ 的（与依赖产物共用）这几份落在原有的「代码批」「与依赖产物共用」行上，不另加行；
#   E161 装置只读 E161_THREADS，capped.sh 清单里有它，这一格不点名它，它没设时取 available_parallelism 的那一行照旧由排除表「代码批」行放行。
# 判不了的：变量里拼出来的命令、读别的文件定的数、跨 5 行以上的回退值、shell 里嵌的 python 字符串（按 shell 判：引号里整句的样本不算，其余照判）、
#   多行的 rust 字符串字面量、跨语句的回退（见上）、shell 的 if / case 多行复合语句（按行判，回退写在另一行里认不出）。排除表的理由成不成立靠 review。
#   线程数取对了之后用在几处、装置的主循环对取样点串不串行，这一格也判不了（只用在一处的装置也可能就是对的：它唯一的重活就在那一处；
#   串行的循环里调的函数自己可能在并行）：由运行时的在用核数告警兜（scripts/hosts/core_usage.py，multi-host-run.sh --peer、detached-run.sh watch、agent-watch.py 三处判）。
# 样本：heavy-thread-defaults-red 的登记写对，scripts/hosts/half.sh 用 $(nproc) / 2、scripts/gate/half.py 用 cpu_count 与 min(16 与 processor_count // 2、
#   .claude/hooks/start.py 用列表写法起 capped.sh 16、.claude/gate.d/stage.sh 用 capped.sh 16、crates/demo/src/lib.rs 用 available_parallelism 与
#   隔了几行的 unwrap_or(8)，逐处点名判红（只在注释、echo 与 print 里提到的不点名）；排除表一行指向不存在的文件、一行锚串不命中、
#   setup.sh 建 git 仓提交一版只有那两行的表、再加一行（表涨了），三样都点名。heavy-thread-defaults-green 的登记写对，脚本经 heavy_thread_default
#   与 default_heavy_threads 取、只在注释、文档串、echo、引号里整句的样本里提到 nproc 与 capped.sh 16，一处真用 $(nproc) 的登记在排除表里，
#   .claude/cargo-command-prefix 写 capped.sh default，判绿并报扫了几份、排除命中几处；scripts/hosts/uses-registry.py 里一行 int(… or 600) 的超时回退值紧跟在
#   SINGLEFS_THREAD_CAP 那一行后面，不点名。heavy-thread-defaults-red 里 scripts/gate/half.py 还有线程变量隔一行才给回退值的 int(cap or 8)，照判（超时那条豁免不吞它）。
#   装置读自己的线程变量那一路，两份样本根里各放一份假的 scripts/hosts/capped.sh（只有 variable_names 清单）：red 的清单没有装置的变量，scripts/experiments/own_device.py（环境里读、
#   没设就 KeyError）、scripts/hosts/own_device.sh（${…:?}）、crates/demo/src/device.rs（env::var 加 expect）三份没有别的被判行，落在点名变量的那一行；scripts/experiments/own_with_cores.py 与
#   crates/demo/src/lib.rs 已有核数、回退值的被判行，同一行再加这一种；own_device.sh 里 LOCAL_WORKERS 是文件自己赋的局部变量，不点名。green 的清单有 E161_THREADS：
#   crates/demo/src/device.rs 读它，不点名，测试模块里的 SAMPLE_TEST_THREADS 不算；scripts/experiments/own_registry.py 读 capped.sh 不设的 SAMPLE_OWN_THREADS 但没设时回退到 default_heavy_threads，不点名；
#   scripts/experiments/own_multiline.py 与 crates/demo/src/own_match.rs 把读和回退写在同一条跨行的语句里（python 的括号续行、rust 的 match 加 Err 分支），不点名；
#   scripts/experiments/own_child_env.py 只往子进程环境里写 ["SAMPLE_CHILD_THREADS"] = 值，scripts/hosts/local_workers.sh 的 WORKERS 与 SAMPLE_INLINE_WORKERS（同一行 ; 之后赋的）是局部变量，都不算读；
#   scripts/experiments/own_in_comparison.py 里 `if "SAMPLE_COMPARED_THREADS" in registry_table:` 是字符串比较，不是读环境变量，但引号里的名字出现就算读，被判行靠排除表放行
#   （W01：scripts/agents/dispatch-prompt.py 的 `in` 比较）；scripts/hosts/env_prefix.sh 里 `true && SAMPLE_PREFIX_THREADS=8 make -j 2` 的 =8 后面跟着命令，是环境前缀，不点名。
#   heavy-thread-defaults-quoted-name-red：线程变量名出现在引号里就点名，不管读环境变量的写法认不认得出：scripts/experiments/wrap_helper.py（包装函数读）、alias_env.py（`ENV = os.environ` 别名）、
#   copied_env.py（os.environ.copy() 拷贝）、in_comparison.py（`in` 比较与没人用的常量）、crates/demo/src/project_wrapper.rs、harness_wrapper.rs（包装函数读）、use_alias.rs（`use std::env::var as …`）、
#   vars_iter.rs（`std::env::vars()`）、lib.rs（#[cfg(test)] mod tests; 之后的读）、tests.rs，以及 shell 的 if_default.sh、or_default.sh（`|| X=8`）、and_default.sh（`&& X=8`），共 15 处（in_comparison.py 的比较和没人用的常量在两行上，各一处），baseline_read.py 是对照。
#   heavy-thread-defaults-per-line-red：排除表按被判行匹配，两张行只放行各自锚串落在的那一行。crates/demo/src/absorbed.rs 第 1 行 available_parallelism 与 scripts/experiments/absorbed.py 第 2 行 os.cpu_count()
#   由排除表放行，同文件新读线程变量的那一行（absorbed.rs:3、absorbed.py:3）借不到它们，各被点名；scripts/hosts/or_var_default.sh（`|| X=$fallback`）、or_arith_default.sh（`|| X=$((4 * 2))`）认成钉数，
#   printenv_read.sh 的 `printenv SAMPLE_PRINTENV_THREADS` 认成读，共 5 处。
#   heavy-thread-defaults-per-line-green：同一批文件，每个新读的那一行都有自己的锚串，判绿并报表 7 行每行都命中；or_registry.sh 里 `|| X=$(heavy_thread_default)`、`|| X=$SINGLEFS_THREAD_CAP` 取自登记位与上限，不点名。
#   heavy-thread-defaults-var-source-green：`|| X=$变量` 的变量在同一份文件里由登记位读法赋值：var_from_registry.sh（default_threads="$(heavy_thread_default)"）、
#   var_after_if.sh（if ! threads="$(heavy_thread_default)"; then …，`|| X=${threads}`）、var_from_cap.sh（cap_jobs="${SINGLEFS_THREAD_CAP}"）、
#   var_stderr_silenced.sh（default_threads="$(heavy_thread_default 2>/dev/null)"，照 run-with-memory-cap.sh 丢掉 stderr），四份都不点名，判绿。
#   heavy-thread-defaults-var-source-forms-red：变量有一处「名=值」从登记位取，但不满足 registry_sourced_variables 三样之一的十二份，`|| X=$default_threads` 那一行各点名一处（共 12 处）：
#   右边不整条是登记位读法的 cap_fallback_via_variable.sh（${SINGLEFS_THREAD_CAP:-$fallback_jobs}）、registry_divided_by_variable.sh（$(( $(heavy_thread_default) / divisor ))）；
#   ${名:=8} 补默认的 colon_equals_fallback.sh；source 了另一份给字面数的 sample_defaults.sh 的 sourced_override.sh；
#   别的赋值写法八份：arith_assignment.sh（(( 名 = 8 ))）、let_assignment.sh（let "名 = 8"）、declare_integer.sh（declare -i 名）、read_assignment.sh（read -r 名 <<< 8）、
#   printf_v_assignment.sh（printf -v 名 %s 8）、for_assignment.sh（for 名 in 8）、plus_equals_assignment.sh（名+=0）、eval_assignment.sh（eval "名=8"）。
#   heavy-thread-defaults-var-source-red：var_from_literal.sh（变量赋了 8）、var_mixed.sh（一处从登记位取、另一处 `&& 变量=8`）、
#   arith_default.sh（`|| X=$((default_threads * 2))`，算术照算钉数，变量从登记位取也一样）、literal_default.sh（`|| X=8`），共 4 处点名。
#   heavy-thread-defaults-wide-cost-red：宽判法的代价，八份不读环境变量的写法仍点名（字符串比较、名字元组、字典实参、没人用的常量、argparse 的 metavar、rust 的比较与 match、`&&` 夹下限），共 8 处；
#   fp_docstring.py（文档串提到）与 fp_prefix.sh（`&& X=8 make` 的环境前缀）是对照，不点名；真树上这类逐行判，不读环境的进排除表并写理由。
#   heavy-thread-defaults-cfg-test-red：crates/demo/src/device_after_cfg_test.rs 第 1 行 #[cfg(test)] 只罩一条 use，第 5 行读 SAMPLE_WORKER_THREADS 没设就 panic，必须点名（共 1 处）；
#   文件末尾另有 #[cfg(test)]、#[allow(dead_code)]、mod tests 的测试模块，里面点名的 SAMPLE_TEST_ONLY_THREADS 不算。
#   heavy-thread-defaults-capped-elsewhere-red：scripts/experiments/mentions_capped.py 读 SAMPLE_OWN_JOBS 没设就 KeyError，同文件另一条语句里有字符串 "scripts/hosts/capped.sh"；
#   scripts/hosts/mentions_capped.sh 读 SAMPLE_SHELL_WORKERS（${…:?}），下一行 bash scripts/hosts/capped.sh default；两份都必须点名（共 2 处）。
# 弄坏开关（CODE_TOOLING_BREAK）：thread-default-strings-scanned（python 与 rust 的字面量与文档串照代码扫）：heavy-thread-defaults-green 判红；
#   thread-default-half-unchecked（不认核数的一半）：heavy-thread-defaults-red 里 scripts/gate/half.py 第 4 行那条 want 找不到；
#   thread-default-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：heavy-thread-defaults-red 里「指向不存在的文件」「锚串一处都不命中」两条 want 找不到；
#   thread-default-exempt-growth-unchecked（不比基准）：heavy-thread-defaults-red 里「排除表涨了」那条 want 找不到；
#   thread-default-timeout-unchecked（超时回退值也按线程数判）：heavy-thread-defaults-green 判红（uses-registry.py 的超时那一行被点名）；
#   thread-default-own-variable-unchecked（不判装置读自己的线程变量）：heavy-thread-defaults-red 里「13 处」那一条与五条「读自己的线程变量」want 找不到；
#   thread-default-capped-list-ignored（capped.sh 的清单读了不认，每个线程变量都算清单外）：heavy-thread-defaults-green 判红（crates/demo/src/device.rs 里 E161_THREADS 被点名）；
#   thread-default-shell-local-unchecked（shell 里文件自己赋了值的变量也算读环境）：heavy-thread-defaults-green 判红（local_workers.sh 的 WORKERS 被点名），heavy-thread-defaults-red 里 scripts/hosts/own_device.sh 那条 want 找不到（点名的变量名多出 LOCAL_WORKERS），heavy-thread-defaults-var-source-green 判红（`|| X=` 赋的 SAMPLE_*_JOBS 被当成读）；
#   thread-default-test-module-unskipped（rust 测试模块里点名的变量也算）：heavy-thread-defaults-green 判红（device.rs 测试模块里 #[test] 函数用 std::env::var("SAMPLE_TEST_THREADS") 读的那个变量被点名），
#     heavy-thread-defaults-cfg-test-red 里 device_after_cfg_test.rs:5 那条 want 找不到（测试模块里 std::env::var("SAMPLE_TEST_ONLY_THREADS") 读的变量也被点名，变量名列表成了 SAMPLE_TEST_ONLY_THREADS、SAMPLE_WORKER_THREADS；同一文件的被判行并成一处，「✗ 1 处」那条 want 仍找得到）；
#     测试模块里只把变量名赋给常量、同文件别处没用到的，不算读，样本要用读环境变量的写法；
#   thread-default-cfg-test-without-mod（行首 #[cfg(test)] 不看后面是不是 mod，一律往后都不算）：heavy-thread-defaults-cfg-test-red 退 0，退出码不对；
#   thread-default-fallback-by-file（整份文件的代码里出现回退字样就算回退了）：heavy-thread-defaults-capped-elsewhere-red 退 0，退出码不对；
#   thread-default-statement-single-line（读它的那条语句只算那一行）：heavy-thread-defaults-green 判红（own_multiline.py、own_match.rs 被点名）；
#   thread-default-subscript-write-counted（["X"] = 值 也算读）：heavy-thread-defaults-green 判红（own_child_env.py 被点名）；
#   thread-default-shell-inline-assignment-unseen（shell 里只认行首的赋值）：heavy-thread-defaults-green 判红（local_workers.sh 的 SAMPLE_INLINE_WORKERS 被点名），heavy-thread-defaults-var-source-green 判红（`|| X=` 赋的 SAMPLE_*_JOBS 被当成读）；
#   thread-default-quoted-name-needs-environment-read（引号里的名字又回到只在读环境变量的写法里才算读，ENVIRONMENT_READ）：heavy-thread-defaults-quoted-name-red 里 wrap_helper.py、alias_env.py、copied_env.py、
#     in_comparison.py、project_wrapper.rs、harness_wrapper.rs、use_alias.rs、vars_iter.rs 那八条 want 与「15 处」找不到；heavy-thread-defaults-green 判红（own_in_comparison.py 排除表那一行一处被判行都不命中）；
#   thread-default-conditional-assignment-unpinned（不认 `|| X=<数>`、`&& X=<数>`）：heavy-thread-defaults-quoted-name-red 里 or_default.sh、and_default.sh 那两条 want 与「15 处」找不到，
#     heavy-thread-defaults-per-line-red 里 or_var_default.sh、or_arith_default.sh 那两条 want 与「5 处」找不到，heavy-thread-defaults-wide-cost-red 里 fp_clamp.sh 那条 want 与「8 处」找不到，
#     heavy-thread-defaults-var-source-red、heavy-thread-defaults-var-source-forms-red 退 0（期望退出 1）；
#   thread-default-external-mod-skipped（`#[cfg(test)] mod tests;` 也当测试模块，往后都不算）：heavy-thread-defaults-quoted-name-red 里 lib.rs:8 那条 want 与「15 处」找不到；
#   thread-default-exempt-by-file（排除表退回按文件并组：新读线程变量这一种挂到文件里已有的每一条被判行上，读它的那一行自己不被点名）：heavy-thread-defaults-per-line-red 里 absorbed.rs:3、absorbed.py:3 那两条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（锚串落在读它那一行上的排除表行，被判行里一处都不命中），heavy-thread-defaults-red 里 lib.rs:7 那条 want 与「13 处」找不到，
#     heavy-thread-defaults-quoted-name-red 里 in_comparison.py 两条 want 与「15 处」找不到；
#   thread-default-conditional-assignment-literal-only（`|| X=<数>` 只认字面正整数，不认 `$变量`、`$((算术))`）：heavy-thread-defaults-per-line-red 里 or_var_default.sh、or_arith_default.sh 那两条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（or_var_default.sh、or_arith_default.sh 两行的排除表行一处被判行都不命中），
#     heavy-thread-defaults-var-source-red 里 var_from_literal.sh、var_mixed.sh、arith_default.sh 那三条 want 与「4 处」找不到，heavy-thread-defaults-var-source-forms-red 退 0（期望退出 1）；
#   thread-default-printenv-unseen（不认 shell 里的 `printenv NAME`）：heavy-thread-defaults-per-line-red 里 printenv_read.sh 那条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（printenv_read.sh 的排除表行一处被判行都不命中）；
#   thread-default-conditional-variable-source-ignored（`|| X=$变量` 不看变量在文件里怎么赋的值，一律算钉数）：heavy-thread-defaults-var-source-green 判红（四份都被点名）；
#   thread-default-registry-substring-accepted（registry_sourced_variables 的 ① 退回改前：右边含登记位读法、去掉之后没有别的数字就算）：heavy-thread-defaults-var-source-forms-red 里
#     cap_fallback_via_variable.sh、registry_divided_by_variable.sh 那两条 want 与「12 处」找不到，heavy-thread-defaults-var-source-green 判红（var_stderr_silenced.sh 的 2>/dev/null 被当成别的数字）；
#   thread-default-defaulted-variable-accepted（不看 ${名:-…}、${名:=…}）：heavy-thread-defaults-var-source-forms-red 里 colon_equals_fallback.sh 那条 want 与「12 处」找不到；
#   thread-default-other-assignment-forms-unseen（不看别的赋值写法，只认「名=值」）：heavy-thread-defaults-var-source-forms-red 里 arith_assignment.sh、let_assignment.sh、declare_integer.sh、
#     read_assignment.sh、printf_v_assignment.sh、for_assignment.sh、plus_equals_assignment.sh、eval_assignment.sh 那八条 want 与「12 处」找不到；
#   thread-default-sourcing-ignored（不看文件 source 了别的文件）：heavy-thread-defaults-var-source-forms-red 里 sourced_override.sh 那条 want 与「12 处」找不到。
cell_heavy_thread_defaults() {
heavy_threads_library_file="$STAGE_REPOSITORY/scripts/hosts/heavy_threads.py"
[[ -f "$heavy_threads_library_file" ]] || { echo "  ✗ 找不到 $heavy_threads_library_file"; echo "     → 怎么办：它随仓走（scripts/hosts/heavy_threads.py），被删了就从 git 找回来；登记位的判法只在那一份里写。"; exit 1; }
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
base_state="none"
base_name=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1 && git rev-parse --verify -q HEAD >/dev/null 2>&1; then
  # shellcheck source=../../scripts/gate/changed-paths.sh
  if ! source "$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"; then
    echo "  ✗ source 不了 $STAGE_REPOSITORY/scripts/gate/changed-paths.sh，取不到改动范围的基准，排除表只缩不涨没法比"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  base_name="$(gate_diff_base gate)"
  if ! listed="$(git -c core.quotepath=false ls-tree --name-only "$base_name" -- .claude/heavy-thread-exempt 2>&1)"; then
    base_state="unreadable:$listed"
  elif [[ -n "$listed" ]]; then
    if git show "$base_name:.claude/heavy-thread-exempt" > "$base_table" 2>/dev/null; then base_state="table"; else base_state="unreadable:git show 退了非 0"; fi
  fi
fi
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$heavy_threads_library_file" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" <<'PY'
import collections, io, os, re, sys, tokenize, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
library_file, broken, base_state, base_name, base_table = sys.argv[2:7]
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(library_file))
try:
    import heavy_threads
except (ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {library_file}：{error}")
    print("     → 怎么办：照它的报错修好（python3 -m py_compile 先过），登记位的判法只在那一份里写。")
    sys.exit(1)

REGISTRATION = ".claude/heavy-thread-default"
EXEMPT_TABLE = ".claude/heavy-thread-exempt"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", "crates")
SCAN_FILES = (".claude/cargo-command-prefix",)
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/",)
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"
THREAD_DEFAULT_KEPT_LITERAL = re.compile(r"nproc|.*capped\.sh|[0-9]+|--(?:threads|jobs|workers|lanes)|threads|jobs|workers|" + THREAD_VARIABLE)
THREAD_DEFAULT_PREFILTER = re.compile(r"nproc|cpu_count|sched_getaffinity|available_parallelism|available_threads|min\s*\(\s*16|\.min\(|capped\.sh|THREAD|JOBS|WORKERS"
                       r"|threads|jobs|workers|lanes|/\s*2")


def thread_default_form(name, pattern):
    return name, re.compile(pattern)


CONDITIONAL_ASSIGNMENT_FORM = "写死的线程数默认（|| / && 赋值）"      # shell 里没设就给默认值的 `[[ -z "$X" ]] && X=8`、`… || X=8`
CONDITIONAL_ASSIGNMENT_LITERAL = r"[1-9][0-9]*"
CONDITIONAL_ASSIGNMENT_VALUE = (CONDITIONAL_ASSIGNMENT_LITERAL if broken == "thread-default-conditional-assignment-literal-only"
                                else CONDITIONAL_ASSIGNMENT_LITERAL + r"|\$\{?(?!SINGLEFS_THREAD_CAP\b)[A-Za-z_][A-Za-z0-9_]*\}?|\$\(\([^\n]*?\)\)")      # 钉数：字面正整数，或 $变量、$((算术))（默认值先放进变量、先算好再赋）；$(命令) 与 $SINGLEFS_THREAD_CAP 不算
CONDITIONAL_ASSIGNMENT_PATTERN = re.compile(r"(?:\|\||&&)[ \t]*\{?[ \t]*(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?" + THREAD_VARIABLE
                                            + r"=[\"']?(?P<value>" + CONDITIONAL_ASSIGNMENT_VALUE + r")[\"']?(?=[ \t]*(?:$|[;)}&|]))")      # 赋值到此为止（后面不跟命令）：`&& X=8 cmd` 是给 cmd 的环境前缀，不是 shell 里的默认值
CONDITIONAL_ASSIGNMENT_VARIABLE = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?")
# shell 里任何变量的赋值（行首、空白或 ; & | ( 之后）；右边取到下一个不在引号与 $( ) 里的 ; & | 或空白为止
SHELL_ANY_ASSIGNMENT = re.compile(r"(?:^|(?<=[\s;&|(!]))(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)="
                                  r"(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|\$\((?:[^()]|\([^()]*\))*\)|[^;&|\"'\s()])*)", re.M)
REGISTRY_READING = re.compile(r"\bheavy_thread_default\b|\bdefault_heavy_threads\b|\$\{?SINGLEFS_THREAD_CAP\b")   # 只有弄坏开关 thread-default-registry-substring-accepted 下还在用
REGISTRY_READING_OTHER_NUMBER = re.compile(r"(?<![\w.])[0-9]+(?![\w.])")   # 同上
# 右边整条是登记位读法："$(heavy_thread_default)"（可带 2>/dev/null）、"$(default_heavy_threads)"、"${SINGLEFS_THREAD_CAP}"，引号与花括号可省
REGISTRY_WHOLE_RIGHT_SIDE = re.compile(r'"?(?:\$\((?:heavy_thread_default(?:[ \t]+2>/dev/null)?|default_heavy_threads)\)|\$\{?SINGLEFS_THREAD_CAP\}?)"?')
VARIABLE_WITH_DEFAULT = re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*):?[-=]")   # ${变量:-…}、${变量:=…}（不带冒号的同算）：它可能先由别处给了值或补了默认
SHELL_COMMAND_START = r"(?:^|[;&|({!]|\b(?:then|do|else|if|while|until)\b)[ \t]*"
SHELL_SOURCE = re.compile(SHELL_COMMAND_START + r"(?:source|\.)[ \t]+(?P<target>[^=\s;&|\n][^;&|\n]*)", re.M)   # 被 source 的那一段取到命令收尾（; & | 或行尾）
REGISTRY_LIBRARY_SOURCE = re.compile(r"/heavy-threads\.sh[\"']?[ \t]*$|^[\"']?heavy-threads\.sh[\"']?[ \t]*$")   # source 登记位那一份（scripts/hosts/heavy-threads.sh）不算 source 了别的文件
ASSIGNMENT_OPERATOR = r"[ \t]*(?:[-+*/%^&|]|<<|>>)?=(?!=)"


def other_assignment_forms(name):
    """不写成「名=值」的赋值写法（同一份文件里出现一处，这个变量就不算登记位取的）：(( 名 = … )) 与 (( 名++ ))、let、declare -i、read、printf -v、for 名 in、名+=、eval。"""
    word = r"(?<![\w${])" + re.escape(name) + r"\b"
    return [
        ("(( 名 = … ))", re.compile(r"\(\([^\n]*?(?:" + word + ASSIGNMENT_OPERATOR + "|" + word + r"[ \t]*(?:\+\+|--)|(?:\+\+|--)[ \t]*" + word + ")")),
        ("let", re.compile(SHELL_COMMAND_START + r"let\b[^\n;&|]*?" + word + r"(?:" + ASSIGNMENT_OPERATOR + r"|[ \t]*(?:\+\+|--))", re.M)),
        ("declare -i", re.compile(SHELL_COMMAND_START + r"(?:declare|typeset|local)[ \t]+(?:-[A-Za-z]+[ \t]+)*?-[A-Za-z]*i[A-Za-z]*\b[^\n;&|]*?" + word, re.M)),
        ("read", re.compile(SHELL_COMMAND_START + r"read\b[^\n;&|<>]*?" + word, re.M)),
        ("printf -v", re.compile(r"\bprintf[ \t]+(?:-[A-Za-z]+[ \t]+)*?-v[ \t]*[\"']?" + re.escape(name) + r"\b")),
        ("for 名 in", re.compile(r"\b(?:for|select)[ \t]+" + re.escape(name) + r"(?:[ \t]+in\b|[ \t]*;|[ \t]*$)", re.M)),
        ("名+=", re.compile(word + r"\+=")),
        ("eval", re.compile(SHELL_COMMAND_START + r"eval\b[^\n]*?" + word, re.M)),
    ]


def registry_sourced_variables(shell_code):
    """shell 文件里只由登记位读法赋值的变量名，三样都满足才算：每一处「名=值」的右边整条是登记位读法（REGISTRY_WHOLE_RIGHT_SIDE）；
    同一份文件里这个名字没出现在 ${名:-…}、${名:=…} 里，也没有别的赋值写法（other_assignment_forms）；文件没有 source 登记位那一份之外的别的文件。
    `|| X=$变量` 的变量在这里面的不算钉数。"""
    if broken != "thread-default-sourcing-ignored" and any(not REGISTRY_LIBRARY_SOURCE.search(found.group("target")) for found in SHELL_SOURCE.finditer(shell_code)):
        return set()
    verdicts = {}
    for found in SHELL_ANY_ASSIGNMENT.finditer(shell_code):
        rhs = found.group("rhs")
        if broken == "thread-default-registry-substring-accepted":
            from_registry = REGISTRY_READING.search(rhs) is not None and not REGISTRY_READING_OTHER_NUMBER.search(REGISTRY_READING.sub("", rhs))
        else:
            from_registry = REGISTRY_WHOLE_RIGHT_SIDE.fullmatch(rhs) is not None
        verdicts[found.group("name")] = verdicts.get(found.group("name"), True) and from_registry
    defaulted = set() if broken == "thread-default-defaulted-variable-accepted" else set(VARIABLE_WITH_DEFAULT.findall(shell_code))
    return {name for name, from_registry in verdicts.items()
            if from_registry and name not in defaulted
            and (broken == "thread-default-other-assignment-forms-unseen" or not any(pattern.search(shell_code) for _form, pattern in other_assignment_forms(name)))}


THREAD_DEFAULT_FORMS = [
    thread_default_form("nproc", r"\bnproc\b"),
    thread_default_form("cpu_count", r"\bcpu_count\s*\("),
    thread_default_form("sched_getaffinity", r"\bsched_getaffinity\b"),
    thread_default_form("available_parallelism", r"\bavailable_parallelism\b"),
    thread_default_form("available_threads（按此刻忙闲现算）", r"\bavailable_threads\b"),
    thread_default_form("min(16", r"\bmin\s*\(\s*16\b|\.min\(\s*16\s*\)"),
    thread_default_form("写死的线程数默认", r"\$\{" + THREAD_VARIABLE + r":-[0-9]+\}"
                        r"|[\"']--(?:threads|jobs|workers|lanes)[\"'][^\n]*?\bdefault\s*=\s*[0-9]+"
                        r"|\b(?=[A-Z0-9_]*(?:DEFAULT|WHEN_UNSET|BY_PROCESSOR))(?=[A-Z0-9_]*(?:THREADS|JOBS|WORKERS))[A-Z0-9_]+\s*(?::\s*\w+\s*)?=\s*[0-9]+\b"
                        r"|[\"'](?:threads|jobs|workers)[\"']\s*:\s*[0-9]+\b"),
    (CONDITIONAL_ASSIGNMENT_FORM, CONDITIONAL_ASSIGNMENT_PATTERN),
]
HALF = re.compile(r"(?<![/*\w])/{1,2}\s*2(?![\w.])")
THREAD_DEFAULT_HALF_CONTEXT = re.compile(r"(?i)nproc|cpu_count|core|processor|parallel|thread|jobs|worker|核")
LITERAL_FALLBACK = re.compile(r"unwrap_or\(\s*[1-9][0-9]*\s*\)|Err\(_\)\s*=>\s*[1-9][0-9]*\b|\bor\s+[1-9][0-9]*\b")
THREAD_VARIABLE_NAMED = re.compile(r"[\"']" + THREAD_VARIABLE + r"[\"']")
FALLBACK_WINDOW = 5
OTHER_QUANTITY = re.compile(r"(?i)timeout|deadline|seconds|duration|超时")   # 回退值所在那一行写着这些词：那个数是别的量（超时秒数一类），不是线程数
CAPPED = re.compile(r"capped\.sh(?P<close>[\"']?)(?:\s*,\s*[\"']?|\s+)[0-9]+\b")
RUST_STRING = re.compile(r'"(?:[^"\\]|\\.)*"')
CAPPED_FILE = "scripts/hosts/capped.sh"
OWN_VARIABLE_QUOTED = re.compile(r"[\"'](" + THREAD_VARIABLE + r")[\"'](?!\s*\]\s*=(?!=))")      # ["X"] = 值 是写给子进程的，不是读
OWN_VARIABLE_QUOTED_WRITE_COUNTED = re.compile(r"[\"'](" + THREAD_VARIABLE + r")[\"']")
ENVIRONMENT_READ = re.compile(r"\benviron\b|\bgetenv\b|\benv::var(?:_os)?\b|\boption_env!|\benv!\s*\(|\bprintenv\b"
                              r"|\b(?:\w+_)?(?:env|environment)\s*(?:\.get\s*\(|\[)|\bin\s+(?:\w+\.)?(?:env|environment)\b")   # 只有弄坏开关 thread-default-quoted-name-needs-environment-read 用：读环境变量的写法：os.environ、os.getenv、env::var、env!、printenv，以及读叫 env / environment 的映射（environment.get(…)、environment[…]、"X" in environment）
OWN_VARIABLE_SHELL = re.compile(r"\$\{?(" + THREAD_VARIABLE + r")")
OWN_VARIABLE_PRINTENV = re.compile(r"\bprintenv\b(?:[ \t]+-[-\w]+)*[ \t]+[\"']?(" + THREAD_VARIABLE + r")")      # shell 里 `printenv NAME` 从环境里读 NAME，文件里赋没赋过值都一样
SHELL_ASSIGNMENT_BODY = r"[ \t]*(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>" + THREAD_VARIABLE + r")=(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|[^;&|\"'\n])*)"
SHELL_ASSIGNMENT = re.compile(r"(?:^|[;&|])" + SHELL_ASSIGNMENT_BODY, re.M)       # 行首，或 ; && || 之后同一行里接着赋值
SHELL_ASSIGNMENT_LINE_START_ONLY = re.compile(r"^" + SHELL_ASSIGNMENT_BODY + r".*$", re.M)
CAP_OR_REGISTRY = re.compile(r"SINGLEFS_THREAD_CAP|heavy_thread_default|default_heavy_threads|heavy-threads\.sh|heavy_threads|capped\.sh")
RUST_MODULE_ITEM = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\b")
RUST_EXTERNAL_MODULE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;")      # `mod tests;`：外置模块只是声明，测试模块的内容在另一个文件里
STATEMENT_LINE_LIMIT = 40
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
strings_scanned = broken == "thread-default-strings-scanned"


def quoted_mask(line):
    """每个字符在不在引号里（shell 的单引号、双引号；双引号里认反斜杠转义）。"""
    mask, quote, index = [False] * len(line), None, 0
    while index < len(line):
        character = line[index]
        if quote:
            mask[index] = True
            if character == "\\" and quote == '"' and index + 1 < len(line):
                mask[index + 1] = True
                index += 2
                continue
            if character == quote:
                quote = None
        elif character in "\"'":
            quote = character
            mask[index] = True
        elif character == "\\":
            index += 2
            continue
        index += 1
    return mask


def kept(literal_text):
    """字面量（连引号与前缀）要不要留着：内容整个是 THREAD_DEFAULT_KEPT_LITERAL 认的那几种才留。"""
    found = re.match(r"(?is)^[a-z]*('''|\"\"\"|'|\")(.*)\1$", literal_text)
    return bool(found) and THREAD_DEFAULT_KEPT_LITERAL.fullmatch(found.group(2)) is not None


def shell_code_lines(text):
    lines = []
    for line in text.split("\n"):
        if line.lstrip().startswith("#"):
            lines.append("")
            continue
        mask = quoted_mask(line)
        for index, character in enumerate(line):
            if character == "#" and not mask[index] and (index == 0 or line[index - 1] in " \t"):
                line = line[:index]
                break
        lines.append(OUTPUT_TEXT.sub(" ", line))
    return lines


def python_code_lines(text, keep_text=False):
    rows = [list(line) for line in text.split("\n")]

    def blank(start, end):
        (start_row, start_column), (end_row, end_column) = start, end
        for row in range(start_row, end_row + 1):
            characters = rows[row - 1]
            first = start_column if row == start_row else 0
            last = end_column if row == end_row else len(characters)
            for column in range(first, min(last, len(characters))):
                characters[column] = " "
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, SyntaxError, IndentationError):
        return shell_code_lines(text)
    scanning_strings = strings_scanned or keep_text
    middle = getattr(tokenize, "FSTRING_MIDDLE", None)
    for token in tokens:
        if token.type == tokenize.COMMENT:
            blank(token.start, token.end)
        elif not scanning_strings and ((token.type == tokenize.STRING and not kept(token.string)) or (middle is not None and token.type == middle)):
            blank(token.start, token.end)
    return [OUTPUT_TEXT.sub(" ", "".join(characters)) for characters in rows]


def rust_code_lines(text, keep_text=False):
    lines = []
    for line in text.split("\n"):
        if not (strings_scanned or keep_text):
            line = RUST_STRING.sub(lambda found: found.group(0) if kept(found.group(0)) else " " * len(found.group(0)), line)
        position = line.find("//")
        lines.append(line if position < 0 else line[:position])
    return lines


def pinned_conditional_assignment(found, registry_variables):
    """`|| X=<值>` 这一处算不算钉数：值是 $变量、而那个变量在同一份文件里只由登记位读法赋值（registry_sourced_variables）的不算。"""
    variable = CONDITIONAL_ASSIGNMENT_VARIABLE.fullmatch(found.group("value"))
    return not (variable and variable.group(1) in registry_variables and broken != "thread-default-conditional-variable-source-ignored")


def forms_of(code, window, text_of_line, registry_variables=frozenset()):
    """text_of_line：这一行连字符串带键名的样子（不含注释），只在要判「往上 5 行里点了线程变量名」那一路时才取；
    registry_variables：这份 shell 文件里只由登记位读法赋值的变量（registry_sourced_variables），`|| X=$变量` 的变量在里面的不算钉数。"""
    names = []
    for name, pattern in THREAD_DEFAULT_FORMS:
        if name == CONDITIONAL_ASSIGNMENT_FORM:
            if broken != "thread-default-conditional-assignment-unpinned" and any(pinned_conditional_assignment(found, registry_variables) for found in pattern.finditer(code)):
                names.append(name)
        elif pattern.search(code):
            names.append(name)
    if broken != "thread-default-half-unchecked" and HALF.search(code) and THREAD_DEFAULT_HALF_CONTEXT.search(code):
        names.append("核数的一半")
    if LITERAL_FALLBACK.search(code):
        if THREAD_VARIABLE_NAMED.search(code):
            names.append("写死的线程数默认（回退值）")
        elif any(THREAD_VARIABLE_NAMED.search(line) for line in window) and (broken == "thread-default-timeout-unchecked" or not OTHER_QUANTITY.search(text_of_line())):
            names.append("写死的线程数默认（回退值）")
    if any(found.group("close") or not quoted_mask(code)[found.start()] for found in CAPPED.finditer(code)):
        names.append("capped.sh <数字>")
    return names


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX:
                    yield directory, os.path.join(current, name), LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]
    for path in SCAN_FILES:
        if os.path.isfile(path):
            yield path, path, "shell"


def rust_test_module_starts(code_lines, index):
    """行首 #[cfg(test)]（第 index 行，0 起）罩着的是不是带着正文的测试模块：它后面第一个非空行（夹着的 #[…] 属性不算）是 mod、且不是分号结尾的外置模块声明（`mod tests;`）才是；
    只罩一条 use、fn、const 的不是。"""
    for piece in [code_lines[index][len("#[cfg(test)]"):]] + code_lines[index + 1:]:
        text = re.sub(r"^(?:#\[[^\]]*\]\s*)+", "", piece.strip())
        if text:
            return RUST_MODULE_ITEM.match(text) is not None and (broken == "thread-default-external-mod-skipped" or RUST_EXTERNAL_MODULE.match(text) is None)
    return False


def python_statement_ranges(text):
    """python 每一行落在哪条逻辑行（起止行，1 起）里；tokenize 读不了返回空表（那时按反斜杠续行算）。"""
    ranges, first = {}, None
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, SyntaxError, IndentationError):
        return ranges
    for token in tokens:
        if token.type in (tokenize.NL, tokenize.COMMENT, tokenize.INDENT, tokenize.DEDENT, tokenize.ENCODING, tokenize.ENDMARKER):
            continue
        if token.type == tokenize.NEWLINE:
            if first is not None:
                ranges.update({row: (first, token.start[0]) for row in range(first, token.start[0] + 1)})
            first = None
        elif first is None:
            first = token.start[0]
    return ranges


def statement_range(language, code_lines, number, python_ranges):
    """第 number 行（1 起）所在那一条语句的起止行：python 取逻辑行；shell 取反斜杠续行；
    rust 往前到上一条语句收尾（; { }）之后，往后到括号配平且以 ; 或 } 收尾、或闭合了外层的块，两头各不超过 STATEMENT_LINE_LIMIT 行。"""
    index = number - 1
    if language == "python" and number in python_ranges:
        return python_ranges[number]
    if language != "rust":
        first, last = index, index
        while first > 0 and code_lines[first - 1].rstrip().endswith("\\"):
            first -= 1
        while last + 1 < len(code_lines) and code_lines[last].rstrip().endswith("\\"):
            last += 1
        return first + 1, last + 1
    first, previous = index, index - 1
    while previous >= 0 and index - previous <= STATEMENT_LINE_LIMIT:
        text = code_lines[previous].strip()
        if text:
            if text.endswith((";", "{", "}")):
                break
            first = previous
        previous -= 1
    depth, last = 0, first
    while last + 1 < len(code_lines) and last - first < STATEMENT_LINE_LIMIT:
        text = code_lines[last]
        depth += sum(text.count(opening) for opening in "([{") - sum(text.count(closing) for closing in ")]}")
        if last >= index and text.strip() and (depth < 0 or (depth == 0 and text.rstrip().endswith((";", "}")))):
            break
        last += 1
    return first + 1, last + 1


problems_registration = []
try:
    registered = heavy_threads.default_heavy_threads({}, registration_file=REGISTRATION)
except heavy_threads.HeavyThreadDefaultError as error:
    registered = None
    problems_registration.append((str(error), error.howto))

# capped.sh 设的线程变量（variable_names 清单）：装置读的线程变量在里面，capped.sh <上限> 的数才到得了装置
capped_names = set()
try:
    capped_block = re.search(r"^variable_names=\(\n(.*?)^\)", open(CAPPED_FILE, encoding="utf-8").read(), re.S | re.M)
except OSError as error:
    capped_block = None
    problems_registration.append((f"读不了 {CAPPED_FILE}：{error}", f"这一格拿它的 variable_names 清单当「capped.sh 设了哪些线程变量」；它随仓走，被删了就从 git 找回来"))
else:
    if capped_block is not None:
        capped_names = set(re.findall(r"^\s+([A-Z][A-Z0-9_]*)\s*$", capped_block.group(1), re.M))
    if not capped_names:
        problems_registration.append((f"{CAPPED_FILE} 里读不出 variable_names=( … ) 清单里的变量名", "清单一行一个变量名、独占一行，写在 variable_names=( 与单独一行的 ) 之间；写法照 capped.sh 现在的样子"))
if broken == "thread-default-capped-list-ignored":
    capped_names = set()       # 弄坏：清单读了不认，每个线程变量都算「capped.sh 不设」
own_variable_checked = broken != "thread-default-own-variable-unchecked"
own_reads = set()              # (路径, 变量名)：装置自己读的线程变量，不管 capped.sh 设没设
own_outside = 0                # 其中 capped.sh 不设的个数（这些要么文件里回退到上限或登记值，要么落在被判行上交给排除表）
own_ignoring = []              # (路径, 变量名)：capped.sh 不设、文件里也没回退到上限或登记值

hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems_registration.append((f"读不了 {path}：{error}", "看它的权限；这一格要读到射程里的每一份"))
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if not THREAD_DEFAULT_PREFILTER.search(text):
        continue
    original = text.split("\n")
    code_lines = {"shell": shell_code_lines, "python": python_code_lines, "rust": rust_code_lines}[language](text)
    text_lines = []                # 连字符串一起留着的版本：回退值那一行用得着时才算，整份文件最多算一次

    def text_line_at(number):
        if not text_lines:
            text_lines.extend({"shell": shell_code_lines, "python": python_code_lines, "rust": rust_code_lines}[language](text, **({} if language == "shell" else {"keep_text": True})))
        return text_lines[number - 1] if number <= len(text_lines) else ""
    window = collections.deque(maxlen=FALLBACK_WINDOW)
    registry_variables = registry_sourced_variables("\n".join(code_lines)) if language == "shell" else frozenset()
    for number, code in enumerate(code_lines, 1):
        if not code.strip():
            continue
        names = forms_of(code, window, lambda number=number: text_line_at(number), registry_variables)
        window.append(code)
        if names:
            hits.setdefault(path, []).append((number, names, original[number - 1] if number <= len(original) else code))
    # 装置自己读的线程变量：代码里点了名（python / rust 的字面量，shell 的 $变量），rust 的测试模块（行首 #[cfg(test)] 后面第一个非空行是 mod，往后）不算
    own_here = {}
    # shell 里文件自己赋了值的变量（WORKERS="${2:?…}" 这种）是局部变量，不是从环境读的；右边读着它自己（X="${X:-4}"）的仍算读
    shell_assignment = SHELL_ASSIGNMENT_LINE_START_ONLY if broken == "thread-default-shell-inline-assignment-unseen" else SHELL_ASSIGNMENT
    shell_assignments = list(shell_assignment.finditer("\n".join(code_lines))) if language == "shell" and broken != "thread-default-shell-local-unchecked" else []
    shell_local = ({found.group("name") for found in shell_assignments if found.group("name") not in found.group("rhs")}
                   - {found.group("name") for found in shell_assignments if found.group("name") in found.group("rhs")})      # 有一处右边读着自己（X="${X:-4}"）的，仍是读环境
    statement_ranges = python_statement_ranges(text) if language == "python" else {}

    def statement_reads_environment(number):
        """第 number 行所在的那条语句里有读环境变量的写法（ENVIRONMENT_READ）；只有弄坏开关 thread-default-quoted-name-needs-environment-read 用它。"""
        first, last = statement_range(language, code_lines, number, statement_ranges)
        return any(ENVIRONMENT_READ.search(code_lines[row - 1]) for row in range(first, last + 1))
    for number, code in enumerate(code_lines, 1):
        if (language == "rust" and code.startswith("#[cfg(test)]") and broken != "thread-default-test-module-unskipped"
                and (broken == "thread-default-cfg-test-without-mod" or rust_test_module_starts(code_lines, number - 1))):
            break
        found_names = [found.group(1) for found in (OWN_VARIABLE_QUOTED_WRITE_COUNTED if broken == "thread-default-subscript-write-counted" else OWN_VARIABLE_QUOTED).finditer(code)]
        if found_names and broken == "thread-default-quoted-name-needs-environment-read" and not statement_reads_environment(number):
            found_names = []      # 弄坏：引号里的名字只在读环境变量的写法里才算读（经包装函数、别名、常量读的认不出）
        if language == "shell":
            found_names += [found.group(1) for found in OWN_VARIABLE_SHELL.finditer(code) if found.group(1) not in shell_local]
            if broken != "thread-default-printenv-unseen":
                found_names += [found.group(1) for found in OWN_VARIABLE_PRINTENV.finditer(code)]
        for name in found_names:
            own_here.setdefault(name, []).append(number)
    outside_here = {name: numbers for name, numbers in own_here.items() if name not in capped_names}
    own_reads.update((path, name) for name in own_here)
    own_outside += len(outside_here)
    if outside_here and own_variable_checked:
        python_ranges = python_statement_ranges(text) if language == "python" else {}
        whole_code = "\n".join(code_lines)

        def falls_back(numbers):
            """这个变量读它的任何一处所在的那条语句里有 SINGLEFS_THREAD_CAP、登记值一类的回退；同文件另一条语句里提到不算（只看代码：注释、文档串里写的不算）。"""
            if broken == "thread-default-fallback-by-file":      # 弄坏：整份文件里出现字样就算回退了
                return CAP_OR_REGISTRY.search(whole_code) is not None
            for number in numbers:
                first, last = (number, number) if broken == "thread-default-statement-single-line" else statement_range(language, code_lines, number, python_ranges)
                if CAP_OR_REGISTRY.search("\n".join(code_lines[first - 1:last])):
                    return True
            return False
        unfallen = {name: numbers[0] for name, numbers in outside_here.items() if not falls_back(numbers)}
        if unfallen:
            own_ignoring.extend((path, name) for name in unfallen)

            def own_kind(variable_names):
                return "读自己的线程变量 " + "、".join(sorted(variable_names)[:3]) + "：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值"
            if broken == "thread-default-exempt-by-file":      # 弄坏：按文件并组，这一种挂到文件里已有的每一条被判行上（新读的那一行不被点名，借已有行的锚串放行）
                if path in hits:
                    hits[path] = [(number, names + [own_kind(unfallen)], line) for number, names, line in hits[path]]
                else:
                    first_number = min(unfallen.values())
                    hits[path] = [(first_number, [own_kind(unfallen)], original[first_number - 1])]
            else:                   # 按被判行：每个变量落在它第一次被读的那一行（同一行的几个变量写在一起），该行已是被判行的并上去，不是的单列一条，排除表要有锚串落在这一行里才放
                by_number = {number: (names, line) for number, names, line in hits.get(path, [])}
                variables_by_number = {}
                for variable_name, number in unfallen.items():
                    variables_by_number.setdefault(number, []).append(variable_name)
                for number, variable_names in variables_by_number.items():
                    names, line = by_number.get(number, ([], original[number - 1] if number <= len(original) else ""))
                    by_number[number] = (names + [own_kind(variable_names)], line)
                hits[path] = [(number, *by_number[number]) for number in sorted(by_number)]

exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or len(found.group("reason")) < 4:
            exempt_problems.append(f"第 {number} 行不是「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）：{line[:120]}")
            continue
        exempt_rows.append((number, found.group("path"), found.group("anchor")))
exempted = 0
unexempted = []
for path, entries in sorted(hits.items()):
    anchors = [anchor for _number, row_path, anchor in exempt_rows if row_path == path]
    for number, names, line in entries:
        if any(anchor in line for anchor in anchors):
            exempted += 1
        else:
            unexempted.append((path, number, names, line))
if broken != "thread-default-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "thread-default-exempt-growth-unchecked":
    base_keys = set()
    for line in open(base_table, encoding="utf-8").read().split("\n"):
        found = EXEMPT_ROW.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.add((found.group("path"), found.group("anchor")))
    grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"取不到基准 {base_name[:12]} 里的 {EXEMPT_TABLE}（{base_state[len('unreadable:'):][:120]}），只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and registered is None and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 与 {REGISTRATION} 都不在")
    sys.exit(77)
failed = False
for reason, howto in problems_registration:
    failed = True
    print(f"  ✗ {reason}")
    print(f"     → 怎么办：{howto}")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处自己定并行度默认值的写法没经登记位取（每台默认重活线程数登记在 {REGISTRATION}）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：shell 先 source scripts/hosts/heavy-threads.sh 再 heavy_thread_default 取（`|| X=$变量` 被点名的，改成 `|| X=$(heavy_thread_default)` 直接取），python 用 scripts/hosts/heavy_threads.py 的 default_heavy_threads() 取，")
    print("               起命令写 bash scripts/hosts/capped.sh default；参数或环境变量给了照给的。装置读自己的线程变量的：把变量名加进 scripts/hosts/capped.sh 的 variable_names，")
    print("               或让装置没设时回退到 SINGLEFS_THREAD_CAP / 登记值（rust 装置 include scripts/experiments/heavy_threads_default/threads.rs，python 用 default_heavy_threads）。确实不是重活（自证输入、线程数就是场景本身、只报核数不起线程）的，")
    print(f"               登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经改成经登记位取就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的默认值经 heavy_thread_default / default_heavy_threads / capped.sh default 取；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in (*SCAN_DIRECTORIES, *SCAN_FILES) if scanned_by_directory[name])
print(f"  ✓ 每台默认重活线程数只经登记位取：登记值 {registered}；扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里"
      f"（表 {len(exempt_rows)} 行，每行都命中）；装置自己读的线程变量 {len(own_reads)} 个，其中 capped.sh 不设的 {own_outside} 个都回退到上限或登记值、或被判行在排除表里"
      f"（capped.sh 清单 {len(capped_names)} 个变量）；{growth_note}")
PY
}

# ── 格 long-run-header-exempt：长活声明（# long-run-hosts:）按现算读的脚本豁免表 .claude/long-run-live-hosts-exempt
# 脚本文件头的 # long-run-hosts: 与派发提示同一种登记值读法（逐字等于 lib_long_run.py 的 CANONICAL_HOSTS_SENTENCE 那一句规范写法，可带句末句号，后面不许接任何说明），只有这张表里登记、登记的 sha256 又对得上的脚本按现算读；
# 判法只在 .claude/hooks/lib_long_run.py 的 exempt-check（用这一道所在仓里那一份，被判的根照传）：一行「<相对仓根的路径>  <脚本文件的 sha256>  # 理由」写对、路径存在、登记的 sha256 等于脚本现在文件的 sha256、
# 脚本文件头按登记值读法还不齐（已经写齐的行留着就是白放行）、按现算读法是齐的（不齐的行不起作用）；只缩不涨：与改动范围的基准
# （scripts/gate/changed-paths.sh 的 gate_diff_base gate）里那一版按「路径 + sha256」成对比，基准里没有的路径、基准里同一路径的 sha256 换了的行（改了脚本正文再重钉，也算涨）都判红；
# 基准那一版还是不带 sha256 的旧写法的行只比路径；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明。
# 退 77：被判的根下没有这张表。
# 样本：long-run-header-exempt-green 的表登记一份文件头按现算写的脚本（点名 --emit-hosts 与 host-resources.py、没点名 heavy-thread-default），判绿并报 1 行；
#   long-run-header-exempt-red 的表六行：指向不存在的路径、脚本已按登记值写齐、脚本按现算读也不齐（只写了分片一样）、写法不对（没有 sha256 与 # 理由）、
#   setup.sh 建 git 仓提交一版不带 grown.sh 那一行的表再加回来（涨了）、一行好的（live.sh），前五样都点名；
#   long-run-header-exempt-sha256-red 的表三行：live.sh 对得上、stale.sh 登记的是 live.sh 的 sha256（脚本登记之后改过一行）、short-sha.sh 的 sha256 只有 63 位（写法不对），判红点名后两行；
#   long-run-header-exempt-repin-red 的表一行：setup.sh 建 git 仓提交改正文之前的 live.sh（slots=7）与登记它的 sha256 的表，工作区改了 live.sh 的正文（slots=8）、把表里的 sha256 换成新文件的（重钉，路径与行数不变），
#   sha256 都对得上，判红点名「登记的 sha256 … 与基准那一版里的 … 不同（豁免表重钉了…）」；
#   long-run-header-exempt-base-green 的表一行：setup.sh 把表与脚本原样提交为基准，工作区没动（路径与 sha256 都没变），判绿并写「与基准那一版比没涨」。
# 弄坏开关（CODE_TOOLING_BREAK，折成 LIB_LONG_RUN_BREAK 去掉 long-run- 前缀的同名项）：
#   long-run-exempt-path-unchecked（不核路径在不在）：long-run-header-exempt-red 里「指向不存在的路径」与「有 5 处不对（共 6 行）」两条 want 找不到；
#   long-run-exempt-needless-unchecked（不核脚本是不是已按登记值写齐、按现算读是不是齐）：long-run-header-exempt-red 里「已经按登记值读法写齐了」「按现算读法也不齐」与「有 5 处不对（共 6 行）」三条 want 找不到；
#   long-run-exempt-growth-unchecked（不比基准）：long-run-header-exempt-red 里「豁免表涨了」与「有 5 处不对（共 6 行）」两条 want 找不到；long-run-header-exempt-repin-red 期望退出 1、实测 0；
#   long-run-exempt-repin-unchecked（比基准时只比路径、不比 sha256）：long-run-header-exempt-repin-red 期望退出 1、实测 0（别的样本不变）；
#   long-run-exempt-sha256-unchecked（不比登记的 sha256）：long-run-header-exempt-sha256-red 里「现在的 sha256 是 …，表里登记的是 …」与「有 2 处不对（共 3 行）」两条 want 找不到。
cell_long_run_header_exempt() {
library="$STAGE_REPOSITORY/.claude/hooks/lib_long_run.py"
[[ -f "$library" ]] || { echo "  ✗ 找不到 $library"; echo "     → 怎么办：它随仓走（.claude/hooks/lib_long_run.py），被删了就从 git 找回来；豁免表的判法只在那一份里写。"; exit 1; }
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版豁免表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
base_arguments=()
base_note=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1 && git rev-parse --verify -q HEAD >/dev/null 2>&1; then
  # shellcheck source=../../scripts/gate/changed-paths.sh
  if ! source "$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"; then
    echo "  ✗ source 不了 $STAGE_REPOSITORY/scripts/gate/changed-paths.sh，取不到改动范围的基准，豁免表只缩不涨没法比"
    echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
    exit 1
  fi
  base_name="$(gate_diff_base gate)"
  if ! listed="$(git -c core.quotepath=false ls-tree --name-only "$base_name" -- .claude/long-run-live-hosts-exempt 2>&1)"; then
    echo "  ✗ 取不到基准 ${base_name::12} 里的 .claude/long-run-live-hosts-exempt（${listed::120}），只缩不涨没比"
    echo "     → 怎么办：看 git 能不能读到基准那一版（git ls-tree ${base_name::12}），修好再跑这一格。"
    exit 1
  elif [[ -n "$listed" ]]; then
    if git show "$base_name:.claude/long-run-live-hosts-exempt" > "$base_table" 2>/dev/null; then
      base_arguments=(--base-table "$base_table")
    else
      echo "  ✗ 取不到基准 ${base_name::12} 里的 .claude/long-run-live-hosts-exempt（git show 退了非 0），只缩不涨没比"
      echo "     → 怎么办：看 git 能不能读到基准那一版（git show ${base_name::12}:.claude/long-run-live-hosts-exempt），修好再跑这一格。"
      exit 1
    fi
  fi
fi
case "${CODE_TOOLING_BREAK:-}" in
  long-run-exempt-path-unchecked) library_break="exempt-path-unchecked" ;;
  long-run-exempt-needless-unchecked) library_break="exempt-needless-unchecked" ;;
  long-run-exempt-growth-unchecked) library_break="exempt-growth-unchecked" ;;
  long-run-exempt-repin-unchecked) library_break="exempt-repin-unchecked" ;;
  long-run-exempt-sha256-unchecked) library_break="exempt-sha256-unchecked" ;;
  *) library_break="" ;;
esac
# 判法的出口：0 好，1 有问题，77 没有这张表；✓ 打 stdout、✗ 与出路打 stderr，这里并成一路给阶段汇总看
if LIB_LONG_RUN_BREAK="$library_break" python3 "$library" exempt-check "$ROOT" ${base_arguments[@]+"${base_arguments[@]}"} 2>&1; then exit 0; else exit "$?"; fi
}

# ── 格 function-units-two-implementations-agree：函数单元的两份实现对拍（crates/singlefs-unit-graph 的 unit-graph 与 scripts/gate/lib_function_units.py）
# 判据：被判的根上 unit-graph dump 与 python 那一份的 dump --language rust、--language wgsl 各一趟，按（文件，起行）配对，逐单元比种类、限定名、起止、指纹
#   （dump 的写法以 Rust 那一份为准，数据单元 data 行同样配对）；再对登记入口逐个比闭包成员与键。登记入口：被判的根 .claude/gate.d/stage-inputs.tsv 第二列、
#   scripts/gate/verus-proof-check.py 与 scripts/mutation/mutation-prune.py 的 JUDGING_ENTRIES 里写成 <文件>.rs::<限定名> 的那几个。python 那一份照 find_entry
#   定入口单元与目标，换成 unit-graph closure <crate>::<模块路径>::<名>（--target <种类>:<名>，lib 不写；lib 与 bin 里按 cfg(test) 编的另给 --cfg test），
#   两边同给一行工具链 host: x86_64-unknown-linux-gnu、不给依赖行；python 那一份的键照 unit-graph 的写法算（lib_function_units.py 的 unit_graph_key）。
#   差一处判红、点名单元（逐处至多列 12 行）。python 那一份解不成入口的（不在 crates/ 的工作区里，例 scripts/experiments/ 下的实验装置）逐个列在成功那句里。
# unit-graph 怎么来：这一道所在仓的 crates/singlefs-unit-graph（连同它 include_str! 读的 .claude/heavy-thread-default）拷进
#   ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-unit-graph-build/ 里一个只有它的工作区，经内存包装（上限 GATE_UNIT_GRAPH_MEMORY_MAX，默认 4G）与
#   capped.sh default 起 cargo build --release --offline -p singlefs-unit-graph（编译目录跨轮留着，源码没变不重编；不在仓的工作区里编，不碰仓的 Cargo.lock）；
#   编出来的 unit-graph 每一趟经同一个内存包装起，包装自己的退出码（245、250–254）判「没跑成」。几道同时编时按那个目录里的 lock 排队（flock）。
# 只供样本（被判的根不是这一道所在的仓、也没设 GATE_STAGED_FROM 时才认）：
#   .unit-graph-recorded/rust-side.json  录好的 Rust 那一侧（dump 与每个入口那一趟 closure 的退出码与输出行，键是 unit-graph 在根之后的参数）：有它就不编、不起 unit-graph
#   .function-units-break                一行，当 python 那一份的 FUNCTION_UNITS_BREAK（真仓与 --staged 下这个变量一律置空）
#   重录（真 unit-graph 跑一趟）：先删掉样本根的 .unit-graph-recorded/，再
#     FUNCTION_UNITS_AGREE_RECORD=<样本根>/.unit-graph-recorded/rust-side.json bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree <样本根>
#     （key-red 那一份的录像是从 green 那一份拷来、把 lib.rs::entry 那一趟的 key 行改了一位，重录之后要照样改回去）
# 样本：function-units-two-implementations-agree-green（一个小 crate：文件开头 #![…]、行内模块、include_str! 读的数据文件、bin 的 main，三个登记入口，判绿）；
#   -dump-red（.function-units-break 是 rust-module-heads-dropped：python 那一份不切模块头，dump 与闭包都点名 mod_head）；
#   -closure-red（rust-constants-dropped：dump 相同，闭包里 Rust 那一份多两个 const）；-key-red（录好的 entry 那一趟的键改了一位：闭包相同、键不同）。
# 弄坏开关 CODE_TOOLING_BREAK：function-units-dump-unchecked 不比 dump（-dump-red 里「只有 Rust 那一份切出：mod_head」那条 want 找不到）；
#   function-units-closures-unchecked 不比闭包与键（-closure-red 判绿）；function-units-keys-unchecked 闭包相同就不比键（-key-red 判绿）。
cell_function_units_two_implementations_agree() {
units_library="$STAGE_REPOSITORY/scripts/gate/lib_function_units.py"
[[ -f "$units_library" ]] || { echo "  ✗ 找不到 $units_library"; echo "     → 怎么办：它随仓走（scripts/gate/lib_function_units.py），被删了就从 git 找回来；别在这一格另写一份。"; exit 1; }
memory_cap="${GATE_UNIT_GRAPH_MEMORY_MAX:-4G}"
memory_wrapper="$STAGE_REPOSITORY/scripts/hosts/run-with-memory-cap.sh"
python_break=""
rust_side=(binary)
if judged_root_is_sample; then
  [[ -f .function-units-break ]] && python_break="$(tr -d '[:space:]' < .function-units-break)"
  [[ -f .unit-graph-recorded/rust-side.json ]] && rust_side=(recording "$ROOT/.unit-graph-recorded/rust-side.json")
fi
if [[ "${rust_side[0]}" == binary ]]; then
  build_home="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-unit-graph-build"
  crate_source="$STAGE_REPOSITORY/crates/singlefs-unit-graph"
  [[ -f "$crate_source/Cargo.toml" ]] || { echo "  ✗ 这一道所在的仓里没有 crates/singlefs-unit-graph"; echo "     → 怎么办：两份实现对拍要编 Rust 那一份；crate 被删了就从 git 找回来，改了名就改这一格的 crate_source。"; exit 1; }
  mkdir -p "$build_home/workspace/crates" "$build_home/workspace/.claude" || { echo "  ✗ 建不了 $build_home"; echo "     → 怎么办：看 \${GATE_CROSS_RUN_TMPDIR:-\${TMPDIR:-/tmp}} 满没满、有没有写权限，修好再跑。"; exit 1; }
  exec 9> "$build_home/lock"
  if command -v flock > /dev/null; then flock 9; fi
  rm -rf "${build_home:?}/workspace/crates/singlefs-unit-graph"
  cp -a "$crate_source" "$build_home/workspace/crates/" && cp -a "$STAGE_REPOSITORY/.claude/heavy-thread-default" "$build_home/workspace/.claude/" \
    || { echo "  ✗ 拷不进 $build_home/workspace"; echo "     → 怎么办：看那个目录的空间与权限，修好再跑。"; exit 1; }
  manifest_text=$'[workspace]\nmembers = ["crates/singlefs-unit-graph"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\noverflow-checks = true\ndebug-assertions = true\n'
  # 清单只在内容变了时重写（mtime 不动，cargo 不因为它重编）
  if [[ ! -f "$build_home/workspace/Cargo.toml" || "$(cat "$build_home/workspace/Cargo.toml")"$'\n' != "$manifest_text" ]]; then
    printf '%s' "$manifest_text" > "$build_home/workspace/Cargo.toml"
  fi
  build_log="$build_home/build.log"
  if ! CARGO_TARGET_DIR="$build_home/target" bash "$memory_wrapper" "$memory_cap" bash "$STAGE_REPOSITORY/scripts/hosts/capped.sh" default \
      cargo build --release --offline --manifest-path "$build_home/workspace/Cargo.toml" -p singlefs-unit-graph > "$build_log" 2>&1; then
    echo "  ✗ 编不出 unit-graph（cargo build --release -p singlefs-unit-graph，日志 $build_log）：$(tail -n 3 "$build_log" | tr '\n' ' ' | cut -c1-300)"
    echo "     → 怎么办：看日志最后几行；内存包装自己的退出码（245、250–254）照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，编译错就修 crates/singlefs-unit-graph。"
    exit 1
  fi
  exec 9>&-
  rust_side=(binary "$build_home/target/release/unit-graph" "$memory_cap" "$memory_wrapper")
fi
FUNCTION_UNITS_BREAK="$python_break" python3 - "$ROOT" "$units_library" "${CODE_TOOLING_BREAK:-}" "${FUNCTION_UNITS_AGREE_RECORD:-}" "${rust_side[@]}" <<'PY'
import ast, collections, importlib.util, json, os, re, subprocess, sys
root, library, broken, record_path, mode = sys.argv[1:6]
HOST_LINE = "host: x86_64-unknown-linux-gnu"
WRAPPER_OWN_EXITS = {245, 250, 251, 252, 253, 254}
ENTRY = re.compile(r"^(.+\.rs)::(.+)$")
LIST_LIMIT = 12
if mode == "binary":
    binary, memory_cap, wrapper = sys.argv[6:9]
    recording = None
else:
    with open(sys.argv[6], encoding="utf-8") as handle:
        recording = json.load(handle)
recorded = {"dump": None, "closures": {}}


def rust(request):
    """unit-graph 的一趟 → (退出码, 输出行)；request 是根之后的参数（录好的输出照它认，不带根）。"""
    name = "\t".join(request)
    if recording is not None:
        entry = recording.get("dump") if request[0] == "dump" else recording["closures"].get(name)
        if entry is None:
            return None, [f"录好的输出里没有这一趟：unit-graph {' '.join(request)}"]
        return entry["exit"], entry["lines"]
    completed = subprocess.run(["bash", wrapper, memory_cap, binary, request[0], root, *request[1:]], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if completed.returncode in WRAPPER_OWN_EXITS:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-3:] if line.strip())
        return completed.returncode, [f"没跑成：内存包装退 {completed.returncode}：{tail[:300]}"]
    lines = [line for line in completed.stdout.split("\n") if line]
    entry = {"exit": completed.returncode, "lines": lines}
    if request[0] == "dump":
        recorded["dump"] = entry
    else:
        recorded["closures"][name] = entry
    return completed.returncode, lines


specification = importlib.util.spec_from_file_location("lib_function_units_two_implementations", library)
units = importlib.util.module_from_spec(specification)
specification.loader.exec_module(units)
problems = []

# dump：按（文件，起行）配对，逐单元比种类、限定名、起止、指纹
exit_code, rust_lines = rust(["dump"])
if exit_code != 0:
    print(f"  ✗ unit-graph dump 没跑成（退 {exit_code}）：{' | '.join(rust_lines[-3:])[:400]}")
    print("     → 怎么办：单跑那一趟看报错；内存包装自己的退出码（245、250–254）照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办")
    sys.exit(1)
python_lines = units.dump_lines(root, "rust")[0] + units.dump_lines(root, "wgsl")[0]


def placed(lines):
    groups = collections.defaultdict(list)
    for line in lines:
        fields = line.split("\t")
        if len(fields) != 4 or fields[0] in ("refused", "key"):
            continue
        file, _colon, span = fields[2].rpartition(":")
        start = span.split("-")[0]
        groups[(file, int(start) if start.isdigit() else 0)].append(line)
    return groups


rust_groups, python_groups = placed(rust_lines), placed(python_lines)
rust_count = sum(len(lines) for lines in rust_groups.values())
by_kind = collections.Counter("WGSL" if line.startswith("wgsl_") else "数据" if line.startswith("data\t") else "Rust"
                              for lines in rust_groups.values() for line in lines)
differences = []
for place in sorted(set(rust_groups) | set(python_groups)):
    rust_only = sorted((collections.Counter(rust_groups.get(place, ())) - collections.Counter(python_groups.get(place, ()))).elements())
    python_only = sorted((collections.Counter(python_groups.get(place, ())) - collections.Counter(rust_groups.get(place, ()))).elements())
    if len(rust_only) == 1 and len(python_only) == 1:
        fields = [label for label, left, right in zip(("种类", "限定名", "起止", "指纹"), rust_only[0].split("\t"), python_only[0].split("\t")) if left != right]
        differences.append(f"{place[0]}:{place[1]} {'、'.join(fields)}不同：Rust「{rust_only[0][:150]}」python「{python_only[0][:150]}」")
        continue
    differences += [f"{place[0]}:{place[1]} 只有 Rust 那一份切出：{line[:200]}" for line in rust_only]
    differences += [f"{place[0]}:{place[1]} 只有 python 那一份切出：{line[:200]}" for line in python_only]
if differences and broken != "function-units-dump-unchecked":
    problems.append(f"dump 有 {len(differences)} 处单元两份不同（Rust {rust_count} 行、python {sum(len(lines) for lines in python_groups.values())} 行）")
    for line in differences[:LIST_LIMIT]:
        print(f"     {line}")
    if len(differences) > LIST_LIMIT:
        print(f"     …另 {len(differences) - LIST_LIMIT} 处")

# 登记入口
entries, sources = [], collections.Counter()
table = os.path.join(root, ".claude", "gate.d", "stage-inputs.tsv")
if os.path.isfile(table):
    with open(table, encoding="utf-8") as handle:
        for raw in handle:
            if raw.startswith("#") or "\t" not in raw:
                continue
            for token in raw.rstrip("\n").split("\t")[1].split():
                if ENTRY.match(token):
                    entries.append(token)
                    sources["stage-inputs.tsv"] += 1
for relative in ("scripts/gate/verus-proof-check.py", "scripts/mutation/mutation-prune.py"):
    path = os.path.join(root, relative)
    if not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8") as handle:
        tree = ast.parse(handle.read())
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "JUDGING_ENTRIES" for target in node.targets):
            for constant in ast.walk(node.value):
                if isinstance(constant, ast.Constant) and isinstance(constant.value, str) and ENTRY.match(constant.value):
                    entries.append(constant.value)
                    sources[relative] += 1
entries = list(dict.fromkeys(entries))
workspace = units.RustWorkspace(root)
compared, unresolved = 0, []
for entry in entries:
    path, qualified = ENTRY.match(entry).groups()
    try:
        unit, target_id = workspace.find_entry(path, qualified, None)
    except units.UsageError as error:
        unresolved.append(f"{entry}（{error}）")
        continue
    target = workspace.targets[target_id]
    module_path = list(target.files.get(unit.file, ((), []))[0])
    if unit.container is not None:
        container = unit.container
        type_name = sorted(container.self_types)[0] if container.kind == "impl" and container.self_types else container.short
        segments = module_path + list(unit.scope or ()) + [type_name, unit.short]
    else:
        segments = module_path + unit.name.split("::")
    request = ["closure", f"{target.crate.lib_name}::{'::'.join(segments)}"]
    if target.kind != "lib":
        request += ["--target", target.kind if target.kind == "build" else f"{target.kind}:{target.name}"]
    if target.kind in ("lib", "bin") and units.rust_unit_is_test(workspace, unit, target_id):
        request += ["--cfg", "test"]
    request += ["--toolchain-line", HOST_LINE]
    exit_code, lines = rust(request)
    if exit_code not in (0, 3):
        problems.append(f"{entry}：unit-graph {' '.join(request)} 退 {exit_code}（{' | '.join(lines[-2:])[:200]}）")
        continue
    try:
        closure = units.rust_closure_of(workspace, unit, target_id)
    except units.UsageError as error:
        problems.append(f"{entry}：python 那一份收不了闭包（{error}）")
        continue
    compared += 1
    if broken == "function-units-closures-unchecked":
        continue
    python_members = [member.dump_text() for member in closure.order if not member.data]
    python_members += [f"data\t{site[2]}\t{site[0]}:{site[1]}-{site[1]}\t{site[3]}" for site in closure.data_sites]
    rust_members = [line for line in lines if not line.startswith(("refused\t", "key\t"))]
    rust_left = collections.Counter(rust_members) - collections.Counter(python_members)
    python_left = collections.Counter(python_members) - collections.Counter(rust_members)
    if rust_left or python_left:
        problems.append(f"{entry}：闭包两份不同（Rust {len(rust_members)} 个、python {len(python_members)} 个；只在 Rust 里 {sum(rust_left.values())} 个、"
                        f"只在 python 里 {sum(python_left.values())} 个）")
        for line in sorted(rust_left.elements())[:LIST_LIMIT // 2]:
            print(f"     {entry} 只在 Rust 闭包里：{line[:200]}")
        for line in sorted(python_left.elements())[:LIST_LIMIT // 2]:
            print(f"     {entry} 只在 python 闭包里：{line[:200]}")
        continue
    if broken == "function-units-keys-unchecked":
        continue
    rust_key = next((line.split("\t", 1)[1] for line in lines if line.startswith("key\t")), None)
    python_key, python_refusals = units.unit_graph_key(closure, [HOST_LINE], [])
    if rust_key != python_key:
        problems.append(f"{entry}：闭包相同、键不同（Rust {rust_key or '拒算'}，python {python_key or '拒算：' + '；'.join(refusal.text() for refusal in python_refusals[:2])}）")
if record_path and recording is None:
    os.makedirs(os.path.dirname(record_path) or ".", exist_ok=True)
    with open(record_path, "w", encoding="utf-8") as handle:
        json.dump(recorded, handle, ensure_ascii=False, indent=0, sort_keys=True)
    print(f"  · 录下了 Rust 那一侧：{record_path}（dump 一趟、closure {len(recorded['closures'])} 趟）")
named = "、".join(f"{source} {count} 个" for source, count in sorted(sources.items())) or "一个都没有"
if problems:
    print(f"  · 比了 dump {rust_count} 个单元、登记入口 {compared} 个（{named}）；解不成 Rust 入口的 {len(unresolved)} 个" + (f"：{'；'.join(unresolved)}" if unresolved else ""))
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）有 {len(problems)} 处不一致")  # gate-lint:summary
    print("     → 怎么办：照上面点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份（定的口径变了就两边一起改）；"
          "单跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree")
    sys.exit(1)
print(f"  ✓ 两份实现对拍：dump 比了 {rust_count} 个单元（{'、'.join(f'{kind} {count}' for kind, count in sorted(by_kind.items()))}），逐个相同；"
      f"登记入口 {compared} 个（{named}），闭包与键逐个相同；解不成 Rust 入口的 {len(unresolved)} 个" + (f"：{'；'.join(unresolved)}" if unresolved else ""))
PY
}

# ── 格 reuse-selftests：登记表 .claude/gate.d/verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了
#
# 判法全在 scripts/gate/lib_reuse_checks.py 的 audit（文件头「四」：登记表的写法、射程 # scan:、light 现跑与 heavy 读上一次输出、本次未跑与不算）；
#   这一格只取并行度（每台默认重活线程数 heavy_thread_default，同 research-script-selftests）、交被判的根。用的是这一道所在仓里那一份 lib_reuse_checks.py
#   （样本根里没有它）；现跑的自证在被判的根里起，样本根里的是样本自己的假自证。
# 样本：reuse-selftests-green（一份 light 假自证打三问都过的规范输出，一份「无判定库：待立」报本次未跑，一份「不算」）判绿；
#   reuse-selftests-red（一份 light 假自证只退 0、不打规范输出，一份注释写了判定库却没登记，一份指到不存在的路径）判红、三样都点名。
# 弄坏开关 CODE_TOOLING_BREAK=reuse-selftest-line-unchecked：折成 REUSE_CHECKS_BREAK=audit-line-unchecked，自证退 0 就算过：
#   reuse-selftests-red 里「输出里没有 REUSE_SELFTEST script=scripts/gate/no-line.sh」那条 want 找不到（另两样照红）。
cell_reuse_selftests() {
  local jobs break_items=""
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），现跑自证的并行度定不了，一份都没判"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default 或 SINGLEFS_THREAD_CAP，再跑这一格；不退回按核数算。"
    exit 1
  fi
  [[ "${CODE_TOOLING_BREAK:-}" == reuse-selftest-line-unchecked ]] && break_items=audit-line-unchecked
  REUSE_CHECKS_BREAK="$break_items" python3 "$STAGE_REPOSITORY/scripts/gate/lib_reuse_checks.py" audit "$ROOT" --jobs "$jobs"
}

stage_cells_run "$ROOT"
