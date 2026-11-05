#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录、跑研究脚本与共用库自己的自证、起各道门禁的 --list；research-gate-lint 格调的上游 gate-lint.sh、shell-lint.sh 要 gawk，缺了由它们自己拒绝（退 78），那一格按红记，不交给 gate.sh 预判；function-units-two-implementations-agree 格编 unit-graph 要的 cargo 缺了那一格自己判红
# gate-stage: 工具层自检（三十七格：研究脚本的自证都过、有阶段在跑；带判定库的脚本自证里复用三问都过；项目规则清单（@ 引用加 read-once 登记）；阶段归属表按格登记；钩子注册与写范围表与定义头；定义的形状（model 与 tools 写明、tools 不给 Agent、四节齐、不抄规则的长行、调度表与定义两向相等）；改动范围只许一份取法；研究脚本与钩子的拒绝出路与 shell 纪律；阶段声称的样本与编造的日期；带编号登记位的样本都排在 doc-lint 之外；kb 登记表；门禁结构；环境检查排在跑测试的阶段之后；目录形态；提交里没有超过单文件上限的文件；阶段在 cd 之前取好自己的位置；并行度默认值只经每台默认重活线程数的登记位取；跑法不写死内存上限与台数；函数单元两份实现对拍；变异门禁的 --list-items 在被判的仓上跑得通；门禁阶段的弄坏开关只在样本根上认；入口脚本带列项、自证与每项一行；弄坏开关文件头三样写全；门禁阶段的弄坏开关逐条复跑、判错的样本与头注点名的相等；研究脚本与钩子平常跑时设了弄坏开关要被拒；登记的脚本与钩子的弄坏开关逐条复跑、点名的项都在失败行里；kb 目录登记位里的每个目录连同子目录都在写回员的写范围里；脚本不经 ~ 与 $HOME 取数据根下共享的那几项；CLAUDE.md 登记的规则都被定义点名；runner 表里套临时服务包装的自证与要临时服务的表里登记成 required 的两边对齐；要临时服务的自证的整条写法在登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、门禁的包装前缀常量这七处逐字相同、包装的解析收得下、不带尖括号占位的 --server-binary）
# gate-fixture: yes
# gate-category: 代码类
# 不写 gate-covers：.claude/gate-not-implemented.tsv 里的两项判的是被测代码的行为，这一道只判工具层自己。
# gate-similar: research-build-and-replay.sh 它编 research/ 的装置、跑它们的单测，不跑研究脚本的自证（stage-mine.py、relabel-item.py、claim-experiment.sh 这几份的自证不要 research 工作区先编好，同别的研究脚本一样在这里的 runner 表里跑）；这一道的 research-script-selftests 格跑 runner 表里研究脚本的自证、research-script-selftest-coverage 格核每份实现了 --selftest 的都有阶段在跑，不编译
# gate-similar: model-differential-and-scenarios.sh 它在真仓上判 harness 用例一条一个场景，crash-case-check.py 的判别力由这一道 research-script-selftests 格跑它的 --selftest；判的对象一个是用例源码，一个是脚本的自证
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
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就三十七格按下面的次序全跑，格名写错退 2）：
# gate-cell: research-script-selftests runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
# gate-cell: research-script-selftest-coverage scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
# gate-cell: reuse-selftests 登记表 verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了
# gate-cell: rules-manifest 项目规则文件与 CLAUDE.md 的 @ 引用加 read-once 登记逐项相等
# gate-cell: rules-named-by-definitions CLAUDE.md 的每条 @ 规则与 read-once 登记的规则，在 .claude/agents/*.md 或 .claude/agent-common.md 里至少被一份按路径点名
# gate-cell: stage-owners 阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根
# gate-cell: agent-write-scope 项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份
# gate-cell: agent-definition-shape 定义写明 model 与 tools、tools 不给 Agent 与 Task、四节齐，定义与共用约束不抄规则的长行，主 agent 的调度表与定义两向相等
# gate-cell: change-range-single-source 门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码；基准那一版照登记的搬迁换算，搬迁登记表写对、指得到
# gate-cell: research-gate-lint 研究脚本、钩子与 .claude/scripts 的拒绝出路、shell 纪律、执行位，终止进程只许点名一个
# gate-cell: fixture-claims 阶段头部的显式标记声称的样本真的在（yes 要目录在，none 要目录不在且登记在豁免表）；文件名与样本正文里的日期不许是编的
# gate-cell: fixture-registrations-excluded 样本目录里有编号登记位（## D<n> / ## E<n> 标题、doc-lint:registry 标记）的阶段，都在 .claude/doc-lint-exclude 里有一行；排除表里指向 fixtures 下不存在的阶段的行判红
# gate-cell: kb-registry kb 目录里的每一份都在 CLAUDE.md「项目本地事实」表里有一行
# gate-cell: gate-structure 每道门禁有格名表，--list 与表一致且不起重活，--check 写错格名退 2
# gate-cell: stage-order 环境检查那一道排在门禁目录里全部跑测试的阶段之后
# gate-cell: directory-layout research/ 与 scripts/ 的顶层、说明文件、层级与不放产物合目录规矩
# gate-cell: committed-file-size 被提交的东西里没有超过 100 MiB 的文件：HEAD 加暂存区里跟踪的每个文件，与还没推上去的每笔提交里新增或改过的 blob
# gate-cell: stage-location-before-cd 门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname "$0" 找旁边的文件
# gate-cell: heavy-thread-defaults 脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨
# gate-cell: hardcoded-quota 脚本、门禁与钩子里不定内存上限与台数（上限、机器由 task-server 按记录定），排除表指得到、只缩不涨
# gate-cell: function-units-two-implementations-agree 函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）原样与产品行为视图两趟：dump 逐单元相同，登记入口的闭包与键逐个相同
# gate-cell: mutation-list-items 变异门禁的 --list-items 在被判的仓上跑通：退 0、报出键对不上的条数，没有「没算出来」、回溯与 PicklingError
# gate-cell: break-switches-sample-only 门禁阶段读弄坏开关都经共用库那一判（开关只在样本根上认），共用库的 stage_cells_run 开头调它
# gate-cell: entry-scripts 登记表 entry-scripts.tsv 里的每份入口脚本都带 --list-items、--selftest 与带齐字段的每项一行（或末行总计）
# gate-cell: break-switch-headers 研究脚本与钩子读了弄坏开关的，文件头开关表三样写全（名字与值、弄坏哪一处、点名的项），存量豁免表指得到、只缩不涨
# gate-cell: stage-break-switch-replay 门禁阶段文件头的每条弄坏开关逐条设上、经 stage-selftest.sh 喂样本，判错的样本与点名的相等
# gate-cell: break-switch-outside-selftest 研究脚本与钩子读了弄坏开关的，平常跑时设上要被拒并点名只在自证里认，存量待改表指得到、只缩不涨
# gate-cell: break-switch-replay 登记表里的脚本与钩子，文件头每条弄坏开关逐条设上跑它的 --selftest，点名的格与实际红的格逐格相同
# gate-cell: kb-directories-write-scope .claude/kb-directories 登记的每个 kb 目录，下一层与子目录里的 .md 都在写回员（kb-scribe）的写范围里
# gate-cell: prune-server-selftest 判定库服务 prune-server/ 的自证过：编服务二进制、起 --selftest，末行 failed=0、cells 大于 0、退出码对得上（目录不在退 77）
# gate-cell: shared-data-root-literals 脚本、钩子、门禁与服务不经 ~ 与 $HOME 取数据根下共享的那几项（经 lib_work_root.py 的 shared_data_root() 取），排除表指得到、只缩不涨
# gate-cell: cp-preserve-mode 项目脚本、钩子、门禁、样本根的搭建脚本与服务里的 cp 不带保留权限的选项（-a、-p、--archive、不带值或带 mode / all 的 --preserve）
# gate-cell: peer-launch-direct 项目脚本与门禁不自己起 multi-host-run.sh --peer 往第二台起活（照服务的分片协议，派哪台由服务定）
# gate-cell: retired-names 退役名单 retired-names.tsv 里的名字在脚本、门禁、钩子、crates 与服务里一处都没有（不留「设了就拒」的放行；搬迁登记表 path-moves.tsv 记的是旧路径，不扫）
# gate-cell: verdict-library-code 登记表 verdict-library-scripts.tsv 写了判定库的行第三列都是 prune-server，点名的脚本里没有取不到库照跑的分支、prune-server 行没有本地库读写
# gate-cell: temporary-service-runners runner 表里套了临时服务包装的自证，与要临时服务的表 selftest-temporary-service.tsv 里登记成 required 的两边对齐（裸写、缺条、包装了表里没有的都点名；own-server 不要求包装）
# gate-cell: temporary-service-wrapper-form 要临时服务的自证的整条写法在七处逐字相同（登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、包装前缀常量）、包装的解析收得下、整条里没有带尖括号占位的 --server-binary
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头），这里只登记格、解析项目根。
#
# 判别力：fixtures/code-tooling.sh/ 下每一格至少一红一绿，每份样本根放 .gate-cells 只点名它判的那一格：
#   research-script-selftests          runner-red、runner-green、runner-guard-red、runner-guard-green、runner-unit-key、runner-plan-exit-red、runner-plan-sentinel-red、runner-red-over-green、
#                                      runner-not-run-green、runner-all-not-run、runner-recorded-before-the-run-ends、
#                                      runner-bare-temporary-service-red、runner-plan-unavailable、runner-item-picked、runner-item-unknown
#   research-script-selftest-coverage  red、green、coverage-claims-red、coverage-claims-green、coverage-nested-red、coverage-suffix-red
#   reuse-selftests                    reuse-selftests-red、reuse-selftests-green
#   rules-manifest                     rules-manifest-red、rules-manifest-green、rules-manifest-read-once-red、rules-manifest-read-once-green
#   rules-named-by-definitions         rules-named-by-definitions-red、rules-named-by-definitions-green
#   stage-owners                       stage-owners-red、stage-owners-green
#   agent-write-scope                  agent-write-scope-red、agent-write-scope-multiedit-red、agent-write-scope-timer-red、agent-write-scope-run-scope-red、
#                                      agent-write-scope-green
#   agent-definition-shape             agent-definition-shape-model-red、agent-definition-shape-tools-red、agent-definition-shape-agent-tool-red、agent-definition-shape-sections-red、
#                                      agent-definition-shape-rule-copy-red、agent-definition-shape-table-red、agent-definition-shape-green
#   change-range-single-source         change-range-single-source-red、change-range-single-source-green、change-range-single-source-path-moves-red、change-range-single-source-path-moves-green、
#                                      change-range-single-source-path-moves-evidence-red、change-range-single-source-path-moves-evidence-green、
#                                      change-range-single-source-path-moves-committed-green、change-range-single-source-path-moves-base-row-green
#   research-gate-lint                 research-gate-lint-red、research-gate-lint-green、research-gate-lint-arithmetic-shift-red、research-gate-lint-pending-outside-green、research-gate-lint-pending-stale-red
#   fixture-claims                     fixture-claims-red、fixture-claims-unnumbered-red、fixture-claims-cells-red、fixture-claims-unmarked-red、
#                                      fixture-claims-marker-malformed-red、fixture-claims-none-with-directory-red、fixture-claims-exempt-red、fixture-claims-green
#   fixture-registrations-excluded     fixture-registrations-excluded-red、fixture-registrations-excluded-stale-red、fixture-registrations-excluded-green
#   kb-registry                        kb-registry-red、kb-registry-green
#   gate-structure                     gate-structure-red、gate-structure-green
#   stage-order                        stage-order-red、stage-order-green
#   directory-layout                   directory-layout-red、directory-layout-green
#   committed-file-size                committed-file-size-red、committed-file-size-unpushed-red、committed-file-size-detached-red、committed-file-size-green、committed-file-size-no-upstream-green
#   stage-location-before-cd           stage-location-before-cd-red、stage-location-before-cd-green
#   heavy-thread-defaults              heavy-thread-defaults-red、heavy-thread-defaults-cfg-test-red、heavy-thread-defaults-capped-elsewhere-red、heavy-thread-defaults-quoted-name-red、heavy-thread-defaults-per-line-red、heavy-thread-defaults-wide-cost-red、heavy-thread-defaults-var-source-red、heavy-thread-defaults-var-source-forms-red、heavy-thread-defaults-per-line-green、heavy-thread-defaults-var-source-green、heavy-thread-defaults-green、heavy-thread-defaults-moved-red、heavy-thread-defaults-moved-green
#   hardcoded-quota                    hardcoded-quota-red、hardcoded-quota-green
#   function-units-two-implementations-agree  function-units-two-implementations-agree-green、-dump-red、-closure-red、-key-red、-product-view-red、-data-directory-green、-doctest-green、-cfg-one-side-red（Rust 那一侧是录好的输出，不编、不起 unit-graph）
#   mutation-list-items                mutation-list-items-red、mutation-list-items-green（setup.sh 现写小工作区，--list-items 只起 git 与 rustc -vV）
#   break-switches-sample-only         break-switches-sample-only-red、break-switches-sample-only-green
#   entry-scripts                      entry-scripts-red、entry-scripts-green
#   break-switch-headers               break-switch-headers-red、break-switch-headers-green
#   stage-break-switch-replay          stage-break-switch-replay-red、stage-break-switch-replay-green、stage-break-switch-replay-progress-green（样本里的小阶段只读一份文件，进度那份另先睡 2 秒，经这一道所在仓的 stage-selftest.sh 喂）、
#                                      stage-break-switch-replay-reuse-three-questions、stage-break-switch-replay-samples-in-key、stage-break-switch-replay-red-over-green、
#                                      stage-break-switch-replay-item-green、stage-break-switch-replay-item-unknown（按条复用与逐项点名，setup.sh 都 git init）
#   break-switch-outside-selftest      break-switch-outside-selftest-red、break-switch-outside-selftest-green、break-switch-outside-selftest-guard-combined-red、
#                                      break-switch-outside-selftest-guard-combined-green
#   break-switch-replay                break-switch-replay-red、break-switch-replay-unnamed-red、break-switch-replay-undecodable-red、break-switch-replay-green、break-switch-replay-slow-baseline-green、
#                                      break-switch-replay-temporary-service-green、break-switch-replay-progress-green、
#                                      break-switch-replay-reuse-three-questions、break-switch-replay-named-in-key、break-switch-replay-red-over-green、
#                                      break-switch-replay-item-co-run-green、break-switch-replay-item-unknown（按条复用与逐项点名，setup.sh 都 git init）
#   kb-directories-write-scope         kb-directories-write-scope-red、kb-directories-write-scope-green
#   prune-server-selftest              prune-server-selftest-green、prune-server-selftest-absent-green、prune-server-selftest-failed-red、prune-server-selftest-empty-red、prune-server-selftest-no-summary-red、prune-server-selftest-exit-red（假的自证是样本根里的 .prune-server-selftest-program，不编）
#   shared-data-root-literals          shared-data-root-literals-red、shared-data-root-literals-data-root-red、shared-data-root-literals-green
#   cp-preserve-mode                   cp-preserve-mode-red、cp-preserve-mode-green
#   peer-launch-direct                 peer-launch-direct-red、peer-launch-direct-green
#   retired-names                      retired-names-red、retired-names-green
#   verdict-library-code               verdict-library-code-red、verdict-library-code-green
#   temporary-service-runners          temporary-service-runners-red、temporary-service-runners-green（样本根自己的要临时服务的表与 .selftest-runners，不起自证）
#   temporary-service-wrapper-form     temporary-service-wrapper-form-green、temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red、temporary-service-wrapper-form-place-missing-red（样本根放齐七处里的文件，不起自证）
#   （不属于哪一格）                   sample-root-bare-red（判的根是样本却不在临时 prune-server 底下：退 2、一格不判；样本根放 .sample-environment-bare 造出裸跑的样子，其余样本都是在临时服务底下被喂的，它们就是这一判的绿侧；写法在文件末尾 code_tooling_sample_service_check 上方）
# 每格的样本放了什么写在那一格的注释里。各份绿样本里 vm-bench.sh、stage-owners.tsv、定义、settings.json 这副架子是并道时为八格全跑补的，留着不碍事。
# 弄坏开关（样本自检时设进环境；每条一行「开关=值  弄坏哪一处」，下面逐行点名打开后判错的样本，fixtures/code-tooling.sh/ 下的目录名；
# 这一道的 stage-break-switch-replay 格逐条复跑，比点名的与实测判错的。每格的样本放了什么、红在哪一句，写在那一格的注释里）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0（共用库别的项见那份文件头）
#     agent-write-scope-multiedit-red、agent-write-scope-red、agent-write-scope-run-scope-red、agent-write-scope-timer-red
#     break-switch-headers-red、break-switch-outside-selftest-red、break-switch-replay-red
#     break-switch-replay-undecodable-red、break-switch-replay-unnamed-red、break-switches-sample-only-red、change-range-single-source-path-moves-evidence-red
#     change-range-single-source-path-moves-red、change-range-single-source-red、committed-file-size-detached-red、committed-file-size-red
#     committed-file-size-unpushed-red、coverage-claims-red、coverage-nested-red、coverage-suffix-red
#     directory-layout-red、entry-scripts-red、fixture-claims-cells-red、fixture-claims-exempt-red
#     fixture-claims-marker-malformed-red、fixture-claims-none-with-directory-red、fixture-claims-red、fixture-claims-unmarked-red
#     fixture-claims-unnumbered-red、fixture-registrations-excluded-red、fixture-registrations-excluded-stale-red、function-units-two-implementations-agree-cfg-one-side-red、function-units-two-implementations-agree-closure-red
#     function-units-two-implementations-agree-dump-red、function-units-two-implementations-agree-key-red、function-units-two-implementations-agree-product-view-red
#     gate-structure-red、heavy-thread-defaults-capped-elsewhere-red
#     heavy-thread-defaults-cfg-test-red、heavy-thread-defaults-moved-red、heavy-thread-defaults-per-line-red、heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red、heavy-thread-defaults-var-source-forms-red、heavy-thread-defaults-var-source-red、heavy-thread-defaults-wide-cost-red
#     hardcoded-quota-red
#     kb-directories-write-scope-red、kb-registry-red
#     mutation-list-items-red、red、research-gate-lint-arithmetic-shift-red、research-gate-lint-pending-stale-red
#     research-gate-lint-red、reuse-selftests-red、rules-manifest-red、runner-guard-red
#     runner-plan-exit-red、runner-plan-sentinel-red、runner-red、runner-red-over-green
#     stage-break-switch-replay-red、stage-location-before-cd-red、stage-order-red、stage-owners-red
#     agent-definition-shape-agent-tool-red、agent-definition-shape-model-red、agent-definition-shape-rule-copy-red、agent-definition-shape-sections-red
#     agent-definition-shape-table-red、agent-definition-shape-tools-red、break-switch-outside-selftest-guard-combined-red
#     rules-manifest-read-once-red
#     prune-server-selftest-empty-red、prune-server-selftest-exit-red、prune-server-selftest-failed-red、prune-server-selftest-no-summary-red
#     shared-data-root-literals-red、shared-data-root-literals-data-root-red
#     cp-preserve-mode-red
#     peer-launch-direct-red
#     retired-names-red、verdict-library-code-red
#     rules-named-by-definitions-red
#     runner-bare-temporary-service-red、temporary-service-runners-red
#     temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-place-missing-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red
#     break-switch-replay-red-over-green、stage-break-switch-replay-red-over-green
#   CODE_TOOLING_BREAK=runner-exit-swallowed  research-script-selftests 回读时把每条的退出码都当 0
#     runner-all-not-run
#     runner-guard-red
#     runner-not-run-green
#     runner-red
#     runner-red-over-green
#   CODE_TOOLING_BREAK=selftest-not-run-as-pass  research-script-selftests 把自证退的 77（本次未跑）当退 0、照绿记
#     runner-all-not-run
#     runner-not-run-green
#   CODE_TOOLING_BREAK=guard-skipped  research-script-selftests 每条不经 selftest-process-guard.py、直接起（判定照「盯了、没扫到」写）
#     runner-guard-green
#     runner-guard-red
#   CODE_TOOLING_BREAK=selftest-key-whole-runner-table  research-script-selftests 每一条自证的键另加表里每一条自证的脚本（整张表一份输入）
#     runner-unit-key
#   CODE_TOOLING_BREAK=selftest-judge-whole-guard  research-script-selftests 的判法取整份 selftest-process-guard.py（每一条都拒算）
#     runner-recorded-before-the-run-ends
#     runner-unit-key
#     runner-red-over-green（输出里找不到「· 按条复用：表里 2 条自证，现判 2、复用 0、拒算 0」：两条都拒算）
#   CODE_TOOLING_BREAK=red-not-recorded  research-script-selftests 判红时不交 verdict-record --red-names（改前的样子：只记绿，判红的不记红、不撤同一把键原有的绿）
#     runner-red-over-green
#   CODE_TOOLING_BREAK=selftest-recorded-at-end  research-script-selftests 每条跑完不当场记，只在整格收尾一次记（改前的样子：中途被停，判完的全丢）
#     runner-recorded-before-the-run-ends
#   CODE_TOOLING_BREAK=bare-temporary-service-selftest-accepted  research-script-selftests 不核只在临时 prune-server 上跑的自证写没写成包装写法（改前的样子：裸写照起、退 2）
#     runner-bare-temporary-service-red
#   CODE_TOOLING_BREAK=temporary-service-table-ignored  认「这份自证要不要临时服务」退回看源码里有没有 selftest_service_problem、不读要临时服务的表（改前的样子：own-server 的被要求包装，表里 required 而源码没写那几个字的认不出；四个格共用 temporary_service_is_required）
#     break-switch-replay-temporary-service-green（样本脚本源码里不写那几个字，不包起就退 2）、temporary-service-runners-green（own-server 裸写那份被点名）、temporary-service-runners-red（「该写成」那条 want 找不到）
#   CODE_TOOLING_BREAK=temporary-service-alignment-unchecked  temporary-service-runners 不判 runner 表与要临时服务的表两边对不对得上（改前的样子：只有 research-script-selftests 格看裸写）
#     temporary-service-runners-red
#   CODE_TOOLING_BREAK=temporary-service-form-unchecked  temporary-service-wrapper-form 有对不上也不判红（改前的样子：没有检查绑住要临时服务的自证的整条写法）
#     temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-place-missing-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red
#   CODE_TOOLING_BREAK=stage-replay-binary-not-passed  stage-break-switch-replay 开跑前不取 prune-server 二进制、套包装起 stage-selftest.sh 时不带 --server-binary（改前的样子：包装不现编，退 2、每条开关都没跑成）
#     stage-break-switch-replay-green、stage-break-switch-replay-item-green、stage-break-switch-replay-progress-green、stage-break-switch-replay-red
#     stage-break-switch-replay-red-over-green、stage-break-switch-replay-reuse-three-questions、stage-break-switch-replay-samples-in-key
#   CODE_TOOLING_BREAK=sample-service-unchecked  判的根是样本时不核在不在临时 prune-server 底下（改前的样子：裸跑也判，样本的判定写进真服务）
#     sample-root-bare-red
#   ADMISSION_BREAK=plan-unavailable-as-failure  归 scripts/gate/admission.py：research-script-selftests 计划退 75 照复用失灵判红、整道退 1（改前的样子）
#     runner-plan-unavailable
#   ADMISSION_BREAK=plan-failure-not-red  归 scripts/gate/admission.py：research-script-selftests 按条复用失灵不判红（退回全部现判）
#     runner-plan-exit-red
#     runner-plan-sentinel-red
#   CODE_TOOLING_BREAK=reuse-selftest-line-unchecked  reuse-selftests 不看规范输出那一行、自证退 0 就算过（折成 REUSE_CHECKS_BREAK=audit-line-unchecked）
#     reuse-selftests-red
#   CODE_TOOLING_BREAK=claims-any-file  research-script-selftest-coverage 退回「全文出现 --selftest 就算声称有」
#     coverage-claims-green
#   CODE_TOOLING_BREAK=selftest-coverage-top-level-only  research-script-selftest-coverage 只扫 scripts/<类>/ 那一层、不进装置目录
#     coverage-claims-green
#     coverage-nested-red
#     green
#   CODE_TOOLING_BREAK=selftest-coverage-name-substring  research-script-selftest-coverage 按文件名当子串认、左边不设边界
#     coverage-suffix-red
#   CODE_TOOLING_BREAK=selftest-coverage-trailing-comments-kept  research-script-selftest-coverage 不去行尾注释（改前的样子：行尾注释里的 --selftest 当实现了）
#     coverage-claims-green
#   CODE_TOOLING_BREAK=numbered-stages-only  research-script-selftest-coverage 与 fixture-claims 只认两位数开头的阶段
#     coverage-claims-green
#     fixture-claims-cells-red
#     fixture-claims-green
#     fixture-claims-unnumbered-red
#   CODE_TOOLING_BREAK=tool-version-dates-judged  fixture-claims 第 ④ 条把工具版本串「(<提交的十六进制> <年-月-日>)」里的日期也当样本正文的日期判（改前的样子）
#     fixture-claims-green
#   CODE_TOOLING_BREAK=bare-red-green-only  fixture-claims 退回只认裸 red / green 样本目录、不按格数红绿
#     fixture-claims-cells-red
#     fixture-claims-green
#   CODE_TOOLING_BREAK=claims-by-prose  fixture-claims 退回按散文子串认声称（标记一概不看、不对豁免表）
#     fixture-claims-exempt-red
#     fixture-claims-green
#     fixture-claims-marker-malformed-red
#     fixture-claims-none-with-directory-red
#     fixture-claims-unmarked-red
#     fixture-claims-unnumbered-red
#   CODE_TOOLING_BREAK=unmarked-stage-passes  fixture-claims 对没写标记的阶段直接跳过、照记通过
#     fixture-claims-unmarked-red
#   CODE_TOOLING_BREAK=owner-rows-ignored  stage-owners 不判「--list 出的每一格都有一行」
#     stage-owners-red
#   CODE_TOOLING_BREAK=structure-exit-ignored  gate-structure 不看 gate-structure-check.py 的退出码
#     gate-structure-red
#   CODE_TOOLING_BREAK=stage-order-ignored  stage-order 不比次序
#     stage-order-red
#   CODE_TOOLING_BREAK=safety-top-level-only  research-gate-lint 的进程安全那一路只扫 scripts/ 顶层、不交 scripts/<类>/ 子目录
#     research-gate-lint-green
#     research-gate-lint-pending-outside-green
#     research-gate-lint-pending-stale-red
#     research-gate-lint-red
#   CODE_TOOLING_BREAK=bare-cd-unchecked  research-gate-lint 不判裸 cd
#     research-gate-lint-arithmetic-shift-red
#     research-gate-lint-green
#     research-gate-lint-pending-outside-green
#     research-gate-lint-red
#   CODE_TOOLING_BREAK=heredoc-in-arithmetic  research-gate-lint 裸 cd 那一项把算术里的 << 也当 heredoc 起始（改前的认法）
#     research-gate-lint-arithmetic-shift-red
#   CODE_TOOLING_BREAK=syntax-unchecked  research-gate-lint 不判语法（不跑 bash -n 与 python 的 compile）
#     research-gate-lint-green
#     research-gate-lint-pending-outside-green
#     research-gate-lint-red
#   CODE_TOOLING_BREAK=dirname-after-cd-ignored  stage-location-before-cd 不判 cd 之后的 dirname
#     stage-location-before-cd-red
#   CODE_TOOLING_BREAK=registrations-unscanned  fixture-registrations-excluded 把样本里的登记位一概认成没有
#     fixture-registrations-excluded-green
#     fixture-registrations-excluded-red
#   CODE_TOOLING_BREAK=committed-size-tracked-half-skipped  committed-file-size 甲（跟踪的文件）不判（折成 scripts/gate/large-file-check.py 的 LARGE_FILE_CHECK_BREAK 去掉 committed-size- 的同名项，下面四条同）
#     committed-file-size-green
#     committed-file-size-no-upstream-green
#     committed-file-size-red
#   CODE_TOOLING_BREAK=committed-size-unpushed-half-skipped  committed-file-size 乙（还没推的提交）不判
#     committed-file-size-detached-red
#     committed-file-size-green
#     committed-file-size-no-upstream-green
#     committed-file-size-unpushed-red
#   CODE_TOOLING_BREAK=committed-size-limit-inclusive  committed-file-size 恰好等于上限的也算超
#     committed-file-size-green
#     committed-file-size-no-upstream-green
#   CODE_TOOLING_BREAK=committed-size-no-upstream-as-pass  committed-file-size 没有上游时乙当成判过（查了 0 笔）
#     committed-file-size-no-upstream-green
#   CODE_TOOLING_BREAK=committed-size-detached-unresolved  committed-file-size 游离的 HEAD 一律当没有上游
#     committed-file-size-detached-red
#   CODE_TOOLING_BREAK=thread-default-strings-scanned  heavy-thread-defaults 把 python 与 rust 的字面量与文档串照代码扫
#     heavy-thread-defaults-green
#     heavy-thread-defaults-red
#     heavy-thread-defaults-wide-cost-red
#   CODE_TOOLING_BREAK=thread-default-half-unchecked  heavy-thread-defaults 不认核数的一半
#     heavy-thread-defaults-red
#   CODE_TOOLING_BREAK=thread-default-exempt-unchecked  heavy-thread-defaults 的排除表不核文件在不在、锚串命不命中
#     heavy-thread-defaults-red
#   CODE_TOOLING_BREAK=thread-default-exempt-growth-unchecked  heavy-thread-defaults 的排除表不比基准
#     heavy-thread-defaults-moved-green
#     heavy-thread-defaults-moved-red
#     heavy-thread-defaults-red
#   CODE_TOOLING_BREAK=thread-default-timeout-unchecked  heavy-thread-defaults 把超时回退值也按线程数判
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-own-variable-unchecked  heavy-thread-defaults 不判装置读自己的线程变量
#     heavy-thread-defaults-capped-elsewhere-red
#     heavy-thread-defaults-cfg-test-red
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red
#     heavy-thread-defaults-wide-cost-red
#   CODE_TOOLING_BREAK=thread-default-capped-list-ignored  heavy-thread-defaults 读了 capped.sh 的清单不认，每个线程变量都算清单外
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-shell-local-unchecked  heavy-thread-defaults 把 shell 里文件自己赋了值的变量也算读环境
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-red
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-test-module-unskipped  heavy-thread-defaults 把 rust 测试模块里点名的变量也算
#     heavy-thread-defaults-cfg-test-red
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-cfg-test-without-mod  heavy-thread-defaults 遇到行首 #[cfg(test)] 不看后面是不是 mod，一律往后都不算
#     heavy-thread-defaults-cfg-test-red
#     heavy-thread-defaults-quoted-name-red
#   CODE_TOOLING_BREAK=thread-default-fallback-by-file  heavy-thread-defaults 整份文件的代码里出现回退字样就算回退了
#     heavy-thread-defaults-capped-elsewhere-red
#   CODE_TOOLING_BREAK=thread-default-statement-single-line  heavy-thread-defaults 读线程变量的那条语句只算那一行
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-subscript-write-counted  heavy-thread-defaults 把 ["X"] = 值 也算读
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-shell-inline-assignment-unseen  heavy-thread-defaults 在 shell 里只认行首的赋值
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-quoted-name-needs-environment-read  heavy-thread-defaults 的引号里的名字退回只在读环境变量的写法里才算读
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red
#     heavy-thread-defaults-wide-cost-red
#   CODE_TOOLING_BREAK=thread-default-conditional-assignment-unpinned  heavy-thread-defaults 不认 `|| X=<数>`、`&& X=<数>`
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-var-source-forms-red
#     heavy-thread-defaults-var-source-red
#     heavy-thread-defaults-wide-cost-red
#   CODE_TOOLING_BREAK=thread-default-external-mod-skipped  heavy-thread-defaults 把 `#[cfg(test)] mod tests;` 也当测试模块，往后都不算
#     heavy-thread-defaults-quoted-name-red
#   CODE_TOOLING_BREAK=thread-default-exempt-by-file  heavy-thread-defaults 的排除表退回按文件并组
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red
#   CODE_TOOLING_BREAK=thread-default-conditional-assignment-literal-only  heavy-thread-defaults 的 `|| X=<数>` 只认字面正整数，不认 `$变量`、`$((算术))`
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-var-source-forms-red
#     heavy-thread-defaults-var-source-red
#   CODE_TOOLING_BREAK=thread-default-printenv-unseen  heavy-thread-defaults 不认 shell 里的 `printenv NAME`
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#   CODE_TOOLING_BREAK=thread-default-conditional-variable-source-ignored  heavy-thread-defaults 的 `|| X=$变量` 不看变量在文件里怎么赋的值，一律算钉数
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-registry-substring-accepted  heavy-thread-defaults 的 registry_sourced_variables ① 退回改前：右边含登记位读法、去掉之后没有别的数字就算
#     heavy-thread-defaults-var-source-forms-red
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-defaulted-variable-accepted  heavy-thread-defaults 不看 ${名:-…}、${名:=…}
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=thread-default-other-assignment-forms-unseen  heavy-thread-defaults 不看别的赋值写法，只认「名=值」
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=thread-default-sourcing-ignored  heavy-thread-defaults 不看文件 source 了别的文件
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=quota-strings-scanned  hardcoded-quota 把 python 与 rust 的字面量与文档串照代码扫
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-sentence-samples-counted  hardcoded-quota 把引号里整句的样本也当跑法
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-memory-cap-option-unchecked  hardcoded-quota 不认 --memory-cap <常量>
#     hardcoded-quota-green
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-memory-variable-unchecked  hardcoded-quota 不认写死的内存上限默认
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-host-count-unchecked  hardcoded-quota 不认写死的台数
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-host-computed-unseen  hardcoded-quota 台数变量另有现算的赋值也照判它的整数初值
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-auto-allowed  hardcoded-quota 照改前不把 auto 当写死的上限
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-exempt-unchecked  hardcoded-quota 的排除表不核文件在不在、锚串命不命中
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-exempt-growth-unchecked  hardcoded-quota 的排除表不比基准
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=path-moves-unchecked  change-range-single-source 不判搬迁登记表（写法、出处在不在、此刻生效的行串没串、有没有搬迁实据）
#     change-range-single-source-path-moves-base-row-green
#     change-range-single-source-path-moves-committed-green
#     change-range-single-source-path-moves-evidence-green
#     change-range-single-source-path-moves-evidence-red
#     change-range-single-source-path-moves-green
#     change-range-single-source-path-moves-red
#   LAG_BASELINE_BREAK=move-evidence-unchecked  归共用库 lib-lag-baseline.py（下面七条同）：不核此刻生效的行串没串、有没有搬迁实据
#     change-range-single-source-path-moves-base-row-green
#     change-range-single-source-path-moves-committed-green
#     change-range-single-source-path-moves-evidence-green
#     change-range-single-source-path-moves-evidence-red
#     change-range-single-source-path-moves-green
#   LAG_BASELINE_BREAK=window-ignored  每一行都当此刻生效（去掉生效窗口）
#     change-range-single-source-path-moves-base-row-green
#     change-range-single-source-path-moves-committed-green
#     change-range-single-source-path-moves-evidence-green
#     change-range-single-source-path-moves-evidence-red
#   LAG_BASELINE_BREAK=occupied-unchecked  不看新前缀在基准里有没有文件（去掉占用核对）
#     change-range-single-source-path-moves-evidence-red
#   LAG_BASELINE_BREAK=chain-allowed  不核此刻生效的行串没串（放开串）
#     change-range-single-source-path-moves-evidence-red
#   LAG_BASELINE_BREAK=untracked-landing-accepted  落点只要在磁盘上，不核被 git 跟踪
#     change-range-single-source-path-moves-evidence-red
#   LAG_BASELINE_BREAK=idle-new-row-unchecked  不在基准登记表里的新行不生效也不判红
#     change-range-single-source-path-moves-evidence-red
#   LAG_BASELINE_BREAK=base-row-stays-effective  基准那一版登记表里的老行照样生效
#     change-range-single-source-path-moves-base-row-green
#     change-range-single-source-path-moves-committed-green
#     change-range-single-source-path-moves-evidence-green
#   LAG_BASELINE_BREAK=path-moves-ignored  不读搬迁登记表，基准那一版不换前缀、不按搬迁之前的路径取
#     heavy-thread-defaults-moved-green
#   CODE_TOOLING_BREAK=function-units-dump-unchecked  function-units-two-implementations-agree 不比 dump
#     function-units-two-implementations-agree-dump-red
#     function-units-two-implementations-agree-product-view-red
#   CODE_TOOLING_BREAK=function-units-closures-unchecked  function-units-two-implementations-agree 不比闭包与键
#     function-units-two-implementations-agree-cfg-one-side-red
#     function-units-two-implementations-agree-closure-red
#     function-units-two-implementations-agree-dump-red
#     function-units-two-implementations-agree-key-red
#     function-units-two-implementations-agree-product-view-red
#   CODE_TOOLING_BREAK=function-units-keys-unchecked  function-units-two-implementations-agree 闭包相同就不比键
#     function-units-two-implementations-agree-key-red
#   CODE_TOOLING_BREAK=function-units-product-view-unchecked  function-units-two-implementations-agree 不跑产品行为视图那一趟
#     function-units-two-implementations-agree-data-directory-green
#     function-units-two-implementations-agree-doctest-green
#     function-units-two-implementations-agree-green
#     function-units-two-implementations-agree-product-view-red
#   CODE_TOOLING_BREAK=function-units-data-kind-ignored  function-units-two-implementations-agree 闭包的数据行一律写 data（读目录的也不写 data-directory，改前的样子）
#     function-units-two-implementations-agree-data-directory-green
#   CODE_TOOLING_BREAK=function-units-doctest-target-named  function-units-two-implementations-agree 给文档测试入口写 --target doctest:<名>（改前的样子，unit-graph 不认）
#     function-units-two-implementations-agree-doctest-green
#   CODE_TOOLING_BREAK=list-items-exit-only  mutation-list-items 只看 --list-items 的退出码
#     mutation-list-items-red
#   CODE_TOOLING_BREAK=break-switch-guard-unchecked  break-switches-sample-only 不判
#     break-switches-sample-only-red
#   CODE_TOOLING_BREAK=entry-item-fields-unchecked  entry-scripts 不核每项那一句的字段
#     entry-scripts-red
#   CODE_TOOLING_BREAK=switch-items-unchecked  break-switch-headers 不判点名的项
#     break-switch-headers-red
#   CODE_TOOLING_BREAK=switch-replay-named-unchecked  stage-break-switch-replay 不比点名的与实测判错的
#     stage-break-switch-replay-red
#     stage-break-switch-replay-red-over-green
#   CODE_TOOLING_BREAK=stage-replay-samples-unkeyed  stage-break-switch-replay 每条的键不带它喂的样本目录的内容（改错的样子：样本改了照复用）
#     stage-break-switch-replay-samples-in-key
#   CODE_TOOLING_BREAK=stage-replay-key-whole-gate-directory  stage-break-switch-replay 每条的键另加有样本目录的每一道阶段的整份文件（改一道、注释也算，全部作废）
#     stage-break-switch-replay-reuse-three-questions
#     stage-break-switch-replay-samples-in-key
#   CODE_TOOLING_BREAK=replay-plan-ignored  break-switch-replay 与 stage-break-switch-replay 不按计划跳过复用的条，点到的全部起（改前的样子：每趟整格重跑）
#     break-switch-replay-reuse-three-questions
#     break-switch-replay-named-in-key
#     stage-break-switch-replay-reuse-three-questions
#     stage-break-switch-replay-samples-in-key
#   CODE_TOOLING_BREAK=replay-red-not-recorded  break-switch-replay 与 stage-break-switch-replay 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     break-switch-replay-red-over-green
#     stage-break-switch-replay-red-over-green
#   CODE_TOOLING_BREAK=items-ignored  逐项点名不起作用，点到的格照跑整格
#     runner-item-picked
#     break-switch-replay-item-co-run-green
#     stage-break-switch-replay-item-green
#   CODE_TOOLING_BREAK=item-unknown-accepted  写错的项悄悄丢掉、不退 2，一项都没认出就照跑整格
#     runner-item-unknown
#     break-switch-replay-item-unknown
#     stage-break-switch-replay-item-unknown
#   CODE_TOOLING_BREAK=outside-selftest-unchecked  break-switch-outside-selftest 把探针的结果一概当守住
#     break-switch-outside-selftest-guard-combined-red
#     break-switch-outside-selftest-red
#   CODE_TOOLING_BREAK=guard-combination-unprobed  break-switch-outside-selftest 不另起「关守卫项,第一个值」那一次探针（改前的样子）
#     break-switch-outside-selftest-guard-combined-green
#     break-switch-outside-selftest-guard-combined-red
#   CODE_TOOLING_BREAK=replay-items-unchecked  break-switch-replay 不比点名的项
#     break-switch-replay-red
#     break-switch-replay-undecodable-red
#   CODE_TOOLING_BREAK=replay-output-strict-decode  break-switch-replay 退回按文本收自证输出、不容坏字节（改前的收法）
#     break-switch-replay-undecodable-red
#   CODE_TOOLING_BREAK=replay-unnamed-rows-unchecked  break-switch-replay 不查多红（失败行比点名的多不判）
#     break-switch-replay-unnamed-red
#   CODE_TOOLING_BREAK=replay-baseline-unsubtracted  break-switch-replay 不起基准趟、不减自证本来就有的失败行
#     break-switch-replay-green
#     break-switch-replay-reuse-three-questions（基准趟那一条不判、不记库，下一趟它照现判：三问 ① 红）
#     break-switch-replay-named-in-key
#   CODE_TOOLING_BREAK=replay-summary-rows-required  break-switch-replay 汇总行也要点名
#     break-switch-replay-green
#     break-switch-replay-item-co-run-green
#   CODE_TOOLING_BREAK=replay-carriage-return-kept  break-switch-replay 把单独的 \r 留在行里
#     break-switch-replay-unnamed-red
#   CODE_TOOLING_BREAK=replay-bad-bytes-first  break-switch-replay 有坏字节就只记坏字节、不再比点名（改前的次序）
#     break-switch-replay-undecodable-red
#   CODE_TOOLING_BREAK=replay-baseline-shares-switch-limit  break-switch-replay 的基准趟也照每条开关的限时跑、不读基准趟那一列（改前的样子）
#     break-switch-replay-slow-baseline-green
#   CODE_TOOLING_BREAK=replay-temporary-service-unwrapped  break-switch-replay 起只在临时 prune-server 上跑的自证也不包一层（改前的样子：裸跑退 2，基准趟就没过）
#     break-switch-replay-temporary-service-green
#   CODE_TOOLING_BREAK=replay-progress-silent  break-switch-replay 与 stage-break-switch-replay 不打进度、不起报进度的线程（改前的样子：整格跑完之前日志里一行不涨）
#     break-switch-replay-progress-green
#     stage-break-switch-replay-progress-green
#   CODE_TOOLING_BREAK=replay-key-whole-registry  break-switch-replay 每条的键另加复跑登记表里每一份脚本的整份文件（改一份、注释也算，全部作废）
#     break-switch-replay-reuse-three-questions
#     break-switch-replay-named-in-key
#   CODE_TOOLING_BREAK=replay-named-unkeyed  break-switch-replay 每条开关的键不带那一条点名的项的原文（改错的样子：头注改了照复用）
#     break-switch-replay-named-in-key
#   CODE_TOOLING_BREAK=replay-siblings-not-co-run  break-switch-replay 同一份只起要现判的那几条开关趟、别的不陪跑（改错的样子：汇总行少了陪跑的开关趟）
#     break-switch-replay-item-co-run-green
#   CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked  kb-directories-write-scope 只探登记目录下一层的 .md、不探子目录
#     kb-directories-write-scope-red
#   AGENT_WRITE_SCOPE_BREAK=any-model  agent-write-scope 关掉定义 model 取值那一条
#     agent-write-scope-red
#   AGENT_WRITE_SCOPE_BREAK=multiedit-ignored  agent-write-scope 的写范围闸注册只要求覆盖 Write 与 Edit、不看 MultiEdit
#     agent-write-scope-multiedit-red
#   AGENT_WRITE_SCOPE_BREAK=timer-monitor-ignored  agent-write-scope 的定时唯一办法闸注册不要求覆盖 Monitor
#     agent-write-scope-timer-red
#   AGENT_WRITE_SCOPE_BREAK=run-scope-unchecked  agent-write-scope 不判范围声明闸注册在 Bash 的 matcher 上
#     agent-write-scope-run-scope-red
#   CODE_TOOLING_BREAK=definition-model-unchecked  agent-definition-shape 不判 frontmatter 有没有 model 一行
#     agent-definition-shape-model-red
#   CODE_TOOLING_BREAK=definition-tools-unchecked  agent-definition-shape 不判 frontmatter 有没有 tools 一行
#     agent-definition-shape-tools-red
#   CODE_TOOLING_BREAK=definition-agent-tool-allowed  agent-definition-shape 不判 tools 里有没有 Agent、Task
#     agent-definition-shape-agent-tool-red
#   CODE_TOOLING_BREAK=definition-sections-unchecked  agent-definition-shape 不判输入、写范围、产出、没做什么四节齐不齐
#     agent-definition-shape-sections-red
#   CODE_TOOLING_BREAK=definition-rule-copies-unchecked  agent-definition-shape 不比定义与共用约束里的行与规则文件逐字相同
#     agent-definition-shape-rule-copy-red
#   CODE_TOOLING_BREAK=definition-table-unchecked  agent-definition-shape 不比主 agent 的调度表与定义目录
#     agent-definition-shape-table-red
#   CODE_TOOLING_BREAK=rules-read-once-ignored  rules-manifest 不认 CLAUDE.md 的 read-once 登记（改前的判法）
#     rules-manifest-read-once-green
#     rules-manifest-read-once-red
#   CODE_TOOLING_BREAK=rules-named-read-once-skipped  rules-named-by-definitions 只判 @ 规则，read-once 登记的不判
#     rules-named-by-definitions-green
#     rules-named-by-definitions-red
#   CODE_TOOLING_BREAK=rules-named-by-basename  rules-named-by-definitions 只写文件名、不带目录的也算点名
#     rules-named-by-definitions-red
#   CODE_TOOLING_BREAK=prune-server-failed-ignored  prune-server-selftest 不看末行的 failed= 与退出码
#     prune-server-selftest-exit-red
#     prune-server-selftest-failed-red
#   CODE_TOOLING_BREAK=prune-server-empty-run-accepted  prune-server-selftest 把 cells=0（一格都没跑）也算过
#     prune-server-selftest-empty-red
#   CODE_TOOLING_BREAK=prune-server-last-line-unchecked  prune-server-selftest 不核末行是恰好一行且在最后，没有末行就只看退出码
#     prune-server-selftest-no-summary-red
#   CODE_TOOLING_BREAK=prune-server-exit-unchecked  prune-server-selftest 不核退出码与末行 failed= 对不对得上
#     prune-server-selftest-exit-red
#   CODE_TOOLING_BREAK=shared-literals-prose-scanned  shared-data-root-literals 把带空白的字面量（出路文字、文档串、样本命令）也照代码扫
#     shared-data-root-literals-green
#   CODE_TOOLING_BREAK=shared-literals-join-form-unseen  shared-data-root-literals 不认 expanduser("~") 拼出来的写法
#     shared-data-root-literals-data-root-red、shared-data-root-literals-red
#   CODE_TOOLING_BREAK=shared-literals-braced-home-unseen  shared-data-root-literals 只认 ~/ 与 $HOME/，不认 ${HOME}/ 与 ${HOME:?}/
#     shared-data-root-literals-red
#   CODE_TOOLING_BREAK=shared-literals-per-host-judged  shared-data-root-literals 把按台的那几项（qemu、rocksdb-prebuilt 这类）也判
#     shared-data-root-literals-green
#   CODE_TOOLING_BREAK=shared-literals-definition-file-judged  shared-data-root-literals 连 .claude/hooks/lib_work_root.py 自己也判
#     shared-data-root-literals-green
#   CODE_TOOLING_BREAK=shared-literals-selftest-bodies-judged  shared-data-root-literals 连自证函数体里造的样本也判
#     shared-data-root-literals-green
#   CODE_TOOLING_BREAK=shared-literals-exempt-unchecked  shared-data-root-literals 的排除表不核文件在不在、锚串命不命中
#     shared-data-root-literals-red
#   CODE_TOOLING_BREAK=shared-literals-exempt-growth-unchecked  shared-data-root-literals 的排除表不比基准
#     shared-data-root-literals-red
#   CODE_TOOLING_BREAK=shared-literals-data-root-unseen  shared-data-root-literals 不判数据根本身与名单外的名字
#     shared-data-root-literals-data-root-red
#   CODE_TOOLING_BREAK=shared-literals-run-records-unseen  shared-data-root-literals 不判经 ~ 或 $HOME 取的运行记录（峰值表、崩溃放量的运行目录）
#     shared-data-root-literals-data-root-red
#   CODE_TOOLING_BREAK=cp-preserve-short-cluster-unseen  cp-preserve-mode 只认单写的 -a、-p，不认几个字母合写的（-rp、-pR）
#     cp-preserve-mode-red
#   CODE_TOOLING_BREAK=cp-preserve-python-unseen  cp-preserve-mode 不认 python 的 argv 列表
#     cp-preserve-mode-red
#   CODE_TOOLING_BREAK=cp-preserve-prose-scanned  cp-preserve-mode 把带空白的字面量（出路文字、样本命令串、文档串）也照代码扫
#     cp-preserve-mode-green
#   CODE_TOOLING_BREAK=cp-preserve-fixture-setups-unscanned  cp-preserve-mode 连样本根的搭建脚本 setup.sh 也不扫
#     cp-preserve-mode-red
#     cp-preserve-mode-green（输出里找不到「扫了 3 份（scripts 2、.claude/gate.d 1）」：.claude/gate.d 下那一份是样本根的 setup.sh，不扫了）
#   CODE_TOOLING_BREAK=peer-launch-variable-unseen  peer-launch-direct 只认字面的 multi-host-run.sh，不认变量名与 python 属性
#     peer-launch-direct-red
#   CODE_TOOLING_BREAK=peer-launch-launcher-scanned  peer-launch-direct 连 scripts/hosts/multi-host-run.sh 自己也扫
#     peer-launch-direct-green
#   CODE_TOOLING_BREAK=peer-launch-prose-scanned  peer-launch-direct 把注释与带空白的字面量也照代码扫
#     peer-launch-direct-green
#   CODE_TOOLING_BREAK=retired-names-comments-unscanned  retired-names 不扫注释行
#     retired-names-red
#   CODE_TOOLING_BREAK=retired-moves-table-scanned  retired-names 连搬迁登记表 path-moves.tsv 也扫（旧路径一栏写着退役的名字）
#     retired-names-green
#   CODE_TOOLING_BREAK=library-code-third-column-unchecked  verdict-library-code 不判第三列
#     verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-silent-run-unseen  verdict-library-code 不搜取不到库照跑的写法
#     verdict-library-code-green、verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-local-library-unseen  verdict-library-code 不搜本地库读写的写法
#     verdict-library-code-green、verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-unregistered-rows-scanned  verdict-library-code 连「不算：」「无判定库：」行点名的脚本也搜
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-explanations-scanned  verdict-library-code 连注释与文档串也照判（改前的样子）
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-break-guards-scanned  verdict-library-code 连弄坏开关护着的分支也照判（改前的样子）
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-guard-names-unchecked  verdict-library-code 包着的块不按文件头开关表认开关名，任何带连字符的词都算
#     verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-else-branches-released  verdict-library-code 往上找包着的块时缩进相同的行也算（else 那一支跟着 if 放过）
#     verdict-library-code-red
#
#   bash .claude/gate.d/code-tooling.sh [项目根]                                三十七格都跑
#   bash .claude/gate.d/code-tooling.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-tooling.sh --check <格名>[,<格名>…] [项目根]       只跑点名的格
#   bash .claude/gate.d/code-tooling.sh --list-items [--check <格名>…] [项目根]  逐行打分项的三格可点的项（格名<TAB>项），不起自证、不算键（这一道「逐项点名」一节）
#   bash .claude/gate.d/code-tooling.sh --item <项> [--item …] [项目根]         只跑点名的项，给几次取并集；项写错列出可点的项、退 2
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd -P)"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
# research-script-selftests 格分批的并行度取每台默认重活线程数（heavy_thread_default，用这一道所在仓里那一份）
source "$STAGE_REPOSITORY/scripts/hosts/heavy-threads.sh"
# 开关复跑两格的判法入口（单元键里的 judge，两格文件头「按条复用」）：令牌照字面写，scripts/gate/lib_function_units.py 的 registered_judging_entries 从这里现取、它的自证逐个收闭包
BREAK_SWITCH_REPLAY_JUDGE_TOKEN="judge:.claude/gate.d/$STAGE_NAME::break_switch_replay_judge_code"
STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN="judge:.claude/gate.d/$STAGE_NAME::stage_break_switch_replay_judge_code"

stage_cell research-script-selftests cell_research_script_selftests "runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）" \
  "按红的那条自证给的下一步修被测脚本，再单独跑那一条命令看它转绿；runner 表在这一道的 SELFTEST_RUNNERS"
stage_cell research-script-selftest-coverage cell_research_script_selftest_coverage "scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑" \
  "把没人跑的自证加进这一道的 SELFTEST_RUNNERS；确实不该每轮跑的登记进 NOT_RUN_HERE 并写明为什么；过期的豁免删掉"
stage_cell reuse-selftests cell_reuse_selftests "登记表 verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了" \
  "照逐条的出路改：自证里补 reuse-three-questions 那一格（scripts/gate/lib_reuse_checks.py 的 report_three_questions 打规范输出），或在 .claude/gate.d/verdict-library-scripts.tsv 补、改那一行；判据在 lib_reuse_checks.py 文件头「四」"
stage_cell rules-manifest cell_rules_manifest "项目规则文件与 CLAUDE.md 的 @ 引用加 read-once 登记逐项相等" \
  "补上 CLAUDE.md 里缺的 @.claude/rules/<文件> 引用（写在代码围栏与行内代码之外），或删掉悬空引用；两侧逐项相等"
stage_cell rules-named-by-definitions cell_rules_named_by_definitions "CLAUDE.md 的每条 @ 规则与 read-once 登记的规则，在 .claude/agents/*.md 或 .claude/agent-common.md 里至少被一份按路径点名" \
  "在用得到它的定义「开工先读：」一行或共用约束里按路径点名那条规则（改定义照 .claude/rules/implementation-workflow.md「改 agent 定义与共用约束，走同一条三步」）；确实没有子 agent 要读的，从 CLAUDE.md 去掉之前先问用户"
stage_cell stage-owners cell_stage_owners "阶段归属表与各道门禁的格逐格一致、派给的 agent 有定义，每道阶段认第一个参数当项目根" \
  "照各道门禁的 --list 在 .claude/gate.d/stage-owners.tsv 逐格补行、改名、删行（四列：门禁文件名、格名、判红时派给谁修、为什么）；列不出格的先修 gate-structure 格报的结构；不认项目根的阶段加 ROOT=\"\${1:-…}\" 与 cd \"\$ROOT\""
stage_cell agent-write-scope cell_agent_write_scope "项目钩子注册在要的 matcher 上，写范围表与定义双向一致，定义头齐全，共用模块不另写一份" \
  "照上面逐条的出路改 .claude/settings.json 的 hooks、.claude/hooks/agent-write-scope.tsv 或定义的 frontmatter；共用模块的函数从模块导入，不另写一份"
stage_cell agent-definition-shape cell_agent_definition_shape "定义写明 model 与 tools、tools 不给 Agent 与 Task、四节齐，定义与共用约束不抄规则的长行，主 agent 的调度表与定义两向相等" \
  "照上面逐条的出路改定义的 frontmatter 与二级标题、删掉抄来的规则行换成「开工先读：」指令、补 .claude/main-agent.md「什么时候派哪个 agent」表；判据与阈值在 code-tooling.sh 里这一格函数上方的注释"
stage_cell change-range-single-source cell_change_range_single_source "门禁阶段不自己算 diff 基准，列路径带 quotepath，调共用取法判退出码；基准那一版照登记的搬迁换算，搬迁登记表写对、指得到" \
  "基准与改动范围只经 scripts/gate/changed-paths.sh 的 gate_diff_base / gate_changed_paths / gate_added_lines 取，列路径的 git 带 -c core.quotepath=false，调共用取法判退出码；搬迁登记表 .claude/gate.d/path-moves.tsv 一行「旧前缀<TAB>新前缀<TAB>YYYY-MM-DD<TAB>records/<文件>.md」，出处写记这次搬迁的那份记录；此刻生效的行（不在 gate 基准那一版的登记表里，gate 基准里旧前缀下有文件、新前缀下没有文件）不许串，基准里旧前缀下每份文件此刻在新前缀下同一相对路径上都在、被 git 跟踪，旧路径搬干净；不在基准那一版登记表里的新行必须生效（判法 lib-lag-baseline.py 的 path_move_problems）"
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
stage_cell committed-file-size cell_committed_file_size "被提交的东西里没有超过 100 MiB 的文件：HEAD 加暂存区里跟踪的每个文件，与还没推上去的每笔提交里新增或改过的 blob" \
  "超限的文件 gzip -n -9 <文件> 压成 .gz 入库、读它的地方改成认 .gz（例：scripts/experiment-apparatus/replay.sh 的 read_stored），引用照 .claude/rules/path-moves.md 全仓改名；已经进了没推的提交的，推之前重做那几笔提交；判法与弄坏开关在 scripts/gate/large-file-check.py 文件头"
stage_cell stage-location-before-cd cell_stage_location_before_cd "门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname \"\$0\" 找旁边的文件" \
  "在 cd 之前取阶段自己的位置存进变量，cd 之后用那个变量；点名的文件:行改完再单跑这一格"
stage_cell heavy-thread-defaults cell_heavy_thread_defaults "脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨" \
  "shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default、python 经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取，起命令里不写线程（整条经 task-server/task submit 交，线程由服务定）；不是重活的登记进 .claude/heavy-thread-exempt（<路径>:<锚串>  # 理由），排除只缩不涨"
stage_cell hardcoded-quota cell_hardcoded_quota "脚本、门禁与钩子里不定内存上限与台数（上限、机器由 task-server 按记录定），排除表指得到、只缩不涨" \
  "删掉脚本里定的上限与台数，整条经 task-server/task submit -- <命令> 交（服务按记录给上限、撞了放大重排，机器与分片也归它）；自证造的假值与这一轮不改的文件，在排除表 .claude/gate.d/hardcoded-quota-exempt.tsv 登记理由（<路径>:<锚串>  # 理由），排除只缩不涨；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell function-units-two-implementations-agree cell_function_units_two_implementations_agree "函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）原样与产品行为视图两趟：dump 逐单元相同，登记入口的闭包与键逐个相同" \
  "照点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份；判法与样本写在这一格的注释里"
stage_cell mutation-list-items cell_mutation_list_items "变异门禁的 --list-items 在被判的仓上跑通：退 0、报出键对不上的条数，没有「没算出来」、回溯与 PicklingError" \
  "照它打的那一句出路改变异表那一行（多半是点名的测试在 cargo 参数选的特性下不编，补特性）；回溯是 scripts/mutation/crates-mutation-rows.py 自己的错，照回溯修；单跑 python3 scripts/mutation/crates-mutation-rows.py --list-items . 到它报出键对不上的条数"
stage_cell break-switches-sample-only cell_break_switches_sample_only "门禁阶段读弄坏开关都经共用库那一判（开关只在样本根上认），共用库的 stage_cells_run 开头调它" \
  "阶段 source .claude/gate.d/lib/stage-cells.sh、格经 stage_cells_run 跑；不经共用库跑格的 source 它取 stage_cells_refuse_unsampled_break_switches，cd 到被判的根之后调（写法在共用库文件头「弄坏开关只在样本根上认」）"
stage_cell entry-scripts cell_entry_scripts "登记表 entry-scripts.tsv 里的每份入口脚本都带 --list-items、--selftest 与带齐字段的每项一行（或末行总计）" \
  "照 .claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」补那份脚本；还没改成的在 .claude/gate.d/entry-scripts.tsv 登记成 pending、写明还缺什么"
stage_cell break-switch-headers cell_break_switch_headers "研究脚本与钩子读了弄坏开关的，文件头开关表三样写全（名字与值、弄坏哪一处、点名的项），存量豁免表指得到、只缩不涨" \
  "照 scripts/mutation/mutate.sh 文件头那张表写（一行「X_BREAK=<值>  <弄坏哪一处>」，下一行起写点名的项）；存量登记在 .claude/gate.d/break-switch-headers-exempt.tsv，改完删那一行"
stage_cell stage-break-switch-replay cell_stage_break_switch_replay "门禁阶段文件头的每条弄坏开关逐条设上、经 stage-selftest.sh 喂样本，判错的样本与点名的相等" \
  "照实测改那一道文件头那一条下面点名的样本（实测是对的才改头注；样本判错是因为判法坏了就修判法）；一时改不准的登记进 .claude/gate.d/stage-break-switch-replay-exempt.tsv，只缩不涨"
stage_cell break-switch-outside-selftest cell_break_switch_outside_selftest "研究脚本与钩子读了弄坏开关的，平常跑时设上要被拒并点名只在自证里认，存量待改表指得到、只缩不涨" \
  "让脚本在参数解析之前先看这个变量，平常跑（不是 --selftest，也不是自证起的子进程）设了就退 2 并点名它只在自证里认；一时改不完的存量登记进 .claude/gate.d/break-switch-outside-selftest-pending.tsv，改好删那一行"
stage_cell break-switch-replay cell_break_switch_replay "登记表里的脚本与钩子，文件头每条弄坏开关逐条设上跑它的 --selftest，点名的格与实际红的格逐格相同" \
  "照实测改那份文件头那一条下面「」里的失败行原文（实测是对的才改头注；自证判错是因为判法坏了就修判法）；接不进复跑的从 .claude/gate.d/break-switch-replay-files.tsv 删掉那一行"
stage_cell kb-directories-write-scope cell_kb_directories_write_scope ".claude/kb-directories 登记的每个 kb 目录，下一层与子目录里的 .md 都在写回员（kb-scribe）的写范围里" \
  "在 .claude/hooks/agent-write-scope.tsv 给 kb-scribe 加一行 <目录>/**（只许 .md 的写成 <目录>/*.md 与 <目录>/**/*.md 两行：* 不跨目录，<目录>/**/*.md 不盖下一层），定义 .claude/agents/kb-scribe.md「写范围」同步改；目录不该写回的从 .claude/kb-directories 撤掉"
stage_cell prune-server-selftest cell_prune_server_selftest "判定库服务 prune-server/ 的自证过：编服务二进制、起 --selftest，末行 failed=0、cells 大于 0、退出码对得上（目录不在退 77）" \
  "红的格照输出里逐格点名的修 prune-server/（归工具实现员），单跑看细节：task-server/task submit --cwd <仓根>/prune-server -- cargo run --release --offline --bin prune-server -- --selftest；预编库取值没判过的先照出路修环境；判据在 code-tooling.sh 里这一格函数上方的注释"
stage_cell shared-data-root-literals cell_shared_data_root_literals "脚本、钩子、门禁与服务不经 ~ 与 \$HOME 取数据根下共享的那几项（经 lib_work_root.py 的 shared_data_root() 取），排除表指得到、只缩不涨" \
  "经 lib_work_root.py 的 shared_data_root() 取：python 载入 .claude/hooks/lib_work_root.py 调 shared_data_path(<名>)，shell 写 python3 <仓根>/.claude/hooks/lib_work_root.py shared <名>（工作根不带参数），取不到就停；确实不是取目录的，在排除表 .claude/gate.d/shared-data-root-literals-exempt.tsv 登记理由（<路径>:<锚串>  # 理由），排除只缩不涨；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell cp-preserve-mode cell_cp_preserve_mode "项目脚本、钩子、门禁、样本根的搭建脚本与服务里的 cp 不带保留权限的选项（-a、-p、--archive、不带值或带 mode / all 的 --preserve）" \
  "要时间戳的写 --preserve=timestamps、要链接的再加 links（整棵树 cp -dR --preserve=timestamps,links），都不要的写 cp -R；python 改用 shutil.copy2 / copytree"
stage_cell peer-launch-direct cell_peer_launch_direct "项目脚本与门禁不自己起 multi-host-run.sh --peer 往第二台起活（照服务的分片协议，派哪台由服务定）" \
  "照 task-server/task_server_lib.py 文件头「分片」改：文件头写「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell retired-names cell_retired_names "退役名单 retired-names.tsv 里的名字在脚本、门禁、钩子、crates 与服务里一处都没有（不留「设了就拒」的放行；搬迁登记表 path-moves.tsv 记的是旧路径，不扫）" \
  "照名单里那个名字的出路改掉那一行（不留兼容、不单留拒绝、没有放行表）；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell verdict-library-code cell_verdict_library_code "登记表 verdict-library-scripts.tsv 写了判定库的行第三列都是 prune-server，点名的脚本里没有取不到库照跑的分支、prune-server 行没有本地库读写" \
  "取不到库名、服务连不上一律退出（连不上退 75），判定只经 prune-server 的客户端读写；一族接完（导进服务、--verify、删旧目录）再翻第三列；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell temporary-service-runners cell_temporary_service_runners "runner 表里套了临时服务包装的自证，与要临时服务的表 selftest-temporary-service.tsv 里登记成 required 的两边对齐（裸写、缺条、包装了表里没有的都点名；own-server 不要求包装）" \
  "照逐条的出路改这一道的 SELFTEST_RUNNERS：表里 required 的写成「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest」，缺的补上；表里没有的包装要么先在 .claude/hooks/selftest-registry.tsv 登记再往 .claude/hooks/selftest-temporary-service.tsv 加一行，要么去掉；own-server 的不要求包装；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell temporary-service-wrapper-form cell_temporary_service_wrapper_form "要临时服务的自证的整条写法在七处逐字相同（登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、包装前缀常量）、包装的解析收得下、整条里没有带尖括号占位的 --server-binary" \
  "照逐处的出路把点名的那一处改回包装用法行给的整条（不带 --server-binary、不写尖括号占位，二进制写在整条之外）；包装改了用法就各处一起改；判法与取的几处在 code-tooling.sh 里这一格函数上方的注释"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
# 逐项点名（这一道「逐项点名」一节）：--list-items 列分项的三格可点的项，--item <项> 只跑点名的项（给几次取并集）；认法在 code_tooling_resolve_items
CODE_TOOLING_ITEMS=(); code_tooling_list_items_requested=0
while (($#)); do
  case "$1" in
    --list-items) code_tooling_list_items_requested=1; shift ;;
    --item|--item=*)
      if [[ "$1" == --item ]]; then
        if (($# < 2)) || [[ -z "${2//[[:space:]]/}" || "$2" == -* ]]; then
          echo "  ✗ --item 后面缺项名"
          echo "     → 怎么办：写成 --item <项>，几项就给几次；可点的项用 bash .claude/gate.d/code-tooling.sh --list-items 看（分项的只有 research-script-selftests、stage-break-switch-replay、break-switch-replay 三格）。"
          exit 2
        fi
        CODE_TOOLING_ITEMS+=("$2"); shift 2
      else
        if [[ -z "${1#--item=}" ]]; then
          echo "  ✗ --item= 后面缺项名"
          echo "     → 怎么办：写成 --item <项> 或 --item=<项>；可点的项用 bash .claude/gate.d/code-tooling.sh --list-items 看。"
          exit 2
        fi
        CODE_TOOLING_ITEMS+=("${1#--item=}"); shift
      fi
      ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <项> 与一个项目根；格名用 --list 看，项用 --list-items 看。"
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


def outside_markdown_code(text):
    """去掉代码围栏里的行与行内代码：那里的 @ 只是在说这个写法，Claude Code 不展开它（rules-manifest、rules-named-by-definitions 两格读 CLAUDE.md 共用）。"""
    import re
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

# 逐行只留代码的三种读法（shell、python、rust），heavy-thread-defaults、hardcoded-quota、shared-data-root-literals 三格共用：去掉注释、出路文字与字符串字面量，
# 内容整个是调用方认的那几种字面量（kept_content）留着。装法照 MANIFEST_LIBRARY：
#   source_lines = types.ModuleType("lib_source_code_lines"); exec(compile(sys.argv[i], "source-code-lines-library", "exec"), source_lines.__dict__)
SOURCE_CODE_LINES_LIBRARY="$(cat <<'PY_SOURCE_CODE_LINES'
import io, re, tokenize


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


def literal_kept(literal_text, kept_content):
    """字面量（连引号与前缀）要不要留着：内容整个是 kept_content 认的那几种才留。"""
    found = re.match(r"(?is)^[a-z]*('''|\"\"\"|'|\")(.*)\1$", literal_text)
    return bool(found) and kept_content.fullmatch(found.group(2)) is not None


def shell_code_lines(text, output_text):
    """shell 去掉整行注释、行尾不在引号里的 # 注释与出路文字（output_text 认的 echo / printf / print( 的字面量）。"""
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
        lines.append(output_text.sub(" ", line))
    return lines


def python_code_lines(text, output_text, kept_content, scan_strings=False):
    """python 经 tokenize 去掉注释、文档串与字符串字面量（内容整个是 kept_content 认的留着；scan_strings 时字面量一概留着），tokenize 读不了的按 shell 读。"""
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
        return shell_code_lines(text, output_text)
    middle = getattr(tokenize, "FSTRING_MIDDLE", None)
    for token in tokens:
        if token.type == tokenize.COMMENT:
            blank(token.start, token.end)
        elif not scan_strings and ((token.type == tokenize.STRING and not literal_kept(token.string, kept_content)) or (middle is not None and token.type == middle)):
            blank(token.start, token.end)
    return [output_text.sub(" ", "".join(characters)) for characters in rows]


RUST_STRING = re.compile(r'"(?:[^"\\]|\\.)*"')


def rust_code_lines(text, kept_content, scan_strings=False):
    """rust 去掉 // 之后的部分与字符串字面量（内容整个是 kept_content 认的留着；scan_strings 时字面量一概留着）。"""
    lines = []
    for line in text.split("\n"):
        if not scan_strings:
            line = RUST_STRING.sub(lambda found: found.group(0) if literal_kept(found.group(0), kept_content) else " " * len(found.group(0)), line)
        position = line.find("//")
        lines.append(line if position < 0 else line[:position])
    return lines
PY_SOURCE_CODE_LINES
)"

# 弄坏开关的文件头表（.claude/rules/test-script-discipline.md「测试脚本自己也要被测试证明有效」；用户 2026-10-01 定取 scripts/mutation/mutate.sh 那种写法），
# break-switch-headers、stage-break-switch-replay 两格共用这一份读法（不各写一份）。装法照 MANIFEST_LIBRARY。
#   一条开关：行首（可带 #）缩进之后「<名字以 _BREAK 结尾的变量>=<值>」，后面紧接两个以上空格写弄坏哪一处，或接一段（…）注；值只认小写起头的字母、数字与 _ . , : -。
#     「X_BREAK=<项>」这种占位写法、值后面只隔一个空格接着写命令的用法示例，不算一条。
#   点名的项：下一行起缩进比开关那一行深的几行（到缩进回到同一层、或下一条开关为止）；研究脚本与钩子写自证的项名（括号里写失败行句首），门禁阶段写样本目录名。
#   三样写全：名字与值、弄坏哪一处（同一行的说明或（…）注）、至少一行点名的项。
BREAK_SWITCH_TABLE_LIBRARY="$(cat <<'PY_BREAK_SWITCH_TABLE'
import re

SWITCH_ENTRY = re.compile(r"^\s*#?(?P<indent>\s*)(?P<name>[A-Z][A-Z0-9_]*_BREAK)=(?P<value>[a-z0-9][A-Za-z0-9_.,:-]*)(?P<rest>.*)$")


def comment_body(row):
    """去掉行首的 # 之后的那一段（python 文档串里的行原样）。"""
    stripped = row.lstrip()
    return stripped[1:] if stripped.startswith("#") else row


def indentation(text):
    return len(text) - len(text.lstrip())


def switch_entries(text):
    """→ [{line, name, value, described, items}]：items 是点名的项那几行（去掉 #）连起来的正文。"""
    rows = text.split("\n")
    entries = []
    for index, row in enumerate(rows):
        found = SWITCH_ENTRY.match(row)
        if not found:
            continue
        rest = found.group("rest")
        if rest and not re.match(r"\s{2,}\S|（|\(", rest):
            continue   # 用法示例：值后面只隔一个空格接着写命令
        depth = indentation(comment_body(row))
        items = []
        for following in rows[index + 1:]:
            body = comment_body(following)
            if not body.strip() or SWITCH_ENTRY.match(following) or indentation(body) <= depth:
                break
            items.append(body.strip())
        entries.append({"line": index + 1, "name": found.group("name"), "value": found.group("value"),
                        "described": bool(rest.strip()), "items": "\n".join(items)})
    return entries


def incomplete(entry):
    """缺的那几样（空表是三样写全）。"""
    return [what for what, ok in (("弄坏哪一处", entry["described"]), ("点名的项", bool(entry["items"].strip()))) if not ok]


def named_tokens(entry, candidates):
    """点名的项里出现的候选名字（整词：前后不接字母、数字、_、-）。"""
    text = entry["items"]
    return {name for name in candidates if re.search(r"(?<![\w-])" + re.escape(name) + r"(?![\w-])", text)}
PY_BREAK_SWITCH_TABLE
)"

# break-switch-replay 格的判法（单元键里的 judge，那一格文件头「按条复用」）：输出怎么收、失败行怎么减基准趟、点名的项怎么比、汇总行怎么认、哪条算绿。
# 格里与开关表的读法拼成一个模块装（break_switch_replay_judge_code 打的就是这两段），只放判法、不起进程；起自证、限时、重跑与进度在格自己那一段。
BREAK_SWITCH_REPLAY_JUDGE_LIBRARY="$(cat <<'PY_BREAK_SWITCH_REPLAY_JUDGE'
import collections, re

# 基准趟与开关趟的失败行按整行比：临时路径、临时文件名与数字每趟都可能不同，比之前换成同一个记号
VOLATILE = re.compile(r"/tmp/[^\s'\"：，、）)\]」]+|\btmp[A-Za-z0-9_]{6,}|[0-9]+")


def quoted_items(text):
    """点名的项：「…」里的原文（嵌套的括号配平）。"""
    found, depth, start = [], 0, None
    for index, character in enumerate(text):
        if character == "「":
            if depth == 0:
                start = index + 1
            depth += 1
        elif character == "」" and depth:
            depth -= 1
            if depth == 0:
                found.append(text[start:index])
    return found


def appears_in(item, row):
    """点名的一项照不照得出这一行：… 是中间省掉的一段，各段按次序出现在同一行里。"""
    pieces = [piece.strip() for piece in re.split(r"…+|\.\.\.", item) if piece.strip()]
    position = 0
    for piece in pieces:
        at = row.find(piece, position)
        if at < 0:
            return False
        position = at + len(piece)
    return bool(pieces)


def decoded_output(streams, broken):
    """[(那一路的名字, 收到的字节或文本)] → (输出, 坏字节的说明)：各路照 UTF-8 解，解不了的照 U+FFFD 换掉接着判、另记一条；
    单独的 \\r 与 \\r\\n 都当换行（弄坏开关 replay-carriage-return-kept 下单独的 \\r 留在行里）。"""
    texts, bad = [], []
    for stream, data in streams:
        if isinstance(data, str):
            texts.append(data)
            continue
        try:
            texts.append(data.decode("utf-8"))
        except UnicodeDecodeError as error:
            bad.append(f"{stream} 里有不合法的 UTF-8（第 {error.start} 字节起：{data[error.start:error.start + 8].hex(' ')}）")
            texts.append(data.decode("utf-8", errors="replace"))
    output = "".join(texts)
    if broken != "replay-carriage-return-kept":
        output = output.replace("\r\n", "\n").replace("\r", "\n")
    return output, bad


def failure_rows(output):
    return [row.strip() for row in output.split("\n") if "✗" in row]


def extra_rows(rows, baseline_rows):
    """这一趟的失败行减去基准趟的（去掉临时路径与数字之后按整行比，按次数抵）。"""
    remaining = collections.Counter(VOLATILE.sub("#", row) for row in baseline_rows)
    extra = []
    for row in rows:
        key = VOLATILE.sub("#", row)
        if remaining[key] > 0:
            remaining[key] -= 1
        else:
            extra.append(row)
    return extra


def baseline_green(record):
    """基准趟（不设开关起的那一趟）算绿：没超时、退 0、没有坏字节。"""
    return not record["timeout"] and record["code"] == 0 and not record["bad"]


def summary_keys_of(records, baseline_rows, broken):
    """一份脚本这一趟的各条开关趟里红了的有一半以上（至少两条）都多红的同一行（去掉临时路径与数字之后）：汇总行，不要求点名。"""
    if broken == "replay-summary-rows-required":
        return frozenset()
    counts, red = collections.Counter(), 0
    for record in records:
        if record["timeout"] or record["code"] in (0, None):
            continue
        red += 1
        counts.update({VOLATILE.sub("#", row) for row in extra_rows(failure_rows(record["output"]), baseline_rows) if row.startswith("✗")})
    return frozenset(key for key, count in counts.items() if count >= 2 and count * 2 > red)


def judge(entry, record, baseline_rows, summary_keys, broken):
    """一条开关趟 → (结局, 说明)：结局是 对、守卫、超时、没红、没点名、不符之一；坏字节另记，不在这里。"""
    if record["timeout"]:
        return "超时", "超过登记的秒数没跑完"
    output, code = record["output"], record["code"]
    if record["bad"] and broken == "replay-bad-bytes-first":
        return "对", ""
    if code == 2 and any(entry["name"] in row and ("自证" in row or "selftest" in row.lower()) for row in output.split("\n")):
        return "守卫", ""
    if code == 0:
        return "没红", "设上之后自证照样退 0（开关没弄坏什么）"
    items = quoted_items(entry["items"])
    if not items:
        return "没点名", "点名的项里没有可比的「…」原文（失败行句首要写在「」里）"
    extra = extra_rows(failure_rows(output), baseline_rows)
    missing = [item for item in items if broken != "replay-items-unchecked" and not any(appears_in(item, row) for row in extra)]
    unnamed = [] if broken == "replay-unnamed-rows-unchecked" else [
        row for row in extra if row.startswith("✗") and VOLATILE.sub("#", row) not in summary_keys and not any(appears_in(item, row) for item in items)]
    if missing or unnamed:
        parts = []
        if missing:
            parts.append("点名的项没在失败行里：" + "；".join(f"「{item[:60]}」" for item in missing[:3]) + (f" 等 {len(missing)} 项" if len(missing) > 3 else ""))
        if unnamed:
            parts.append("失败行没被点名：" + "；".join(f"「{row[:80]}」" for row in unnamed[:3]) + (f" 等 {len(unnamed)} 行" if len(unnamed) > 3 else ""))
        return "不符", "；".join(parts)
    return "对", ""


def switch_green(kind, record):
    """一条开关判完算绿：对上（对、守卫）且没有坏字节。"""
    return kind in ("对", "守卫") and not record["bad"]
PY_BREAK_SWITCH_REPLAY_JUDGE
)"

# stage-break-switch-replay 格的判法（单元键里的 judge，那一格文件头「按条复用」）：stage-selftest.sh 的输出里判错的样本怎么认、算不算跑成、点名的与实测比出哪一句红。
# 格里与开关表的读法拼成一个模块装（stage_break_switch_replay_judge_code 打的就是这两段），只放判法、不起进程。
STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY="$(cat <<'PY_STAGE_BREAK_SWITCH_REPLAY_JUDGE'
def wrong_samples(output, stage, samples):
    """stage-selftest.sh 报「✗ <阶段> <样本> …」的那几份（只认这一道的样本目录名）。"""
    wrong = set()
    for line in output.splitlines():
        fields = line.split()
        if len(fields) >= 3 and fields[0] == "✗" and fields[1] == stage[:-3] and fields[2] in samples:
            wrong.add(fields[2])
    return wrong


def run_finished(code, output):
    """stage-selftest.sh 跑成了：退 0 或 1，且打了「判得都对」或「个用例判错」那一句。"""
    return code in (0, 1) and ("判得都对" in output or "个用例判错" in output)


def problem_of(stage, entry, switch, named, wrong, finished, code, output, broken):
    """一条复跑的结局 → 判红的那一句；对上了 → None。逐条进度、记库与收尾的判红用同一份。"""
    if not finished:
        return f"{stage}:{entry['line']} {switch}：stage-selftest.sh 没跑成（退 {code}；末尾：{' | '.join(output.strip().splitlines()[-2:])[:200]}）"
    if named != wrong and broken != "switch-replay-named-unchecked":
        return f"{stage}:{entry['line']} {switch}：点名的与实测判错的不等（点了没错：{'、'.join(sorted(named - wrong)) or '无'}；错了没点：{'、'.join(sorted(wrong - named)) or '无'}）"
    return None
PY_STAGE_BREAK_SWITCH_REPLAY_JUDGE
)"

# 两格开关复跑的判法入口：单元键收这两个函数的闭包（读到的顶层变量连同 here-doc 正文，改一句判法就作废），格里也照它们打的文本装判法，判的与进键的是同一份
break_switch_replay_judge_code() { printf '%s\n' "$BREAK_SWITCH_TABLE_LIBRARY" "$BREAK_SWITCH_REPLAY_JUDGE_LIBRARY"; }
stage_break_switch_replay_judge_code() { printf '%s\n' "$BREAK_SWITCH_TABLE_LIBRARY" "$STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY"; }

# 按条复用的共用段（stage-break-switch-replay、break-switch-replay 两格，不各写一份；research-script-selftests 格是 shell 写的那一份，样板同它）：
#   经 scripts/gate/admission.py 的 verdict-plan 算键、查库（复用哨兵在它里面抽），verdict-record 照键记绿、记红撤绿，失灵与服务不可用交 plan-failure 判；
#   样本根另放的 .unit-verdict-sample/prior/（上一版的树）先照判绿记进样本自己的判定库，.unit-verdict-sample/red-over-green 在时计划另设 ADMISSION_BREAK=verdict-lookup-skipped
#   （不读库、一律现判），判红记库之后再算一次计划、逐条打现判的原因（门禁样本根里 admission.py 才认这个开关）。装法照 BREAK_SWITCH_TABLE_LIBRARY。
UNIT_VERDICT_ROUNDS_LIBRARY="$(cat <<'PY_UNIT_VERDICT_ROUNDS'
import os, subprocess, threading

REUSE, JUDGE, REFUSED = "reuse", "judge", "refused"
FORCED_REASON_PREFIX = "强制现判"
STATE_WORDS = {REUSE: "复用", JUDGE: "现判", REFUSED: "拒算"}


def admission_run(admission, arguments, output_path, environment_changes=None):
    """起一趟 admission.py <子命令>：标准输出与标准错误写进 output_path → (退出码, 输出)。"""
    environment = dict(os.environ)
    environment.update(environment_changes or {})
    with open(output_path, "w", encoding="utf-8") as handle:
        done = subprocess.run(["python3", admission, *arguments], stdout=handle, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, env=environment)
    with open(output_path, encoding="utf-8", errors="replace") as handle:
        return done.returncode, handle.read()


def last_lines(text, count=2, width=300):
    return " ".join(line.strip() for line in text.strip().split("\n")[-count:])[:width]


def plan_rows(output):
    """verdict-plan 逐条那几行：[(名字, 决定, 原因)]。"""
    rows = []
    for line in output.split("\n"):
        fields = line.split("\t", 3)
        if len(fields) == 4 and fields[1] in (REUSE, JUDGE, REFUSED):
            rows.append((fields[0], fields[1], fields[3]))
    return rows


class ItemVerdicts:
    """一格一趟的按条复用。ok：计划算出来了；red：复用失灵或配置错（这一格判红）；unavailable：prune-server 不可用（这一格退 75）；problem：算不出的那一句。"""

    def __init__(self, admission, root, kind, scratch, caller, sample, how, record_red=True):
        self.admission, self.root, self.kind, self.scratch, self.caller = admission, root, kind, scratch, caller
        self.sample, self.how, self.record_red = sample, how, record_red
        self.decisions, self.names, self.problem = {}, [], ""
        self.ok = self.red = self.unavailable = False
        self.lock, self.serial = threading.Lock(), 0

    def path(self, name):
        return os.path.join(self.scratch, name)

    def marker(self, name):
        return self.sample and os.path.exists(os.path.join(self.root, ".unit-verdict-sample", name))

    def failure(self, command, exit_code, output_path):
        """失灵没有交 plan-failure 判（admission.py 文件头「按条复用失灵」）：它打 ✗ 与出路；退 75 记不可用，别的非 0 记红。"""
        done = subprocess.run(["python3", self.admission, "plan-failure", self.caller, command, str(exit_code), output_path],
                              capture_output=True, text=True, errors="replace", stdin=subprocess.DEVNULL)
        text = (done.stdout + done.stderr).rstrip("\n")
        if text:
            print(text, flush=True)
        if done.returncode == 75:
            self.unavailable = True
        elif done.returncode != 0:
            self.red = True

    def seed_prior(self):
        """样本根的 .unit-verdict-sample/prior/：拿上一版的树算同一批请求的键、照判绿记进样本自己的判定库。"""
        prior = os.path.join(self.root, ".unit-verdict-sample", "prior")
        if not (self.sample and os.path.isdir(prior)):
            return True
        code, output = admission_run(self.admission, ["verdict-plan", prior, self.kind, self.path("requests"), self.path("seed.plan")], self.path("seed.out"))
        if code != 0:
            self.problem = f"样本：上一版的树上算不出键（{last_lines(output)}）"
            self.failure("verdict-plan", code, self.path("seed.out"))
            return False
        with open(self.path("seed.names"), "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in self.names))
        code, output = admission_run(self.admission, ["verdict-record", prior, self.kind, self.path("seed.plan"), self.path("seed.names"), "--how", "样本：上一版"],
                                     self.path("seed.record"))
        if code != 0:
            self.problem = f"样本：上一版那几条没记进库（{last_lines(output)}）"
            self.failure("verdict-record", code, self.path("seed.record"))
            return False
        print(f"  · 样本：上一版（.unit-verdict-sample/prior）的 {len(self.names)} 条照判绿记进这一版样本的判定库", flush=True)
        return True

    def plan(self, request_lines):
        """算键、查库；交算没算出来。哨兵那几行（抽中谁、种子）照打进日志。"""
        self.names = [line.split("\t", 1)[0] for line in request_lines]
        with open(self.path("requests"), "w", encoding="utf-8") as handle:
            handle.write("".join(line + "\n" for line in request_lines))
        if not self.seed_prior():
            return False
        changes = {"ADMISSION_BREAK": "verdict-lookup-skipped"} if self.marker("red-over-green") else {}
        code, output = admission_run(self.admission, ["verdict-plan", self.root, self.kind, self.path("requests"), self.path("plan")], self.path("plan.out"), changes)
        if code != 0 or not any(line.startswith("PLAN ") for line in output.split("\n")):
            self.problem = f"admission.py verdict-plan {'退 ' + str(code) if code else '没打 PLAN 那一行'}（{last_lines(output)}）"
            self.failure("verdict-plan", code, self.path("plan.out"))
            return False
        for line in output.split("\n"):
            if "哨兵" in line and "\t" not in line:
                print(line.rstrip(), flush=True)
        self.decisions = {name: (decision, reason) for name, decision, reason in plan_rows(output)}
        os.symlink("plan", self.path("plan-per-entry"))
        self.ok = True
        return True

    def decision(self, name):
        if not self.ok:
            return JUDGE, "按条复用算不出，照判"
        return self.decisions.get(name, (JUDGE, "计划里没有它"))

    def state(self, name):
        """复用三问认的结局（scripts/gate/lib_reuse_checks.py 文件头「一」）：复用、现判、强制现判、拒算。"""
        decision, reason = self.decision(name)
        if decision == JUDGE and reason.startswith(FORCED_REASON_PREFIX):
            return "强制现判"
        return STATE_WORDS.get(decision, "现判")

    def record(self, green, red, final=False):
        """照计划里的键记：判绿记绿，判红记红、撤同一把键原有的绿（record_red 是 False 时不交判红的，改前的样子）。
        一份判完就记的那几趟用不带复用哨兵状态的计划（plan-per-entry），收尾那一趟用带它的（哨兵在那一趟判）。"""
        if not self.ok or not (green or red):
            return
        with self.lock:
            self.serial += 1
            serial = self.serial
        green_path, red_path, output_path = self.path(f"record-{serial}.green"), self.path(f"record-{serial}.red"), self.path(f"record-{serial}.out")
        with open(green_path, "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in green))
        with open(red_path, "w", encoding="utf-8") as handle:
            handle.write("".join(name + "\n" for name in red))
        arguments = ["verdict-record", self.root, self.kind, self.path("plan" if final else "plan-per-entry"), green_path, "--how", self.how]
        if red and self.record_red:
            arguments += ["--red-names", red_path]
        code, output = admission_run(self.admission, arguments, output_path)
        if code == 0:
            if final and green:
                print(f"  · 判绿记库：{last_lines(output, 1)}", flush=True)
            if final and red and self.record_red:
                print(f"  · 判红记库：{len(red)} 条记红、撤掉同一把键原有的绿，下一趟这几条现判（{last_lines(output, 1)}）", flush=True)
            return
        print(f"  ! 判定记库：admission.py verdict-record 没成（{last_lines(output)}），下一趟这几条照判", flush=True)
        self.failure("verdict-record", code, output_path)

    def reread_after_red(self):
        """样本根带 .unit-verdict-sample/red-over-green 时：判红记库之后不设 ADMISSION_BREAK 再算一次计划，打复用几条、现判几条，现判的逐条写原因。"""
        if not self.marker("red-over-green"):
            return
        code, output = admission_run(self.admission, ["verdict-plan", self.root, self.kind, self.path("requests"), self.path("reread.plan")], self.path("reread.out"))
        if code != 0:
            print(f"  ✗ 样本档：判红记库之后再算计划没成（{last_lines(output)}）")
            print("     → 怎么办：看括号里 admission.py verdict-plan 报的错，修好样本或判定库，再单跑这一份样本")
            return
        rows = plan_rows(output)
        print(f"  · 样本档：判红记库之后再算一次计划：复用 {sum(1 for row in rows if row[1] == REUSE)} 条、现判 {sum(1 for row in rows if row[1] == JUDGE)} 条", flush=True)
        for name, decision, reason in rows:
            if decision == JUDGE:
                print(f"    现判「{name}」：{reason}", flush=True)

    def write_states(self, path, names):
        """复用三问那一趟的各条结局（一行「名字<TAB>结局」）；path 空不写。"""
        if path:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("".join(f"{name}\t{self.state(name)}\n" for name in names))
PY_UNIT_VERDICT_ROUNDS
)"

# code_tooling_three_question_rounds <格名> <跑一趟的函数>：复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）。被判的根是样本、带 .unit-verdict-sample/three-questions/
#   （unrelated.sh：改一处不在任何判定闭包里的输入；change.sh：改闭包里的一处；affected：一行一条闭包含它的判定名）时，两格不照常跑一趟，改成在同一份判定库上跑四趟：
#   头一趟、同一份输入、跑 unrelated.sh 之后、跑 change.sh 之后；每趟照真跑那一条路算键、查库、起、记库，各条的结局（复用、现判、强制现判、拒算）写一份，
#   再交 lib_reuse_checks.py three-questions 判三问、打规范输出「REUSE_SELFTEST script=.claude/gate.d/code-tooling.sh …」（门禁 code-tooling 的 reuse-selftests 格经
#   verdict-library-scripts.tsv 那一行起这两份样本、认这一行）。四趟都判绿、三问都过退 0；有一趟没判绿、或三问有一问不过退 1。
code_tooling_three_question_rounds() {
  local cell="$1" pass="$2" marker=".unit-verdict-sample/three-questions" round rounds_scratch rounds_red=0 affected
  rounds_scratch="$(mktemp -d)" || { echo "  ✗ 建不了复用三问的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一份样本；一趟都没跑。"; return 1; }
  for round in first second unrelated unit; do
    if [[ "$round" == unrelated || "$round" == unit ]]; then
      local change_script="$marker/unrelated.sh"
      [[ "$round" == unit ]] && change_script="$marker/change.sh"
      if ! bash "$change_script"; then
        echo "  ✗ 样本的 $change_script 没跑成，复用三问判不了"
        echo "     → 怎么办：照它的报错改样本；它只许改样本根里的文件。"
        rm -rf -- "$rounds_scratch"
        return 1
      fi
    fi
    echo "── 复用三问：$round 那一趟（$cell）"
    if ! "$pass" "$rounds_scratch/$round.tsv"; then
      echo "  · 复用三问：$round 那一趟这一格没判绿（上面几行）"
      rounds_red=1
    fi
  done
  affected="$(sed '/^[[:space:]]*$/d' "$marker/affected" | paste -sd, -)"
  if ! python3 "$STAGE_REPOSITORY/scripts/gate/lib_reuse_checks.py" three-questions --script ".claude/gate.d/$STAGE_NAME" --first "$rounds_scratch/first.tsv" \
         --second "$rounds_scratch/second.tsv" --unrelated "$rounds_scratch/unrelated.tsv" --unit-change "$rounds_scratch/unit.tsv" --affected "$affected"; then
    echo "  ✗ $cell 的复用三问没过（红在哪一问写在上面 REUSE_SELFTEST 那一行与它上面几行）"
    echo "     → 怎么办：照 scripts/gate/lib_reuse_checks.py 文件头「一」看是哪一问：① 查库或跳过复用的那一段，② 键里多进了闭包外的输入，③ 键少进或多进了闭包里的输入"
    rounds_red=1
  fi
  rm -rf -- "$rounds_scratch"
  return "$rounds_red"
}

# ── 格 research-script-selftests：runner 表里每一条研究脚本与共用库的自证都通过（分批并行跑）
#
# 判据：SELFTEST_RUNNERS 里每一条自证都退 0；表全不全归 research-script-selftest-coverage 格。lib/stage-cells.sh 不是阶段，它的自证只在这张表里跑。
#   自证退 77 是「本次未跑」（它自己造不出前提，例如 crates-mutation-rows.py 并发领片号那一格两次调用没重叠）：那一条不判绿也不判红，不记进判定库（下一趟照跑），
#   成功句与汇总里逐条列名、往 $GATE_NOT_RUN_FILE 各报一行；这一趟真起的每一条都退 77、又没有复用的，整格退 77（一条都没判过）。
#   样本 runner-not-run-green（两条，一条退 0、一条退 77）必须判绿并列出本次未跑的那一条；runner-all-not-run（两条都退 77）整格退 77。
#   弄坏开关 CODE_TOOLING_BREAK=selftest-not-run-as-pass：77 当 0，照绿记（改前的样子），这两份样本都判错。
#   每一条经 scripts/gate/selftest-process-guard.py 起（登记表取被判那棵树的 .claude/hooks/selftest-registry.tsv）：登记过的，
#   跑的时候它整棵进程树里起了真 cargo / rustc（版本查询之外）、真清场（gpu-quiesce-run.sh 读真多机配置）或私有清场脚本，守卫退 3，这一条照没过记；
#   经 PATH 起的 cargo、rustc 被守卫的替身当场拒、真的那一份不起。判据、射程与本次未查写在守卫文件头。别的（表外的、
#   不是单独一个 --selftest 的）守卫原样 exec、本次未盯，成功句逐条列名。每条的判定写 <序号>.guard，按表次序回读、数份数，缺一份整格红。
#
# 分批并行：SELFTEST_RUNNERS_STARTED_ALONE 里最慢的那几条先各自单独起、跑到完；其余按表次序每批 N 条并行，一批收齐再起下一批，
#   N = 每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；
#   取不了这一格判红）减去单独起的条数，至少 1。
#
# 按条复用：表里每一条自证一条判定（scripts/gate/admission.py verdict-plan / verdict-record，种类 code-tooling.sh:research-script-selftests），
#   键是那份脚本整份的单元闭包（它 import / source 到的模块与函数、读的数据文件、解释器版本，算法在 admission.py 文件头「单元键」）、
#   这一条命令原文（line:command=）、selftest-registry.tsv 里那份脚本登记的种类（line:selftest-registration=）与判法的闭包（judge:，在这一道所在的仓里算）；不带路径与行号。
#   判法取 selftest-process-guard.py 里判盯不盯的那几个函数（SELFTEST_JUDGE_ENTRIES：认哪条命令要盯、真工具链的位置、替身、进程树与 execve 记录怎么判），
#   不取整份程序：它的 main / guard 起的是调用方给的那条命令（subprocess 的命令折不出），整份闭包必拒算、每一条都现判；被起的那条命令已由入口与 line:command= 进键。键相同、判过绿的不起，表全部复用就一条都不起；拒算的（闭包收不全）照跑、不记。
#   现判而判绿、守卫给了判定的照键记进 prune-server 那一份判定库（unit-verdicts 族，种类 singlefs-unit-verdicts/code-tooling.sh:research-script-selftests，
#   库名固定、库根不进键，scripts/gate/admission.py 文件头「判定库」；盯了而 execve 跟踪没跑的不记；样本根在临时 prune-server 底下起，另占一份命名空间）。
#   记红、撤绿、下趟现判：现判而判红的（自证退非 0、不是 77）照同一把键记红、撤掉这把键原有的绿（selftest_record_verdicts 经 admission.py verdict-record --red-names，
#   打一行「· 判红记库：…」），下一趟这一条现判、原因以「上次判红」起头；退 77 的不是判定，不记；拒算的没有键，不记。
#   一条判完就记：每条自证跑完，起它的那个后台作业写完退出码就当场照键记这一条（selftest_record_one，判法与回读同一份 selftest_outcome；
#   admission.py verdict-record 每把键一份文件、同目录临时文件写完 fsync 再改名换上），这一格中途被停，已经跑完的那几条都在库里，下一趟它们复用、只补没判的；
#   当场记的那一趟不判复用哨兵，收尾照旧把这一趟的判绿、判红名单整份再交一次 verdict-record（同一把键写同样的内容），哨兵与「· 判绿记库」那一行在收尾那一趟。
#   样本 runner-recorded-before-the-run-ends（两条，样本根的 .selftest-environment 设 SINGLEFS_THREAD_CAP=1：每批一条、按表次序）：
#   a 退 0；b 在样本自己的判定库（git common-dir 底下）里找 a 那一条的判绿记录，找到退 0、找不到退 1：整格收尾之前 a 已经在库里，判绿、两条都记。
#   成功句报「表里 N 条：现判 J、复用 M、拒算 R、本次未跑 0」，现判与拒算逐条列为什么（键对不上时是变了的单元），没过的那一条另报为什么现判。
#   样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库：runner-unit-key 三条自证、上一版只有 c 那一份不同，
#   必须只现判 c、另两条复用。
#   按条复用失灵（verdict-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），这一格把那一趟的输出交给它；它判失灵，这一格照旧全部现判跑完、结论判红。
#   样本 runner-plan-exit-red（.unit-verdict-sample/plan-failure 写 exit-1：verdict-plan 退 1）与 runner-plan-sentinel-red（写 sentinel-reuse-failed：哨兵报复用失灵）
#   各三条自证都退 0，必须判红、点名失灵在哪，三条照样跑完。
#   样本 runner-red-over-green（记红、撤绿、下趟现判）：表两条（a 自证退 0、b 自证在样本根有开关文件 .red-switch 时退 1），.unit-verdict-sample/prior/ 是这一版逐字拷的一份
#   （上一版两条照判绿记进库，同一把键），.unit-verdict-sample/red-over-green 在：这一格的计划另设 ADMISSION_BREAK=verdict-lookup-skipped（不读库）照样两条都现判，b 判红、照键记红盖掉绿，
#   样本档接着不设它再算一次计划：复用 a、现判 b、原因以「上次判红」起头，整格判红。
#   每条的输出与退出码各写自己的文件，跑完按表次序回读、数份数（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
#   几份自证抢同一份状态（同一个临时路径、同一个端口）的，只在并行下红：那一条挪进 SELFTEST_RUNNERS_STARTED_ALONE 单独起，或让它的自证各用各的临时目录。
#   计划或记库退 75（prune-server 不可用）：plan-failure 判不可用、退 75（admission.py 文件头「按条复用失灵」），这一格不现判、退 75；整道由 lib/stage-cells.sh 汇总成 75。
#   样本 runner-plan-unavailable（.unit-verdict-sample/plan-failure 写 exit-75）三条自证都退 0，这一格退 75、一条都不起，整道 75、汇总写 prune-server 不可用。
#   逐项点名（这一道「逐项点名」一节）：项是 <脚本路径>，表里只留起那几份脚本的自证，算键、复用、记库照整格的走法。样本 runner-item-picked（三条里点一份，只跑那一条）、
#   runner-item-unknown（项写错，退 2）。
#
# 只在临时 prune-server 上跑的自证（.claude/hooks/selftest-temporary-service.tsv 登记成 required 的：自证开跑先调 selftest_service_problem，scripts/prune-server/prune_server_client.py 文件头「命令行」，裸跑退 2、不写真服务；
#   认哪些是 required 读这张表，不看源码里有没有那几个字，读法在 TEMPORARY_SERVICE_TABLE 的注释；登记成 own-server 的两份自己起真服务，不要求包装）：
#   表里写成包装写法「$SELFTEST_TEMPORARY_SERVICE_PREFIX <原命令>」，不写二进制；裸写的整格判红、一条都不起，点名该写成的那一句（样本 runner-bare-temporary-service-red，样本根放自己的要临时服务的表）。
#   runner 表与要临时服务的表两边对不对得上归 temporary-service-runners 格（文件头开关表里的样本）。
#   键与登记表里的登记都按 -- 后面那条原命令认（line:command= 仍是表里那一行整句），另加 file:scripts/prune-server/prune_server_client_selftest.py。
#   这一趟要起的里面有包装写法的，开跑前取一次 prune-server 二进制（code_tooling_prune_server_binary：PRUNE_SERVER_BINARY 设了用它，没设照 prune-server-selftest 格的编法
#   编进这一格的临时目录、跑完随它删；取不到整格判红），每条照「包装 --server-binary <那一份> -- 守卫 -- <原命令>」起：守卫盯的仍是原命令那一棵。
#   弄坏开关 CODE_TOOLING_BREAK=bare-temporary-service-selftest-accepted：不核写法（改前的样子），runner-bare-temporary-service-red 判错。
#
# 样本：每份样本都放 setup.sh 做 git init（判定库的名字取被判那棵树的 git common-dir，stage-selftest 拷出的临时目录不是 git 工作树时 admission.py verdict-plan 退 2）。
#   被判的根是样本时（judged_root_is_sample），样本根的 .selftest-runners（一行一条命令，空行与 # 开头的行不算）换掉 runner 表，
#   单独起的那一路不走；真仓与 --staged 下放了它也不认。runner-red 四条里第二条必红，必须判红、点名它并列出输出，另三条照样跑完、
#   报 4 条里 1 条没过；runner-green 三条都退 0，判绿并报跑了 3 条。
#   runner-guard-red 与 runner-guard-green 各放一份自己的 .claude/hooks/selftest-registry.tsv，把一份样本自证登记成 selftest：
#   red 那一份经 PATH 起 cargo build（清单不在），必须被守卫的替身拒、这一条退 3 判红，报 2 条里 1 条没过；green 那一份只用样本自己的假 cargo，
#   判绿并报盯了 1 条、本次未盯 1 条（另一条没登记）。
SELFTEST_RUNNERS=(
  "python3 scripts/agents/checklist-specs.py --selftest" "bash scripts/hosts/two-logs.sh --selftest" "python3 scripts/hosts/two_logs.py --selftest" "bash scripts/gate/baseline-diff.sh --selftest" "python3 scripts/kb/batch-target.py --selftest"
  "python3 scripts/kb/quote-kb.py --selftest" "python3 scripts/kb/kb-sections.py --selftest"
  "python3 scripts/crash/check-segment-registry.py --selftest" "python3 scripts/kb/replace-once.py --selftest"
  "python3 scripts/kb/replace-batch.py --selftest" "python3 scripts/experiment-apparatus/e152-tables.py --selftest"
  "python3 scripts/agents/agent-watch.py --selftest" "python3 scripts/kb/quote-rust-items.py --selftest"
  "python3 scripts/kb/sweep-term.py --selftest" "python3 scripts/kb/archive-past-rounds.py --selftest"
  "python3 scripts/kb/insert-row.py --selftest" "bash scripts/hosts/cache-keepalive.sh --selftest"
  "python3 scripts/kb/ledger-append-section.py --selftest" "python3 scripts/kb/append-record.py --selftest" "python3 scripts/agents/lib_session_directory.py --selftest" "python3 scripts/agents/lib_session_registry.py --selftest"
  "bash scripts/agents/watch.sh --selftest" "python3 scripts/kb/stale-candidates.py --selftest"
  "python3 scripts/kb/stale-candidates.py --benchmark" "python3 scripts/hosts/test-environment-check.py --selftest"
  "bash scripts/kb/verify-citations.sh --selftest"
  "python3 scripts/agents/agent-handover.py --selftest" "python3 scripts/kb/decision-slim-check.py --selftest"
  "python3 scripts/kb/pdf-text.py --selftest" "python3 scripts/kb/rules-sweep-audit.py --selftest"
  "python3 scripts/agents/dispatch-prompt.py --selftest" "python3 scripts/agents/three-way-materials.py --selftest"
  "python3 scripts/gate/lib_rebuilt_packages.py --selftest"
  "python3 scripts/gate/lib_function_units.py --selftest"
  "python3 scripts/gate/checker-direction-overlap.py --selftest"
  "bash scripts/agents/three-way-rerun-template.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest"
  "python3 scripts/agents/agent-cost.py --selftest" "bash scripts/devices/snapshot-check.sh --selftest" "bash scripts/devices/e152-run.sh --selftest"
  "python3 scripts/agents/subagent_transcript.py --selftest"
  "bash scripts/gate/changed-paths.sh --selftest"
  "bash scripts/gate/gate-staged.sh --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/admission.py --selftest"
  "bash scripts/hosts/capped.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/mutation-shard-run.sh --selftest"
  "bash scripts/hosts/multi-host-run.sh --selftest" "bash scripts/hosts/run-with-memory-cap.sh --selftest" "bash scripts/hosts/run-build-directory.sh --selftest" "python3 scripts/agents/copy-originals.py --selftest"
  "python3 task-server/task_server_lib.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/replay.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/mutate.sh --selftest" "python3 scripts/agents/cite-check.py --selftest"
  "python3 scripts/agents/snapshot-tree.py --selftest"
  "python3 scripts/kb/kb-spec-check.py --selftest" "python3 scripts/crash/crash-case-check.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/crash-flow-coverage.py --selftest"
  "bash scripts/mutation/prove-red.sh --selftest" "python3 scripts/gate/apply-writer-patch.py --selftest"
  "python3 scripts/gate/lib_libtest_result_lines.py --selftest" "python3 scripts/gate/rerun-failed-tests.py --selftest"
  "python3 scripts/gate/test-targets-parallel.py --selftest"
  "python3 scripts/gate/closeout-status.py --selftest" "python3 scripts/hosts/compile-then-swap.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest"
  "bash .claude/gate.d/lib/stage-cells.sh --selftest" "python3 scripts/gate/gate-structure-check.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-item.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-two-hosts.sh --selftest"
  "bash scripts/crash/gpu-quiesce-run.sh --selftest"
  "python3 scripts/hosts/gpu-card-health.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay.sh --selftest"
  "python3 scripts/crash/gpu-exemptions-check.py --selftest"
  "python3 scripts/crash/cpu-judging-sites-check.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/broad-mutation.py --selftest" "python3 scripts/mutation/mutation-prune.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e165-pin-red-examples.py --selftest"
  "bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest"
  "python3 scripts/experiment-apparatus/e165-compare-kill-bounds.py --selftest"
  "python3 scripts/gate/hook-swap.py --selftest" "python3 scripts/hosts/host-dispatch.py --selftest"
  "python3 scripts/gate/selftest-process-guard.py --selftest" "bash scripts/devices/vm-bench.sh --selftest"
  "python3 scripts/hosts/source-stamp.py --selftest" "python3 scripts/hosts/memory_peaks_lib.py --selftest"
  "python3 scripts/hosts/minimal-copy.py --selftest" "python3 scripts/hosts/lib_repository_copy.py --selftest" "bash scripts/hosts/work-directory-ownership.sh --selftest"
  "bash scripts/devices/qemu-blklogwrites-check.sh --selftest" "bash scripts/devices/build-patched-qemu.sh --selftest"
  "python3 scripts/gate/stage-mine.py --selftest"
  "python3 scripts/kb/relabel-item.py --selftest" "bash scripts/kb/claim-experiment.sh --selftest" "python3 scripts/kb/lib_experiment_number_directories.py --selftest"
  "python3 scripts/hosts/target-prune.py --selftest" "python3 scripts/gate/crate-sources-reachable.py --selftest"
  "python3 scripts/gate/directory-layout-check.py --selftest"
  "python3 scripts/gate/data-name-lint.py --selftest" "python3 scripts/gate/wallclock-assert-lint.py --selftest"
  "bash scripts/agents/remind.sh --selftest"
  "python3 .claude/hooks/lib_workspace_cargo_lock.py --selftest"
  "python3 .claude/hooks/lib_prompt_scratch_line.py --selftest" "python3 .claude/hooks/lib_work_root.py --selftest"
  "bash scripts/hosts/heavy-threads.sh --selftest" "python3 scripts/hosts/heavy_threads.py --selftest"
  "python3 scripts/hosts/multi_host_addresses.py --selftest" "bash scripts/experiment-apparatus/e185-r2-segment-run.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-lib.sh --selftest" "python3 scripts/kb/doc-consolidate.py --selftest" "python3 scripts/kb/history-consolidate.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/lib_reuse_checks.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/lkmm-selftest.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/device-code-key.py --selftest" "python3 scripts/gate/shared-index-stale-check.py --selftest"
  "bash scripts/experiment-apparatus/e180-reading-shards.sh --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e182-single-device-long-run.sh --selftest"
  "bash scripts/experiment-apparatus/e181-landing-check-long-run.sh --selftest" "bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest"
  "bash scripts/experiment-apparatus/e56-sweep.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh --selftest"
  "bash scripts/experiment-apparatus/e183-capacity-cells.sh --selftest" "bash scripts/experiment-apparatus/e183-sweep-history-shards.sh --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --selftest"
  "python3 scripts/hosts/rocksdb-prebuilt.py --selftest"
  "python3 scripts/kb/milestone-close-owed-check.py --selftest" "python3 scripts/kb/move-directory-in-use-check.py --selftest"
  "python3 scripts/hosts/clean-session-scratch.py --selftest" "bash scripts/gate/staged-tree-worktree.sh --selftest"
  "bash scripts/crash/crash-point-replay-shard-configuration-check.sh --selftest" "python3 scripts/kb/path-successor.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune-server-load-simulation.py --selftest"
  "python3 .claude/gate.d/lib-lag-baseline.py --selftest"
  "python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py --selftest"
  "python3 scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/report.py --selftest"
  "python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/contrast_revision_6_against_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/contrast_segment_two_revision_10.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_tree_lines_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/cross_host_o3_check.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_10_placeholders.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_placeholders.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_11.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_8.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_11.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_11.py --selftest"
  "python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py --selftest"
)
# 包装写法的前缀（这一格文件头「只在临时 prune-server 上跑的自证」）与认它要包的那一句（被起的脚本源码里出现它就要包）
SELFTEST_TEMPORARY_SERVICE_PREFIX="python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --"
SELFTEST_TEMPORARY_SERVICE_MARK="selftest_service_problem"
# selftest_inner_command <表里的一条>：包装写法打 -- 后面那条原命令，别的原样打
selftest_inner_command() {
  if [[ "$1" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]]; then printf '%s\n' "${1#"$SELFTEST_TEMPORARY_SERVICE_PREFIX "}"; else printf '%s\n' "$1"; fi
}
# 要临时服务的自证登记表（TEMPORARY_SERVICE_TABLE，写法在它的文件头，读法是 .claude/hooks/lib_project_code_runs.py 的 read_temporary_service_table，这里不另写一份解析）：
#   这一道里凡是判「这份自证要不要临时服务」的地方（research-script-selftests 格 runner 表里该不该套包装、reuse-selftests 与 break-switch-replay 两格要不要先取二进制、
#   temporary-service-runners 格两边对齐）一律读这张表登记成 required 的，不看源码里有没有 $SELFTEST_TEMPORARY_SERVICE_MARK：
#   own-server 的两份源码里也有这几个字，它们的 --selftest 自己起真服务、裸跑不退 2，不许被要求套包装（runner 表里包着它们只为把二进制交下去，包不包都不判）。
#   被判的根是样本时表可以不放（当作空表）；真仓与 --staged 的树里表不在、读不了或有写坏的行，用到它的格判红。
#   弄坏开关 CODE_TOOLING_BREAK=temporary-service-table-ignored：退回看源码里有没有那几个字（改前的样子）：own-server 的被要求包装，表里是 required 而源码没写那几个字的认不出。
TEMPORARY_SERVICE_TABLE=".claude/hooks/selftest-temporary-service.tsv"
declare -A TEMPORARY_SERVICE_JUDGEMENT=()
# temporary_service_table_load：把被判的根里的表读进 TEMPORARY_SERVICE_JUDGEMENT（路径 → required 或 own-server）；读不成打出为什么与出路、返回 1
temporary_service_table_load() {
  local output key judgement
  TEMPORARY_SERVICE_JUDGEMENT=()
  if [[ ! -f "$TEMPORARY_SERVICE_TABLE" ]]; then
    judged_root_is_sample && return 0
    echo "  ✗ 没有要临时服务的自证登记表 $TEMPORARY_SERVICE_TABLE，哪份自证要套临时 prune-server 判不了"
    echo "     → 怎么办：表是新立的，写法在它的文件头；确认它进了这一棵树（--staged 判的是暂存区，要先 git add 这个文件）"
    return 1
  fi
  if ! output="$(python3 - "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$TEMPORARY_SERVICE_TABLE" <<'PY'
import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_project_code_runs", sys.argv[1])
library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(library)
rows, problems = library.read_temporary_service_table(sys.argv[2])
if problems:
    print("\n".join(problems))
    sys.exit(1)
for key, judgement in sorted(rows.items()):
    print(f"{key}\t{judgement}")
PY
  )"; then
    echo "  ✗ 要临时服务的自证登记表 $TEMPORARY_SERVICE_TABLE 读不了或有写坏的行："
    printf '%s\n' "$output" | sed 's/^/      /'   # gate-lint:detail
    echo "     → 怎么办：照上面逐行改表（一行 <路径> required 或 own-server 加 # 理由，写法在表的文件头）；判法在 .claude/hooks/lib_project_code_runs.py 的 temporary_service_table_at"
    return 1
  fi
  while IFS=$'\t' read -r key judgement; do
    [[ -n "$key" ]] && TEMPORARY_SERVICE_JUDGEMENT[$key]="$judgement"
  done <<<"$output"
  return 0
}
# temporary_service_is_required <路径>：表里这份脚本登记成 required（它的 --selftest 开跑先调客户端核对，没有临时服务就退 2）就返回 0
temporary_service_is_required() {
  if [[ "${CODE_TOOLING_BREAK:-}" == temporary-service-table-ignored ]]; then
    [[ -f "$1" ]] && grep -qF -- "$SELFTEST_TEMPORARY_SERVICE_MARK" "$1"
    return
  fi
  [[ "${TEMPORARY_SERVICE_JUDGEMENT[$1]:-}" == required ]]
}
# temporary_service_bare_runners：${runners[@]} 里裸写（没套包装写法）而表里是 required 的，一行一条打出
temporary_service_bare_runners() {
  local runner
  local -a bare_words=()
  for runner in "${runners[@]}"; do
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && continue
    read -r -a bare_words <<<"$runner"
    if temporary_service_is_required "${bare_words[1]:-}"; then printf '%s\n' "$runner"; fi
  done
  return 0
}
# code_tooling_prune_server_binary <编译目录>：打一份 prune-server 二进制的路径（research-script-selftests 与 reuse-selftests 两格各一趟取一次）：
#   PRUNE_SERVER_BINARY 设了（非空，绝对路径）用它（与 scripts/prune-server/prune_server_client_selftest.py 的 resolve_binary 认同一个变量）；
#   没设照 prune-server-selftest 格的编法（经 capped.sh、--offline、被判那棵树的 prune-server/）编进 <编译目录>/target（调用的那一格自己的临时目录，跑完随它删）。
#   取不到退 1，为什么打在标准错误
code_tooling_prune_server_binary() {
  if [[ -n "${PRUNE_SERVER_BINARY:-}" ]]; then
    [[ "$PRUNE_SERVER_BINARY" == /* && -x "$PRUNE_SERVER_BINARY" ]] || { echo "PRUNE_SERVER_BINARY=$PRUNE_SERVER_BINARY 不是可执行文件的绝对路径" >&2; return 1; }
    printf '%s\n' "$PRUNE_SERVER_BINARY"
    return 0
  fi
  if ! CARGO_TARGET_DIR="$1/target" \
      cargo build --release --offline --manifest-path "$ROOT/prune-server/Cargo.toml" --bin prune-server >"$1/prune-server-build.log" 2>&1; then
    echo "cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server 没成：$(tail -n 3 "$1/prune-server-build.log" | tr '\n' ' ' | cut -c1-300)" >&2
    return 1
  fi
  [[ -x "$1/target/release/prune-server" ]] || { echo "编完不见 $1/target/release/prune-server" >&2; return 1; }
  printf '%s\n' "$1/target/release/prune-server"
}
SELFTEST_RUNNERS_STARTED_ALONE=("python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest" "python3 scripts/agents/agent-watch.py --selftest"
  "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest")

# selftest_plan_runners <草稿目录>：table_runners 每一条一条判定请求（这一格文件头「按条复用」），经 scripts/gate/admission.py verdict-plan 算键、查库；
# 结论放进 selftest_decision[<命令>]（reuse、judge、refused）与 selftest_reason[<命令>]，计划文件是 <草稿目录>/plan；算不出返回 1、原因放进 selftest_plan_problem。
# 样本根另放了 .unit-verdict-sample/prior/（上一版的树）的，先拿上一版算键、照判绿记进样本自己的判定库。
# 弄坏开关 CODE_TOOLING_BREAK=selftest-key-whole-runner-table：每一条的键另加表里每一条自证的脚本（file:），一份脚本改了整张表都作废；
#   =selftest-judge-whole-guard：判法取整份 selftest-process-guard.py（改前的样子），每一条都拒算
SELFTEST_JUDGE_ENTRIES=(
  scripts/gate/selftest-process-guard.py::watch_judgement scripts/gate/selftest-process-guard.py::default_settings
  scripts/gate/selftest-process-guard.py::write_shims scripts/gate/selftest-process-guard.py::Watch.scan scripts/gate/selftest-process-guard.py::ExecTrace.poll
)
selftest_plan_runners() {
  local scratch="$1" runner tier name decision key reason entry plan_exit
  local -a words=() table_files=() judge_tokens=() plan_environment=() wrapper_token=()
  # 样本根带 .unit-verdict-sample/red-over-green：库里已有上一版照判绿记进来的同一把键，这一格的计划另设 ADMISSION_BREAK=verdict-lookup-skipped（不读库、一律现判）
  #   照样现判（这一格文件头「按条复用」记红、撤绿）；门禁样本根里 admission.py 才认这个开关
  judged_root_is_sample && [[ -f .unit-verdict-sample/red-over-green ]] && plan_environment=(env ADMISSION_BREAK=verdict-lookup-skipped)
  for entry in "${SELFTEST_JUDGE_ENTRIES[@]}"; do judge_tokens+=("judge:$entry"); done
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-judge-whole-guard ]] && judge_tokens=("judge:scripts/gate/selftest-process-guard.py")
  local admission="$STAGE_REPOSITORY/scripts/gate/admission.py" kind="$STAGE_NAME:research-script-selftests"
  selftest_plan_problem=""; selftest_plan_failure=()
  : >"$scratch/requests"
  if [[ "${CODE_TOOLING_BREAK:-}" == selftest-key-whole-runner-table ]]; then
    for runner in "${table_runners[@]}"; do read -r -a words <<<"$(selftest_inner_command "$runner")"; [[ -f "${words[1]:-}" ]] && table_files+=("file:${words[1]}"); done
  fi
  for runner in "${table_runners[@]}"; do
    read -r -a words <<<"$(selftest_inner_command "$runner")"
    wrapper_token=()
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && wrapper_token=("file:scripts/prune-server/prune_server_client_selftest.py")
    tier="$(awk -F'\t' -v script="${words[1]:-}" '$1 == script { print $2; exit }' .claude/hooks/selftest-registry.tsv 2>/dev/null)"
    { printf '%s\t%s' "$runner" "${words[1]:-${words[0]:-}}"
      printf '\t%s' "${judge_tokens[@]}" "judge-root:$STAGE_REPOSITORY" "line:command=$runner" "line:selftest-registration=${tier:-没登记}" \
        ${wrapper_token[@]+"${wrapper_token[@]}"} ${table_files[@]+"${table_files[@]}"}
      printf '\n'; } >>"$scratch/requests"
  done
  if judged_root_is_sample && [[ -d .unit-verdict-sample/prior ]]; then
    if python3 "$admission" verdict-plan "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/requests" "$scratch/seed.plan" >"$scratch/seed.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版的树上算不出键（$(tail -n 2 "$scratch/seed.out" | tr '\n' ' ' | cut -c1-300)）"
      selftest_plan_failure=(verdict-plan "$plan_exit" "$scratch/seed.out"); return 1
    fi
    cut -f1 "$scratch/requests" >"$scratch/seed.names"
    if python3 "$admission" verdict-record "$ROOT/.unit-verdict-sample/prior" "$kind" "$scratch/seed.plan" "$scratch/seed.names" --how "样本：上一版" >"$scratch/seed.record" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      selftest_plan_problem="样本：上一版那几条没记进库（$(tail -n 2 "$scratch/seed.record" | tr '\n' ' ')）"
      selftest_plan_failure=(verdict-record "$plan_exit" "$scratch/seed.record"); return 1
    fi
    echo "  · 样本：上一版（.unit-verdict-sample/prior）的 $(grep -c . "$scratch/seed.names") 条自证照判绿记进这一版样本的判定库"
  fi
  if ${plan_environment[@]+"${plan_environment[@]}"} python3 "$admission" verdict-plan "$ROOT" "$kind" "$scratch/requests" "$scratch/plan" >"$scratch/plan.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
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
# plan-failure 退 75 是 prune-server 不可用（不是复用失灵）：记 selftest_service_unavailable=1，这一格退 75（这一格文件头「按条复用」）
selftest_plan_failure_verdict() {
  local verdict_exit
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" plan-failure "$STAGE_NAME 的 research-script-selftests 格" "$@"; then return 0; else verdict_exit=$?; fi
  if ((verdict_exit == 75)); then selftest_service_unavailable=1; else selftest_plan_red=1; fi
}

# 记库时交给 admission.py verdict-record 的跑法（当场记的一条与收尾那一趟同一句）
SELFTEST_RECORD_HOW="经 scripts/gate/selftest-process-guard.py 起"

# selftest_outcome <序号> <命令>：读 <逐条目录>/<序号>.exit 与 .guard，定这一条的结局（这一格文件头「判据」「按条复用」）；回读与当场记库同一份判法。
# 放进 selftest_rc（照弄坏开关 runner-exit-swallowed、selftest-not-run-as-pass 改过的退出码）、selftest_guard_verdict（守卫判定的头一行）与 selftest_result：
#   not-run（退 77）、red（退非 0）、green-record（现判而判绿、守卫给了判定：记库；盯了而 execve 跟踪没跑的不在这里）、green（判绿而不记）
selftest_outcome() {
  selftest_rc="$(<"$scratch/$1.exit")"
  [[ "${CODE_TOOLING_BREAK:-}" == runner-exit-swallowed ]] && selftest_rc=0
  [[ "$selftest_rc" == 77 && "${CODE_TOOLING_BREAK:-}" == selftest-not-run-as-pass ]] && selftest_rc=0
  selftest_guard_verdict=""
  [[ -f "$scratch/$1.guard" ]] && selftest_guard_verdict="$(head -n 1 "$scratch/$1.guard")"
  if [[ "$selftest_rc" == 77 ]]; then
    selftest_result=not-run
  elif [[ "$selftest_rc" != 0 ]]; then
    selftest_result=red
  elif [[ "${selftest_decision[$2]:-}" == judge && ( "$selftest_guard_verdict" == not-watched$'\t'* || ( "$selftest_guard_verdict" == watched$'\t'* && "$selftest_guard_verdict" != *$'\ttrace=no'* ) ) ]]; then
    selftest_result=green-record
  else
    selftest_result=green
  fi
}

# selftest_record_one <序号> <命令>：一条自证跑完当场照键记（这一格文件头「按条复用」一条判完就记），在起它的那个后台作业里、退出码写完之后调：
# 判绿该记的记绿，判红的记红、撤同一把键原有的绿（弄坏开关 red-not-recorded 下不记红），退 77、复用、拒算的不记。经 admission.py verdict-record 写
# （每把键一份文件，同目录临时文件写完 fsync 再改名换上），计划文件用不带复用哨兵状态的那个名字（<算键目录>/plan-per-entry 指向 plan：哨兵在收尾那一趟判一次）。
# 输出写 <序号>.record、退出码写 <序号>.record-exit。中途被停，已经跑完的那几条都在库里，下一趟它们复用、只补没判的。
# 弄坏开关 CODE_TOOLING_BREAK=selftest-recorded-at-end：不当场记，只在收尾一次记（改前的样子：中途被停，判完的全丢）
selftest_record_one() {
  local names="$scratch/$1.record-names" record_exit
  local -a lists=()
  ((selftest_plan_ok)) || return 0
  [[ "${CODE_TOOLING_BREAK:-}" == selftest-recorded-at-end ]] && return 0
  selftest_outcome "$1" "$2"
  case "$selftest_result" in
    green-record) printf '%s\n' "$2" >"$names"; lists=("$names") ;;
    red)
      [[ "${CODE_TOOLING_BREAK:-}" == red-not-recorded ]] && return 0
      printf '%s\n' "$2" >"$names"; : >"$names.green"; lists=("$names.green" --red-names "$names") ;;
    *) return 0 ;;
  esac
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-record "$ROOT" "$STAGE_NAME:research-script-selftests" "$selftest_plan_scratch/plan-per-entry" "${lists[@]}" \
       --how "$SELFTEST_RECORD_HOW" >"$scratch/$1.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  printf '%s\n' "$record_exit" >"$scratch/$1.record-exit"
}

# selftest_record_verdicts <草稿目录> <判绿的命令文件> <判红的命令文件>：现判而判绿的那几条照键记进库，现判而判红的（自证退非 0、不是 77）照同一把键记红、
# 撤掉这把键原有的绿（这一格文件头「按条复用」记红、撤绿、下趟现判），经 admission.py verdict-record 的 --red-names；拒算的没有键，不记。
# 记不下只报、不判红，复用哨兵报红（强制现判那一条判定不稳）的经 plan-failure 判红。弄坏开关 CODE_TOOLING_BREAK=red-not-recorded 下不交 --red-names（改前的样子：只记绿）
selftest_record_verdicts() {
  local record_exit
  local -a red_option=()
  [[ -s "$2" || -s "$3" ]] || return 0
  [[ -s "$3" && "${CODE_TOOLING_BREAK:-}" != red-not-recorded ]] && red_option=(--red-names "$3")
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-record "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/plan" "$2" ${red_option[@]+"${red_option[@]}"} \
       --how "$SELFTEST_RECORD_HOW" >"$1/record.out" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    [[ -s "$2" ]] && echo "  · 判绿记库：$(tail -n 1 "$1/record.out")"
    ((${#red_option[@]})) && echo "  · 判红记库：$(grep -c . "$3") 条记红、撤掉同一把键原有的绿，下一趟这几条现判（$(tail -n 1 "$1/record.out")）"
  else
    echo "  ! 判定记库：admission.py verdict-record 没成（$(tail -n 2 "$1/record.out" | tr '\n' ' ')），下一趟这几条照跑"
    selftest_plan_failure_verdict verdict-record "$record_exit" "$1/record.out"
  fi
}

# selftest_reread_after_red <草稿目录>：样本根带 .unit-verdict-sample/red-over-green 时（这一格文件头「按条复用」），判红记库之后不设 ADMISSION_BREAK 再算一次表的计划，
# 打复用几条、现判几条，现判的逐条写原因（记红盖掉了绿：红的那一条答「上次判红」；没盖掉：它答复用）
selftest_reread_after_red() {
  [[ -f .unit-verdict-sample/red-over-green ]] || return 0
  if python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" verdict-plan "$ROOT" "$STAGE_NAME:research-script-selftests" "$1/requests" "$1/reread.plan" >"$1/reread.out" 2>&1; then
    echo "  · 样本档：判红记库之后再算一次计划：复用 $(awk -F'\t' '$2 == "reuse" { count++ } END { print count + 0 }' "$1/reread.out") 条、现判 $(awk -F'\t' '$2 == "judge" { count++ } END { print count + 0 }' "$1/reread.out") 条"
    awk -F'\t' '$2 == "judge" { printf "    现判「%s」：%s\n", $1, $4 }' "$1/reread.out"
  else
    echo "  ✗ 样本档：判红记库之后再算计划没成（$(tail -n 2 "$1/reread.out" | tr '\n' ' ')）"
    echo "     → 怎么办：看括号里 admission.py verdict-plan 报的错，修好样本或判定库，再单跑这一份样本"
  fi
}

# selftest_runner_table_load：取这一趟的 runner 表（research-script-selftests 与 temporary-service-runners 两格共用）：
#   设全局 runners（表）、table_source（表取自哪，报错时写）、started_alone（单独起的那几条）；被判的根是样本且有 .selftest-runners 时，换成样本根里的（单独起的那一路不走）
selftest_runner_table_load() {
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
}

cell_research_script_selftests() {
selftest_runner_table_load
# 逐项点名（这一道「逐项点名」一节）：--item 点到这一格的，表里只留起那几份脚本的自证（按包装 -- 后面那条原命令的第二个词认），单独起的那一路只留还在表里的；
# 键、判法、复用与记库照整格的走法，点名单跑一条与它在整格里跑的判定相同
if [[ -n "${CODE_TOOLING_PICKED[research-script-selftests]:-}" ]]; then
  declare -A picked_selftest_scripts=()
  while IFS= read -r line; do [[ -n "$line" ]] && picked_selftest_scripts[$line]=1; done <<<"${CODE_TOOLING_PICKED[research-script-selftests]}"
  picked_selftest_runners=(); picked_alone=()
  for runner in "${runners[@]}"; do
    read -r -a words <<<"$(selftest_inner_command "$runner")"
    if [[ -n "${words[1]:-}" && -n "${picked_selftest_scripts[${words[1]}]+set}" ]]; then picked_selftest_runners+=("$runner"); fi
  done
  for runner in ${started_alone[@]+"${started_alone[@]}"}; do
    for line in ${picked_selftest_runners[@]+"${picked_selftest_runners[@]}"}; do [[ "$line" == "$runner" ]] && picked_alone+=("$runner"); done
  done
  echo "  · 只跑点名的 ${#picked_selftest_scripts[@]} 份脚本的自证：表里 ${#runners[@]} 条留 ${#picked_selftest_runners[@]} 条（表取自 $table_source）"
  runners=(${picked_selftest_runners[@]+"${picked_selftest_runners[@]}"}); started_alone=(${picked_alone[@]+"${picked_alone[@]}"})
  table_source="$table_source里 --item 点名的那几条"
fi
# 样本根的 .selftest-environment（一行一条，空行与 # 开头的行不算）：`名字=值` 导出、`!名字` 摘掉，在这一格开跑前套上，当作这一格被起的那一刻外面的环境；
# 只在被判的根是样本时认，真仓与 --staged 下放了它也不认。样本借它造外面的环境（设上或摘掉一个变量）
if judged_root_is_sample && [[ -f .selftest-environment ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    case "$line" in
      '!'*) unset "${line#!}" ;;
      *=*) export "${line%%=*}=${line#*=}" ;;
    esac
  done < .selftest-environment
fi
# 样本里的假自证要读样本那一份库（在临时 prune-server 里样本那一份命名空间）时，从 CODE_TOOLING_SAMPLE_STAGE_REPOSITORY 取这一道所在仓里的 admission.py 与客户端；只给样本设
judged_root_is_sample && export CODE_TOOLING_SAMPLE_STAGE_REPOSITORY="$STAGE_REPOSITORY"
if ((${#runners[@]} == 0)); then
  echo "  ✗ runner 表一条都没有（取自 $table_source）"
  echo "     → 怎么办：跑到 0 条自证而报绿，与判过了一模一样；表写在这一道的 SELFTEST_RUNNERS，样本写在样本根的 .selftest-runners。"
  exit 1
fi
# 只在临时 prune-server 上跑的自证要写成包装写法（这一格文件头「只在临时 prune-server 上跑的自证」）：要临时服务的表里登记成 required 而裸写的，整格判红、一条都不起
temporary_service_table_load || exit 1
mapfile -t bare_temporary_service < <(temporary_service_bare_runners)
if ((${#bare_temporary_service[@]})) && [[ "${CODE_TOOLING_BREAK:-}" != bare-temporary-service-selftest-accepted ]]; then
  echo "  ✗ 这几条自证只在临时 prune-server 上跑（$TEMPORARY_SERVICE_TABLE 登记成 required：开跑先调 $SELFTEST_TEMPORARY_SERVICE_MARK，裸跑退 2），runner 表里却是裸写的（取自 $table_source）："
  for runner in "${bare_temporary_service[@]}"; do echo "      「$runner」该写成「$SELFTEST_TEMPORARY_SERVICE_PREFIX $runner」"; done   # gate-lint:detail
  echo "     → 怎么办：照上面逐条改成包装写法（不写 --server-binary：二进制由这一格一趟取一份交给每一条）；这一格一条都没起"
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
: >"$selftest_plan_scratch/red.names"
table_runners=("${runners[@]}")
declare -A selftest_decision=() selftest_reason=()
selftest_plan_ok=0; selftest_judged=0; selftest_reused=0; selftest_refused=0; selftest_plan_red=0; selftest_service_unavailable=0; selftest_plan_failure=()
if selftest_plan_runners "$selftest_plan_scratch"; then
  selftest_plan_ok=1
  # 一条判完就记（selftest_record_one）用的计划文件名：指向同一份计划，旁边没有复用哨兵状态（哨兵只在收尾那一趟判）
  ln -s plan "$selftest_plan_scratch/plan-per-entry"
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
  if ((selftest_service_unavailable)); then
    rm -rf -- "${selftest_plan_scratch:?}"
    echo "  ✗ research 脚本的自证这一格没判：prune-server 不可用（上面那一句），表里 ${#table_runners[@]} 条一条都没起，这一格退 75"
    echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
    exit 75
  fi
fi
if ((${#runners[@]} == 0)); then
  rm -rf -- "${selftest_plan_scratch:?}"
  echo "  ✓ research 脚本的自证都通过（表取自 $table_source；表里 ${#table_runners[@]} 条都有键相同的判定，全部复用，这一格一条都没起：现判 0、复用 ${selftest_reused}、拒算 0、本次未跑 0）"
  exit 0
fi

if ! thread_cap="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），自证分批的并行度定不了，一条自证都没跑"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
  exit 1
fi
batch_size=$((thread_cap - ${#alone_set[@]}))
((batch_size >= 1)) || batch_size=1

# 这一趟要起的里面有包装写法的：开跑前取一次 prune-server 二进制，交给每一条（这一格文件头「只在临时 prune-server 上跑的自证」）
selftest_server_binary=""; selftest_wrapped_count=0
for runner in "${runners[@]}"; do [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] && selftest_wrapped_count=$((selftest_wrapped_count + 1)); done
if ((selftest_wrapped_count)); then
  if ! selftest_server_binary="$(code_tooling_prune_server_binary "$selftest_plan_scratch" 2>"$selftest_plan_scratch/server-binary.err")"; then
    echo "  ✗ 取不到 prune-server 二进制（$(tail -n 3 "$selftest_plan_scratch/server-binary.err" | tr '\n' ' ' | cut -c1-400)），要包在临时服务底下起的 $selftest_wrapped_count 条一条都没起"
    echo "     → 怎么办：修好 prune-server/ 的编译（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）再跑这一格；二进制不另指，这一格自己编一份"
    exit 1
  fi
  echo "  · 临时 prune-server 的二进制：$selftest_server_binary（这一趟取一次，交给包装写法的 $selftest_wrapped_count 条）"
fi
scratch="$(mktemp -d)" || { echo "  ✗ 建不了放逐条输出与退出码的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条自证都没跑。"; exit 1; }
started_pids=()
selftest_process_guard="$STAGE_REPOSITORY/scripts/gate/selftest-process-guard.py"
# start_selftest <表里的序号> <命令>：后台起一条，输出写 <序号>.out、退出码写 <序号>.exit、守卫的判定写 <序号>.guard，进程号记进 started_pids；
# 退出码写完，同一个后台作业里当场照键记这一条（selftest_record_one）
start_selftest() {
  # runner 表每条是「解释器 脚本 参数」，按空白切词起，与原来逐条顺序跑时同一个起法；经守卫起：登记过的盯着它的进程树，别的守卫原样 exec
  local wrap_prefix=() inner
  # 包装写法：包装在守卫外面起、二进制交这一趟取的那一份，守卫盯 -- 后面那条原命令（这一格文件头「只在临时 prune-server 上跑的自证」）
  inner="$(selftest_inner_command "$2")"
  [[ "$inner" != "$2" ]] && wrap_prefix=(python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary "$selftest_server_binary" --)
  if [[ "${CODE_TOOLING_BREAK:-}" == guard-skipped ]]; then
    { printf 'watched\tfindings=0\tscan=no\texamined=0\texit=-\n' > "$scratch/$1.guard"; ${wrap_prefix[@]+"${wrap_prefix[@]}"} $inner > "$scratch/$1.out" 2>&1; printf '%s\n' "$?" > "$scratch/$1.exit"
      selftest_record_one "$1" "$2"; } &
  else
    { ${wrap_prefix[@]+"${wrap_prefix[@]}"} python3 "$selftest_process_guard" --registry-root "$ROOT" --verdict-file "$scratch/$1.guard" -- $inner > "$scratch/$1.out" 2>&1
      printf '%s\n' "$?" > "$scratch/$1.exit"
      selftest_record_one "$1" "$2"; } &
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

collected=0; failed_count=0; uncollected=(); watched_count=0; unwatched=(); unjudged=(); untraced=(); not_run_runners=()
for index in "${!runners[@]}"; do
  runner="${runners[$index]}"
  if [[ ! -f "$scratch/$index.exit" ]]; then uncollected+=("$runner"); continue; fi
  collected=$((collected + 1))
  selftest_outcome "$index" "$runner"
  rc="$selftest_rc"; guard_verdict="$selftest_guard_verdict"
  if [[ -f "$scratch/$index.record-exit" && "$(<"$scratch/$index.record-exit")" != 0 ]]; then
    echo "  ! 「$runner」跑完当场记库没成（$(tail -n 2 "$scratch/$index.record" | tr '\n' ' ')），收尾那一趟再记一次"
  fi
  case "$guard_verdict" in
    watched$'\t'*)
      watched_count=$((watched_count + 1))
      # 判定文件末尾的 trace=no：守卫没能经 strace 跟踪 execve（strace 不在或起不了），这一条只有采样与替身那一半，不当绿报
      [[ "$guard_verdict" == *$'\ttrace=no'* ]] && untraced+=("$runner") ;;
    not-watched$'\t'*) unwatched+=("$runner") ;;
    *) unjudged+=("$runner") ;;
  esac
  if [[ "$selftest_result" == not-run ]]; then
    # 本次未跑：不判绿也不判红、不记库，下面逐条列名、报进汇总（这一格文件头「判据」）
    not_run_runners+=("$runner")
  elif [[ "$selftest_result" == red ]]; then
    failed_count=$((failed_count + 1))
    printf '%s\n' "$runner" >>"$selftest_plan_scratch/red.names"   # 判红：跑完当场已照键记红、撤绿（selftest_record_one），收尾 selftest_record_verdicts 连同复用哨兵再交一次
    echo "  ✗ $runner 没过（退出码 $rc）："
    sed 's/^/    /' "$scratch/$index.out"   # gate-lint:detail
    ((selftest_plan_ok)) && echo "    这一条这一趟为什么现判：${selftest_reason[$runner]:-（计划里没有它）}"
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
  elif [[ "$selftest_result" == green-record ]]; then
    # 现判而判绿、守卫给了判定（盯了而 execve 跟踪没跑的不记：那一条只验了一半）：跑完当场已记（selftest_record_one），收尾连同复用哨兵再交一次
    printf '%s\n' "$runner" >>"$selftest_plan_scratch/green.names"
  fi
done
rm -rf "${scratch:?}"
if ((selftest_plan_ok)); then
  selftest_record_verdicts "$selftest_plan_scratch" "$selftest_plan_scratch/green.names" "$selftest_plan_scratch/red.names"
  ((failed_count)) && selftest_reread_after_red "$selftest_plan_scratch"
fi
if ((collected != ${#runners[@]})); then
  echo "  ✗ runner 表派出去 ${#runners[@]} 条，只收回 $collected 份退出码；没收回的：${uncollected[*]}"
  echo "     → 怎么办：临时目录在跑的过程中被删了或写不进去；看 \$TMPDIR 再重跑这一格，收不全的一轮不算判过。"
  exit 1
fi
if ((${#unjudged[@]})); then
  echo "  ✗ scripts/gate/selftest-process-guard.py 没给 ${#unjudged[@]} 条写判定（盯没盯说不清）：$(printf '「%s」' "${unjudged[@]}")"
  echo "     → 怎么办：单跑 python3 scripts/gate/selftest-process-guard.py --verdict-file <文件> -- <那条命令> 看它报什么；判定不全的一轮不算判过。"
  exit 1
fi
not_run_text=""
if ((${#not_run_runners[@]})); then
  not_run_text="：$(printf '「%s」' "${not_run_runners[@]}")"
  echo "  · 本次未跑 ${#not_run_runners[@]} 条（自证退 77：自己造不出前提，不算通过、不记库，下一趟照跑）$not_run_text"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for runner in "${not_run_runners[@]}"; do printf '%s\n' "$STAGE_NAME 的 research-script-selftests 格：$runner 退 77，本次未跑" >> "$GATE_NOT_RUN_FILE"; done
  fi
fi
if ((failed_count)); then
  echo "  ✗ runner 表 ${#runners[@]} 条里 $failed_count 条没过（上面按表次序逐条列着）；本次未跑 ${#not_run_runners[@]} 条$not_run_text"
  echo "     → 怎么办：逐条按那份自证给的下一步修；只在并行下红、单跑绿的，是几份自证抢同一份状态，照这一格文件头那一段办。"
  exit 1
fi
if ((selftest_service_unavailable)); then
  echo "  ✗ research 脚本的自证这一格：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75"
  echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
  exit 75
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
selftest_counts="表里 ${#table_runners[@]} 条：现判 ${selftest_judged}、复用 ${selftest_reused}、拒算 ${selftest_refused}、本次未跑 ${#not_run_runners[@]}"
((selftest_plan_ok)) || selftest_counts="表里 ${#table_runners[@]} 条，按条复用算不出、全部现判，本次未跑 ${#not_run_runners[@]}"
if ((${#not_run_runners[@]} == ${#runners[@]} && selftest_reused == 0)); then
  echo "  ! research 脚本的自证这一趟一条都没判过：起的 ${#runners[@]} 条都退 77（本次未跑$not_run_text），也没有复用的；这一格记本次未跑，不记通过"
  echo "     → 怎么办：照那几条自证打的原因造出前提（机器闲一点再跑这类），再跑 bash .claude/gate.d/code-tooling.sh --check research-script-selftests"
  exit 77
fi
selftest_verdict_text="research 脚本的自证都通过"
((${#not_run_runners[@]})) && selftest_verdict_text="research 脚本的自证判了的都通过、本次未跑 ${#not_run_runners[@]} 条$not_run_text"
echo "  ✓ ${selftest_verdict_text}（这一格跑了 ${#runners[@]} 条，表取自 $table_source；${selftest_counts}；单独起 ${#alone_pids[@]} 条，其余 $((${#runners[@]} - ${#alone_pids[@]})) 条按每批至多 $batch_size 条并行、共 $batches 批；登记过的 $watched_count 条经 selftest-process-guard.py 盯着进程树、经 strace 记下每一次 execve，没起真 cargo / rustc / qemu / herd7 / verus（版本查询除外）与真清场，本次未盯 ${#unwatched[@]} 条$unwatched_text$untraced_text）"
}

# ── 格 temporary-service-runners：runner 表里套了临时服务包装的自证，与要临时服务的表里登记成 required 的两边对齐
#
# 判据：.claude/hooks/selftest-temporary-service.tsv（读法在 TEMPORARY_SERVICE_TABLE 的注释，一份表、认 required 与 own-server 两种）与 runner 表（SELFTEST_RUNNERS；样本根的 .selftest-runners 换掉它，
#   和 research-script-selftests 格同一个取法 selftest_runner_table_load）逐条对一遍，对不上的逐条点名，整格判红：
#   ① 表里 required 而 runner 表里裸写（该写成包装写法；research-script-selftests 格也在起任何一条之前拒，同一个函数 temporary_service_bare_runners）；
#   ② 表里 required 而 runner 表里一条都没有（没人跑它的自证）；
#   ③ runner 表里写成包装写法而表里没有这一行（包装只给表里登记的套）。
#   own-server 的两份不许被要求包装：runner 表里包着它们只为把二进制交下去，包不包都不判，源码里有没有 $SELFTEST_TEMPORARY_SERVICE_MARK 也不看。
#   表不在（样本根可以不放，当作空表）、读不了或有写坏的行：红，出路写在 temporary_service_table_load 里。
# 样本：temporary-service-runners-red（样本根的表四行：一份 required 而 runner 表裸写、一份 required 而 runner 表里没有、一份 required 且写成包装写法、一份 own-server 包着；
#   runner 表另有一条包装写法而表里没有这一行：①②③ 各一处，必须判红并逐条点名，另两份不点名）；
#   temporary-service-runners-green（required 的包着、own-server 的一份包着一份裸写，裸写那份源码里有那几个字，runner 表里另有一条不在表里的裸写：必须判绿，报 required、own-server 与包装写法各几份）。
# 弄坏开关 CODE_TOOLING_BREAK=temporary-service-alignment-unchecked：不判两边对不对得上（只有 research-script-selftests 格看裸写，改前的样子），temporary-service-runners-red 判绿（期望退出 1，实测 0）；
#   =temporary-service-table-ignored（文件头开关表）：认 required 退回看源码，temporary-service-runners-red 里「该写成」那条 want 找不到，temporary-service-runners-green 判红（own-server 裸写那份被点名）。
cell_temporary_service_runners() {
  selftest_runner_table_load
  temporary_service_table_load || exit 1
  local runner script
  local -a words=() bare=() unregistered=() absent=()
  local -A runner_scripts=()
  local required_count=0 own_server_count=0 wrapped_count=0 problem_count
  mapfile -t bare < <(temporary_service_bare_runners)
  for runner in "${runners[@]}"; do
    read -r -a words <<<"$(selftest_inner_command "$runner")"
    script="${words[1]:-空}"
    runner_scripts[$script]=1
    [[ "$runner" == "$SELFTEST_TEMPORARY_SERVICE_PREFIX "* ]] || continue
    wrapped_count=$((wrapped_count + 1))
    [[ -n "${TEMPORARY_SERVICE_JUDGEMENT[$script]:-}" ]] || unregistered+=("$runner")
  done
  while IFS= read -r script; do
    [[ -n "$script" ]] || continue
    if [[ "${TEMPORARY_SERVICE_JUDGEMENT[$script]}" == required ]]; then
      required_count=$((required_count + 1))
      [[ -n "${runner_scripts[$script]+set}" ]] || absent+=("$script")
    else
      own_server_count=$((own_server_count + 1))
    fi
  done < <(printf '%s\n' "${!TEMPORARY_SERVICE_JUDGEMENT[@]}" | sort)
  problem_count=$((${#bare[@]} + ${#absent[@]} + ${#unregistered[@]}))
  if ((problem_count)) && [[ "${CODE_TOOLING_BREAK:-}" != temporary-service-alignment-unchecked ]]; then
    echo "  ✗ runner 表与要临时服务的表 $TEMPORARY_SERVICE_TABLE 有 $problem_count 处对不上（runner 表 ${#runners[@]} 条取自 $table_source，其中套包装的 $wrapped_count 条；表里 required $required_count 份、own-server $own_server_count 份）："
    for runner in ${bare[@]+"${bare[@]}"}; do echo "      「$runner」裸写，表里是 required：该写成「$SELFTEST_TEMPORARY_SERVICE_PREFIX $runner」"; done   # gate-lint:detail
    for script in ${absent[@]+"${absent[@]}"}; do echo "      $script：表里是 required，runner 表里一条都没有（没人跑它的自证）：补上「$SELFTEST_TEMPORARY_SERVICE_PREFIX <解释器> $script --selftest」"; done   # gate-lint:detail
    for runner in ${unregistered[@]+"${unregistered[@]}"}; do echo "      「$runner」写成了包装写法，表里没有这一行：包装只给表里登记的套"; done   # gate-lint:detail
    echo "     → 怎么办：照上面逐条改 runner 表（这一道的 SELFTEST_RUNNERS）：required 的写成包装写法、缺的补上；包装写法而表里没有的，要么这份自证确实只在临时 prune-server 上跑（先在 .claude/hooks/selftest-registry.tsv 登记，再往 $TEMPORARY_SERVICE_TABLE 加一行 required 或 own-server），要么去掉包装；own-server 的不要求包装"
    exit 1
  fi
  echo "  ✓ 要临时服务的表与 runner 表两边对齐：表里 required ${required_count} 份都在 runner 表里写成包装写法，own-server ${own_server_count} 份不要求包装；runner 表 ${#runners[@]} 条，套包装的 ${wrapped_count} 条都在表里"
}

# ── 格 temporary-service-wrapper-form：要临时服务的自证的整条写法，各处逐字相同、包装的解析收得下、不带尖括号占位的 --server-binary
#
# 判据：整条写法散在几处（读表的代码都跳过 # 行，没有别的检查绑住它），这一格逐处取出来与包装自己认的用法比；只读文件，不起 prune-server、不起 cargo。
#   比的对象是包装 scripts/prune-server/prune_server_client_selftest.py：文件头用法行「<解释器> <包装> --with-temporary-service [--server-binary <…>] -- <命令> [<参数>…]」去掉方括号里的可选项再接上 --，
#   就是整条该有的前缀（不带 --server-binary：任务服务把整条认成会编的、给编译目录，自己编了的二进制才另加 --server-binary，写在整条之外）；
#   with_temporary_service 里给 unknown 赋值的那一句（认不出的参数退 2）原样取出来，对每一处的选项求值，这里不另抄一份解析。
#   取包装写法的法子：找「<路径以 prune_server_client_selftest.py 结尾的词> --with-temporary-service」，往后取到第一个单独的 -- 为止，前面粘着的解释器词连在一起当前缀；
#   后面没有单独 -- 的是行文里提到它，不算整条；选项恰好是用法行里的 [--server-binary <…>] 的是用法写法，也不算整条。
#   取的七处（红行点名是哪一处、文件与行、原文）：
#     登记表文件头：.claude/hooks/selftest-temporary-service.tsv 的注释行；
#     拒绝说明：.claude/hooks/lib_project_code_runs.py 的 temporary_service_command 对一个 .py 键与一个 .sh 键各打的整条（载入库调它，不读源码）；
#     骨架：scripts/agents/dispatch-prompt.py 里给 TEMPORARY_SERVICE_WRAPPER 赋的字符串；
#     客户端文件头：scripts/prune-server/prune_server_client.py 的文件头；
#     自证登记表：.claude/hooks/selftest-registry.tsv 的注释行，加上要临时服务的表里登记成 required 的那几行的理由（own-server 行的理由说它自己怎么起真服务，不判）；
#     runner 表：这一道的 SELFTEST_RUNNERS（样本根的 .selftest-runners 换掉它）里套了包装的每一条；
#     门禁的包装前缀常量：SELFTEST_TEMPORARY_SERVICE_PREFIX。
#   判红四种：选项里带尖括号（`--server-binary <编好的 prune-server>` 这类占位，照抄进 shell 尖括号被当成输入重定向、命令起不来）；包装的解析认不出这些选项；
#   前缀与整条该有的逐字不同（含写了 --server-binary）；某一处一条整条写法都没有。包装文件读不出用法行或认选项的那一句时，整格红、点名包装。
#   表不在、读不了或有写坏的行：红，出路写在 temporary_service_table_load 里。
# 样本：temporary-service-wrapper-form-green（七处都对：自证登记表里 own-server 行的理由带 --server-binary <…> 占位，不判；客户端文件头另有一行用法写法，不算整条）；
#   -binary-placeholder-red（自证登记表里 required 行的理由写成带 --server-binary <编好的 prune-server> 占位，只点名那一处）；
#   -prefix-differs-red（骨架的包装前缀少了 scripts/，只点名骨架）；
#   -unparsable-red（runner 表一条写了 --memory 4G，包装认不出，只点名 runner 表）；
#   -place-missing-red（拒绝说明打的是裸命令、没有包装，两个键各点名一次）。
# 弄坏开关 CODE_TOOLING_BREAK=temporary-service-form-unchecked：有问题也不判红（改前的样子：没有检查绑住这几处），四份红样本判绿（期望退出 1，实测 0）。
cell_temporary_service_wrapper_form() {
  selftest_runner_table_load
  temporary_service_table_load || exit 1
  local output status script required_text runner_text kind text canonical="" tally=""
  local -a problem_lines=()
  required_text="$(for script in "${!TEMPORARY_SERVICE_JUDGEMENT[@]}"; do [[ "${TEMPORARY_SERVICE_JUDGEMENT[$script]}" == required ]] && printf '%s\n' "$script"; done | sort)"
  runner_text="$(printf '%s\n' ${runners[@]+"${runners[@]}"})"
  output="$(python3 - "$SELFTEST_TEMPORARY_SERVICE_PREFIX" "$required_text" "$runner_text" "${table_source%%（*}" <<'PY_WRAPPER_FORM'
import ast, importlib.util, re, sys

sys.dont_write_bytecode = True   # 载入被判的钩子库不在样本根与真仓里留 __pycache__
constant_prefix, required_text, runner_text, runner_label = sys.argv[1:5]
WRAPPER_PATH = "scripts/prune-server/prune_server_client_selftest.py"
CLIENT_PATH = "scripts/prune-server/prune_server_client.py"
DISPATCH_PATH = "scripts/agents/dispatch-prompt.py"
LIBRARY_PATH = ".claude/hooks/lib_project_code_runs.py"
TABLE_PATH = ".claude/hooks/selftest-temporary-service.tsv"
REGISTRY_PATH = ".claude/hooks/selftest-registry.tsv"
USAGE_LINE = re.compile(r"^[ \t]*(?P<head>\S+[ \t]+\S*prune_server_client_selftest\.py[ \t]+--with-temporary-service)(?:[ \t]+\[[^\]\n]*\])?[ \t]+--[ \t]", re.M)
ANCHOR = re.compile(r"(?:(?P<launcher>\S+)[ \t]+)?(?P<path>\S*prune_server_client_selftest\.py)[ \t]+--with-temporary-service(?P<rest>[^\n]*)")
LEADING = "\"'`(["
CLOSERS = "\"'`」』”）)"
problems = []
tallies = {}


def fail(place, location, found, reason):
    problems.append((place, location, found, reason))


def finish():
    print(f"CANONICAL\t{canonical}")
    print("TALLY\t" + "、".join(f"{place} {wholes} 条" + (f"（另有用法行 {usage} 行）" if usage else "") for place, (wholes, usage, _mentions) in tallies.items()))
    for place, location, found, reason in problems:
        print(f"PROBLEM\t{place}（{location}）：原文「{found}」，{reason}")
    sys.exit(1 if problems else 0)


def read_text(path, place):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    except (OSError, UnicodeDecodeError) as error:
        fail(place, path, "（读不到）", f"{type(error).__name__}：{error}")
        return None


def parse_tree(text, path, place):
    if text is None:
        return None
    try:
        return ast.parse(text)
    except SyntaxError as error:
        fail(place, path, "（语法错）", f"{error}")
        return None


# 包装自己的用法行与认选项的那一句：整条该是什么、认得下哪些选项都从这里取，这一格不另抄一份
canonical = "（读不出）"
unknown_of = None
wrapper_source = read_text(WRAPPER_PATH, "包装")
wrapper_tree = parse_tree(wrapper_source, WRAPPER_PATH, "包装")
if wrapper_tree is not None:
    usage = USAGE_LINE.search(ast.get_docstring(wrapper_tree, clean=False) or "")
    if usage is None:
        fail("包装", WRAPPER_PATH, "（文件头里找不到用法行）", "文件头要有一行「<解释器> <包装> --with-temporary-service -- <命令> [<参数>…]」（可选项写在方括号里），这一格从它取整条的前缀")
    else:
        canonical = usage.group("head") + " --"
    for function in ast.walk(wrapper_tree):
        if isinstance(function, ast.FunctionDef) and function.name == "with_temporary_service":
            for statement in ast.walk(function):
                if isinstance(statement, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "unknown" for target in statement.targets):
                    unknown_code = compile(ast.Expression(statement.value), WRAPPER_PATH, "eval")
                    unknown_of = lambda options, code=unknown_code: eval(code, {"options": options})
    if unknown_of is None:
        fail("包装", WRAPPER_PATH, "（找不到）", "with_temporary_service 里找不到给 unknown 赋值的那一句，认不出的选项怎么判这一格读不到")
if wrapper_tree is None or canonical == "（读不出）":
    finish()


def judge_options(options, text):
    if any("<" in token or ">" in token for token in options):
        return "带尖括号占位的 --server-binary：照抄进 shell 时尖括号被当成输入重定向、命令起不来；整条不带 --server-binary，编好的二进制写在整条之外"
    unknown = unknown_of(options) if options and unknown_of is not None else []
    if unknown:
        return f"包装的解析认不出 {' '.join(unknown)}（认不出的参数退 2）"
    if text != canonical:
        return "整条不带 --server-binary，编好的二进制写在整条之外" if options else "前缀与包装用法行写的不同"
    return None


def ascii_tail(word):
    """词前面粘着的全角括号与引号、行文里的汉字（「的包装写法（python3」）不算词：取最后一个非 ASCII 字符之后的那一截，再去掉 ASCII 的引号与括号。"""
    return re.sub(r"^.*[^\x00-\x7f]", "", word).lstrip(LEADING)


def scan(place, lines):
    """lines 是 (位置, 一行文字)：逐行找包装写法，取到第一个单独的 -- 为止的那一段当前缀；后面没有单独 -- 的是行文里提到它，不算整条；
    选项整个写在方括号里（[--server-binary <…>]，用法行里表示可选项的写法）的是用法写法，也不算整条。"""
    wholes = usage = prose = 0
    for location, line in lines:
        for found in ANCHOR.finditer(line):
            tokens = found.group("rest").split()
            separator = next((index for index, token in enumerate(tokens) if token.rstrip(CLOSERS) == "--"), None)
            if separator is None:
                prose += 1
                continue
            options = tokens[:separator]
            if options and options[0].startswith("[") and options[-1].endswith("]"):
                usage += 1
                continue
            wholes += 1
            text = " ".join(word for word in (ascii_tail(found.group("launcher") or ""), ascii_tail(found.group("path")), "--with-temporary-service", *options, "--") if word)
            reason = judge_options(options, text)
            if reason:
                fail(place, location, text, reason)
    total = tallies.setdefault(place, [0, 0, 0])
    total[0] += wholes
    total[1] += usage
    total[2] += prose
    return wholes


def scan_each(place, lines, empty_location):
    if scan(place, lines) == 0:
        fail(place, empty_location, "（没有）", "这一处没有写出整条包装写法")


def numbered(text, path, first=1):
    return [(f"{path}:{number}", line) for number, line in enumerate(text.split("\n"), first)]


table_text = read_text(TABLE_PATH, "登记表文件头")
if table_text is not None:
    scan_each("登记表文件头", [(location, line) for location, line in numbered(table_text, TABLE_PATH) if line.lstrip().startswith("#")], TABLE_PATH)

library_source = read_text(LIBRARY_PATH, "拒绝说明")
library_tree = parse_tree(library_source, LIBRARY_PATH, "拒绝说明")
if library_tree is not None:
    definitions = [node.lineno for node in ast.walk(library_tree) if isinstance(node, ast.FunctionDef) and node.name == "temporary_service_command"]
    try:
        spec = importlib.util.spec_from_file_location("lib_project_code_runs_for_wrapper_form", LIBRARY_PATH)
        library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(library)
        for key in ("scripts/gate/sample-script.py", "scripts/gate/sample-script.sh"):
            location = f"{LIBRARY_PATH}:{definitions[0] if definitions else 0} temporary_service_command(\"{key}\")"
            scan_each("拒绝说明", [(location, library.temporary_service_command(key))], location)
    except Exception as error:   # 库载不进来、函数不在、调用抛异常都算这一处读不到
        fail("拒绝说明", LIBRARY_PATH, "（取不到）", f"{type(error).__name__}：{error}")

dispatch_tree = parse_tree(read_text(DISPATCH_PATH, "骨架"), DISPATCH_PATH, "骨架")
if dispatch_tree is not None:
    wrappers = [(node.lineno, node.value.value) for node in dispatch_tree.body
                if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == "TEMPORARY_SERVICE_WRAPPER" for target in node.targets)
                and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str)]
    if wrappers:
        scan_each("骨架", [(f"{DISPATCH_PATH}:{wrappers[0][0]} TEMPORARY_SERVICE_WRAPPER", wrappers[0][1])], DISPATCH_PATH)
    else:
        fail("骨架", DISPATCH_PATH, "（找不到）", "找不到给 TEMPORARY_SERVICE_WRAPPER 赋字符串的那一句，骨架拼的整条取自它")

client_tree = parse_tree(read_text(CLIENT_PATH, "客户端文件头"), CLIENT_PATH, "客户端文件头")
if client_tree is not None:
    header = ast.get_docstring(client_tree, clean=False)
    if header is None:
        fail("客户端文件头", CLIENT_PATH, "（没有文件头）", "客户端要有文件头，「命令行」一段给整条写法")
    else:
        scan_each("客户端文件头", numbered(header, CLIENT_PATH, client_tree.body[0].lineno), CLIENT_PATH)

# 自证登记表：文件头的注释行，加上要临时服务的表里登记成 required 的那几行的理由；own-server 行的理由说的是它自己怎么起真服务，不判
registry_text = read_text(REGISTRY_PATH, "自证登记表")
if registry_text is not None:
    required = set(required_text.split("\n"))
    scan_each("自证登记表", [(location, line) for location, line in numbered(registry_text, REGISTRY_PATH)
                            if line.lstrip().startswith("#") or line.split("\t", 1)[0] in required], REGISTRY_PATH)

scan_each("runner 表", [(f"{runner_label} 第 {number} 条", runner) for number, runner in enumerate(runner_text.split("\n"), 1) if runner], runner_label)
scan_each("门禁的包装前缀常量", [("code-tooling.sh 的 SELFTEST_TEMPORARY_SERVICE_PREFIX", constant_prefix)], "code-tooling.sh 的 SELFTEST_TEMPORARY_SERVICE_PREFIX")
finish()
PY_WRAPPER_FORM
  )"
  status=$?
  while IFS=$'\t' read -r kind text; do
    case "$kind" in
      CANONICAL) canonical="$text" ;;
      TALLY) tally="$text" ;;
      PROBLEM) problem_lines+=("$text") ;;
    esac
  done <<<"$output"
  if ((status > 1)) || [[ -z "$canonical" ]]; then
    echo "  ✗ 要临时服务的自证的整条写法判不了（python 退 $status，没有打出整条该有的前缀）：${output:-（没有输出）}"   # gate-lint:detail
    echo "     → 怎么办：看上面 python 的报错；判法与取的几处写在 code-tooling.sh 里这一格函数上方的注释"
    exit 1
  fi
  if ((${#problem_lines[@]})) && [[ "${CODE_TOOLING_BREAK:-}" != temporary-service-form-unchecked ]]; then
    echo "  ✗ 要临时服务的自证的整条写法有 ${#problem_lines[@]} 处对不上包装的用法（用法行与认选项的那一句取自 scripts/prune-server/prune_server_client_selftest.py 文件头；整条该是「$canonical <解释器> <脚本> --selftest」）："
    for text in "${problem_lines[@]}"; do echo "      $text"; done   # gate-lint:detail
    echo "     → 怎么办：把点名的那一处改回整条该有的写法「$canonical <解释器> <脚本> --selftest」（整条不带 --server-binary、不写尖括号占位；自己编了、两台都读得到的二进制另加 --server-binary，写在整条之外）；用法以包装文件头那一行为准，包装改了用法就各处一起改；新增了写整条的一处，加进这一格的取法（这一格函数上方的注释列了全部几处）"
    exit 1
  fi
  echo "  ✓ 要临时服务的自证的整条写法各处逐字相同（前缀「$canonical」，不带 --server-binary），包装的解析收得下：$tally"
}

# ── 格 research-script-selftest-coverage：scripts/ 里代码实现了 --selftest 的脚本，都有门禁阶段在跑或登记了为什么不跑
#
# 判据：scripts/ 里实现了 --selftest 的脚本（递归到装置目录那一层），要么有阶段在调它（脚本名后直接跟 --selftest，或先赋给变量再 "$变量" --selftest；
#   脚本名取它在这一批里唯一的最短路径尾，左边不许紧挨字母、数字、点与连字符：crash-point-replay.sh 不算调了 replay.sh），
#   要么登记进 NOT_RUN_HERE 写明为什么；NOT_RUN_HERE 里那份脚本已经有阶段在跑、或不再实现 --selftest 的，是过期豁免，也红。
# 「实现了」只认 .sh / .py 经 code_of 去掉注释与出路文字之后的代码，不列具体写法（==、case、match 都认）；行尾注释同样去掉（shell 不在引号里、前面是空白的 #
#   之后那一段，python 的注释记号；例：共用库函数头后面「# 调用方的 --selftest 起的：…」不算实现了）；「有阶段在调」只认 code_of 之后的代码。
# 阶段取门禁目录顶层全部 *.sh，与 gate.sh 起的是同一批。
#
# 样本：red 放一份只在某个阶段的 echo 出路里被提到的 --selftest 脚本，必须报它没人跑；green 放一份真被阶段调用的与 NOT_RUN_HERE 每一行的那份脚本，判绿并报对数。
#   NOT_RUN_HERE 是这一道自己的表，绿样本里缺了哪一行的脚本，那一行就判过期：往表里加一行，同时往 green、coverage-claims-green 里各放一份那个路径的脚本、改两份 expect 的份数。
#   coverage-claims-green 放一份只在注释、行尾注释与出路里提 --selftest 的 .py、一份只在行尾注释里提它的共用库 .sh、一份提到它的 .tsv（都不算实现了）
#   与一份由不带编号的阶段 tooling-runs.sh 在跑的 .sh，
#   判绿并报 10 份中 1 份（另九份是 NOT_RUN_HERE 那九行）；coverage-claims-red 放一份用 match 语句实现 --selftest、没人跑的 .py，必须报它没人跑。
#   coverage-nested-red 放一份装置目录那一层的 scripts/experiment-apparatus/sample_rig/drive.py（实现了 --selftest、没人跑），必须报它没人跑；
#   coverage-suffix-red 放 scripts/experiment-apparatus/replay.sh 与 scripts/crash/crash-point-replay.sh 两份，阶段只跑后一份的自证，必须报前一份没人跑。
NOT_RUN_HERE=(
  "scripts/crash/lib_crash_point_replay_items.py	它自己没有自证：用法里提到的 --selftest 指 scripts/crash/crash-case-check.py 的自证（那一条已在这张表里）"
  "scripts/kb/lib_notation_check.py	它自己没有自证：只被 import，调它的三个脚本（replace-once.py、insert-row.py、replace-batch.py）各自的 --selftest 走一遍拒绝与放行"
  "scripts/mutation/lib_mutation_units.py	它自己没有自证：只被 import、没有命令行入口，判法由 crates-mutation-rows.py 与 mutate.sh 两边的 --selftest 覆盖"
  "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/fake_row_driver.py	它自己没有自证：rows_orchestrator.py --selftest 起它当假的逐行驱动"
  "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_selftest_harness.py	它自己没有自证：只被修订 8 那几份 import，没有命令行入口，起的是它们各自的 --selftest"
  "scripts/gate/large-file-check.py	它自己没有自证：文件头那句「这份脚本没有 --selftest」被算成实现了；判法由这一道 committed-file-size 格的五份样本与五条弄坏开关证红"
  "scripts/kb/lib_atomic_replace.py	它自己没有自证：只被 import，代码里的 --selftest 是认祖先进程命令行的那个串；自证在 replace-once.py、insert-row.py、replace-batch.py 各自的 --selftest"
  "scripts/prune-server/prune_server_client.py	它自己没有自证：命令行认到 --selftest 只为退 2、指到 prune_server_client_selftest.py（那一份的 --selftest 在 runner 表里）"
)
cell_research_script_selftest_coverage() {
coverage="$(python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "${NOT_RUN_HERE[@]}" <<'PY_COVERAGE'
import glob, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
code_of = code_text.code_of
broken = sys.argv[3]
exempt = dict(row.split("\t", 1) for row in sys.argv[4:])
EVERY_STRING = re.compile(r"(?s).*")   # python 只去注释记号，字符串字面量一概留着（scan_strings）：实现 --selftest 的那个串就在字面量里
def without_trailing_comments(path, text):
    """去掉行尾注释（shell 不在引号里、前面是空白的 #，python 的注释记号）：行尾注释里提到的 --selftest 不算实现了。"""
    if broken == "selftest-coverage-trailing-comments-kept":
        return text
    if path.endswith(".sh"):
        return "\n".join(source_lines.shell_code_lines(text, code_text.OUTPUT_TEXT))
    return "\n".join(source_lines.python_code_lines(text, code_text.OUTPUT_TEXT, EVERY_STRING, scan_strings=True))
def implements_selftest(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    if broken == "claims-any-file":
        return "--selftest" in text
    if not path.endswith((".sh", ".py")) or "--selftest" not in text:
        return False
    return "--selftest" in code_of(without_trailing_comments(path, text))
scan_pattern = "scripts/*/*" if broken == "selftest-coverage-top-level-only" else "scripts/**/*"
claimed = sorted(path for path in glob.glob(scan_pattern, recursive=True)
                 if os.path.isfile(path) and implements_selftest(path))
def unique_tail(script):
    # 在 claimed 里只指它一份的最短路径尾：文件名不重名就是文件名，重名才带上一层层目录
    parts = script.split("/")
    for size in range(1, len(parts) + 1):
        tail = "/".join(parts[-size:])
        if sum(1 for other in claimed if other == tail or other.endswith("/" + tail)) == 1:
            return tail
    return script
tail_of = {script: (os.path.basename(script) if broken == "selftest-coverage-name-substring" else unique_tail(script)) for script in claimed}
left_boundary = "" if broken == "selftest-coverage-name-substring" else r"(?<![\w.-])"
run = set()
stage_pattern = "[0-9][0-9]-*.sh" if broken == "numbered-stages-only" else "*.sh"
for stage in sorted(glob.glob(os.path.join(".claude/gate.d", stage_pattern))):
    code = code_of(open(stage, encoding="utf-8").read())
    variables = dict(re.findall(r'^\s*(\w+)="([^"\n]*\.(?:py|sh))"', code, re.M))
    for script in claimed:
        tail = re.escape(tail_of[script])
        if re.search(left_boundary + tail + r'["\']?\s+--selftest', code):
            run.add(script)
        for variable, value in variables.items():
            if re.search(left_boundary + tail + "$", value) and re.search(r'"\$' + variable + r'"\s+--selftest', code):
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
# 判据：.claude/rules/ 下的文件集合与 CLAUDE.md 里登记的集合逐项相等；登记有两种：@.claude/rules/<文件名> 引用（常驻），
#   与 <!-- doc-lint:read-once <文件名> … --> 登记（不常驻、开会话读一次；写法与上游 doc-lint「K. 规则清单」那一段同一种：同一个正则认注释、
#   注释里每个 [A-Za-z0-9._-]+.md 是一个文件名）。没被登记的规则读不进上下文。
#   read-once 里的名字：在 .claude/rules/ 下的算项目规则；不在 .claude/rules/ 而在上游副本 .claude/singlefs-ai-sop/rules/ 下的归上游 doc-lint 的规则清单判，
#   这一格不判、成功句报条数；两处都没有的判悬空。
#   代码围栏（``` 或 ~~~ 起止）与行内代码里的 @ 与 read-once 注释都不算登记：Claude Code 不在代码里展开 @ 导入，代码里的注释只是在说这个写法。
# 样本：rules-manifest-red 放一份没被引用的规则、一份只在代码围栏里与一份只在行内代码里被 @ 的规则，必须逐个报出；rules-manifest-green 两侧逐项相等，判绿。
#   rules-manifest-read-once-green：一份规则 @ 引用、一份只在 read-once 里登记，read-once 里另登记一份上游副本里的规则，判绿并报 read-once 1 条、归上游 1 条；
#   rules-manifest-read-once-red：read-once 登记了一份 .claude/rules/ 与上游副本里都没有的规则，判红点名它。
# 弄坏开关 CODE_TOOLING_BREAK=rules-read-once-ignored：不认 read-once 登记（改前的判法），rules-manifest-read-once-green 判红（期望退出 0，实测 1：只在 read-once 里登记的那一份报没被引用），
#   rules-manifest-read-once-red 判绿（期望退出 1，实测 0：悬空的登记看不见）。
cell_rules_manifest() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
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
outside_code = manifest.outside_markdown_code

UPSTREAM_RULES = ".claude/singlefs-ai-sop/rules"
# 与上游 doc-lint「K. 规则清单」同一种写法：注释的认法与注释里文件名的认法
READ_ONCE_COMMENT = re.compile(r"<!-- *doc-lint:read-once[^>]*-->")
READ_ONCE_NAME = re.compile(r"[A-Za-z0-9._-]+\.md")
with open(MD, encoding="utf-8", errors="replace") as handle:
    text = outside_code(handle.read())
at_referenced = {os.path.basename(match) for match in re.findall(r"@\.claude/rules/[A-Za-z0-9._-]+\.md", text)}
read_once = set() if sys.argv[2] == "rules-read-once-ignored" else {
    name for comment in READ_ONCE_COMMENT.findall(text) for name in READ_ONCE_NAME.findall(comment)}
upstream_on_disk = set(fnmatch.filter(os.listdir(UPSTREAM_RULES), "*.md")) if os.path.isdir(UPSTREAM_RULES) else set()
read_once_upstream = sorted(name for name in read_once if name not in on_disk and name in upstream_on_disk)
read_once_project = read_once - set(read_once_upstream)
referenced = sorted(at_referenced | read_once_project)
missing, extra = manifest.two_way(on_disk, referenced)

if missing:
    print(f"  ✗ 这些规则文件存在，但 {MD} 既没有 @ 引用、也没在 read-once 里登记——不会被读进上下文，等于没写：")   # gate-lint:detail
    for name in missing:
        print(f"     {RULES}/{name}")   # gate-lint:detail
dangling_at = [name for name in extra if name in at_referenced]
dangling_read_once = [name for name in extra if name not in at_referenced]
if dangling_at:
    print(f"  ✗ {MD} 引用了这些规则，但文件不存在——引用悬空：")   # gate-lint:detail
    for name in dangling_at:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if dangling_read_once:
    print(f"  ✗ {MD} 的 read-once 登记了这些规则，但 {RULES}/ 与上游副本 {UPSTREAM_RULES}/ 里都没有——登记悬空：")
    for name in dangling_read_once:
        print(f"     {RULES}/{name}")   # gate-lint:detail
if missing or extra:
    print("     → 怎么办：补上缺的 @ 引用或 read-once 登记（<!-- doc-lint:read-once <文件名> … -->），或删掉悬空的引用与登记；两侧必须逐项相等。")
    sys.exit(1)
print(f"  ✓ 项目规则清单一致（{len(on_disk)} 条，与 {MD} 的登记逐项相符：@ 引用 {len(at_referenced)} 条、只在 read-once 里登记 {len(read_once_project - at_referenced)} 条；"
      f"read-once 里另有 {len(read_once_upstream)} 条归上游副本 {UPSTREAM_RULES}/，由上游 doc-lint 的规则清单判）")
PY
}

# ── 格 rules-named-by-definitions：CLAUDE.md 的每条 @ 规则与 read-once 规则，至少被一份定义按路径点名
# 为什么：定义都开了 omitClaudeMd，子 agent 只读定义与共用约束点名的规则；一条规则一份都没点名，就没有子 agent 读它（records/2026-10-29-subagent拆分提案.md 第一百六十六节 ②）。
# 判据：CLAUDE.md（代码围栏与行内代码之外，与 rules-manifest 同一份去法 outside_markdown_code）里登记的规则：
#   @.claude/rules/<名>.md 与 @.claude/singlefs-ai-sop/rules/<名>.md 照写的路径；read-once 注释里的每个文件名，.claude/rules/ 下有的取 .claude/rules/<名>，
#   没有而上游副本 .claude/singlefs-ai-sop/rules/ 下有的取那一份，两处都没有的归 rules-manifest 判悬空、这里不判；
#   每一条的路径在 .claude/agents/*.md 或 .claude/agent-common.md 里至少出现一次（按整段路径认，前后不接路径里的字），一份都没有的逐条点名判红。
#   CLAUDE.md 不在、或一条规则都没登记：退 77。
# 管不到的：点名的那一处是不是真让子 agent 读它（「开工先读：」一行与别处顺带提到都算）；一条规则该被哪几份定义点名。
# 样本：rules-named-by-definitions-red（样本仓：CLAUDE.md @ 两条项目规则与一条上游规则，read-once 一条项目规则；共用约束与一份定义只点名其中两条，
#   一条 @ 规则只被定义写成不带目录的文件名、一条 read-once 规则谁都没提，两条逐条点名判红）；
#   rules-named-by-definitions-green（同一副架子，四条都被按路径点名；一条只写在代码围栏里的 @ 不算登记、不判），判绿报「查了 4 条规则、2 份定义」。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-read-once-skipped：只判 @ 规则，read-once 登记的不判；
#   rules-named-by-definitions-red 输出里找不到 read-once 那一条（「.claude/rules/once-only.md」），rules-named-by-definitions-green 输出里找不到「查了 4 条规则」（只剩 3 条）。
# 弄坏开关 CODE_TOOLING_BREAK=rules-named-by-basename：只写文件名、不带目录的也算点名；rules-named-by-definitions-red 输出里找不到「.claude/rules/basename-only.md」那一条。
# 退役判据：定义不再开 omitClaudeMd（子 agent 照读 CLAUDE.md 与它 @ 的规则）时删掉这一格。
cell_rules_named_by_definitions() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys, types

manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken = sys.argv[2]
MD = "CLAUDE.md"
RULES = ".claude/rules"
UPSTREAM_RULES = ".claude/singlefs-ai-sop/rules"
if not os.path.isfile(MD):
    print(f"  ! 找不到 {MD}，这一格无对象可判")
    sys.exit(77)
with open(MD, encoding="utf-8", errors="replace") as handle:
    text = manifest.outside_markdown_code(handle.read())
registered = list(dict.fromkeys(re.findall(r"@(\.claude/(?:singlefs-ai-sop/)?rules/[A-Za-z0-9._-]+\.md)", text)))
if broken != "rules-named-read-once-skipped":
    for comment in re.findall(r"<!-- *doc-lint:read-once[^>]*-->", text):
        for name in re.findall(r"[A-Za-z0-9._-]+\.md", comment):
            for directory in (RULES, UPSTREAM_RULES):
                if os.path.isfile(os.path.join(directory, name)):
                    if f"{directory}/{name}" not in registered:
                        registered.append(f"{directory}/{name}")
                    break
if not registered:
    print(f"  ! {MD} 里一条规则都没登记（@ 引用与 read-once 都没有），这一格无对象可判")
    sys.exit(77)
definitions = sorted(glob.glob(".claude/agents/*.md")) + ([".claude/agent-common.md"] if os.path.isfile(".claude/agent-common.md") else [])
definition_texts = {}
for path in definitions:
    with open(path, encoding="utf-8", errors="replace") as handle:
        definition_texts[path] = handle.read()
unnamed, named_counts = [], []
for rule in registered:
    if broken == "rules-named-by-basename":
        pattern = re.compile(r"(?<![\w.-])" + re.escape(os.path.basename(rule)) + r"(?![\w.-])")
    else:
        pattern = re.compile(r"(?<![\w./-])" + re.escape(rule) + r"(?![\w.-])")
    naming = [path for path, body in definition_texts.items() if pattern.search(body)]
    if naming:
        named_counts.append(len(naming))
    else:
        unnamed.append(rule)
if unnamed:
    for rule in unnamed:
        print(f"  ✗ {rule}：{MD} 登记了它，.claude/agents/*.md 与 .claude/agent-common.md 里一份都没按路径点名")   # gate-lint:detail
    print(f"  ✗ {len(unnamed)} 条规则没有一份定义点名（查了 {len(registered)} 条规则、{len(definitions)} 份定义），定义开了 omitClaudeMd，这几条没有子 agent 会读")
    print("     → 怎么办：在用得到它的定义「开工先读：」一行或共用约束里按路径点名它（改定义照 .claude/rules/implementation-workflow.md「改 agent 定义与共用约束，走同一条三步」，归工具实现员照判决改）；"
          "这条规则确实没有子 agent 要读的，从 CLAUDE.md 的 @ 与 read-once 里去掉之前先问用户")
    sys.exit(1)
print(f"  ✓ {MD} 登记的规则都至少被一份定义点名：查了 {len(registered)} 条规则、{len(definitions)} 份定义，被点名最少的一条 {min(named_counts)} 份")
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

# ── 格 agent-write-scope：钩子注册在要的 matcher 上、写范围表与定义双向一致、每个定义带 omitClaudeMd 与「开工先读：」、共用切词与项目代码判定模块不另写一份
#
# 判据，任一条不成立判红（「注册」指 settings.json 里有一条 matcher 覆盖那个工具或事件、命令指向那个钩子）：
#   ① PreToolUse 里命令指向 write-guard.sh 的注册，matcher 合起来覆盖 Write、Edit 与 MultiEdit（WRITE_GUARD_TOOLS）；
#   ③ tools 含 Write 或 Edit 的每个定义，在 .claude/hooks/agent-write-scope.tsv 里至少一条路径模式；④ 表里每个 agent 名都有定义，每行至少两列；
#   ⑤ Bash 注册 bash-command-detector.sh；⑥ Agent 注册 runner-dispatch-guard.sh；⑦ SendMessage 注册 continuation-guard.sh；
#   ⑧ 每个定义的 frontmatter 有 omitClaudeMd: true，正文有一行以「开工先读：」开头；
#   ⑨ Bash 注册上游 claude-hooks/pattern-process-guard.sh，且那个文件在（它的判别力归上游 selftest）；
#   ⑩ SessionStart 有一条 matcher 覆盖 compact、一条同时覆盖 startup 与 resume（可以是同一条），命令都指向 session-start.sh；
#   ⑫ Bash 注册 project-code-guard.sh；⑭ AskUserQuestion 注册 ask-user-claim-guard.sh；⑯ SubagentHandback 注册 handback-guard.sh；
#   ⑲ Bash 注册 run-scope-guard.sh（范围声明闸；注在不覆盖 Bash 的 matcher 上的不算）；
#   ⑰ PreToolUse 里命令指向 timer-only-remind-guard.sh 的注册，matcher 合起来覆盖 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor 与 Bash（TIMER_GUARD_TOOLS）；
#   ⑱ Stop 里有命令指向 remind-rearm-stop-guard.sh 的注册（循环提醒到点之后自启动靠它）；
#   ⑬ 共用模块 lib_shell_words.py、lib_project_code_runs.py 顶层定义的每个函数名，.claude/hooks/ 下别的文件里不许再 def 一份；名字从模块现读，
#      读不到或一个函数都没有也红；不算的名字登记在 NOT_SHARED_JUDGMENT，登记了而模块里没有的也红；
#   ⑮ frontmatter 的 model 只许 ALLOWED_MODELS 里的取值。
# 钩子的自证不在这里跑，归上游 hooks-registered.sh；它只核有一条注册指向钩子、不核挂在要的 matcher 上，所以 matcher 留在这一格判。
#
# 样本：agent-write-scope-red 的 .claude/hooks/ 里放一份手抄 shell_tokens 的 hook 与一份手抄 judged_by_name 的 hook，必须各报出文件与行号；
#   agent-write-scope-multiedit-red 的 settings.json 只把 write-guard.sh 注册在 "Write|Edit" 上，必须报没覆盖 MultiEdit；
#   agent-write-scope-timer-red 的 settings.json 把 timer-only-remind-guard.sh 注册在不含 Monitor 的 matcher 上、Stop 里没有 remind-rearm-stop-guard.sh，必须报没挂全工具（没覆盖 Monitor）与没注册停止钩子；
#   agent-write-scope-run-scope-red 是 green 那一份，只把 run-scope-guard.sh 注册在 "Write|Edit|MultiEdit" 上（不覆盖 Bash），必须报没注册范围声明闸；
#   agent-write-scope-green 的 .claude/hooks/ 里放两份从共用模块导入、只写自己判定的 hook（连同被判仓自己那两份模块），必须判绿，成功行数进去的别的文件数要对。
# 共用模块的函数名取自这一道脚本旁边 ../hooks/ 下那两份（取自 $0 的目录，不取项目根）：样本仓里放坏的模块碰不到它。
cell_agent_write_scope() {
HOOKS_DIRECTORY="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)"
table_output="$(python3 - "$MANIFEST_LIBRARY" "$HOOKS_DIRECTORY/lib_shell_words.py" "$HOOKS_DIRECTORY/lib_project_code_runs.py" <<'PY'
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
    code_guard_registered = [entry for entry in entries
                        if matcher_covers(entry.get("matcher", ""), "Bash")
                        and any(command_invokes(hook.get("command"), "project-code-guard.sh") for hook in entry.get("hooks") or [])]
    if not code_guard_registered:
        failed = True
        print(f"  ✗ {settings_path} 没注册项目代码闸：PreToolUse 里没有 matcher 覆盖 Bash、命令指向 project-code-guard.sh 的一条")
        print('     → 怎么办：在 hooks.PreToolUse 的 Bash 那一条里加 command "bash \\"$CLAUDE_PROJECT_DIR\\"/.claude/hooks/project-code-guard.sh"。')
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
# ⑮ frontmatter 的 model 只许这几个取值（行尾可带 YAML 注释）：写成别的，换环境之后派发报 model_not_found、那一类 agent 全部起不来
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
    "lib_project_code_runs.py": {
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
    ("lib_project_code_runs.py", "共用项目代码判定模块", sys.argv[3],
     "手抄的判定会分叉：执行前的闸 project-code-guard.sh 与看门狗 agent-watch.py 对同一条命令判得不一样",
     "从 `.claude/hooks/lib_project_code_runs.py` 导入，不另写一份（照 project-code-guard.sh 开头按文件路径 importlib 导入的写法）；那份不够用就改它，再跑它与 project-code-guard.sh 的 --selftest。",
     "classify、judged_by_name 这些判定要定义在 lib_project_code_runs.py 里，project-code-guard.sh 与看门狗从它导入。"),
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
read -r _ writer_count pattern_count shared_function_count scanned_hook_file_count project_code_function_count project_code_scanned_file_count not_shared_judgment_names <<<"$(printf '%s\n' "$table_output" | grep '^TABLE_OK ')"
echo "  ✓ 写范围闸、Bash 检出 hook、项目代码闸、续派闸、续做闸、交回闸、弹窗断言闸与定时唯一办法闸注册在要的 matcher 上，提醒自启动的停止钩子注册着，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着，表与定义一致（${writer_count} 个有 Write 或 Edit 的定义、${pattern_count} 条路径模式），共用切词模块的 ${shared_function_count} 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 ${scanned_hook_file_count} 个文件），共用项目代码判定模块的 ${project_code_function_count} 个函数只在 lib_project_code_runs.py 里定义（查了 .claude/hooks/ 下另外 ${project_code_scanned_file_count} 个文件；${not_shared_judgment_names} 不算，见 NOT_SHARED_JUDGMENT）；钩子的自证归上游 hooks-registered.sh 跑"
}

# ── 格 agent-definition-shape：定义的形状。还拆分提案 records/2026-10-23-subagent拆分提案.md 第五节「新门禁阶段」那张表里没人判的四条
#   （权限写明、四样齐、不抄规则、登记）与第七节 ①；那张表的 frontmatter 一条归上游 doc-lint 的 B2，写范围一致、model 取值、omitClaudeMd 与「开工先读：」归 agent-write-scope 格。
#
# 判据，任一条不成立判红（定义是 .claude/agents/*.md，frontmatter 是第 1 行的 --- 到下一行 --- 之间）：
#   ① frontmatter 有一行 model:（取值认不认得归 agent-write-scope ⑮）；
#   ② frontmatter 有一行 tools:（没写就继承全部工具，Agent、Write、Edit 都在里面）；
#   ③ tools 里没有 Agent、Task（`tools: A, B`、`tools: [A, B]` 与下面几行 `- A` 的 YAML 列表都认；`Agent(…)` 这种带括号的按括号前那一段认）；
#   ④ 正文（frontmatter 之后、代码围栏之外）有以「输入」「写范围」「产出」「没做什么」起头的二级标题，四样各至少一个（`## 输入（主 agent 必须给）` 这种带尾巴的算）；
#   ⑤ 定义、.claude/agent-common.md（在就判）与 .claude/main-agent.md 里的每一行，去掉首尾空白与行首的列表记号（- * + > 与 1. 1)）之后，
#      与 .claude/rules/*.md、.claude/singlefs-ai-sop/rules/*.md 里某一行同样处理之后逐字相同、又不短于 RULE_COPY_MINIMUM 个字符的，判红：规则只指路径，不抄进定义；
#      只由表格分隔与围栏字符（| - : ` ~ = 与空白）组成的行不算。
#      阈值的量法（2026-10-26 在本仓工作区上量）：把这三处的每一行照上面去掉记号，找规则文件 28 份里逐字相同的行，只有一行：.claude/main-agent.md 第 93 行
#      「|---|---|」（9 个字符，与 .claude/rules/fs-design.md 的表格分隔同形）；不算表格分隔之后一行都没有。所以零误报的阈值是不小于 10 的任何数；
#      取 20：标题、表头、一两个词的短行（「## 推论」这类）同形不算抄，20 个字符在中文里是一整句。
#   ⑥ .claude/main-agent.md「## 什么时候派哪个 agent」一节里表格行（以 | 起头）反引号点名的、形似定义名的词（小写字母与数字，连字符隔开），与 .claude/agents/*.md 两向相等
#      （manifest.two_way）：每个定义都被点名；点名的词都有定义。点名了却不是定义名的词，指名门禁阶段（.claude/gate.d/<词>.sh 在）或 crate（crates/<词>/ 在）的自动不算，
#      其余登记在 NOT_DEFINITION_NAMES（写为什么）；登记了而又是定义名的红；被判的根不是样本时，登记了而那一节没点名的也红（不起作用的排除项）。这份文件或这一节没有，红。
# 管不到的：四节底下写没写对东西、tools 给得对不对、规则抄了半行或换了说法、调度表那一行写得对不对；表外别处点名的定义名不算登记。
#
# 样本（每份样本根放 .gate-cells 只点名这一格，定义、主 agent 与规则文件都是样本自己的小份）：
#   agent-definition-shape-model-red：一份定义 frontmatter 没写 model，必须点名它；agent-definition-shape-tools-red：一份没写 tools，必须点名它；
#   agent-definition-shape-agent-tool-red：一份 tools 写 Agent、一份写成 YAML 列表、列表里有 Task(…)，必须各点名一次；
#   agent-definition-shape-sections-red：一份缺「产出」一节、「没做什么」写成三级标题，另一份的「写范围」只在代码围栏里，必须各点名缺的那几样；
#   agent-definition-shape-rule-copy-red：一份定义与样本的 .claude/agent-common.md 各抄了样本规则文件里的一整行（一份带了不同的列表记号），必须各报文件:行与规则文件:行；
#   agent-definition-shape-table-red：调度表没点名一份定义、点名了一个没有定义的词，必须两样都点名；
#   agent-definition-shape-green：tools 写成 YAML 列表、四节带尾巴、正文与 description 里提到 Agent、定义里有一行与规则文件同形而短于阈值、表格分隔同形，
#   调度表里另点名了 NOT_DEFINITION_NAMES 里的 cfg 与样本里一道门禁阶段的名字，必须判绿并报定义份数与点名的词数。
cell_agent_definition_shape() {
python3 - "$MANIFEST_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$(judged_root_is_sample && echo sample || echo repository)" <<'PY'
import glob, os, re, sys, types
manifest = types.ModuleType("lib_manifest")
exec(compile(sys.argv[1], "manifest-library", "exec"), manifest.__dict__)
broken, root_kind = sys.argv[2], sys.argv[3]
RULE_COPY_MINIMUM = 20          # ⑤ 的阈值，量法见这一格函数上方的注释
REQUIRED_SECTIONS = ("输入", "写范围", "产出", "没做什么")
FORBIDDEN_TOOLS = ("Agent", "Task")
TABLE_SECTION = "## 什么时候派哪个 agent"
# ⑥ 调度表里点名、不是定义名的词（门禁阶段名与 crate 名自动不算，不用登记）
NOT_DEFINITION_NAMES = {
    "cfg": "Rust 的条件编译属性（验证包那一行说 tests/ 里 cfg 点到的特性）",
    "gpu": "cargo 特性名（验证包那一行说带 gpu 特性的那一趟）",
}
DEFINITION_NAME = re.compile(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*")
LIST_MARK = re.compile(r"^(?:(?:[-*+>]|\d+[.)])\s+)+")
STRUCTURE_ONLY = re.compile(r"[\s|\-:`~=]*")
failed = False


def report(title, rows, howto):
    global failed
    failed = True
    print(f"  ✗ {title}")  # gate-lint:summary
    for row in rows:
        print(f"     {row}")  # gate-lint:detail
    print(f"     → 怎么办：{howto}")


def split_definition(text):
    """→ (frontmatter 的行, 正文的行, 正文第一行的行号)；第 1 行不是 --- 的没有 frontmatter。"""
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return [], lines, 1
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            return lines[1:index], lines[index + 1:], index + 2
    return lines[1:], [], len(lines) + 1


def tools_of(head):
    """→ tools 一行列出的工具名集合；没有这一行交 None。"""
    index = next((number for number, line in enumerate(head) if line.startswith("tools:")), None)
    if index is None:
        return None
    inline = head[index][len("tools:"):].strip().strip("[]")
    names = [piece for piece in inline.split(",") if piece.strip()]
    for line in head[index + 1:]:
        item = re.match(r"^\s*-\s*(.+?)\s*$", line)
        if not item:
            break
        names.append(item.group(1))
    return {name.strip().strip("'\"").split("(", 1)[0].strip() for name in names}


def headings_outside_fences(body):
    fence, found = None, []
    for line in body:
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            if fence is None:
                fence = marker.group(1)[0]
            elif stripped.startswith(fence * 3):
                fence = None
            continue
        if fence is None and line.startswith("## "):
            found.append(line[3:].strip())
    return found


def comparable(line):
    return LIST_MARK.sub("", line.strip()).strip()


def section_lines(body, prefix):
    """→ 以 prefix 起头的二级标题那一节（到下一个二级标题为止）里代码围栏之外的行；没有这一节交 None。"""
    fence, inside, found = None, False, None
    for line in body:
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            if fence is None:
                fence = marker.group(1)[0]
            elif stripped.startswith(fence * 3):
                fence = None
            continue
        if fence is not None:
            continue
        if line.startswith("## "):
            inside = line[3:].strip().startswith(prefix)
            if inside and found is None:
                found = []
            continue
        if inside:
            found.append(line)
    return found


definitions = sorted(glob.glob(".claude/agents/*.md"))
names = [os.path.basename(path)[:-3] for path in definitions]
missing_model, missing_tools, forbidden, missing_sections = [], [], [], []
for path, name in zip(definitions, names):
    head, body, _ = split_definition(open(path, encoding="utf-8").read())
    if not any(line.startswith("model:") for line in head) and broken != "definition-model-unchecked":
        missing_model.append(name)
    tools = tools_of(head)
    if tools is None:
        if broken != "definition-tools-unchecked":
            missing_tools.append(name)
    elif broken != "definition-agent-tool-allowed":
        given = sorted(tools & set(FORBIDDEN_TOOLS))
        if given:
            forbidden.append(f"{name}：tools 里有 {'、'.join(given)}")
    headings = headings_outside_fences(body)
    lacking = [section for section in REQUIRED_SECTIONS if not any(heading.startswith(section) for heading in headings)]
    if lacking and broken != "definition-sections-unchecked":
        missing_sections.append(f"{name}：缺 {'、'.join(lacking)}")
if not definitions:
    report("没有 .claude/agents/*.md：这一格一份定义都没有可判", [], "定义放在 .claude/agents/<名字>.md；样本里也要放至少一份。")
if missing_model:
    report("这些定义的 frontmatter 没写 model——派发时取默认模型，定义里说的模型与真跑的对不上：", missing_model,
           "在 frontmatter 里加一行 model: opus、sonnet、haiku 或 inherit（取值认不认得归 agent-write-scope 格）。")
if missing_tools:
    report("这些定义的 frontmatter 没写 tools——没写就继承全部工具，Agent、Write、Edit 都在里面：", missing_tools,
           "在 frontmatter 里加一行 tools:，只列它干活要的工具（观测类与推论类不给 Write、Edit）。")
if forbidden:
    report("这些定义的 tools 里有 Agent 或 Task——子 agent 只由主 agent 派，不再往下派：", forbidden,
           "从 tools 里删掉 Agent、Task；要分活的写进报告交主 agent 派（.claude/agent-common.md「派发」）。")
if missing_sections:
    report("这些定义缺以「输入」「写范围」「产出」「没做什么」起头的二级标题（代码围栏里的不算）：", missing_sections,
           "补上缺的那几节：输入是派发时必须给的、写范围是它能写的路径、产出是交什么、没做什么是固定不做的；写成 ## 起头的二级标题。")

# ⑤ 不抄规则
rule_files = sorted(glob.glob(".claude/rules/*.md") + glob.glob(".claude/singlefs-ai-sop/rules/*.md"))
rule_lines = {}
for path in rule_files:
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        text = comparable(line)
        if len(text) >= RULE_COPY_MINIMUM and not STRUCTURE_ONLY.fullmatch(text):
            rule_lines.setdefault(text, f"{path}:{number}")
judged_files = definitions + [path for path in (".claude/agent-common.md", ".claude/main-agent.md") if os.path.isfile(path)]
copies, judged_lines = [], 0
for path in judged_files:
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        judged_lines += 1
        text = comparable(line)
        if text in rule_lines and broken != "definition-rule-copies-unchecked":
            copies.append(f"{path}:{number} 与 {rule_lines[text]} 逐字相同（{len(text)} 个字符）：{text[:60]}")
if copies:
    report(f"定义与共用约束里这些行与规则文件逐字相同、不短于 {RULE_COPY_MINIMUM} 个字符——规则抄进定义，改规则时这里不跟着改就分叉：", copies,
           "删掉抄来的那一行，改成一条读取指令：「开工先读：`<规则文件>`「<小节>」」（.claude/agent-common.md「规则怎么读」）。")

# ⑥ 调度表与定义两向相等
main_path = ".claude/main-agent.md"
table_names, not_definition_seen, auto_skipped = [], set(), set()
if not os.path.isfile(main_path):
    report(f"没有 {main_path}：调度表「{TABLE_SECTION[3:]}」无从比对", [], f"主 agent 的调度表写在 {main_path} 的「{TABLE_SECTION}」一节，一行一种情形、反引号点名派谁。")
else:
    in_section, seen_section = False, False
    for line in open(main_path, encoding="utf-8").read().split("\n"):
        if line.startswith("## "):
            in_section = line.strip() == TABLE_SECTION
            seen_section = seen_section or in_section
            continue
        if not in_section or not line.lstrip().startswith("|"):
            continue
        for token in re.findall(r"`([^`\n]+)`", line):
            if not DEFINITION_NAME.fullmatch(token):
                continue
            if token in NOT_DEFINITION_NAMES:
                not_definition_seen.add(token)
            elif token not in names and (os.path.isfile(f".claude/gate.d/{token}.sh") or os.path.isdir(f"crates/{token}")):
                auto_skipped.add(token)
            elif token not in table_names:
                table_names.append(token)
    if not seen_section:
        report(f"{main_path} 里没有「{TABLE_SECTION}」一节：调度表与定义目录无从比对", [], f"把调度表写在 {main_path} 的「{TABLE_SECTION}」一节（标题逐字），一行一种情形、反引号点名派谁。")
    elif broken != "definition-table-unchecked":
        unnamed, dangling = manifest.two_way(names, table_names)
        if unnamed:
            report(f"这些定义在 {main_path}「{TABLE_SECTION[3:]}」表里没被反引号点名——主 agent 照表派活，表里没有的类型没人派：", unnamed,
                   f"在那张表里给它加一行（什么时候派它），或删掉不再用的定义；只在表外别处提到不算。")
        if dangling:
            report(f"{main_path}「{TABLE_SECTION[3:]}」表里反引号点名的这些词在 .claude/agents/ 里没有定义：", dangling,
                   "改成现在的定义名；不是定义名的词（命令、特性名这类）去掉反引号，或登记进 code-tooling.sh 这一格的 NOT_DEFINITION_NAMES 并写为什么。")
        contradictory = sorted(set(NOT_DEFINITION_NAMES) & set(names))
        stale = sorted(set(NOT_DEFINITION_NAMES) - not_definition_seen) if root_kind == "repository" else []
        if contradictory or stale:
            report("code-tooling.sh 这一格的 NOT_DEFINITION_NAMES 有不起作用或自相矛盾的登记：",
                   [f"{token}：同名的定义在 .claude/agents/ 里" for token in contradictory] + [f"{token}：那一节没点名它" for token in stale],
                   "删掉这几项；同名的定义在的，那个词就是定义名，不该登记成不是。")

if failed:
    sys.exit(1)
skipped = sorted(not_definition_seen | auto_skipped)
print(f"  ✓ 定义的形状：{len(definitions)} 份定义都写明 model 与 tools、tools 里没有 Agent 与 Task、输入写范围产出没做什么四节齐；"
      f"定义与共用约束 {len(judged_files)} 份文件 {judged_lines} 行里没有与规则文件（{len(rule_files)} 份）逐字相同、不短于 {RULE_COPY_MINIMUM} 个字符的行；"
      f"{main_path} 的调度表点名 {len(table_names)} 个定义名，与 .claude/agents/ 两向相等（表里另有不是定义名的 {len(skipped)} 个词：{'、'.join(skipped) or '没有'}）")
PY
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
#   ④ 搬迁登记表 .claude/gate.d/path-moves.tsv（被判的根里那一份；只缩不涨的表与基准那一版比时照它换算，读法在 lib-lag-baseline.py 文件头「搬迁登记」）：
#      每一行「<旧前缀><TAB><新前缀><TAB><YYYY-MM-DD><TAB>records/<文件>.md」写对、出处文件在；此刻生效的行（这一行不在 gate 基准那一版的登记表里，gate 基准那一版里旧前缀下有文件、新前缀下没有文件；
#      提交进基准之后不再生效）之间，任一行的旧前缀或新前缀不许等于、包含或落在另一行的旧前缀或新前缀下面；生效的每一行有搬迁实据：基准里旧前缀下的每份文件，
#      此刻在新前缀下照这一行换前缀的同一相对路径上都在、被 git 跟踪（在索引里，或是没被忽略的工作区文件），新前缀在基准里没有文件，旧前缀此刻不在磁盘上
#      （换算之后相同的份数只报不判）；不在基准那一版登记表里的新行不生效的判红「这一行不是一次搬迁」，已在基准那一版登记表里的行不生效不核；
#      被判的根不在 git 仓里、或在仓里但不是仓根时判不了哪几行生效，不核，成功句写明。
#      判法是 lib-lag-baseline.py 的 path_move_report（用这一道所在仓里那一份；gate 基准由它在被判的根上经 scripts/gate/changed-paths.sh 的 gate_diff_base gate 取）；表不在算 0 行。
# 为什么：取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath），收完之后要有东西拦下一份拷贝。
#
# 样本：change-range-single-source-red 把三条判据该抓的写法各放几种（自己取 merge-base、@{u}...、HEAD~1、HEAD^、换了行的参数列表续行里的 "merge-base"、
#   不带或带 =true 的 quotepath、包装函数名叫 git 而这一行没调它、test 的 -z、printf "$(git …)"、换行的参数列表、不判退出码的共用取法），逐条点名判红；
#   change-range-single-source-green 放五种合法写法与 unset GATE_BASE、merge-base --is-ancestor、HEAD^{tree}、判了退出码的调用，判绿。
#   change-range-single-source-path-moves-red（不在 git 仓里）的搬迁登记表五行：日期不是 YYYY-MM-DD、只有三列、出处不在各一行，逐行点名判红；
#   前两行写法对，旧前缀还在磁盘上、新前缀不在磁盘上都不核（判不了哪几行生效）。
#   change-range-single-source-path-moves-green（不在 git 仓里）的表两行写法对、出处在：判绿报 2 行，成功句写判不了哪几行此刻生效、串与实据没核。
#   change-range-single-source-path-moves-evidence-red：setup.sh 建 git 仓提交两份实现 tools/real-a、tools/real-b、起 A 的 ops/launch.sh、old/x/f.sh、old/y/g.sh、rig/run.sh
#   与写着 build/ 的 .gitignore 作基准；删掉 tools/real-a、ops/launch.sh 改成起 B，old/x、old/y 下两份搬到 new/ 下、另拷一份 other/f.sh，删掉 rig、放一份没进 git 的 build/rig/run.sh；
#   表五行：tools/real-a → tools/real-b（新前缀下基准里本来就有文件，不生效，又是新行，判红「这一行不是一次搬迁」）、tools/real-a → tools/gone（落点不在）、old → new 与 old/x → other（串了）、
#   rig → build/rig（落点被 .gitignore 忽略），四处逐条点名判红。
#   change-range-single-source-path-moves-evidence-green：setup.sh 建 git 仓，基准里有 scripts/old-rig 下三份与一张登记了 scripts/ancient-rig → scripts/old-rig 的表；
#   git mv 到 scripts/new-rig，三份里两份只换了路径、一份另改了一行，表加一行 scripts/old-rig → scripts/new-rig：判绿，成功句写此刻生效 1 行（3 份里 2 份换算之后相同）、
#   不生效 1 行（第 2 行）。
#   change-range-single-source-path-moves-committed-green：setup.sh 建 git 仓，把 scripts/old-rig 搬到 scripts/new-rig、登记，连同登记表一起提交（这一批提交之后）：
#   这一行不生效，判绿，成功句写此刻生效的 0 行、不生效 1 行。
#   change-range-single-source-path-moves-base-row-green：setup.sh 建 git 仓，搬迁与登记表一起提交之后，scripts/new-rig 又改名、旧名字 scripts/old-rig 下另起一份新东西，再提交一次：
#   基准树满足「旧前缀下有文件、新前缀下没有文件」，可那一行已在基准那一版的登记表里，不生效，判绿，成功句写此刻生效的 0 行、点名「已在基准 HEAD 那一版的登记表里」。
# 弄坏开关 CODE_TOOLING_BREAK=path-moves-unchecked：不判 ④，change-range-single-source-path-moves-red 与 change-range-single-source-path-moves-evidence-red 判绿（期望退出 1，实测 0），
#   change-range-single-source-path-moves-green、change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 里「✓ 搬迁登记表 …」那条 want 找不到。
# 共用库 lib-lag-baseline.py 的开关 LAG_BASELINE_BREAK 各项（写法在那份文件头）在这一格样本上红在哪一句；逐条点名判错的样本在文件头开关表：
#   move-evidence-unchecked：change-range-single-source-path-moves-evidence-red 判绿（期望退出 1，实测 0），
#     change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 里「此刻生效」那条 want 找不到，
#     change-range-single-source-path-moves-green 里「判不了哪几行此刻生效」那条 want 找不到（不核时成功句换了说法）
#   window-ignored：change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1：
#     已提交的那一行当生效、实据红）；change-range-single-source-path-moves-evidence-red 里「有 4 处不对」那条 want 找不到
#   occupied-unchecked：change-range-single-source-path-moves-evidence-red 里「第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件」那条 want 找不到（tools/real-a → tools/real-b 当生效，与下一行串了）
#   chain-allowed：change-range-single-source-path-moves-evidence-red 里「第 4 行与第 5 行串了」与「有 4 处不对」两条 want 找不到
#   untracked-landing-accepted：change-range-single-source-path-moves-evidence-red 里「没被 git 跟踪」与「有 4 处不对」两条 want 找不到
#   idle-new-row-unchecked：change-range-single-source-path-moves-evidence-red 里「这一行不是一次搬迁」与「有 4 处不对」两条 want 找不到（不在基准登记表里的新行不生效，不判红）
#   base-row-stays-effective：change-range-single-source-path-moves-evidence-green（基准登记表里的 scripts/ancient-rig → scripts/old-rig 当新行，新前缀下基准里早有文件）、
#     change-range-single-source-path-moves-committed-green 与 change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1：基准登记表里的老行照样生效、实据红）
cell_change_range_single_source() {
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
python3 - "$STAGE_NAME" "$STAGE_DIRECTORY/lib-lag-baseline.py" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, importlib.util, os, re, sys

own_name = sys.argv[1]
lag_baseline_library, broken = sys.argv[2], sys.argv[3]
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
# ④ 搬迁登记表（判法 lib-lag-baseline.py 的 path_move_problems，用这一道所在仓里那一份；表取被判的根里那一份）
move_count, move_problems, move_note = 0, [], ''
if broken != 'path-moves-unchecked':
    try:
        lag_baseline_spec = importlib.util.spec_from_file_location('lag_baseline', lag_baseline_library)
        lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
        lag_baseline_spec.loader.exec_module(lag_baseline)
        move_count, move_problems, move_note = lag_baseline.path_move_report('.')
    except (OSError, ImportError, AttributeError, SyntaxError, ValueError) as error:
        move_problems = [(f'载不进 {lag_baseline_library}：{type(error).__name__}: {error}',
                          '它随仓走（.claude/gate.d/lib-lag-baseline.py），被删了就从 git 找回来；搬迁登记表的判法只在那一份里写')]
if move_problems:
    failed = True
    print(f'  ✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 {len(move_problems)} 处不对（共 {move_count} 行）：')
    for what, _howto in move_problems:
        print(f'      {what}')   # gate-lint:detail
    print('     → 怎么办：' + '；'.join(dict.fromkeys(howto for _what, howto in move_problems)))
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
if broken == 'path-moves-unchecked':
    print('  ! 弄坏开关 path-moves-unchecked 生效：搬迁登记表 .claude/gate.d/path-moves.tsv 没判')
else:
    print(f'  ✓ 搬迁登记表 .claude/gate.d/path-moves.tsv {move_count} 行：写法对、出处都在；{move_note}')
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
#   这里经它的 --scan-scripts 扫，射程比两个 lint 多一个 .claude/gate.d/（它的样本目录不扫）。--scan-scripts 只扫给它的那一层：待改清单 .claude/process-safety-pending 里
#   所在目录这一趟没扫的行判本次未判（它点名那一行并往 $GATE_NOT_RUN_FILE 报一行），不判成「一处要放行的都没有」。
# 样本：research-gate-lint-red 放一个只带一句话的 die 和一个靠子 shell 赋值往外带值的函数；它的 .claude/scripts/ 里再放一个不带出路的拒绝和一处 pkill -f
#   （这个目录进了射程才报得出来）；进程安全那一路有 scripts/gate/stop-processes.sh（自己任务的进程组、循环里 kill、systemctl 带通配）、
#   scripts/gate/stop.py（循环 os.kill、os.killpg）与 .claude/gate.d/50-stop-group.sh（按 cgroup.procs 批量发、没核 scope 也没标），都必须逐处报出。
#   research-gate-lint-green 的 die 带第二个参数、不靠子 shell 带值，.claude/scripts/ 里放一个干净的包装（成功行的脚本数要把它数进去），
#   scripts/gate/stop-own.sh 只停点名的一个、按 cgroup 批量发的那一段核过 singlefs-memory-cap-*.scope 又标了 own-scope，必须判绿。
#   research-gate-lint-pending-outside-green 是 green 加 scripts/gate/deeper/killer.sh（循环里 kill，所在目录进程安全那一路不扫）和 .claude/process-safety-pending 的一行点名它：
#   清单里落在这一趟没扫的目录下的行判本次未判（bash-command-detector.sh --scan-scripts 点名并往 $GATE_NOT_RUN_FILE 报一行），不判成「一处要放行的都没有」，必须判绿；
#   research-gate-lint-pending-stale-red 是 green 加 .claude/process-safety-pending 一行点名 scripts/gate/sample.sh（扫了的目录里、一处要放行的都没有），必须仍判红。
#   弄坏开关 BASH_COMMAND_DETECTOR_BREAK=pending-outside-scanned-as-stale 归钩子、由钩子自己的 --selftest 证红，不列进文件头开关表：钩子平常跑设了它就退 2（只在自证里认），
#   经这一道喂样本证不出那一条判据（2026-10-24 实测：进程安全那一路没跑成，research-gate-lint 的 green、red、pending-stale-red、pending-outside-green 一起判错）。
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
# 射程到 scripts/<类>/ 这一层为止：实验装置那一层（scripts/experiment-apparatus/<装置>/）与 scripts/devices/qemu/、scripts/mutation/tables/ 不进（搬迁前它们不在 research/scripts/ 里，也不扫）。
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
#      工具版本串里的日期不判：一行里写成「(<7 到 40 位十六进制> <年-月-日>)」的（rustc -V、cargo -V 的样子），那个日期是那一版工具的发布日，
#      不是写样本的日子（例 formal-verification.sh 样本里的 rustc-version）；成功句另报不判的个数。弄坏开关 tool-version-dates-judged 下照判。
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
#   绿样本退 77 并写了够长的 not-green-by-design，必须判绿。fixture-claims-green 另放一份 rustc-version（版本串里的日期早于沙箱第一个提交 7 天以上），
#   必须不判、成功句报「工具版本串里的日期 1 个」。
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
  scanned=0; dated=0; bodies=0; body_dates=0; version_dated=0; skipped_own=0; unreadable_bodies=()
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
    # 工具版本串里的日期（rustc / cargo -V 的「(<提交的十六进制> <年-月-日>)」）是那一版工具的发布日，不是写样本的日子：照「行号:日期」记下，下面不判
    if version_hits="$(grep -a -noE "\([0-9a-f]{7,40} $date_re\)" "$body" 2>/dev/null)"; then :; else version_hits=""; fi
    if [[ "${CODE_TOOLING_BREAK:-}" == tool-version-dates-judged ]]; then version_hits=""; fi
    version_dates=" "
    while IFS= read -r version_hit; do
      [[ -n "$version_hit" ]] || continue
      version_dates+="${version_hit%%:*}:$(grep -oE "$date_re" <<<"${version_hit#*:}") "
    done <<<"$version_hits"
    while IFS= read -r hit; do
      [[ -n "$hit" ]] || continue
      line_number="${hit%%:*}"; found="${hit##*:}"
      if [[ "$version_dates" == *" $line_number:$found "* ]]; then version_dated=$((version_dated + 1)); continue; fi
      body_dates=$((body_dates + 1))
      if why="$(date_out_of_range "$found" "$start_date")"; then
        printf 'BADBODY\t%s:%s\t%s\t%s\n' "$body" "$line_number" "$found" "$why"
      fi
      # 整行去重，不按行号去重：按行号会丢掉同一行上的第二个日期。
    done < <(sort -u <<<"$hits")
  done < <(git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/' | sort -z -u)
  printf 'COUNT\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$scanned" "$dated" "$bodies" "$body_dates" "$start_date" "$skipped_own" "$own_fixture" "$start_source" "$version_dated" "${unreadable_bodies[*]:-（没有）}"
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
  IFS=$'\t' read -r _ scanned dated bodies body_dates start_date skipped_own own_fixture start_source version_dated unreadable_list < <(grep '^COUNT' "$date_report")
  # 计数缺一个就判红：报着空数的成功句照样绿，比没有这一条更糟。
  if ! [[ "$scanned" =~ ^[0-9]+$ && "$dated" =~ ^[0-9]+$ && "$bodies" =~ ^[0-9]+$ && "$body_dates" =~ ^[0-9]+$ && "$skipped_own" =~ ^[0-9]+$ && "$version_dated" =~ ^[0-9]+$ ]]; then
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
    printf '    工具版本串里的日期 %s 个（「(<提交的十六进制> <年-月-日>)」，那一版工具的发布日）不判\n' "$version_dated"
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

# ── 格 committed-file-size：被提交的东西里没有超过 100 MiB 的文件（单文件超过 GitHub 的推送上限，推不上去，事后只能重做提交）
# 判法只在 scripts/gate/large-file-check.py（用这一道所在仓里那一份，上限常量 SINGLE_FILE_SIZE_LIMIT_BYTES 全仓只在那里定义），被判的根照传。
#   甲  被判的根（gate.sh --staged 时是 HEAD 加暂存区的临时树）里 git 跟踪的每个文件；
#   乙  当前分支相对上游还没推上去的每笔提交里新增或改过的 blob（拦绕过门禁提交的，和先提交大文件、后一笔提交又删掉的）；
#       没有上游时乙这一半本次未跑：判法往 $GATE_NOT_RUN_FILE 报一行，成功句写明乙没跑，不报成判过。
#   红行逐个列路径与大小，乙另列提交号；出路是 gzip -n -9 压成 .gz 入库、读它的地方改成认 .gz、已进没推提交的重做提交。
# 样本：不真造 100 MiB 的文件。样本根放 .large-file-limit（一个字节数，判法的 --limit-bytes）把门限压小；被判的根是这一道所在的仓或 --staged 的临时树时不认
#   （与 .gate-cells 同一种认法，judged_root_is_sample）。五份样本的 setup.sh 各建一个小 git 仓，门限 1024 字节：
#   committed-file-size-green          3 个跟踪的文件，edge.bin 恰好 1024 字节（等于放行），上游在基准提交、1 笔没推的小提交：甲查 3 个，乙查 1 笔提交 1 个 blob；
#   committed-file-size-red            甲红：基准提交里的 old-big.out（2048）与已 git add 未提交的 staged-big.bin（3000）都点名，汇总括号里只有「甲」；没跟踪的 untracked-big.bin 不判；
#   committed-file-size-unpushed-red   乙红：没推的第一笔提交加 big.bin（2048）、第二笔又删了，HEAD 的树干净，乙点名第一笔提交与 big.bin，汇总括号里只有「乙」；
#   committed-file-size-detached-red   同上，最后 git checkout --detach（--staged 临时树的样子），靠指着 HEAD 的本地分支找上游；
#   committed-file-size-no-upstream-green  没设上游：打「乙（还没推的提交）本次未跑」，甲照判，整格退 0，成功句写「乙 本次未跑」。
# 弄坏开关 CODE_TOOLING_BREAK=committed-size-<项>（折成 LARGE_FILE_CHECK_BREAK=<项>）：
#   committed-size-tracked-half-skipped   甲不判：committed-file-size-red 判绿（期望退出 1，实测 0）；
#   committed-size-unpushed-half-skipped  乙不判：committed-file-size-unpushed-red、committed-file-size-detached-red 判绿；
#   committed-size-limit-inclusive        恰好等于上限的也算超：committed-file-size-green、committed-file-size-no-upstream-green 判红；
#   committed-size-no-upstream-as-pass    没有上游时乙当成判过（查了 0 笔）：committed-file-size-no-upstream-green 里「乙（还没推的提交）本次未跑」那条 want 找不到；
#   committed-size-detached-unresolved    游离的 HEAD 一律当没有上游：committed-file-size-detached-red 判绿。
cell_committed_file_size() {
  local limit_arguments=() break_item="" sample_limit
  local checker="$STAGE_REPOSITORY/scripts/gate/large-file-check.py"
  [[ -f "$checker" ]] || { echo "  ✗ 找不到 $checker"; echo "     → 怎么办：它随仓走（scripts/gate/large-file-check.py），被删了就从 git 找回来；--staged 时它要先暂存。"; exit 1; }
  if judged_root_is_sample && [[ -f .large-file-limit ]]; then
    sample_limit="$(tr -d "[:space:]" < .large-file-limit)"
    limit_arguments=(--limit-bytes "$sample_limit")
  fi
  case "${CODE_TOOLING_BREAK:-}" in committed-size-*) break_item="${CODE_TOOLING_BREAK#committed-size-}" ;; esac
  LARGE_FILE_CHECK_BREAK="$break_item" python3 "$checker" "$ROOT" --report-as "$STAGE_NAME 的 committed-file-size 格" ${limit_arguments[@]+"${limit_arguments[@]}"}
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
    print(f"     → 怎么办：给这几道改名，让它们按 gate.sh 的 find … | sort 排到 {environment_stage} 前面（名字按 sort 的次序排在它前面）；")
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

# 排除表只缩不涨要的基准那一版，heavy-thread-defaults、hardcoded-quota、shared-data-root-literals 三格共用。在被判的根里调，设两个变量：
#   base_name   改动范围的基准（scripts/gate/changed-paths.sh 的 gate_diff_base gate）；不在 git 仓里、没有 HEAD 时是空的；
#   base_state  table（取到了，写进第二个参数那份文件）、none（不在 git 仓里、没有 HEAD、基准里没有这张表）、unreadable:<为什么>（git 取不到）。
#   取法是 lib-lag-baseline.py show（照登记的搬迁换算路径；退 0 有表、退 3 基准里没有这张表、别的是 git 取不到，问题句在 stderr）。
#   source 不了 changed-paths.sh 时打出路、exit 1（格函数在自己的子 shell 里，结束的是那一格）。
exempt_table_baseline() { # <表的相对路径> <放基准那一版的文件>
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
    if base_problem="$(python3 "$STAGE_DIRECTORY/lib-lag-baseline.py" show "$base_name" "$1" 2>&1 >"$2")"; then
      base_state="table"
    else
      base_exit=$?
      if ((base_exit == 3)); then base_state="none"; else base_state="unreadable:${base_problem:-lib-lag-baseline.py show 退 $base_exit}"; fi
    fi
  fi
}

# ── 格 heavy-thread-defaults：scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里自己定并行度默认值的写法，都经每台默认重活线程数的登记位取
#
# 登记位：.claude/heavy-thread-default（写法在它的文件头），判它在不在、写没写对经 scripts/hosts/heavy_threads.py 的 default_heavy_threads
#   （用这一道所在仓里那一份，登记文件取被判的根里那一份，不认调用方的 SINGLEFS_THREAD_CAP）。
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里的 *.sh、*.py、*.rs；
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
#   （scripts/gate/changed-paths.sh 的 gate_diff_base gate）里那一版比，基准里没有的「路径:锚串」判红；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 show）。
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
#   串行的循环里调的函数自己可能在并行）：由运行时的在用核数兜：每次尝试的平均在用核数由 task-server 记进 task status（只记不报）。
# 样本：heavy-thread-defaults-red 的登记写对，scripts/hosts/half.sh 用 $(nproc) / 2、scripts/gate/half.py 用 cpu_count 与 min(16 与 processor_count // 2、
#   .claude/hooks/start.py 用列表写法起 capped.sh 16、.claude/gate.d/stage.sh 用 capped.sh 16、crates/demo/src/lib.rs 用 available_parallelism 与
#   隔了几行的 unwrap_or(8)，逐处点名判红（只在注释、echo 与 print 里提到的不点名）；排除表一行指向不存在的文件、一行锚串不命中、
#   setup.sh 建 git 仓提交一版只有那两行的表、再加一行（表涨了），三样都点名。heavy-thread-defaults-green 的登记写对，脚本经 heavy_thread_default
#   与 default_heavy_threads 取、只在注释、文档串、echo、引号里整句的样本里提到 nproc 与 capped.sh 16，一处真用 $(nproc) 的登记在排除表里，
#   判绿并报扫了几份、排除命中几处；scripts/hosts/uses-registry.py 里一行 int(… or 600) 的超时回退值紧跟在
#   SINGLEFS_THREAD_CAP 那一行后面，不点名。heavy-thread-defaults-red 里 scripts/gate/half.py 还有线程变量隔一行才给回退值的 int(cap or 8)，照判（超时那条豁免不吞它）。
#   装置读自己的线程变量那一路，两份样本根里各放一份假的 scripts/hosts/capped.sh（只有 variable_names 清单）：red 的清单没有装置的变量，scripts/experiment-apparatus/own_device.py（环境里读、
#   没设就 KeyError）、scripts/hosts/own_device.sh（${…:?}）、crates/demo/src/device.rs（env::var 加 expect）三份没有别的被判行，落在点名变量的那一行；scripts/experiment-apparatus/own_with_cores.py 与
#   crates/demo/src/lib.rs 已有核数、回退值的被判行，同一行再加这一种；own_device.sh 里 LOCAL_WORKERS 是文件自己赋的局部变量，不点名。green 的清单有 E161_THREADS：
#   crates/demo/src/device.rs 读它，不点名，测试模块里的 SAMPLE_TEST_THREADS 不算；scripts/experiment-apparatus/own_registry.py 读 capped.sh 不设的 SAMPLE_OWN_THREADS 但没设时回退到 default_heavy_threads，不点名；
#   scripts/experiment-apparatus/own_multiline.py 与 crates/demo/src/own_match.rs 把读和回退写在同一条跨行的语句里（python 的括号续行、rust 的 match 加 Err 分支），不点名；
#   scripts/experiment-apparatus/own_child_env.py 只往子进程环境里写 ["SAMPLE_CHILD_THREADS"] = 值，scripts/hosts/local_workers.sh 的 WORKERS 与 SAMPLE_INLINE_WORKERS（同一行 ; 之后赋的）是局部变量，都不算读；
#   scripts/experiment-apparatus/own_in_comparison.py 里 `if "SAMPLE_COMPARED_THREADS" in registry_table:` 是字符串比较，不是读环境变量，但引号里的名字出现就算读，被判行靠排除表放行
#   （W01：scripts/agents/dispatch-prompt.py 的 `in` 比较）；scripts/hosts/env_prefix.sh 里 `true && SAMPLE_PREFIX_THREADS=8 make -j 2` 的 =8 后面跟着命令，是环境前缀，不点名。
#   heavy-thread-defaults-quoted-name-red：线程变量名出现在引号里就点名，不管读环境变量的写法认不认得出：scripts/experiment-apparatus/wrap_helper.py（包装函数读）、alias_env.py（`ENV = os.environ` 别名）、
#   copied_env.py（os.environ.copy() 拷贝）、in_comparison.py（`in` 比较与没人用的常量）、crates/demo/src/project_wrapper.rs、harness_wrapper.rs（包装函数读）、use_alias.rs（`use std::env::var as …`）、
#   vars_iter.rs（`std::env::vars()`）、lib.rs（#[cfg(test)] mod tests; 之后的读）、tests.rs，以及 shell 的 if_default.sh、or_default.sh（`|| X=8`）、and_default.sh（`&& X=8`），共 15 处（in_comparison.py 的比较和没人用的常量在两行上，各一处），baseline_read.py 是对照。
#   heavy-thread-defaults-per-line-red：排除表按被判行匹配，两张行只放行各自锚串落在的那一行。crates/demo/src/absorbed.rs 第 1 行 available_parallelism 与 scripts/experiment-apparatus/absorbed.py 第 2 行 os.cpu_count()
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
#   heavy-thread-defaults-capped-elsewhere-red：scripts/experiment-apparatus/mentions_capped.py 读 SAMPLE_OWN_JOBS 没设就 KeyError，同文件另一条语句里有字符串 "scripts/hosts/capped.sh"；
#   scripts/hosts/mentions_capped.sh 读 SAMPLE_SHELL_WORKERS（${…:?}），下一行 bash scripts/hosts/capped.sh default；两份都必须点名（共 2 处）。
#   heavy-thread-defaults-moved-green：setup.sh 提交 scripts/old-rig/report_cores.sh（$(nproc) 只打核数）与登记它的排除表，再 git mv 到 scripts/new-rig、表里那一行改成新路径，
#   .claude/gate.d/path-moves.tsv 登记 scripts/old-rig → scripts/new-rig：基准那一版的表经 lib-lag-baseline.py show 照登记换成新前缀，判绿并写「那一版比没涨」；
#   heavy-thread-defaults-moved-red：同一个搬迁，不放搬迁登记表，表里换了路径的那一行是基准里没有的，判红点名「排除表涨了」。
# 弄坏开关（CODE_TOOLING_BREAK）：thread-default-strings-scanned（python 与 rust 的字面量与文档串照代码扫）：heavy-thread-defaults-green 判红；
#   thread-default-half-unchecked（不认核数的一半）：heavy-thread-defaults-red 里 scripts/gate/half.py 第 4 行那条 want 找不到；
#   thread-default-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：heavy-thread-defaults-red 里「指向不存在的文件」「锚串一处都不命中」两条 want 找不到；
#   thread-default-exempt-growth-unchecked（不比基准）：heavy-thread-defaults-red 里「排除表涨了」那条 want 找不到；heavy-thread-defaults-moved-red 期望退出 1、实测 0；
#     heavy-thread-defaults-moved-green 里「那一版比没涨」那条 want 找不到；
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
exempt_table_baseline .claude/heavy-thread-exempt "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$heavy_threads_library_file" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" "$SOURCE_CODE_LINES_LIBRARY" <<'PY'
import collections, io, os, re, sys, tokenize, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[7], "source-code-lines-library", "exec"), source_lines.__dict__)
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
quoted_mask = source_lines.quoted_mask      # 逐行只留代码的读法在共用库 SOURCE_CODE_LINES_LIBRARY，这一格只给它认哪些字面量（THREAD_DEFAULT_KEPT_LITERAL）


def shell_code_lines(text):
    return source_lines.shell_code_lines(text, OUTPUT_TEXT)


def python_code_lines(text, keep_text=False):
    return source_lines.python_code_lines(text, OUTPUT_TEXT, THREAD_DEFAULT_KEPT_LITERAL, strings_scanned or keep_text)


def rust_code_lines(text, keep_text=False):
    return source_lines.rust_code_lines(text, THREAD_DEFAULT_KEPT_LITERAL, strings_scanned or keep_text)


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
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
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
    print("               起命令里不写线程（整条经 task-server/task submit 交，服务套 capped.sh <每台默认重活线程数>）；参数或环境变量给了照给的。装置读自己的线程变量的：把变量名加进 scripts/hosts/capped.sh 的 variable_names，")
    print("               或让装置没设时回退到 SINGLEFS_THREAD_CAP / 登记值（rust 装置 include scripts/experiment-apparatus/heavy_threads_default/threads.rs，python 用 default_heavy_threads）。确实不是重活（自证输入、线程数就是场景本身、只报核数不起线程）的，")
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
    print("     → 怎么办：排除只缩不涨。新写的默认值经 heavy_thread_default / default_heavy_threads 取，起命令里不写线程（线程由服务定）；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 每台默认重活线程数只经登记位取：登记值 {registered}；扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里"
      f"（表 {len(exempt_rows)} 行，每行都命中）；装置自己读的线程变量 {len(own_reads)} 个，其中 capped.sh 不设的 {own_outside} 个都回退到上限或登记值、或被判行在排除表里"
      f"（capped.sh 清单 {len(capped_names)} 个变量）；{growth_note}")
PY
}

# ── 格 hardcoded-quota：scripts/、.claude/gate.d/、.claude/hooks/ 里自己定内存上限或台数（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03：上限与机器归 task-server，脚本里不定）
#
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/ 里的 *.sh、*.py、*.rs；跳过 target/、__pycache__/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/（样本是造出来的输入）。
# 只看代码：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY（shell 去掉注释与出路文字；python 经 tokenize、rust 按引号去掉注释与字符串字面量），
#   留下的字面量只有内容整个是 run-with-memory-cap.sh 结尾的路径、--memory-cap / --hosts（可带 =常量）、内存上限写法（正整数加 K / M / G / T）、整数，
#   或名字像内存上限变量（以 MEMORY_CAP / MEMORY_MAX / MEMORY_LIMIT / MEM_CAP 收尾，不分大小写）、台数变量（以 HOSTS / HOST_COUNT / MACHINE_COUNT / PEER_COUNT 收尾）的那几种。
# 算写死的写法（每处记一个或几个种类，一行一处）：
#   run-with-memory-cap.sh <常量>：包装后面直接跟上限写法；包装不在引号里，或它所在的字面量就在它收尾（"…/run-with-memory-cap.sh" 16G、python 列表写法 "run-with-memory-cap.sh", "16G"）；
#   --memory-cap <常量>：同上的选项写法、整个字面量是「--memory-cap=常量」，以及 argparse 的 "--memory-cap" … default=<常量>；
#   写死的内存上限默认：shell 的 ${<内存上限变量>:-<常量>}（:= 与不带冒号的同算，双引号里也算）、不在引号里的 <内存上限变量>=<常量>（行首、空白或 ; & | ( 之后，环境前缀也算）；
#     python / rust 同一行里有内存上限变量的名字（标识符或引号里的名字）又有一个引号里的上限写法（"6G"）；
#   写死的台数：台数变量赋两台以上的整数（shell 不在引号里的 X=2、${X:-2}；python / rust 的 X = 2、let X = 2、关键字实参 X=2），"hosts": 2 这类键，--hosts <整数> 与它的 argparse 默认值；
#     同一份文件里这个变量另有一处赋值的右边不是整数字面量的（hosts=$(host_count) 这类现算的），整数只是没算之前的初值，不判；写 0、1 的是开关或「只在本机」，不判。
# 上限写 auto 照常量判：内存包装只认 正整数加 K / M / G / T，auto 退 251（scripts/hosts/run-with-memory-cap.sh 文件头），上限归 task-server，写 auto 同样是脚本自己定上限。
# 这一格只判写成常量与 auto 的那几种；从多机配置读、上限放在变量里、经 multi-host-run.sh 起而不写 --memory-cap、台数现算的不在这一格判：
#   那几种还在 scripts/hosts/、scripts/experiment-apparatus/ 的几份里有，判了就要给每一处加排除行，而排除只缩不涨；那几份照
#   records/2026-10-30-编译绕过内存总上限.md 收尾批 M03、M06 删干净之后再把这几种并进来。
#   引号里整句拿来当样本的、注释、文档串与出路文字里写的不算；自证里造的假值、写给人照抄的例子、这一轮不改的文件，逐行登记进排除表。
# 排除表 .claude/gate.d/hardcoded-quota-exempt.tsv：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中，都判红；只缩不涨：与改动范围的基准那一版比（exempt_table_baseline，照登记的搬迁换算路径），基准里没有的「路径:锚串」判红；
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红。
# 判不了的：变量里拼出来的命令（上限或包装路径放在变量里：["bash", RUNNER, "16G"]）、读别的文件定的数、表格与 heredoc 里写的上限（e184 的档表那类按 | 分列的行）、
#   跨行的写法（续行之后才写常量）、shell 里嵌的 python 字符串（按 shell 判）、台数写成「for host in 本机 第二台」这类逐台列出来的；排除表的理由成不成立靠 review。
# 样本：hardcoded-quota-red 的 scripts/experiment-apparatus/ 下 fixed_wrapper.sh（包装后跟 16G、引号收尾的包装路径后跟 4G）、fixed_default.sh（${…:-24G}、环境前缀 =40G、局部变量 =8G）、
#   fixed_hosts.sh（HOSTS=2、${SAMPLE_HOSTS:-3}），scripts/gate/fixed_driver.py（DEVICE_MEMORY_CAP = "6G"、host_count = 2、列表写法的包装与 "8G"、argparse 默认 "4G"、"hosts": 3），
#   .claude/hooks/fixed_hook.sh（包装后跟 12G）、.claude/gate.d/fixed_stage.sh（${GATE_SAMPLE_MEMORY_MAX:-8G}）与 scripts/experiment-apparatus/auto_cap.sh（包装后跟 auto、${…:-auto}），共 16 处逐处点名；排除表一行放行自证假值（--memory-cap 256M）、
#   一行指向不存在的文件、一行锚串不命中、setup.sh 建 git 仓提交一版不带 grown-row 那一行的表再加回来（表涨了），后三样都点名。
#   hardcoded-quota-green：从多机配置读的上限、包装只带 --status、multi-host-run.sh 不写 --memory-cap、hosts=1 与 emit_hosts=1 / two_hosts=0 这类开关、
#   peer_count=2 之后又照 --emit-hosts 现算，引号里整句的样本、注释、echo、文档串里写的常量，
#   .claude/gate.d/fixtures/ 下写死的样本，都不点名；排除表一行放行自证假值，判绿并报扫了 6 份、表 1 行每行都命中。
# 弄坏开关（CODE_TOOLING_BREAK）：quota-strings-scanned（python 与 rust 的字面量与文档串照代码扫）：hardcoded-quota-green 判红（sentence_samples.py 文档串里的包装与 --memory-cap 被点名）；
#   quota-sentence-samples-counted（引号里整句的样本也算）：hardcoded-quota-green 判红（sentence_samples.sh 的 feed 那一行被点名）；
#   quota-memory-cap-option-unchecked（不认 --memory-cap <常量>）：hardcoded-quota-red 里 fixed_driver.py:15 那条 want 与「16 处」找不到；
#     hardcoded-quota-green 判红（排除表那一行的锚串落在 --memory-cap 256M 那一行上，那一行不再是被判行，锚串一处都不命中）；
#   quota-memory-variable-unchecked（不认写死的内存上限默认）：hardcoded-quota-red 里 fixed_default.sh、fixed_stage.sh、fixed_driver.py:5 那几条 want 与「16 处」找不到；
#   quota-host-count-unchecked（不认写死的台数）：hardcoded-quota-red 里 fixed_hosts.sh、fixed_driver.py:6、fixed_driver.py:16 那几条 want 与「16 处」找不到；
#   quota-host-computed-unseen（台数变量另有现算的赋值也照判它的整数初值）：hardcoded-quota-green 判红（computed_run.sh 的 peer_count=2 被点名）；
#   quota-auto-allowed（照改前不把 auto 当写死的上限）：hardcoded-quota-red 里 auto_cap.sh 那两条 want 与「16 处」找不到；
#   quota-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：hardcoded-quota-red 里「指向不存在的文件」「一处被判行都不命中」两条 want 找不到；
#   quota-exempt-growth-unchecked（排除表不比基准）：hardcoded-quota-red 里「排除表涨了」那条 want 找不到。
cell_hardcoded_quota() {
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/hardcoded-quota-exempt.tsv "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken, base_state, base_name, base_table = sys.argv[3:7]

EXEMPT_TABLE = ".claude/gate.d/hardcoded-quota-exempt.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
SIZE = r"[0-9]+[KMGT]" if broken == "quota-auto-allowed" else r"(?:[0-9]+[KMGT]|auto)"      # systemd 的 MemoryMax 写法与 auto：包装不认 auto（退 251），写 auto 也是脚本自己定上限
MEMORY_NAME = r"(?i:\b[A-Za-z0-9_]*(?:MEMORY_CAP|MEMORY_MAX|MEMORY_LIMIT|MEM_CAP)\b)"      # 名字以这几样收尾：REPLAY_MEMORY_CAP、GATE_VERUS_MEMORY_MAX、memory_cap；RUN_WITH_MEMORY_CAP_RESERVE 这类包装自己的设置不算
HOST_NAME = r"(?i:\b[A-Za-z0-9_]*(?:HOSTS|HOST_COUNT|MACHINE_COUNT|PEER_COUNT)\b)"
HOST_COUNT_LITERAL = r"(?:[2-9]|[1-9][0-9]+)"      # 两台以上：写 0、1 的是开关或「只在本机」，不算写死台数
KEPT_LITERAL = re.compile(r".*run-with-memory-cap\.sh|--memory-cap(?:=" + SIZE + r")?|--hosts(?:=[0-9]+)?|" + SIZE + r"|[0-9]+|" + MEMORY_NAME + "|" + HOST_NAME)
PREFILTER = re.compile(r"(?i)memory-cap|memory_cap|memory_max|memory_limit|mem_cap|hosts|host_count|machine_count|peer_count")
WRAPPER_FORM = "run-with-memory-cap.sh <常量>"
OPTION_FORM = "--memory-cap <常量>"
MEMORY_VARIABLE_FORM = "写死的内存上限默认"
HOST_FORM = "写死的台数"
# 跟在包装或选项后面的常量：包装或选项不在引号里，或它所在的那个字面量就在它收尾（"…/run-with-memory-cap.sh" 16G、python 的列表写法 "run-with-memory-cap.sh", "16G"），
# 或整个字面量就是「选项=常量」；引号里整句拿来当样本、当出路文字的不算
WRAPPER = re.compile(r"run-with-memory-cap\.sh(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+)" + SIZE + r"(?![\w.])")
OPTION = re.compile(r"--memory-cap(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+|=)" + SIZE + r"(?P<tail>[\"']?)(?![\w.])")
OPTION_DEFAULT = re.compile(r"[\"']--memory-cap[\"'][^\n]*?\bdefault\s*=\s*[\"']?" + SIZE + r"(?![\w.])")
HOST_OPTION = re.compile(r"--hosts(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+|=)" + HOST_COUNT_LITERAL + r"(?P<tail>[\"']?)(?![\w.])")
HOST_OPTION_DEFAULT = re.compile(r"[\"']--hosts[\"'][^\n]*?\bdefault\s*=\s*" + HOST_COUNT_LITERAL + r"(?![\w.])")
SHELL_DEFAULTED = r"\$\{(?P<name>{name}):?[-=]{value}\}"
SHELL_MEMORY_DEFAULTED = re.compile(SHELL_DEFAULTED.replace("{name}", MEMORY_NAME).replace("{value}", SIZE))
SHELL_HOST_DEFAULTED = re.compile(SHELL_DEFAULTED.replace("{name}", HOST_NAME).replace("{value}", HOST_COUNT_LITERAL))
# shell 的赋值（行首、空白或 ; & | ( 之后，可带 export / local / declare / readonly，环境前缀 X=16G 命令 也算）：右边取到下一个不在引号与 $( ) 里的 ; & | ) 或空白为止
SHELL_ASSIGNMENT = re.compile(r"(?:^|(?<=[\s;&|(]))(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)="
                              r"(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|\$\((?:[^()]|\([^()]*\))*\)|[^;&|\"'\s()])*)")
# python / rust 的赋值：NAME = 值、NAME: 类型 = 值、let [mut] NAME = 值、const NAME: 类型 = 值；右边取到行尾
CODE_ASSIGNMENT = re.compile(r"(?:\blet[ \t]+(?:mut[ \t]+)?|\b(?:const|static)[ \t]+)?\b(?P<name>[A-Za-z_][A-Za-z0-9_]*)[ \t]*(?::[ \t]*[^=\n]+?)?(?<![=!<>+\-*/%&|^])=(?!=)[ \t]*(?P<rhs>[^\n]*)")
QUOTED_SIZE = re.compile(r"[\"']" + SIZE + r"[\"']")
QUOTED_HOST_KEY = re.compile(r"[\"']" + HOST_NAME + r"[\"']\s*:\s*" + HOST_COUNT_LITERAL + r"(?![\w.])")
QUOTED_MEMORY_NAME = re.compile(r"[\"']" + MEMORY_NAME + r"[\"']")
BARE_MEMORY_NAME = re.compile(MEMORY_NAME)
INTEGER_LITERAL = re.compile(r"[\"']?[0-9]+[\"']?")
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
strings_scanned = broken == "quota-strings-scanned"


def standalone(code, found, mask):
    """跟在包装或选项后面的常量算不算写在跑法里：包装或选项不在引号里，或它所在的字面量就在它收尾，或整个字面量就是「选项=常量」。"""
    if broken == "quota-sentence-samples-counted":
        return True
    start = found.start()
    if not mask[start] or found.group("close"):
        return True
    tail = found.groupdict().get("tail")
    return bool(tail) and start > 0 and code[start - 1] in "\"'"


def computed_host_names(language, code):
    """这份文件里有哪一处赋值的右边不是整数字面量的台数变量（hosts=$(host_count) 这类现算的）：它在别处给的整数只是没算之前的初值，不判。"""
    if broken == "quota-host-computed-unseen":
        return set()
    pattern = SHELL_ASSIGNMENT if language == "shell" else CODE_ASSIGNMENT
    names = set()
    for found in pattern.finditer(code):
        if re.fullmatch(HOST_NAME, found.group("name")) and not INTEGER_LITERAL.fullmatch(found.group("rhs").strip().rstrip(";,")):
            names.add(found.group("name"))
    return names


def forms_of(language, code, computed):
    names = []
    mask = source_lines.quoted_mask(code)
    if any(standalone(code, found, mask) for found in WRAPPER.finditer(code)):
        names.append(WRAPPER_FORM)
    if broken != "quota-memory-cap-option-unchecked" and (any(standalone(code, found, mask) for found in OPTION.finditer(code)) or OPTION_DEFAULT.search(code)):
        names.append(OPTION_FORM)
    if broken != "quota-memory-variable-unchecked":
        if language == "shell":
            assigned = any(re.fullmatch(MEMORY_NAME, found.group("name")) and not mask[found.start("name")]
                           and re.fullmatch(r"[\"']?" + SIZE + r"[\"']?", found.group("rhs")) for found in SHELL_ASSIGNMENT.finditer(code))
            if assigned or SHELL_MEMORY_DEFAULTED.search(code):
                names.append(MEMORY_VARIABLE_FORM)
        elif QUOTED_SIZE.search(code) and (QUOTED_MEMORY_NAME.search(code) or BARE_MEMORY_NAME.search(re.sub(r"[\"'][^\"']*[\"']", " ", code))):
            names.append(MEMORY_VARIABLE_FORM)
    if broken != "quota-host-count-unchecked":
        if language == "shell":
            assigned = any(re.fullmatch(HOST_NAME, found.group("name")) and found.group("name") not in computed and not mask[found.start("name")]
                           and re.fullmatch(r"[\"']?" + HOST_COUNT_LITERAL + r"[\"']?", found.group("rhs")) for found in SHELL_ASSIGNMENT.finditer(code))
            defaulted = any(found.group("name") not in computed for found in SHELL_HOST_DEFAULTED.finditer(code))
        else:
            assigned = any(re.fullmatch(HOST_NAME, found.group("name")) and found.group("name") not in computed
                           and re.match(HOST_COUNT_LITERAL + r"(?![\w.])[ \t]*(?:[;,)]|$)", found.group("rhs").strip()) for found in CODE_ASSIGNMENT.finditer(code))
            defaulted = QUOTED_HOST_KEY.search(code) is not None
        if assigned or defaulted or any(standalone(code, found, mask) for found in HOST_OPTION.finditer(code)) or HOST_OPTION_DEFAULT.search(code):
            names.append(HOST_FORM)
    return names


def code_lines_of(language, text):
    if language == "shell":
        return source_lines.shell_code_lines(text, OUTPUT_TEXT)
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, KEPT_LITERAL, strings_scanned)
    return source_lines.rust_code_lines(text, KEPT_LITERAL, strings_scanned)


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


problems = []
hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if not PREFILTER.search(text):
        continue
    original = text.split("\n")
    code_lines = code_lines_of(language, text)
    computed = computed_host_names(language, "\n".join(code_lines))
    for number, code in enumerate(code_lines, 1):
        if not code.strip():
            continue
        names = forms_of(language, code, computed)
        if names:
            hits.setdefault(path, []).append((number, names, original[number - 1] if number <= len(original) else code))

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
if broken != "quota-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "quota-exempt-growth-unchecked":
    base_keys = set()
    for line in open(base_table, encoding="utf-8").read().split("\n"):
        found = EXEMPT_ROW.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.add((found.group("path"), found.group("anchor")))
    grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py、*.rs，也没有 {EXEMPT_TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处在脚本里自己定了内存上限或台数（上限与机器归 task-server）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：删掉脚本里定的上限与台数，整条经 task-server/task submit -- <命令> 交：服务按记录给上限、撞了放大重排，机器与分片也归它（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03）。")
    print(f"               确实不是跑法（自证造的假值、写给人照抄的文档例子）的，登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经删掉（上限交给 task-server）就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的跑法不定上限与台数，整条经 task-server/task submit 交；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 内存上限与台数都不写死：扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里（表 {len(exempt_rows)} 行，每行都命中）；{growth_note}")
PY
}

# ── 格 shared-data-root-literals：scripts/、.claude/hooks/、.claude/gate.d/、.claude/scripts/、task-server/ 里经 ~ 或 $HOME 取数据根下共享的那几项
#   （records/2026-10-31-task-server巡检.md 问题 73：第二台的 ~ 是另一个目录，经它取的写读都落进第二台本地那一份，两台读不到同一份）
#
# 名单：这一道所在仓里 .claude/hooks/lib_work_root.py 的 SHARED_DATA_ENTRIES（共享的，判）与 PER_HOST_DATA_ENTRIES（按台的，不判），读不出就判红、出路是先换上那一版库。
#   运行记录另判（.claude/rules/verification.md 两节：运行记录、产物放共享根，不经 ~ 与 $HOME 取）：这一格里的 RUN_RECORD_ENTRIES 点名的几项（峰值表 memory-peaks.tsv、
#   崩溃放量的运行目录 crash-amplification）分台靠共享表里的「哪一台」列、不靠 ~，经 ~ 或 $HOME 取的一律照「运行记录按台取」判，不看 lib_work_root.py 把它登在哪张名单里。
#   数据根本身（~/.local/share/singlefs 后面不接名字，或接的是变量）与名单外的名字也判：取数据根只经 lib_work_root.py（欠账 C949（任务只在一台机器上跑得起）的静态那一半）。
# 扫哪些：被判的根下这五处的 *.sh、*.py；跳过 target/、__pycache__/、.git/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/（样本是造出来的输入），
#   跳过 .claude/hooks/lib_work_root.py 自己（数据根只在它里面定义）。
# 只看代码（放行规则）：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY。shell 去掉整行注释、行尾注释与出路文字（echo / printf / howto / bad / die / say 的字面量），
#   引号里带空白的那一段是句子（出路文字、样本命令），去掉；嵌在 shell 里的 python 的三引号整段带空白的同样去掉（文档串）。
#   python 经 tokenize 去掉注释与带空白的字符串字面量（文档串、出路文字），内容不带空白的字面量照代码判（路径就在这里头）；f-string 的字面段一概不判。
#   自证函数体（python 的 def …selftest…( 按缩进收尾，shell 的 …selftest…() { 收到同一缩进的 }）里造的样本不判。
# 算经 ~ 或 $HOME 取的写法（每处一行，两种起法）：
#   ~/、$HOME/、${HOME}/ 与 ${HOME:?}/ 这类带修饰的写法起头、后面跟 /.local/share/singlefs（后面不再接字母、数字、_ . -），再接 /<名字> 或不接；
#   expanduser("~") 后面紧接 ".local", "share", "singlefs"[, "<名字>"] 或 ".local/share/singlefs[/<名字>]"（os.path.join 的写法）。
#   按名字归类（一行里几处各归各的，种类去重后点名）：运行记录的名字（连同以它起头的 <名字>-<…>）点名「运行记录按台取」；共享的名字照起法点名；按台的名字不点名；
#   不接名字或接的是变量点名「数据根本身经 ~ 或 $HOME 取」；名单两半里都没有的名字点名「数据根下名单外的一项」。
# 排除表 .claude/gate.d/shared-data-root-literals-exempt.tsv：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中，都判红；只缩不涨：与改动范围的基准那一版比（exempt_table_baseline），基准里没有的「路径:锚串」判红；
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红。
# 判不了的：变量里拼出来的路径（home="$HOME"; "$home/.local/share/singlefs/work"）、f-string 里的、带空白的命令串里的（bash -c "… $HOME/.local/share/singlefs/work …"）、
#   Path.home() 拼的、XDG_DATA_HOME 下拼的、读别的文件定的；自证函数之外造的样本与认路径文字的代码（不取目录）逐行登记进排除表，理由成不成立靠 review。
# 样本：shared-data-root-literals-red 的 scripts/hosts/home_literal.sh（$HOME/、~/、${HOME:?}/、${HOME}/ 各一处）、scripts/gate/home_literal.py（~/ 起头的字面量、
#   expanduser("~") 的两种拼法）、.claude/hooks/home_hook.sh、.claude/gate.d/home_stage.sh、.claude/scripts/home_tool.sh、task-server/home_server.py 各一处，共 11 处逐处点名；
#   scripts/agents/text_reader.py 认路径文字的一行由排除表放行；排除表另一行指向不存在的文件、一行锚串不命中、setup.sh 建 git 仓提交一版不带 grown-row 那一行的表再加回来（表涨了），后三样都点名。
#   shared-data-root-literals-green：经 lib_work_root.py 取的、注释、echo、带空白的出路字面量、文档串、按台的 qemu 与 rocksdb-prebuilt、自证函数体里的样本、
#   样本自己的 .claude/hooks/lib_work_root.py、.claude/gate.d/fixtures/ 下的，都不点名；排除表一行放行认路径文字的那一行，判绿并报扫了 3 份、表 1 行每行都命中。
#   shared-data-root-literals-data-root-red：scripts/hosts/data_root.sh（数据根本身一处、名单外的名字一处、crash-amplification 与 memory-peaks.tsv 两处运行记录）、
#   scripts/gate/data_root.py（expanduser("~") 拼出来的数据根本身一处、以 crash-amplification- 起头的运行目录一处），共 6 处逐处点名；没有排除表。
# 弄坏开关（CODE_TOOLING_BREAK）：shared-literals-prose-scanned（带空白的字面量也照代码扫）：shared-data-root-literals-green 判红（出路字面量、文档串与 refuse 的参数被点名）；
#   shared-literals-join-form-unseen（不认 expanduser("~") 的拼法）：shared-data-root-literals-red 里 home_literal.py:4、:5 两条 want 与「11 处」找不到；
#   shared-literals-braced-home-unseen（不认 ${HOME}/ 与 ${HOME:?}/）：shared-data-root-literals-red 里 home_literal.sh:5、:6 两条 want 与「11 处」找不到；
#   shared-literals-per-host-judged（按台的也判）：shared-data-root-literals-green 判红（qemu 与 rocksdb-prebuilt 被点名）；
#   shared-literals-definition-file-judged（连定义那一份也判）：shared-data-root-literals-green 判红（样本的 lib_work_root.py 被点名）；
#   shared-literals-selftest-bodies-judged（连自证函数体也判）：shared-data-root-literals-green 判红（两份自证里造的样本被点名）；
#   shared-literals-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：shared-data-root-literals-red 里「指向不存在的文件」「一处被判行都不命中」两条 want 找不到；
#   shared-literals-exempt-growth-unchecked（排除表不比基准）：shared-data-root-literals-red 里「排除表涨了」那条 want 找不到；
#   shared-literals-data-root-unseen（数据根本身与名单外的名字不判）：shared-data-root-literals-data-root-red 里那三条 want 与「6 处」找不到；
#   shared-literals-run-records-unseen（运行记录不判）：shared-data-root-literals-data-root-red 里那三条运行记录的 want 与「6 处」找不到。
#   shared-literals-join-form-unseen 另让 shared-data-root-literals-data-root-red 里 data_root.py:4 那条 want 与「6 处」找不到。
cell_shared_data_root_literals() {
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/shared-data-root-literals-exempt.tsv "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" "$STAGE_REPOSITORY/.claude/hooks/lib_work_root.py" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken, base_state, base_name, base_table, work_root_library = sys.argv[3:8]

EXEMPT_TABLE = ".claude/gate.d/shared-data-root-literals-exempt.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/hooks", ".claude/gate.d", ".claude/scripts", "task-server")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
DEFINITION_FILE = ".claude/hooks/lib_work_root.py"
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python"}
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
WITHOUT_WHITESPACE = re.compile(r"\S*")      # python 的字面量：内容里没有空白的留着当代码判（路径），带空白的是句子（文档串、出路文字），去掉
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
PYTHON_SELFTEST_DEF = re.compile(r"^(?P<indent>[ \t]*)(?:async[ \t]+)?def[ \t]+\w*self_?test\w*[ \t]*\(")
SHELL_SELFTEST_FUNCTION = re.compile(r"^(?P<indent>[ \t]*)(?:function[ \t]+)?[A-Za-z_]\w*self_?test\w*[ \t]*\([ \t]*\)[ \t]*\{?[ \t]*$")
prose_scanned = broken == "shared-literals-prose-scanned"

# 名单只在 .claude/hooks/lib_work_root.py 里（这一道所在仓里那一份）：共享的判，按台的不判
try:
    import importlib.util
    specification = importlib.util.spec_from_file_location("lib_work_root_for_shared_data_root_literals", work_root_library)
    work_roots = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(work_roots)
    shared_entries = tuple(work_roots.SHARED_DATA_ENTRIES)
    per_host_entries = tuple(work_roots.PER_HOST_DATA_ENTRIES)
except (OSError, ImportError, AttributeError) as error:
    print(f"  ✗ 读不出数据根的两张名单（{work_root_library} 的 SHARED_DATA_ENTRIES、PER_HOST_DATA_ENTRIES）：{error}")
    print("     → 怎么办：名单只在 .claude/hooks/lib_work_root.py 里定义；那份库还是没有这两张名单的那一版，就先经 scripts/gate/hook-swap.py 把它换上，再跑这一格。")
    sys.exit(1)
# 运行记录（.claude/rules/verification.md「判定库只有一份」：用时表、峰值表、占盘时序、上次红的清单，「所有任务都要多机可用」：运行记录与产物）放共享根；
#   分台靠共享表里的「哪一台」列，不靠各台自己的 ~：这几样经 ~ 或 $HOME 取就是按台取，不论 lib_work_root.py 把它登在哪张名单里（崩溃放量的运行目录已挪到工作根下）
RUN_RECORD_ENTRIES = ("memory-peaks.tsv", "crash-amplification")
data_root_judged = broken != "shared-literals-data-root-unseen"
run_records_judged = broken != "shared-literals-run-records-unseen"
HOME_PREFIX = r"(?:~|\$HOME\b)" if broken == "shared-literals-braced-home-unseen" else r"(?:~|\$HOME\b|\$\{HOME(?:[:?+=-][^}]*)?\})"
NAME = r"(?:/(?P<entry>[A-Za-z0-9_.-]*))?"
LITERAL_FORM = re.compile(r"(?<![\w$])" + HOME_PREFIX + r"/\.local/share/singlefs(?![A-Za-z0-9_.-])" + NAME)
QUOTE = r"[\"']"
EXPANDED_HOME = r"expanduser\(\s*" + QUOTE + r"~" + QUOTE + r"\s*\)\s*,\s*"
JOIN_FORMS = (re.compile(EXPANDED_HOME + QUOTE + r"\.local" + QUOTE + r"\s*,\s*" + QUOTE + r"share" + QUOTE + r"\s*,\s*" + QUOTE + r"singlefs" + QUOTE
                         + r"(?:\s*,\s*" + QUOTE + r"(?P<entry>[A-Za-z0-9_.-]*))?"),
              re.compile(EXPANDED_HOME + QUOTE + r"\.local/share/singlefs(?![A-Za-z0-9_.-])" + NAME))
LITERAL_NAME = "~/、$HOME/ 或 ${HOME}/ 起头的字面量"
JOIN_NAME = "expanduser(\"~\") 拼出来的"
DATA_ROOT_NAME = "数据根本身经 ~ 或 $HOME 取"
UNREGISTERED_NAME = "数据根下名单外的一项"
RUN_RECORD_NAME = "运行记录按台取"


def per_host_entry_of(entry):
    """名字是按台的那一项（连同以它起头的 <名字>-<…>）就交那一项，不是交 None。"""
    return next((name for name in per_host_entries if entry == name or entry.startswith(name + "-")), None)


def run_record_entry_of(entry):
    """名字是运行记录那一项（连同以它起头的 <名字>-<…>）就交那一项，不是交 None。"""
    return next((name for name in RUN_RECORD_ENTRIES if entry == name or entry.startswith(name + "-")), None)


def classified(entry, form_name):
    """一处命中按名字归类：运行记录不论登在哪张名单里都点名「运行记录按台取」；共享的照 form_name；按台的只在弄坏开关 per-host-judged 时判；
    没给名字是数据根本身；名单外的另记。不判交 None。"""
    if entry and run_record_entry_of(entry) is not None:
        return RUN_RECORD_NAME if run_records_judged else None
    if entry and entry in shared_entries:
        return form_name
    per_host = per_host_entry_of(entry) if entry else None
    if per_host is not None:
        return form_name if broken == "shared-literals-per-host-judged" else None
    if not data_root_judged:
        return None
    return DATA_ROOT_NAME if not entry else UNREGISTERED_NAME


def blank_regions(text, pattern, keep):
    """把 pattern 命中、keep 判为不留的那几段换成空格（换行照留，行号不变）。"""
    def replace(found):
        return found.group(0) if keep(found) else re.sub(r"[^\n]", " ", found.group(0))
    return pattern.sub(replace, text)


def without_prose_strings(line):
    """shell 的一行：引号里带空白的那一段是句子（出路文字、样本命令），换成空格；不带空白的照留（路径与赋值的右边）。"""
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def selftest_lines(original):
    """自证函数体里的行号（python 的 def …selftest…( 按缩进收尾，shell 的 …selftest…() { 收到同一缩进的 }）：里面造的是样本，不判。"""
    if broken == "shared-literals-selftest-bodies-judged":
        return set()
    inside, index = set(), 0
    while index < len(original):
        python_start, shell_start = PYTHON_SELFTEST_DEF.match(original[index]), SHELL_SELFTEST_FUNCTION.match(original[index])
        if python_start:
            indent = len(python_start.group("indent").expandtabs())
            end = index + 1
            while end < len(original) and (not original[end].strip() or len(original[end].expandtabs()) - len(original[end].expandtabs().lstrip()) > indent):
                end += 1
            inside.update(range(index + 1, end + 1))
            index = end
        elif shell_start:
            closing = re.compile(r"^" + re.escape(shell_start.group("indent")) + r"\}[ \t;]*(?:#.*)?$")
            end = index + 1
            while end < len(original) and not closing.match(original[end]):
                end += 1
            inside.update(range(index + 1, end + 2))
            index = end + 1
        else:
            index += 1
    return inside


def code_lines_of(language, text):
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, prose_scanned)
    if not prose_scanned:   # shell 里嵌的 python 的文档串：三引号里带空白的整段是句子
        text = blank_regions(text, TRIPLE_QUOTED, lambda found: not any(character.isspace() for character in found.group(2)))
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


def forms_of(code):
    names = []
    for found in LITERAL_FORM.finditer(code):
        name = classified(found.group("entry"), LITERAL_NAME)
        if name and name not in names:
            names.append(name)
    if broken != "shared-literals-join-form-unseen":
        for form in JOIN_FORMS:
            for found in form.finditer(code):
                name = classified(found.group("entry"), JOIN_NAME)
                if name and name not in names:
                    names.append(name)
    return names


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                path = os.path.join(current, name)
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX and (path != DEFINITION_FILE or broken == "shared-literals-definition-file-judged"):
                    yield directory, path, LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


problems = []
hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "share" not in text or "singlefs" not in text:
        continue
    original = text.split("\n")
    samples = selftest_lines(original)
    for number, code in enumerate(code_lines_of(language, text), 1):
        if number in samples or not code.strip():
            continue
        names = forms_of(code)
        if names:
            hits.setdefault(path, []).append((number, names, original[number - 1] if number <= len(original) else code))

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
if broken != "shared-literals-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "shared-literals-exempt-growth-unchecked":
    base_keys = set()
    for line in open(base_table, encoding="utf-8").read().split("\n"):
        found = EXEMPT_ROW.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.add((found.group("path"), found.group("anchor")))
    grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py，也没有 {EXEMPT_TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处经 ~ 或 $HOME 取了数据根下共享的那几项（第二台的 ~ 是另一个目录，写的读的都落进它本地那一份）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：经 lib_work_root.py 的 shared_data_root() 取：python 载入 .claude/hooks/lib_work_root.py 调 shared_data_path(<名>)（工作根调 configured_work_root()），")
    print("               shell 写 \"$(python3 <仓根>/.claude/hooks/lib_work_root.py shared <名>)\"（工作根不带参数），取不到就停、不退回 ~ 下；名单在那份库文件头「数据根」表。")
    print(f"               确实不是取目录的（认会话记录里写的路径文字、这一轮不改的文件），登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经改成经 lib_work_root.py 取就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的取法经 lib_work_root.py 的 shared_data_root() 取；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 数据根下共享的几项都不经 ~ 与 $HOME 取：扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里（表 {len(exempt_rows)} 行，每行都命中）；{growth_note}"
      f"；数据根本身、名单外的名字与运行记录（{'、'.join(RUN_RECORD_ENTRIES)}）一起查了")
PY
}

# ── 格 cp-preserve-mode：项目脚本里带保留权限的 cp（records/2026-10-31-task-server巡检.md 问题 82：从 NFS 共享树往第二台本地盘拷时，cp 保留权限那一步
#   退非 0、报 Operation not supported，后面接 && 的那一步不跑、整条命令红；本机拷的是本地盘不撞，只在第二台红）
# 扫哪些：被判的根下 scripts/、.claude/hooks/、.claude/gate.d/、.claude/scripts/、task-server/、prune-server/ 里的 *.sh、*.py 与不带后缀、第一行是 #! 起 bash / sh / python 的文件；
#   跳过 target/、__pycache__/、.git/；.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/ 下只扫样本根的搭建脚本（fixtures/<阶段>/<样本>/setup.sh，喂样本时照真跑），
#   别的样本文件是造出来的输入、不扫；规范副本 .claude/singlefs-ai-sop/ 不在射程里（改在上游，照 CLAUDE.md「规范从哪来」）。
# 只看代码：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY（同 shared-data-root-literals 格）：shell 去掉整行注释、行尾注释与出路文字，引号里带空白的那一段（样本命令、
#   出路文字）去掉，嵌在 shell 里的 python 的三引号整段带空白的同样去掉；python 经 tokenize 去掉注释与带空白的字符串字面量，不带空白的字面量照代码判
#   （argv 列表里的 "cp"、"-a" 就在这里头）。自证函数体里的照判（自证真起 cp）。
# 算保留权限的写法（每处一行，两种）：
#   shell：命令词是 cp（前面是行首、空白、; & | ( { ` / 或 $(），到 ; & | ( ) < > 为止的参数里有一项是 -a、-p（单个 - 起头、几个字母合写的也算，例 -ar、-rp）、
#     --archive、不带值的 --preserve，或 --preserve= 的值里有 mode 或 all；
#   python：argv 列表里 "cp"（或 'cp'）之后的列表项里有同样的一项（整项是带引号的那一个词）。
# 改法：要时间戳的写 --preserve=timestamps，要链接的再加 links（整棵树写 cp -dR --preserve=timestamps,links），都不要的写 cp -R 或 cp；python 改用 shutil.copy2、
#   shutil.copytree(…, symlinks=True)，或 argv 写成同样的选项。没有豁免表：射程里的每一处都改。
# 判不了的：变量里拼出来的命令、带空白的命令串里的（bash -c "cp -a …"、subprocess 的 shell=True 字符串）、install -p、rsync -a 这类别的工具（不报这个错），靠 review。
# 样本：cp-preserve-mode-red 的 scripts/hosts/copy_tree.sh（-a、-rp 合写、--archive、if 里的 --preserve=mode,timestamps、$( 里不带值的 --preserve、/bin/cp -p）、
#   scripts/gate/copy_tree.py（argv 列表里的 "-a" 与 '--preserve=all'）、task-server/sample_server（不带后缀、#! 起 bash 的文件里的 -pR）、
#   .claude/gate.d/fixtures/sample-stage/case/setup.sh（样本根的搭建脚本里的 -a），共 10 处逐处点名；同一份里样本根的别的文件（data.sh 里的 -a）不点名。
#   cp-preserve-mode-green：-dR --preserve=timestamps,links、--preserve=timestamps、-R、-P、--parents、出路文字、注释、带空白的样本命令串、python 的文档串与 shutil、
#   scp -p 与 rsync -a、mkdir -p、样本根下不是 setup.sh 的文件、.claude/singlefs-ai-sop/ 下的 cp -a，都不点名；判绿并报扫了 3 份。
# 弄坏开关（CODE_TOOLING_BREAK）：cp-preserve-short-cluster-unseen（只认单写的 -a、-p，不认合写）：cp-preserve-mode-red 里 copy_tree.sh:4、task-server/sample_server:3
#   两条 want 与「10 处」找不到；cp-preserve-python-unseen（不认 python 的 argv 列表）：cp-preserve-mode-red 里 copy_tree.py:4、:5 两条 want 与「10 处」找不到；
#   cp-preserve-prose-scanned（带空白的字面量也照代码扫）：cp-preserve-mode-green 判红（样本命令串被点名）；
#   cp-preserve-fixture-setups-unscanned（样本根的 setup.sh 也不扫）：cp-preserve-mode-red 里 setup.sh:3 那条 want 与「10 处」找不到；cp-preserve-mode-green 里「扫了 3 份」找不到（只扫了 2 份）。
cell_cp_preserve_mode() {
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken = sys.argv[3]

SCAN_DIRECTORIES = ("scripts", ".claude/hooks", ".claude/gate.d", ".claude/scripts", "task-server", "prune-server")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
FIXTURE_SETUP = re.compile(r"\.claude/(?:gate\.d|hooks)/fixtures/[^/]+/[^/]+/setup\.sh")
FIXTURE_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
SHEBANG = re.compile(r"#!\S*(?:\s+\S+)?\s*.*\b(?P<interpreter>bash|sh|python3?)\b")
WITHOUT_WHITESPACE = re.compile(r"\S*")      # python 的字面量：内容里没有空白的留着当代码判（argv 的一项），带空白的是句子，去掉
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
prose_scanned = broken == "cp-preserve-prose-scanned"
SHORT_FLAG = r"-[ap]" if broken == "cp-preserve-short-cluster-unseen" else r"-[A-Za-z]*[ap][A-Za-z]*"
PRESERVING = r"(?:" + SHORT_FLAG + r"|--archive|--preserve|--preserve=[A-Za-z,]*(?:mode|all)[A-Za-z,]*)"
SHELL_COMMAND = re.compile(r"(?:^|(?<=[\s;&|({`/])|(?<=\$\())cp(?=\s)(?P<arguments>[^;&|()<>]*)")
SHELL_FLAG = re.compile(r"(?:^|(?<=\s))(?P<flag>" + PRESERVING + r")(?=\s|$)")
PYTHON_LIST = re.compile(r"""(?<![\w"'])["']cp["'](?P<items>(?:\s*,\s*[^,\]\)]+)*)""")
PYTHON_ITEM = re.compile(r"""(["'])(?P<content>.*?)\1""")   # argv 列表里带引号的一项（逗号写在引号里的也整项取）
PRESERVING_WORD = re.compile(PRESERVING)


def without_prose_strings(line):
    """shell 的一行：引号里带空白的那一段是句子（出路文字、样本命令），换成空格；不带空白的照留。"""
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def language_of(path, name):
    suffix = os.path.splitext(name)[1]
    if suffix in (".sh", ".py"):
        return "shell" if suffix == ".sh" else "python"
    if suffix:
        return None
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            first = handle.readline()
    except OSError:
        return None
    found = SHEBANG.match(first)
    if not found:
        return None
    return "python" if found.group("interpreter").startswith("python") else "shell"


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES)
            for name in sorted(files):
                path = os.path.join(current, name)
                if path.startswith(FIXTURE_PREFIXES) and (broken == "cp-preserve-fixture-setups-unscanned" or not FIXTURE_SETUP.fullmatch(path)):
                    continue
                language = language_of(path, name)
                if language:
                    yield directory, path, language


def code_lines_of(language, text):
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, prose_scanned)
    if not prose_scanned:   # shell 里嵌的 python 的文档串：三引号里带空白的整段是句子
        text = TRIPLE_QUOTED.sub(lambda found: found.group(0) if not any(character.isspace() for character in found.group(2))
                                 else re.sub(r"[^\n]", " ", found.group(0)), text)
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


def preserving_flags(code):
    flags = []
    for command in SHELL_COMMAND.finditer(code):
        flags += [found.group("flag") for found in SHELL_FLAG.finditer(command.group("arguments"))]
    if broken != "cp-preserve-python-unseen":
        for listing in PYTHON_LIST.finditer(code):
            flags += [item.group("content") for item in PYTHON_ITEM.finditer(listing.group("items")) if PRESERVING_WORD.fullmatch(item.group("content"))]
    return flags


problems, hits = [], []
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "cp" not in text:
        continue
    original = text.split("\n")
    for number, code in enumerate(code_lines_of(language, text), 1):
        for flag in preserving_flags(code):
            hits.append((path, number, flag, original[number - 1] if number <= len(original) else code))

if scanned == 0:
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有要扫的脚本")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if hits:
    failed = True
    print(f"  ✗ {len(hits)} 处 cp 带了保留权限（-a、-p、--archive、不带值的 --preserve、带 mode 或 all 的 --preserve=）：从 NFS 共享树拷到第二台本地盘时保留权限那一步退非 0"
          "（Operation not supported），命令跟着红：")  # gate-lint:summary
    for path, number, flag, line in hits:
        print(f"     {path}:{number}  {flag}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：要时间戳的写 --preserve=timestamps，要链接的再加 links（整棵树写 cp -dR --preserve=timestamps,links），都不要的写 cp -R 或 cp；")
    print("               python 改用 shutil.copy2、shutil.copytree(…, symlinks=True)，或 argv 写成同样的选项。这一格没有豁免表，射程里的每一处都改。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 项目脚本里没有带保留权限的 cp：扫了 {scanned} 份（{by_directory}）")
PY
}

# ── 格 peer-launch-direct：项目脚本里直接起 multi-host-run.sh --peer 往第二台起活（records/2026-10-31-task-server巡检.md 问题 90、94：驱动自己连第二台，
#   绕过服务的排队、额度与挑机器；.claude/rules/implementation-workflow.md「分片」一条写的是「任务里面不自己连第二台」）
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/scripts/ 里的 *.sh、*.py 与不带后缀、第一行是 #! 起 bash / sh / python 的文件；跳过 target/、__pycache__/、.git/；
#   .claude/gate.d/fixtures/ 下只扫样本根的搭建脚本（fixtures/<阶段>/<样本>/setup.sh）；scripts/hosts/multi-host-run.sh 自己（连同它的自证）不扫：它就是往第二台起活的那一份，
#   task-server 派第二台的尝试经它起；task-server/ 与钩子不在射程里（服务是唯一起它的一方，钩子里的是判法的样本输入）。规范副本 .claude/singlefs-ai-sop/ 不扫。
# 只看代码：逐行读法同 cp-preserve-mode（共用库 SOURCE_CODE_LINES_LIBRARY：shell 去掉注释、出路文字与引号里带空白的一段，python 去掉注释与带空白的字符串字面量，
#   不带空白的字面量照代码判）。一行代码里既有起它的那个词（multi-host-run，连字符或下划线写的，大小写不论：脚本路径、变量名 MULTI_HOST_RUN、context.multi_host_run 都算）
#   又有一个单独的词 --peer（前后不接字母、数字、减号与下划线）就点名。
# 改法：照服务的分片协议（task-server/task_server_lib.py 文件头「分片」）：脚本文件头写一行「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并，
#   计划与产物目录在共享工作根下；各片派哪台由服务定，脚本里不再起 multi-host-run.sh --peer。没有豁免表：射程里的每一处都改。
# 判不了的：命令写在带空白的字符串里（bash -c "… multi-host-run.sh --peer …"）、拆成几行拼的 argv、经变量起而那一行不带这两样的，靠 review。
# 样本：peer-launch-direct-red 的 scripts/experiment-apparatus/driver.sh（字面路径、$MULTI_HOST_RUN 两处）、scripts/experiment-apparatus/driver.py（argv 列表里 context.multi_host_run 与 "--peer"）、
#   .claude/gate.d/fixtures/sample-stage/case/setup.sh（样本根的搭建脚本里一处），共 4 处逐处点名；
#   peer-launch-direct-green：分片协议写法的驱动、注释与出路文字里的 multi-host-run.sh --peer、scripts/hosts/multi-host-run.sh 自己、--peer-memory-cap 这类别的词、
#   样本根下不是 setup.sh 的文件，都不点名；判绿并报扫了 3 份。
# 弄坏开关（CODE_TOOLING_BREAK）：peer-launch-variable-unseen（只认字面的 multi-host-run.sh，不认变量名与 python 属性）：peer-launch-direct-red 里 driver.sh:5、driver.py:4
#   两条 want 与「4 处」找不到；peer-launch-launcher-scanned（multi-host-run.sh 自己也扫）：peer-launch-direct-green 判红；
#   peer-launch-prose-scanned（带空白的字面量与注释也照代码扫）：peer-launch-direct-green 判红。
cell_peer_launch_direct() {
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken = sys.argv[3]

SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/scripts")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
LAUNCHER = "scripts/hosts/multi-host-run.sh"
FIXTURE_SETUP = re.compile(r"\.claude/gate\.d/fixtures/[^/]+/[^/]+/setup\.sh")
FIXTURE_PREFIX = ".claude/gate.d/fixtures/"
SHEBANG = re.compile(r"#!\S*(?:\s+\S+)?\s*.*\b(?P<interpreter>bash|sh|python3?)\b")
WITHOUT_WHITESPACE = re.compile(r"\S*")
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
prose_scanned = broken == "peer-launch-prose-scanned"
LAUNCHER_WORD = re.compile(r"multi-host-run\.sh" if broken == "peer-launch-variable-unseen" else r"(?i)multi[-_]host[-_]run")
PEER_WORD = re.compile(r"(?<![\w-])--peer(?![\w-])")


def without_prose_strings(line):
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def language_of(path, name):
    suffix = os.path.splitext(name)[1]
    if suffix in (".sh", ".py"):
        return "shell" if suffix == ".sh" else "python"
    if suffix:
        return None
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            first = handle.readline()
    except OSError:
        return None
    found = SHEBANG.match(first)
    if not found:
        return None
    return "python" if found.group("interpreter").startswith("python") else "shell"


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES)
            for name in sorted(files):
                path = os.path.join(current, name)
                if path == LAUNCHER and broken != "peer-launch-launcher-scanned":
                    continue
                if path.startswith(FIXTURE_PREFIX) and not FIXTURE_SETUP.fullmatch(path):
                    continue
                language = language_of(path, name)
                if language:
                    yield directory, path, language


def code_lines_of(language, text):
    if prose_scanned:
        return text.split("\n")
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, False)
    text = TRIPLE_QUOTED.sub(lambda found: found.group(0) if not any(character.isspace() for character in found.group(2))
                             else re.sub(r"[^\n]", " ", found.group(0)), text)
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


problems, hits = [], []
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "--peer" not in text:
        continue
    original = text.split("\n")
    for number, code in enumerate(code_lines_of(language, text), 1):
        if LAUNCHER_WORD.search(code) and PEER_WORD.search(code):
            hits.append((path, number, original[number - 1] if number <= len(original) else code))

if scanned == 0:
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有要扫的脚本")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if hits:
    failed = True
    print(f"  ✗ {len(hits)} 处项目脚本直接起 multi-host-run.sh --peer 往第二台起活（绕过服务的排队、额度与挑机器）：")  # gate-lint:summary
    for path, number, line in hits:
        print(f"     {path}:{number}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：照 task-server/task_server_lib.py 文件头「分片」改：文件头写「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并，")
    print("               计划与产物目录在共享工作根下，各片派哪台由服务定；脚本里不再起它。这一格没有豁免表，射程里的每一处都改。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 项目脚本里没有直接起 multi-host-run.sh --peer 的：扫了 {scanned} 份（{by_directory}）")
PY
}

# ── 格 retired-names：退役的名字（判定库服务化与多机改动退掉的变量、参数、子命令、目录名、错误种类、特性、脚本名）还留在代码、注释、出路文字与登记表里
#   （records/2026-11-01-判定库服务化.md 批次 22：选库变量、本地库目录、导入与拷库的说法在服务化之后还活在 27 份文件里，没有一格会红）
# 名单：被判的根里 .claude/gate.d/retired-names.tsv（名字<TAB>种类<TAB>出路，写法在它的文件头）；名单读不出、一行写不对判红。名字照字面认：
#   前一个字符不是字母、数字、_ 与 -，后一个字符不是字母、数字、_ 与 -（以 - 起头的参数名里那一段不另算）。
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/、.claude/scripts/、crates/、prune-server/ 里的每一份文本文件（前 8 KiB 里有 NUL 字节的当二进制、不扫）；
#   跳过 target/、__pycache__/、.git/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/（样本是造出来的输入），与名单自己、
#   搬迁登记表 .claude/gate.d/path-moves.tsv（它的旧路径一栏记的就是搬走之前的名字，搬迁换算要靠旧路径认；写法在 .claude/rules/path-moves.md）。
#   整行原文都判：代码、注释、出路文字、文档串、登记表的说明列（说明里还写着退役的名字，就是知识没跟上代码）。
# 没有放行：「设了就拒」的拒绝分支也不留（不兼容旧东西，一个门禁），名单里的名字在别处出现就改掉；也没有放行表。
# 判不了的：拼出来的名字（"SINGLEFS_" "VERDICT_…" 分开写）、编码过的、在扫的目录之外（规则与 kb 归 doc 那几道）；名字退役之后又被别的意思拿去用的，改名单那一行，靠 review。
# 比过（gate-similar）：doc-text.sh 的旧术语那一格判 kb 与文档里登记过改名的术语旧名，射程是文档、名单是 .claude/kb/term-renames.md，不读代码；
#   break-switch-outside-selftest 格判弄坏开关平常跑时被拒，不判退役的名字还在不在；shared-data-root-literals 格判数据根的取法。三格的对象与名单都不同，并不进去。
# 样本：retired-names-red（名单三个名字；scripts/hosts/uses_old.sh 一行代码读退役变量、一行注释写它，scripts/gate/uses_old.py 一个退役参数，.claude/hooks/sample_hook.sh
#   读第三个退役变量，共 4 处逐处点名）；
#   retired-names-green（名字前后接了字母或 - 的不算、样本目录 .claude/gate.d/fixtures/ 下的不扫、名单与搬迁登记表不扫；判绿并报扫了 5 份）。
# 弄坏开关（CODE_TOOLING_BREAK）：retired-names-comments-unscanned（注释行不扫）：retired-names-red 里 uses_old.sh:3 那条 want 与「4 处」找不到；
#   retired-moves-table-scanned（连搬迁登记表也扫）：retired-names-green 判红（期望退出 0，实测 1：path-moves.tsv:2 上的退役名字被点名）。
cell_retired_names() {
python3 - "${CODE_TOOLING_BREAK:-}" <<'PY'
import collections, os, re, sys
broken = sys.argv[1]

NAMES_TABLE = ".claude/gate.d/retired-names.tsv"
MOVES_TABLE = ".claude/gate.d/path-moves.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", ".claude/scripts", "crates", "prune-server")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
OWN_TABLES = {NAMES_TABLE} if broken == "retired-moves-table-scanned" else {NAMES_TABLE, MOVES_TABLE}
BINARY_PROBE_BYTES = 8192
COMMENT_LINE = re.compile(r"^\s*(?:#|//)")
comments_unscanned = broken == "retired-names-comments-unscanned"


def read_names():
    names, problems = [], []
    if not os.path.isfile(NAMES_TABLE):
        return None, [f"名单 {NAMES_TABLE} 不在"]
    for number, line in enumerate(open(NAMES_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 3 or not all(field.strip() for field in fields) or fields[0] != fields[0].strip():
            problems.append(f"{NAMES_TABLE} 第 {number} 行不是「名字<TAB>种类<TAB>出路」三列都不空：{line[:120]}")
            continue
        names.append((fields[0], fields[2]))
    return names, problems


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                path = os.path.join(current, name)
                if path not in OWN_TABLES:
                    yield directory, path


names, problems = read_names()
if names is None:
    print(f"  ✗ {problems[0]}：退役的名字没处读")
    print("     → 怎么办：它随仓走（.claude/gate.d/retired-names.tsv），被删了就从 git 找回来；写法在它的文件头。")
    sys.exit(1)
way_out = dict(names)
NAME_PATTERN = re.compile("|".join(r"(?<![A-Za-z0-9_-])" + re.escape(name) + r"(?![A-Za-z0-9_-])"
                                   for name in sorted(way_out, key=len, reverse=True))) if way_out else None

hits = collections.OrderedDict()   # 路径 → [(行号, 名字们, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path in scanned_files():
    try:
        with open(path, "rb") as handle:
            data = handle.read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    if b"\0" in data[:BINARY_PROBE_BYTES]:
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if NAME_PATTERN is None:
        continue
    text = data.decode("utf-8", "replace")
    if not NAME_PATTERN.search(text):
        continue
    lines = text.split("\n")
    for number, line in enumerate(lines, 1):
        if comments_unscanned and COMMENT_LINE.match(line):
            continue
        found = sorted(set(NAME_PATTERN.findall(line)))
        if found:
            hits.setdefault(path, []).append((number, found, line))

unallowed = [(path, number, found, line) for path, entries in hits.items() for number, found, line in entries]

if scanned == 0:
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有要扫的文件")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：名单照 .claude/gate.d/retired-names.tsv 文件头的写法改；读不了的文件看它的权限，这一格要读到射程里的每一份。")
if unallowed:
    failed = True
    print(f"  ✗ {len(unallowed)} 处还写着退役的名字（名单 {NAMES_TABLE}）：")  # gate-lint:summary
    for path, number, found, line in unallowed:
        print(f"     {path}:{number}  {'、'.join(found)}  「{line.strip()[:160]}」")  # gate-lint:detail
    for name in sorted({name for _path, _number, found, _line in unallowed for name in found}):
        print(f"     {name}：{way_out[name]}")  # gate-lint:detail
    print("     → 怎么办：照上面每个名字的出路改掉那一行（代码删掉、注释与出路文字照现行写法重写），不留兼容、不单留拒绝、没有放行表。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 退役的名字都删干净了：扫了 {scanned} 份（{by_directory}），名单 {len(way_out)} 个名字")
PY
}

# ── 格 verdict-library-code：登记表 verdict-library-scripts.tsv 与它点名的脚本对账（.claude/rules/verification.md「判定库只有一份」；
#   欠账 C948（判定另存别处的库）的会红那一半：取不到库就照跑不记的分支、第三列写 prune-server 的脚本里还有本地库读写、第三列还不是 prune-server）
# 读什么：被判的根里 .claude/gate.d/verdict-library-scripts.tsv（列的写法在它文件头）与 .claude/gate.d/verdict-library-code-forms.tsv（写法表：种类、名字、正则）。
#   登记表不在、一行写法也没有：退 77；写法表不在、一行写不对、正则编不过：判红。
# 判什么，三样：
#   ① 第三列：第三列写了一份库（不以「不算：」「无判定库：」起头）而不以「prune-server」起头的行判红（接完一族才翻成 prune-server，翻之前这一格照红）。
#   ② 取不到库照跑：第三列写了一份库的每一行点名的那一份，逐行只取代码照写法表里种类 silent-run 的正则搜，命中就点名；
#      出路文字照判（红句里说出「取不到库就照跑、不记」，那个分支就还在）。
#   ③ 本地库读写：第三列以 prune-server 起头的行点名的那一份，另照种类 local-library 的正则搜（git 目录下的库目录、按 git common-dir 拼库目录、
#      python 里自己开 RocksDB 或 sqlite），命中就点名。
#   ②③ 只判代码，放过两样（主 agent 2026-11-02 定）：
#      纯说明不判：shell 的整行注释与行尾注释（不在引号里、前面是空白的 # 之后那一段），python 的注释与文档串（模块、类、函数开头那一句字符串）；
#      只在自证里认的弄坏开关护着的分支不判：开关按名字认，取那一份脚本文件头开关表里登记的值（读法同 break-switch-headers 格，BREAK_SWITCH_TABLE_LIBRARY），
#      写在命中那一行的代码里，或写在包着那一行的块的开头行里（往上逐层取缩进更浅的那一行：if / elif / while 的条件、case 的分支），就放过并计数；
#      else 那一行不写开关名，else 那一支照判；文件头开关表里没登记的名字照判。开关平常跑时被拒由 break-switch-outside-selftest 格判，这一格不重判。
#   同一份脚本登记了几行的，① 逐行判、②③ 按那份脚本判一次（几行里有一行是 prune-server 就照 ③ 判）。点名的路径不在判红。
#   库根的三种选法（选库变量、库根参数、打库根的子命令）由 retired-names 格判（名单里登记了），这一格不重判。
# 判不了的：拼出来的写法、写法表外的说法、经别的库间接开的本地库（例：调一份没登记的共用库去写文件）、不按缩进写的块（一行写完的 if、续行）；
#   自证函数里的本地库读写照判（只放过开关护着的分支与说明）；靠 review。
# 比过（gate-similar）：reuse-selftests 格读同一张登记表，但它跑每份的自证核复用三问（判法在 scripts/gate/lib_reuse_checks.py），不读脚本正文；
#   这一格只读正文与第三列、不起任何自证，并进 reuse-selftests 会让一格静态判据跟着自证一起跑、一起本次未跑。retired-names 格判退役名字，不读登记表。
# 样本：verdict-library-code-red（登记表四行：两行 prune-server 各点名一份脚本；silent.sh 文件头开关表登记 SAMPLE_BREAK=silent-run-tolerated，
#   两句取不到库照跑的、一句在那个开关护着的 if 的 else 那一支、一句在文件头没登记的名字 unlisted-mode 护着的块里，local.py 一处 common-dir 拼库目录与一处 sqlite；
#   一行第三列还写库根的 git common-dir（点名的 old_root.sh 里有 git 目录下的库目录，不是 prune-server 行、③ 不判它）；共点名照跑 4 处、本地库 2 处、第三列 1 行）；
#   verdict-library-code-green（prune-server 行点名的脚本里只有「本地不建不读不写」「不读不写真服务」「按条复用失灵：全部现判、一条都不记库」与不接库目录的
#   git common-dir；clean.sh 的注释与行尾注释、clean.py 的文档串与注释里写着取不到库照跑的说法，clean.sh 文件头登记的开关 SAMPLE_BREAK=silent-run-tolerated
#   护着的 if 里有两句（一句在条件下一层、一句再深一层）；「不算：」行点名的脚本里有取不到库照跑的那一句，不判；判绿并报查了 2 行登记、2 份脚本、放过 2 处）。
# 弄坏开关（CODE_TOOLING_BREAK）：library-code-third-column-unchecked（不判第三列）：verdict-library-code-red 里第三列那条 want 找不到；
#   library-code-silent-run-unseen（不搜 silent-run）：verdict-library-code-red 里 silent.sh 四条 want 找不到，verdict-library-code-green 成功句里的写法种数对不上；
#   library-code-local-library-unseen（不搜 local-library）：verdict-library-code-red 里 local.py 两条 want 找不到，verdict-library-code-green 成功句里的写法种数对不上；
#   library-code-unregistered-rows-scanned（「不算：」「无判定库：」行点名的也搜）：verdict-library-code-green 判红（clean_mention.sh 被点名）；
#   library-code-explanations-scanned（注释与文档串也照判，改前的样子）：verdict-library-code-green 判红（clean.sh、clean.py 的说明被点名）；
#   library-code-break-guards-scanned（开关护着的分支也照判，改前的样子）：verdict-library-code-green 判红（clean.sh 开关护着的两句被点名）；
#   library-code-guard-names-unchecked（包着的块不按文件头开关表认，任何带连字符的词都当开关名）：verdict-library-code-red 里 unlisted-mode 护着的那条 want 与「4 处」找不到；
#   library-code-else-branches-released（往上找包着的块时缩进相同的行也算，else 那一支跟着 if 放过）：verdict-library-code-red 里 else 那一支的那条 want 与「4 处」找不到。
cell_verdict_library_code() {
python3 - "${CODE_TOOLING_BREAK:-}" "$SOURCE_CODE_LINES_LIBRARY" "$BREAK_SWITCH_TABLE_LIBRARY" <<'PY'
import ast, collections, io, os, re, sys, tokenize, types
broken = sys.argv[1]
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
switch_table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[3], "break-switch-table-library", "exec"), switch_table.__dict__)
explanations_scanned = broken == "library-code-explanations-scanned"
guards_scanned = broken == "library-code-break-guards-scanned"
guard_names_unchecked = broken == "library-code-guard-names-unchecked"
else_branches_released = broken == "library-code-else-branches-released"

REGISTRY = ".claude/gate.d/verdict-library-scripts.tsv"
FORMS = ".claude/gate.d/verdict-library-code-forms.tsv"
NOT_LIBRARY_PREFIXES = ("不算：", "无判定库：")
SERVICE_PREFIX = "prune-server"
KINDS = ("silent-run", "local-library")

if not os.path.isfile(REGISTRY):
    print(f"  ⊘ 本次无对象可判：没有 {REGISTRY}")
    sys.exit(77)
problems = []
library_rows = []                  # (行号, 路径, 第三列)
for number, line in enumerate(open(REGISTRY, encoding="utf-8").read().split("\n"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) < 4:
        continue   # 列数不对归 reuse-selftests 格判
    if broken != "library-code-unregistered-rows-scanned" and fields[2].startswith(NOT_LIBRARY_PREFIXES):
        continue
    library_rows.append((number, fields[0], fields[2]))
forms = collections.defaultdict(list)
if not os.path.isfile(FORMS):
    problems.append(f"写法表 {FORMS} 不在")
else:
    for number, line in enumerate(open(FORMS, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 3 or fields[0] not in KINDS or not fields[1].strip() or not fields[2]:
            problems.append(f"{FORMS} 第 {number} 行不是「种类（{' / '.join(KINDS)}）<TAB>名字<TAB>正则[<TAB># 说明]」：{line[:120]}")
            continue
        try:
            forms[fields[0]].append((fields[1], re.compile(fields[2])))
        except re.error as error:
            problems.append(f"{FORMS} 第 {number} 行的正则编不过：{error}")
if not library_rows and not forms:
    print(f"  ⊘ 本次无对象可判：{REGISTRY} 里没有写了判定库的行，也没有写法表")
    sys.exit(77)

third_column = []
service_paths, all_paths = set(), []
for number, path, location in library_rows:
    if not location.startswith(SERVICE_PREFIX):
        if broken != "library-code-third-column-unchecked" and not location.startswith(NOT_LIBRARY_PREFIXES):
            third_column.append((number, path, location))
    else:
        service_paths.add(path)
    if path not in all_paths:
        all_paths.append(path)
silent_forms = [] if broken == "library-code-silent-run-unseen" else forms["silent-run"]
local_forms = [] if broken == "library-code-local-library-unseen" else forms["local-library"]
hits = []                          # (路径, 行号, 种类, 名字, 原行)
released = 0                       # 命中在弄坏开关护着的分支里、放过的处数


def code_lines_of(path, text):
    """只留代码（行号不变）：shell 去掉整行注释与行尾注释（不在引号里、前面是空白的 # 之后），python 去掉注释与文档串；出路文字照留（红句里的那一句就是分支还在）。"""
    if path.endswith(".py"):
        rows = [list(line) for line in text.split("\n")]
        try:
            tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
            tree = ast.parse(text)
        except (tokenize.TokenError, SyntaxError, IndentationError, ValueError):
            tokens, tree = None, None
        if tokens is not None:
            for token in tokens:
                if token.type == tokenize.COMMENT:
                    (row, start), (_end_row, end) = token.start, token.end
                    rows[row - 1][start:end] = [" "] * (end - start)
            for node in ast.walk(tree):
                body = getattr(node, "body", None)
                if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and body \
                        and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
                    for row in range(body[0].lineno, body[0].end_lineno + 1):
                        rows[row - 1] = []
            return ["".join(characters) for characters in rows]
    kept = []
    for line in text.split("\n"):
        if line.lstrip().startswith("#"):
            kept.append("")
            continue
        mask = source_lines.quoted_mask(line)
        for index, character in enumerate(line):
            if character == "#" and not mask[index] and (index == 0 or line[index - 1] in " \t"):
                line = line[:index]
                break
        kept.append(line)
    return kept


def guard_pattern(text):
    """那一份脚本文件头开关表里登记的弄坏开关的值（只在自证里认的那几条，按名字认），拼成按词边界认的正则；一条都没登记交 None。"""
    values = sorted({entry["value"] for entry in switch_table.switch_entries(text)}, key=len, reverse=True)
    if not values:
        return None
    return re.compile(r"(?<![A-Za-z0-9_-])(?:" + "|".join(re.escape(value) for value in values) + r")(?![A-Za-z0-9_-])")


SWITCH_LIKE = re.compile(r"(?<![A-Za-z0-9_-])[a-z0-9]+(?:-[a-z0-9]+)+(?![A-Za-z0-9_-])")   # 弄坏开关 guard-names-unchecked 下包着的块认的：任何带连字符的词


def indentation(line):
    return len(line.expandtabs()) - len(line.expandtabs().lstrip())


def guarded_by_break_switch(code, index, guard):
    """命中那一行的代码、或包着它的块的开头行（往上逐层取缩进更浅的那一行：if / elif / while 的条件、case 的分支）写着开关名，就是开关护着的分支；
    else 那一行不写开关名，else 那一支不算被护着。"""
    if guard is not None and guard.search(code[index]):
        return True
    block_guard = SWITCH_LIKE if guard_names_unchecked else guard
    if block_guard is None:
        return False
    depth = indentation(code[index])
    for row in range(index - 1, -1, -1):
        if not code[row].strip():
            continue
        level = indentation(code[row])
        if level > depth or (level == depth and not else_branches_released):
            continue
        if block_guard.search(code[row]):
            return True
        depth = level
        if depth == 0 and not else_branches_released:
            return False
    return False

for path in all_paths:
    if not os.path.isfile(path):
        problems.append(f"{REGISTRY} 点名的 {path} 不在")
        continue
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    lines = text.split("\n")
    code = lines if explanations_scanned else code_lines_of(path, text)
    guard = guard_pattern(text)
    checked = [("取不到库照跑", name, pattern) for name, pattern in silent_forms]
    if path in service_paths:
        checked += [("本地库读写", name, pattern) for name, pattern in local_forms]
    for number, line in enumerate(code, 1):
        for kind, name, pattern in checked:
            if pattern.search(line):
                if not guards_scanned and guarded_by_break_switch(code, number - 1, guard):
                    released += 1
                else:
                    hits.append((path, number, kind, name, lines[number - 1]))
                break

failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：写法表照 .claude/gate.d/verdict-library-code-forms.tsv 文件头改，登记表点名的脚本搬了就改登记表那一行的路径（.claude/gate.d/verdict-library-scripts.tsv）。")
if third_column:
    failed = True
    print(f"  ✗ {len(third_column)} 行写了判定库而第三列不是 prune-server（判定库只有 prune-server 那一份）：")  # gate-lint:summary
    for number, path, location in third_column:
        print(f"     {REGISTRY} 第 {number} 行 {path}  「{location[:120]}」")  # gate-lint:detail
    print("     → 怎么办：把那一族接完（.claude/rules/verification.md「判定库只有一份」的三件：入口与门禁只走服务、旧库最后一次导进服务并 --verify、删掉导进去的那几类目录），")
    print("               再把第三列改写成「prune-server（<族> 族，键空间 …）」；接完之前这一格照红，不设过渡、不留放行。")
silent_hits = [entry for entry in hits if entry[2] == "取不到库照跑"]
local_hits = [entry for entry in hits if entry[2] == "本地库读写"]
if silent_hits:
    failed = True
    print(f"  ✗ {len(silent_hits)} 处带判定库的脚本取不到库还照跑、不记：")  # gate-lint:summary
    for path, number, _kind, name, line in silent_hits:
        print(f"     {path}:{number}  {name}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：取不到库名、服务连不上一律退出（连不上退 75，别的退 2 或 1，红句点名为什么），不往下现判；说那个分支的注释与出路文字一并删掉。")
if local_hits:
    failed = True
    print(f"  ✗ {len(local_hits)} 处第三列写 prune-server 的脚本里还有本地库读写：")  # gate-lint:summary
    for path, number, _kind, name, line in local_hits:
        print(f"     {path}:{number}  {name}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：判定只经 prune-server 的客户端读写（scripts/prune-server/prune_server_client.py），库名只当名字、本地不建不拼库目录；")
    print("               并回即删的中转目录按 scripts/gate/admission.py library-files 的写法走，自证的库对临时服务起。")
if failed:
    sys.exit(1)
print(f"  ✓ 登记表与脚本对得上：查了 {len(library_rows)} 行写了判定库的登记、{len(all_paths)} 份脚本（第三列 prune-server 的 {len(service_paths)} 份），"
      f"第三列都是 prune-server，取不到库照跑的写法 {len(silent_forms)} 种、本地库读写的写法 {len(local_forms)} 种在代码里一处都没有"
      f"（注释与文档串不判；弄坏开关护着的分支放过 {released} 处）")
PY
}

# ── 格 function-units-two-implementations-agree：函数单元的两份实现对拍（crates/singlefs-unit-graph 的 unit-graph 与 scripts/gate/lib_function_units.py）
# 判据：被判的根上 unit-graph dump 与 python 那一份的 dump --language rust、--language wgsl 各一趟，按（文件，起行）配对，逐单元比种类、限定名、起止、指纹
#   （dump 的写法以 Rust 那一份为准，数据单元 data 行同样配对）；再对登记入口逐个比闭包成员与键。登记入口：被判的根 .claude/gate.d/stage-inputs.tsv 第二列、
#   scripts/gate/verus-proof-check.py 与 scripts/mutation/mutation-prune.py 的 JUDGING_ENTRIES 里写成 <文件>.rs::<限定名> 的那几个。python 那一份照 find_entry
#   定入口单元与目标，换成 unit-graph closure <crate>::<模块路径>::<名>（--target <种类>:<名>，lib 不写、build 只写种类；lib 与 bin 里按 cfg(test) 编的另给 --cfg test；
#   文档测试入口的路径是所挂条目或模块在 lib 模块树里的路径（lib_function_units.py 的 doctest_entry_paths 头一条），目标写不带名字的 --target doctest，
#   那个路径点到的全部文档测试一起当入口；lib 模块树之外的与挂在 crate 根上的写不出这一趟，列进解不成的），
#   两边同给一行工具链 host: x86_64-unknown-linux-gnu、不给依赖行；python 那一份的键照 unit-graph 的写法算（lib_function_units.py 的 unit_graph_key）。
#   差一处判红、点名单元（逐处至多列 12 行）。python 那一份解不成入口的（不在 crates/ 的工作区里，例 scripts/experiment-apparatus/ 下的实验装置）逐个列在成功那句里。
#   再走产品行为视图那一趟（lib_function_units.py 文件头「判定种类」）：unit-graph dump --judgment-kind product-behavior 与 python 那一份
#   dump_lines(…, judgment_kind=product-behavior, product_dump=<Rust 那一份交的 dump 行>) 照同样逐单元比，登记入口逐个比 closure 另带
#   --judgment-kind product-behavior 与 python 那一份 RustWorkspace(根, PRODUCT_BEHAVIOR, <同一份 dump 行>) 的闭包与键；差一处判红，点名的行带「产品行为视图：」。
#   python 那一份的剥法就是 Rust 交的这一份（不另写），这一趟比的是两份的配对、只给 Verus 看的单元停在哪、闭包的边与键。
# unit-graph 怎么来：这一道所在仓的 crates/singlefs-unit-graph（连同它 include_str! 读的 .claude/heavy-thread-default）拷进
#   这一格自己的临时目录（${TMPDIR:-/tmp} 下 mktemp -d）里一个只有它的工作区，经
#   capped.sh default 起 cargo build --release --offline -p singlefs-unit-graph（编译目录是这一格自己的、跑完连同临时目录删，用户 2026-10-23 定案「自己用完自己删」，
#   不跨轮复用；不在仓的工作区里编，不碰仓的 Cargo.lock）；编出来的 unit-graph 每一趟直接起（内存上限归起这一道的 task-server，这一格不另套包装）。
# 只供样本（被判的根不是这一道所在的仓、也没设 GATE_STAGED_FROM 时才认）：
#   .unit-graph-recorded/rust-side.json  录好的 Rust 那一侧（dump 与每个入口那一趟 closure 的退出码与输出行，键是 unit-graph 在根之后的参数）：有它就不编、不起 unit-graph
#   .function-units-break                一行，当 python 那一份的 FUNCTION_UNITS_BREAK（真仓与 --staged 下这个变量一律置空）
#   重录（真 unit-graph 跑一趟）：先删掉样本根的 .unit-graph-recorded/，再
#     FUNCTION_UNITS_AGREE_RECORD=<样本根>/.unit-graph-recorded/rust-side.json bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree <样本根>
#     （key-red 那一份的录像重录之后把原样那一趟 lib.rs::entry 的 key 行改一位；product-view-red 的录像与 green 的相同）
# 样本：function-units-two-implementations-agree-green（一个小 crate：文件开头 #![…]、行内模块、include_str! 读的数据文件、bin 的 main、一块 verus!（spec 与 proof 函数、
#   带 requires / ensures 与 proof 块的执行函数），六个登记入口，两趟都判绿）；
#   -product-view-red（green 的拷贝，.function-units-break 是 product-view-kept-formal：python 产品行为视图照原样，dump 与闭包都点名只给 Verus 看的单元）；
#   -dump-red（.function-units-break 是 rust-module-heads-dropped：python 那一份不切模块头，dump 与闭包都点名 mod_head）；
#   -closure-red（rust-constants-dropped：dump 相同，闭包里 Rust 那一份多两个 const）；-key-red（录好的 entry 那一趟的键改了一位：闭包相同、键不同）；
#   -data-directory-green（两个登记入口：一个读 crates/ 之下的数据目录、一个 include_str! 读数据文件，Rust 那一侧是真 unit-graph 录下来的；闭包的数据行行首读目录的是 data-directory、读文件的是 data，两份逐字相同；
#   弄坏开关 function-units-data-kind-ignored 下读目录那一个入口的闭包两份不同、这份样本判红）；
#   -doctest-green（一个登记的文档测试入口：crates/fdocsample/src/lib.rs 里 documented 挂两个文档测试，入口登记在样本的 scripts/mutation/mutation-prune.py 的
#   JUDGING_ENTRIES（限定名带空格，stage-inputs.tsv 第二列按空白切、登记不了），Rust 那一侧录的是 closure fdocsample::documented --target doctest 两趟，两趟都判绿；
#   弄坏开关 function-units-doctest-target-named 下录好的输出里没有那一趟、点名那个入口判红）；
#   -cfg-one-side-red（crates/fsample 的 Cargo.toml 是 default = []，Rust 那一侧的录像是特性开着时录的：entry 里挂在 gpu 特性下的 gpu_boost 只在 Rust 闭包里，闭包两份不同）。
# 弄坏开关 CODE_TOOLING_BREAK：function-units-dump-unchecked 不比 dump（-dump-red 里「只有 Rust 那一份切出：mod_head」那条 want 找不到）；
#   function-units-closures-unchecked 不比闭包与键（-closure-red 与 -cfg-one-side-red 判绿）；function-units-keys-unchecked 闭包相同就不比键（-key-red 判绿）；
#   function-units-product-view-unchecked 不跑产品行为视图那一趟（-product-view-red 判绿，-green 成功那句里没有产品行为视图那一段）；
#   function-units-data-kind-ignored 闭包的数据行一律写 data（-data-directory-green 判红：读目录的入口，Rust 一侧行首是 data-directory）；
#   function-units-doctest-target-named 文档测试入口照改前的样子写 --target doctest:<名>（unit-graph 不认；-doctest-green 判红：「crates/fdocsample/src/lib.rs::documented::<doctest line 3>：unit-graph closure」那一行）。
cell_function_units_two_implementations_agree() {
units_library="$STAGE_REPOSITORY/scripts/gate/lib_function_units.py"
[[ -f "$units_library" ]] || { echo "  ✗ 找不到 $units_library"; echo "     → 怎么办：它随仓走（scripts/gate/lib_function_units.py），被删了就从 git 找回来；别在这一格另写一份。"; exit 1; }
python_break=""
rust_side=(binary)
if judged_root_is_sample; then
  [[ -f .function-units-break ]] && python_break="$(tr -d '[:space:]' < .function-units-break)"
  [[ -f .unit-graph-recorded/rust-side.json ]] && rust_side=(recording "$ROOT/.unit-graph-recorded/rust-side.json")
fi
if [[ "${rust_side[0]}" == binary ]]; then
  build_home="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-unit-graph-build.XXXXXX")" || { echo "  ✗ 建不了这一格的临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
  # shellcheck disable=SC2064  # 路径在这里就展开（%q 过）
  trap "rm -rf -- $(printf '%q' "$build_home")" EXIT   # 这一格自己的工作区与编译目录，收尾删
  crate_source="$STAGE_REPOSITORY/crates/singlefs-unit-graph"
  [[ -f "$crate_source/Cargo.toml" ]] || { echo "  ✗ 这一道所在的仓里没有 crates/singlefs-unit-graph"; echo "     → 怎么办：两份实现对拍要编 Rust 那一份；crate 被删了就从 git 找回来，改了名就改这一格的 crate_source。"; exit 1; }
  mkdir -p "$build_home/workspace/crates" "$build_home/workspace/.claude" || { echo "  ✗ 建不了 $build_home"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
  cp -dR --preserve=timestamps,links "$crate_source" "$build_home/workspace/crates/" && cp --preserve=timestamps "$STAGE_REPOSITORY/.claude/heavy-thread-default" "$build_home/workspace/.claude/" \
    || { echo "  ✗ 拷不进 $build_home/workspace"; echo "     → 怎么办：看那个目录的空间与权限，修好再跑。"; exit 1; }
  manifest_text=$'[workspace]\nmembers = ["crates/singlefs-unit-graph"]\nresolver = "2"\n\n[profile.release]\nopt-level = 3\noverflow-checks = true\ndebug-assertions = true\n'
  printf '%s' "$manifest_text" > "$build_home/workspace/Cargo.toml"
  build_log="$build_home/build.log"
  if ! CARGO_TARGET_DIR="$build_home/target" \
      cargo build --release --offline --manifest-path "$build_home/workspace/Cargo.toml" -p singlefs-unit-graph > "$build_log" 2>&1; then
    # build_home 连同里面的日志随 EXIT trap 删掉：判红时把日志拷到 TMPDIR 下留着，出路指那一份
    kept_build_log="$(mktemp "${TMPDIR:-/tmp}/singlefs-unit-graph-build.XXXXXX.log")" && cp -- "$build_log" "$kept_build_log" || kept_build_log="（没留下：拷不出来）"
    echo "  ✗ 编不出 unit-graph（cargo build --release -p singlefs-unit-graph，日志留在 $kept_build_log）：$(tail -n 3 "$build_log" | tr '\n' ' ' | cut -c1-300)"
    echo "     → 怎么办：看那份日志，看完自己删；编译错就修 crates/singlefs-unit-graph。"
    exit 1
  fi
  rust_side=(binary "$build_home/target/release/unit-graph")
fi
FUNCTION_UNITS_BREAK="$python_break" python3 - "$ROOT" "$units_library" "${CODE_TOOLING_BREAK:-}" "${FUNCTION_UNITS_AGREE_RECORD:-}" "${rust_side[@]}" <<'PY'
import ast, collections, importlib.util, json, os, re, subprocess, sys
root, library, broken, record_path, mode = sys.argv[1:6]
HOST_LINE = "host: x86_64-unknown-linux-gnu"
ENTRY = re.compile(r"^(.+\.rs)::(.+)$")
LIST_LIMIT = 12
if mode == "binary":
    binary = sys.argv[6]
    recording = None
else:
    with open(sys.argv[6], encoding="utf-8") as handle:
        recording = json.load(handle)
recorded = {"dump": None, "closures": {}}


def rust(request):
    """unit-graph 的一趟 → (退出码, 输出行)；request 是根之后的参数（录好的输出照它认，不带根）。"""
    name = "\t".join(request)
    if recording is not None:
        entry = recording.get("dump") if request == ["dump"] else recording["closures"].get(name)
        if entry is None:
            return None, [f"录好的输出里没有这一趟：unit-graph {' '.join(request)}"]
        return entry["exit"], entry["lines"]
    completed = subprocess.run([binary, request[0], root, *request[1:]], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    lines = [line for line in completed.stdout.split("\n") if line]
    entry = {"exit": completed.returncode, "lines": lines}
    if request == ["dump"]:
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
    print("     → 怎么办：单跑那一趟看报错")
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


def compare_dumps(rust_lines, python_lines, label):
    """两份 dump 按（文件，起行）配对逐单元比，不同的记一条 problem（label 是这一趟的前缀，原样那一趟是空的）→ (Rust 那一份的单元数, 各类的数)。"""
    rust_groups, python_groups = placed(rust_lines), placed(python_lines)
    rust_count = sum(len(lines) for lines in rust_groups.values())
    by_kind = collections.Counter("WGSL" if line.startswith("wgsl_") else "数据" if line.startswith("data\t") else "Rust"
                                  for lines in rust_groups.values() for line in lines)
    differences = []
    for place in sorted(set(rust_groups) | set(python_groups)):
        rust_only = sorted((collections.Counter(rust_groups.get(place, ())) - collections.Counter(python_groups.get(place, ()))).elements())
        python_only = sorted((collections.Counter(python_groups.get(place, ())) - collections.Counter(rust_groups.get(place, ()))).elements())
        if len(rust_only) == 1 and len(python_only) == 1:
            fields = [name for name, left, right in zip(("种类", "限定名", "起止", "指纹"), rust_only[0].split("\t"), python_only[0].split("\t")) if left != right]
            differences.append(f"{place[0]}:{place[1]} {'、'.join(fields)}不同：Rust「{rust_only[0][:150]}」python「{python_only[0][:150]}」")
            continue
        differences += [f"{place[0]}:{place[1]} 只有 Rust 那一份切出：{line[:200]}" for line in rust_only]
        differences += [f"{place[0]}:{place[1]} 只有 python 那一份切出：{line[:200]}" for line in python_only]
    if differences and broken != "function-units-dump-unchecked":
        problems.append(f"{label}dump 有 {len(differences)} 处单元两份不同（Rust {rust_count} 行、python {sum(len(lines) for lines in python_groups.values())} 行）")
        for line in differences[:LIST_LIMIT]:
            print(f"     {label}{line}")
        if len(differences) > LIST_LIMIT:
            print(f"     {label}…另 {len(differences) - LIST_LIMIT} 处")
    return rust_count, by_kind


rust_count, by_kind = compare_dumps(rust_lines, python_lines, "")

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
def compare_entries(workspace, label, view_request):
    """登记入口逐个比两份的闭包成员与键（label 是这一趟的前缀，view_request 是 unit-graph closure 另带的选项）→ (比了几个, 解不成 Rust 入口的)。"""
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
        if target.kind == "doctest":
            # 文档测试入口照 unit-graph 的写法（crates/singlefs-unit-graph 的 resolve_entry_units）：路径是所挂条目或模块在 lib 模块树里的路径
            # （doctest_entry_paths 的头一条），那个路径点到的全部文档测试一起当入口，python 那一份的 rust_closure_of 照 doctest_entry_group 取同一组；
            # lib 模块树之外的与挂在 crate 根上的（unit-graph 的入口路径在 crate 名之后至少要一段）写不出这一趟，列进解不成的
            library = workspace.lib_target(target.crate)
            owner_paths = units.doctest_entry_paths(library, unit) if library is not None else []
            if not owner_paths or not owner_paths[0]:
                unresolved.append(f"{entry}（文档测试{'不在 lib 目标的模块树里' if not owner_paths else '挂在 crate 根上'}，unit-graph closure 写不出它的入口路径）")
                continue
            segments = list(owner_paths[0])
        elif unit.container is not None:
            container = unit.container
            type_name = sorted(container.self_types)[0] if container.kind == "impl" and container.self_types else container.short
            segments = module_path + list(unit.scope or ()) + [type_name, unit.short]
        else:
            segments = module_path + unit.name.split("::")
        request = ["closure", f"{target.crate.lib_name}::{'::'.join(segments)}"]
        if target.kind == "doctest":
            # unit-graph 的 TargetSelector::parse 只认不带名字的 doctest；弄坏开关 function-units-doctest-target-named：照改前的样子写 doctest:<名>
            request += ["--target", f"doctest:{target.name}" if broken == "function-units-doctest-target-named" else "doctest"]
        elif target.kind != "lib":
            request += ["--target", target.kind if target.kind == "build" else f"{target.kind}:{target.name}"]
        if target.kind in ("lib", "bin") and units.rust_unit_is_test(workspace, unit, target_id):
            request += ["--cfg", "test"]
        request += ["--toolchain-line", HOST_LINE] + list(view_request)
        exit_code, lines = rust(request)
        if exit_code not in (0, 3):
            problems.append(f"{label}{entry}：unit-graph {' '.join(request)} 退 {exit_code}（{' | '.join(lines[-2:])[:200]}）")
            continue
        try:
            closure = units.rust_closure_of(workspace, unit, target_id)
        except units.UsageError as error:
            problems.append(f"{label}{entry}：python 那一份收不了闭包（{error}）")
            continue
        compared += 1
        if broken == "function-units-closures-unchecked":
            continue
        python_members = [member.dump_text() for member in closure.order if not member.data]
        # 数据行的行首照读它的那一处的种类写（site[5]：读文件的 data，读目录的 data-directory，Rust 那一份 output.rs 的 data_placed_line 同样）；弄坏开关：一律写 data（改前的样子）
        python_members += [f"{'data' if broken == 'function-units-data-kind-ignored' else site[5]}\t{site[2]}\t{site[0]}:{site[1]}-{site[1]}\t{site[3]}" for site in closure.data_sites]
        rust_members = [line for line in lines if not line.startswith(("refused\t", "key\t"))]
        rust_left = collections.Counter(rust_members) - collections.Counter(python_members)
        python_left = collections.Counter(python_members) - collections.Counter(rust_members)
        if rust_left or python_left:
            problems.append(f"{label}{entry}：闭包两份不同（Rust {len(rust_members)} 个、python {len(python_members)} 个；只在 Rust 里 {sum(rust_left.values())} 个、"
                            f"只在 python 里 {sum(python_left.values())} 个）")
            for line in sorted(rust_left.elements())[:LIST_LIMIT // 2]:
                print(f"     {label}{entry} 只在 Rust 闭包里：{line[:200]}")
            for line in sorted(python_left.elements())[:LIST_LIMIT // 2]:
                print(f"     {label}{entry} 只在 python 闭包里：{line[:200]}")
            continue
        if broken == "function-units-keys-unchecked":
            continue
        rust_key = next((line.split("\t", 1)[1] for line in lines if line.startswith("key\t")), None)
        python_key, python_refusals = units.unit_graph_key(closure, [HOST_LINE], [])
        if rust_key != python_key:
            problems.append(f"{label}{entry}：闭包相同、键不同（Rust {rust_key or '拒算'}，python {python_key or '拒算：' + '；'.join(refusal.text() for refusal in python_refusals[:2])}）")
    return compared, unresolved


compared, unresolved = compare_entries(units.RustWorkspace(root), "", [])
# 产品行为视图那一趟（lib_function_units.py 文件头「判定种类」）：python 那一份的产品行为视图吃的就是 Rust 那一份交的 dump，比的是配对、只给 Verus 看的单元停在哪、闭包的边与键
product_summary = "产品行为视图那一趟没比（弄坏开关 function-units-product-view-unchecked）"
if broken != "function-units-product-view-unchecked":
    product_request = ["dump", "--judgment-kind", "product-behavior"]
    exit_code, rust_product_lines = rust(product_request)
    if exit_code != 0:
        problems.append(f"产品行为视图：unit-graph {' '.join(product_request)} 没跑成（退 {exit_code}）：{' | '.join(rust_product_lines[-3:])[:300]}")
        product_summary = "产品行为视图那一趟的 dump 没跑成"
    else:
        python_product_lines = (units.dump_lines(root, "rust", judgment_kind=units.PRODUCT_BEHAVIOR, product_dump=rust_product_lines)[0]
                                + units.dump_lines(root, "wgsl")[0])
        product_count, _product_kinds = compare_dumps(rust_product_lines, python_product_lines, "产品行为视图：")
        product_workspace = units.RustWorkspace(root, units.PRODUCT_BEHAVIOR, rust_product_lines)
        product_compared, _product_unresolved = compare_entries(product_workspace, "产品行为视图：", ["--judgment-kind", "product-behavior"])
        # 只给 Verus 看的单元产品行为视图压根不切（lib_function_units.py 文件头「判定种类」），两份都不列的个数 = 形式化视图那一趟比的个数 − 这一趟比的个数
        ghost_count = rust_count - product_count
        product_summary = (f"产品行为视图那一趟 dump 比了 {product_count} 个单元（只给 Verus 看的 {ghost_count} 个两份都不列）、"
                           f"登记入口 {product_compared} 个，闭包与键逐个相同")
if record_path and recording is None:
    os.makedirs(os.path.dirname(record_path) or ".", exist_ok=True)
    with open(record_path, "w", encoding="utf-8") as handle:
        json.dump(recorded, handle, ensure_ascii=False, indent=0, sort_keys=True)
    print(f"  · 录下了 Rust 那一侧：{record_path}（dump 一趟、产品行为视图的 dump 与各趟 closure 共 {len(recorded['closures'])} 趟）")
named = "、".join(f"{source} {count} 个" for source, count in sorted(sources.items())) or "一个都没有"
if problems:
    print(f"  · 比了 dump {rust_count} 个单元、登记入口 {compared} 个（{named}）；{product_summary.replace('，闭包与键逐个相同', '')}；"
          f"解不成 Rust 入口的 {len(unresolved)} 个" + (f"：{'；'.join(unresolved)}" if unresolved else ""))
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）有 {len(problems)} 处不一致")  # gate-lint:summary
    print("     → 怎么办：照上面点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份（定的口径变了就两边一起改）；"
          "单跑这一格：bash .claude/gate.d/code-tooling.sh --check function-units-two-implementations-agree")
    sys.exit(1)
print(f"  ✓ 两份实现对拍：dump 比了 {rust_count} 个单元（{'、'.join(f'{kind} {count}' for kind, count in sorted(by_kind.items()))}），逐个相同；"
      f"登记入口 {compared} 个（{named}），闭包与键逐个相同；{product_summary}；解不成 Rust 入口的 {len(unresolved)} 个"
      + (f"：{'；'.join(unresolved)}" if unresolved else ""))
PY
}

# ── 格 reuse-selftests：登记表 .claude/gate.d/verdict-library-scripts.tsv 里每份带判定库的脚本，自证的复用三问都过；声称复用的都登记了
#
# 判法全在 scripts/gate/lib_reuse_checks.py 的 audit（文件头「四」：登记表的写法、射程 # scan:、一律现跑、本次未跑与不算、只在临时服务上跑的包一层起）；
#   登记表里有只在临时 prune-server 上跑的自证时，这一格先取一次 prune-server 二进制（code_tooling_prune_server_binary）交 audit 的 --server-binary；
#   这一格只取并行度（每台默认重活线程数 heavy_thread_default，同 research-script-selftests）、交被判的根。用的是这一道所在仓里那一份 lib_reuse_checks.py
#   （样本根里没有它）；现跑的自证在被判的根里起，样本根里的是样本自己的假自证。
# 样本：reuse-selftests-green（一份假自证打三问都过的规范输出，一份「无判定库：待立」报本次未跑，一份「不算」）判绿；
#   reuse-selftests-red（一份假自证只退 0、不打规范输出，一份注释写了判定库却没登记，一份指到不存在的路径）判红、三样都点名。
# 弄坏开关 CODE_TOOLING_BREAK=reuse-selftest-line-unchecked：折成 REUSE_CHECKS_BREAK=audit-line-unchecked，自证退 0 就算过：
#   reuse-selftests-red 里「输出里没有 REUSE_SELFTEST script=scripts/gate/no-line.sh」那条 want 找不到（另两样照红）。
cell_reuse_selftests() {
  local jobs break_items=""
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），现跑自证的并行度定不了，一份都没判"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  [[ "${CODE_TOOLING_BREAK:-}" == reuse-selftest-line-unchecked ]] && break_items=audit-line-unchecked
  # 登记表里有自证只在临时 prune-server 上跑的脚本（要临时服务的表里登记成 required，audit 包一层起）：一趟取一次二进制交 --server-binary
  local server_option=() server_binary needs_server=0 registered build_scratch=""
  temporary_service_table_load || exit 1
  while IFS= read -r registered; do
    [[ -f "$registered" ]] && temporary_service_is_required "$registered" && needs_server=1
  done < <(awk -F'\t' '!/^#/ && NF >= 2 { print $1 }' .claude/gate.d/verdict-library-scripts.tsv 2>/dev/null)
  if ((needs_server)); then
    build_scratch="$(mktemp -d)" || { echo "  ✗ 建不了编 prune-server 的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格"; exit 1; }
    # 路径在设 trap 这一刻展开：build_scratch 是这个函数的局部变量，EXIT 时已出作用域
    trap "rm -rf -- $(printf '%q' "$build_scratch")" EXIT
    if ! server_binary="$(code_tooling_prune_server_binary "$build_scratch")"; then
      echo "  ✗ 取不到 prune-server 二进制（原因在上一行），登记表里只在临时服务上跑的自证现跑不了，一份都没判"
      echo "     → 怎么办：修好 prune-server/ 的编译再跑这一格；二进制不另指，这一格自己编一份"
      exit 1
    fi
    server_option=(--server-binary "$server_binary")
  fi
  REUSE_CHECKS_BREAK="$break_items" python3 "$STAGE_REPOSITORY/scripts/gate/lib_reuse_checks.py" audit "$ROOT" --jobs "$jobs" ${server_option[@]+"${server_option[@]}"}
}

# ── 格 mutation-list-items：还 03-third-txn.md 待优化第 57 行（变异门禁 --list-items 在主工作区抛 PicklingError、取不出键对不上的行数）。
# 判据：在被判的根上跑这一道所在仓里的 scripts/mutation/crates-mutation-rows.py --list-items <根>（门禁 crates-mutation-replay.sh --list-items exec 的就是它，
#   不编、不跑，只起 git 与 rustc -vV），要：退 0；输出里有「· 键对不上 N 条」那一行（键算出来了）；没有「键对不对得上没算出来」、没有 Traceback 与 PicklingError。
#   键算不出来时那份脚本照旧退 0、只打一句「没算出来」（列项照列），平常跑看不出来，提交前要取的数就取不到；这一格在真仓上跑它，把这一种当红。
#   PicklingError 的来源（算闭包的工作进程把按路径载入的模块里的异常往回传）已在 crates-mutation-rows.py 修掉、由它的自证（弄坏开关 closure-error-masked）证红；
#   这一格补的是「在真仓上跑」那一半。真仓上一趟 78 到 168 秒（2026-10-22 在副本上量了两趟，机器同时在跑别的变异；5260 行）。
# 被判的根里没有 crates/mutations.tsv：退 77。
# 样本：mutation-list-items-red（setup.sh 现写一个小工作区，变异表一行点名只在 gpu 特性下编的测试、cargo 参数没带特性：键算不出来，那份脚本退 0，这一格判红）；
#   mutation-list-items-green（同一个小工作区，变异表一行点名普通测试：报「键对不上 1 条」）。
# 弄坏开关 CODE_TOOLING_BREAK=list-items-exit-only：只看退出码，mutation-list-items-red 判绿（期望退出 1，实测 0）。
cell_mutation_list_items() {
  [[ -f crates/mutations.tsv ]] || { echo "  ! 被判的根里没有 crates/mutations.tsv，这一格无对象可判"; exit 77; }
  local listing="$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py" output list_exit=0 summary
  [[ -f "$listing" ]] || { echo "  ✗ 找不到 $listing"; echo "     → 怎么办：它随仓走（scripts/mutation/crates-mutation-rows.py），被删了就从 git 找回来。"; exit 1; }
  output="$(python3 "$listing" --list-items "$ROOT" 2>&1)" || list_exit=$?
  summary="$(grep -m1 -E '^  · 键对不上 [0-9]+ 条' <<<"$output")"
  if [[ "${CODE_TOOLING_BREAK:-}" == list-items-exit-only ]]; then
    summary="${summary:-（弄坏开关 list-items-exit-only：不看输出）}"
  elif [[ "$list_exit" -eq 0 && ( -z "$summary" || "$output" == *"键对不对得上没算出来"* || "$output" == *Traceback* || "$output" == *PicklingError* ) ]]; then
    list_exit=1
  fi
  if [[ "$list_exit" -ne 0 ]]; then
    tail -n 12 <<<"$output" | sed 's/^/    /'   # gate-lint:detail
    echo "  ✗ crates-mutation-rows.py --list-items 在 $ROOT 上没跑通（退 ${list_exit}；键对不上的条数没报出来、或有「没算出来」、Traceback、PicklingError，末尾 12 行在上面）"
    echo "     → 怎么办：照上面那一句的出路改（多半是变异表某一行点名的测试在 cargo 参数选的特性下不编，给那一行补特性），改完单跑"
    echo "               python3 scripts/mutation/crates-mutation-rows.py --list-items . 到它报出「键对不上 N 条」；回溯是脚本自己的错，照回溯修 crates-mutation-rows.py。"
    exit 1
  fi
  echo "  ✓ crates-mutation-rows.py --list-items 在被判的根上跑通：列了 $(grep -c $'\t第 [0-9]* 行，改 ' <<<"$output") 行，${summary#  · }"
}

# ── 格 break-switches-sample-only：还 checks-owed.md C622（门禁的弄坏开关平常跑也认）门禁那一半。
# 判据：被判的根里 .claude/gate.d/ 顶层每道阶段（*.sh），代码里（注释之外）读了名字以 _BREAK 结尾的变量的，要经共用库那一判：
#   调了 stage_cells_run（它开头调 stage_cells_refuse_unsampled_break_switches），或自己调了 stage_cells_refuse_unsampled_break_switches（不经共用库跑格的那一道）；
#   共用库 .claude/gate.d/lib/stage-cells.sh 自己要定义这个函数、stage_cells_run 里调它。判法与「样本根」的认法只在共用库一处（它文件头「弄坏开关只在样本根上认」）。
# 射程只到门禁阶段：scripts/ 与 .claude/hooks/ 里读 *_BREAK 的研究脚本与钩子「只在自证起的子进程里认」那一半是 break-switch-outside-selftest 格（形态各家不同，没有共用的一判可认，那一格逐份起探针）。
# 一道读 *_BREAK 的阶段都没有：退 77。
# 样本：break-switches-sample-only-red（一道阶段读 SAMPLE_STAGE_BREAK、不调 stage_cells_run 也不调那一判；共用库的 stage_cells_run 没调那一判，两处点名判红）；
#   break-switches-sample-only-green（一道经 stage_cells_run、一道自己调那一判、一道不读开关，报「读开关的 2 道都经共用库那一判」）。
# 弄坏开关 CODE_TOOLING_BREAK=break-switch-guard-unchecked：不判，break-switches-sample-only-red 判绿（期望退出 1，实测 0）。
cell_break_switches_sample_only() {
  [[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，这一格无对象可判"; exit 77; }
  python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
broken = sys.argv[2]
GUARD, RUNNER, LIBRARY = "stage_cells_refuse_unsampled_break_switches", "stage_cells_run", ".claude/gate.d/lib/stage-cells.sh"
SWITCH = re.compile(r"\b[A-Z][A-Z0-9_]*_BREAK\b")
problems, readers, routes = [], [], {}
for path in sorted(glob.glob(".claude/gate.d/*.sh")):
    code = code_text.code_of(open(path, encoding="utf-8", errors="replace").read())
    if not SWITCH.search(code):
        continue
    readers.append(path)
    if re.search(r"\b" + RUNNER + r"\b", code):
        routes[path] = RUNNER
    elif re.search(r"\b" + GUARD + r"\b", code):
        routes[path] = GUARD
    elif broken != "break-switch-guard-unchecked":
        problems.append(f"{path} 读了 {SWITCH.search(code).group(0)} 这类弄坏开关，不经 {RUNNER} 跑格、也没调 {GUARD}：环境里漏一个开关，平常跑也照弄坏的判法判")
if readers and broken != "break-switch-guard-unchecked":
    library = code_text.code_of(open(LIBRARY, encoding="utf-8", errors="replace").read()) if os.path.isfile(LIBRARY) else ""
    runner_body = re.search(RUNNER + r"\(\)\s*\{(.*?)\n\}", library, re.S)
    if not re.search(GUARD + r"\(\)\s*\{", library):
        problems.append(f"{LIBRARY} 里没有定义 {GUARD}（共用的那一判）")
    if not runner_body or not re.search(r"\b" + GUARD + r"\b", runner_body.group(1)):
        problems.append(f"{LIBRARY} 的 {RUNNER} 里没调 {GUARD}：经共用库跑格的阶段平常跑也认开关")
if not readers:
    print(f"  ! .claude/gate.d/ 顶层一道读 *_BREAK 的阶段都没有，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 弄坏开关只在样本根上认：{len(problems)} 处没经共用库那一判（读开关的阶段 {len(readers)} 道）")  # gate-lint:summary
    print(f"     → 怎么办：阶段 source {LIBRARY}，格经 {RUNNER} 跑；不经共用库跑格的，source 它取 {GUARD}、cd 到被判的根之后调它"
          "（写法在共用库文件头「弄坏开关只在样本根上认」，样子照 crash-point-replay.sh）")
    sys.exit(1)
by_runner = sum(1 for route in routes.values() if route == RUNNER)
print(f"  ✓ 读开关的 {len(readers)} 道都经共用库那一判（经 {RUNNER} {by_runner} 道、自己调 {GUARD} {len(readers) - by_runner} 道）")
PY
}

# ── 格 entry-scripts：还 checks-owed.md C641（两份入口脚本没按入口脚本格式核）的那一格。
# 登记表 .claude/gate.d/entry-scripts.tsv（被判的根里那一份）一行一份入口脚本（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）：
#   <相对仓根的路径><TAB><每项一行或末行总计的前缀><TAB><那一行要带的字段，逗号连><TAB># 理由
#   前缀那一列写 pending 的是认了是入口脚本、还没改成这个格式的（字段那一列写 -）：不判，逐份报本次未判（往 $GATE_NOT_RUN_FILE 报一行），成功句里点名。
# 判据（逐份读源码，注释行与 python 的文档串不算；不起脚本）：文件在；代码里有 --list-items 与 --selftest 两个参数；代码里有那个前缀，且它那一句（前缀所在行的前 3 行到后 3 行）
#   带齐字段那一列的每个字段。项名写错退 2、列项不起重活、三条完整性闸会不会红，由各份自己的 --selftest 证（research-script-selftests 格跑），这一格只判带没带。
# 表不在、一行都没有：退 77。表里写坏的行（不是四段、理由不到 4 个字）判红。
# 样本：entry-scripts-red（一份没有 --list-items、一份每项那一句少 red=、一行指到不在的文件、一行写坏，逐条点名判红）；
#   entry-scripts-green（两份带齐、一份 pending，报「核了 2 份……pending 1 份本次未判」）。
# 弄坏开关 CODE_TOOLING_BREAK=entry-item-fields-unchecked：不核那一句的字段，entry-scripts-red 里「少了字段 red=」那条 want 找不到。
cell_entry_scripts() {
  local registry=".claude/gate.d/entry-scripts.tsv"
  [[ -f "$registry" ]] || { echo "  ! 没有 $registry，这一格无对象可判"; exit 77; }
  python3 - "$registry" "${CODE_TOOLING_BREAK:-}" "${GATE_NOT_RUN_FILE:-}" <<'PY'
import ast, os, re, sys
registry, broken, not_run_file = sys.argv[1:4]
problems, judged, pending = [], [], []
for number, line in enumerate(open(registry, encoding="utf-8"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    columns = line.rstrip("\n").split("\t")
    reason = columns[3].lstrip("#").strip() if len(columns) == 4 else ""
    if len(columns) != 4 or not columns[3].startswith("#") or len(reason) < 4:
        problems.append(f"{registry}:{number} 不是「<路径><TAB><前缀><TAB><字段，逗号连><TAB># 理由>」（理由至少 4 个字）")
        continue
    path, prefix, fields = columns[0], columns[1], [field for field in columns[2].split(",") if field and field != "-"]
    if not os.path.isfile(path):
        problems.append(f"{registry}:{number} 指到的 {path} 不在")
        continue
    if prefix == "pending":
        pending.append((path, reason))
        continue
    rows = open(path, encoding="utf-8", errors="replace").read().split("\n")
    documentation = set()
    if path.endswith(".py"):
        # python 的文档串（模块、函数、类开头那一段字符串）照注释办：文件头的「输出」一段写着字段，代码里没打也会被认成打了
        try:
            tree = ast.parse("\n".join(rows))
        except SyntaxError as error:
            problems.append(f"{path}：python 语法不过（{error}），判不了")
            continue
        for node in ast.walk(tree):
            body = getattr(node, "body", None)
            if isinstance(body, list) and body and isinstance(body[0], ast.Expr) and isinstance(getattr(body[0], "value", None), ast.Constant) and isinstance(body[0].value.value, str):
                documentation.update(range(body[0].lineno - 1, body[0].end_lineno))
    code = [row for index, row in enumerate(rows) if not row.lstrip().startswith("#") and index not in documentation]
    text = "\n".join(code)
    missing = [argument for argument in ("--list-items", "--selftest") if argument not in text]
    if missing:
        problems.append(f"{path}：代码里没有 {'、'.join(missing)}（入口脚本要能列项、要有自证）")
    # 打那一行的句子：带输出的词（echo、printf、print、say），不是 grep 它的那一种（自证里核输出的那几句也带前缀）
    starts = [index for index, row in enumerate(code) if prefix in row and re.search(r"\b(echo|printf|print|say)\b", row) and "grep" not in row]
    if not starts:
        problems.append(f"{path}：代码里没有打「{prefix}」那一句（echo、printf、print、say）")
    elif fields and broken != "entry-item-fields-unchecked":
        best = min(([field for field in fields if field not in "\n".join(code[max(start - 3, 0):start + 4])] for start in starts), key=len)
        if best:
            problems.append(f"{path}：「{prefix}」那一句少了字段 {'、'.join(best)}（判了、红了、复用了多少与秒，取不出的写取不出、不写 0）")
    judged.append(path)
for path, reason in pending:
    print(f"  ! {path} 登记成入口脚本、还没改成入口脚本格式，本次未判：{reason}")
    if not_run_file and os.path.exists(not_run_file):
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"code-tooling.sh entry-scripts：{path} 还没改成入口脚本格式，本次未判\n")
if not judged and not pending and not problems:
    print(f"  ! {registry} 一行都没有，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 入口脚本格式：{len(problems)} 处不对（登记 {len(judged) + len(pending)} 份）")  # gate-lint:summary
    print("     → 怎么办：照上面逐条补：--list-items 逐行列可点名的项、不编不跑；项名写错退 2；--selftest 证三条完整性闸各会红；"
          "每项一行带项名、退出码、判了、红了、复用了多少与秒，末行总计（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）；"
          f"还没改成的登记成 pending、写明还缺什么；表 {registry} 的写法在这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 入口脚本格式：核了 {len(judged)} 份（{'、'.join(judged)}），都有 --list-items、--selftest 与带齐字段的那一句；pending {len(pending)} 份本次未判"
      + (f"（{'、'.join(path for path, _ in pending)}）" if pending else ""))
PY
}

# ── 格 break-switch-headers：还 checks-owed.md C625（弄坏开关文件头三样没有门禁）的格式那一半。
# 判据：被判的根里 scripts/ 与 .claude/hooks/ 下（fixtures 目录除外）的 .sh、.py，代码里读了名字以 _BREAK 结尾的变量的
#   （shell 的 $X_BREAK / ${X_BREAK…}，python 的 environ.get("X_BREAK") / environ["X_BREAK"] / getenv("X_BREAK") 与 `= "X_BREAK"` 的常量，注释行不算），
#   文件头里要有至少一条 mutate.sh 写法的开关（读法在 BREAK_SWITCH_TABLE_LIBRARY），每一条三样写全：名字与值、弄坏哪一处、下一行起至少一行点名的项。
#   立账时还没改成这种写法的存量逐份登记在 .claude/gate.d/break-switch-headers-exempt.tsv（「<相对仓根的路径><TAB># 理由>」），登记了的不判；
#   一行指到的文件不在、或那份已经写全了（或不再读开关），判红：那一行删掉。表只缩不涨（不判涨，靠 review）。
# 管不到的：逐个打开开关、看自证红在点名的那一项（复跑方）——研究脚本与钩子那一半是 break-switch-replay 格（只复跑登记表里的），门禁阶段那一半是 stage-break-switch-replay 格。
# 一份读开关的都没有：退 77。
# 样本：break-switch-headers-red（一份读开关却没有表、一份一条没写弄坏哪一处、一份一条没点名的项、豁免表一行指到写全了的、一行指到不在的，逐条点名判红）；
#   break-switch-headers-green（一份写全、一份登记在豁免表、一份只在注释里提开关不算读，报「读开关的 2 份：写全 1 份、豁免 1 份」）。
# 弄坏开关 CODE_TOOLING_BREAK=switch-items-unchecked：不判点名的项，break-switch-headers-red 里「点名的项」那条 want 找不到。
cell_break_switch_headers() {
  local exempt=".claude/gate.d/break-switch-headers-exempt.tsv"
  python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$exempt" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, sys, types
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
exempt_path, broken = sys.argv[2:4]
READ = re.compile(r"\$\{?[A-Z][A-Z0-9_]*_BREAK\b|environ(?:\.get)?\s*[\(\[]\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']|getenv\(\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']"
                  r"|^\s*[A-Z_][A-Z0-9_]*\s*=\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']", re.M)
paths = sorted(path for directory in ("scripts", ".claude/hooks") for extension in ("sh", "py")
               for path in glob.glob(f"{directory}/**/*.{extension}", recursive=True) if "/fixtures/" not in path)
problems, exempt = [], {}
if os.path.isfile(exempt_path):
    for number, line in enumerate(open(exempt_path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        path, _, reason = line.rstrip("\n").partition("\t")
        if len(reason.lstrip("#").strip()) < 4:
            problems.append(f"{exempt_path}:{number} 不是「<路径><TAB># 理由>」（理由至少 4 个字）")
            continue
        exempt[path] = number
readers, complete, exempted = [], [], []
for path in paths:
    text = open(path, encoding="utf-8", errors="replace").read()
    code = "\n".join(row for row in text.split("\n") if not row.lstrip().startswith("#"))
    if not READ.search(code):
        continue
    readers.append(path)
    entries = table.switch_entries(text)
    gaps = [(entry, [what for what in table.incomplete(entry) if not (what == "点名的项" and broken == "switch-items-unchecked")]) for entry in entries]
    gaps = [(entry, missing) for entry, missing in gaps if missing]
    written = bool(entries) and not gaps
    if path in exempt:
        exempted.append(path)
        if written:
            problems.append(f"{exempt_path}:{exempt[path]} {path} 的开关表已经三样写全了，豁免那一行删掉")
        continue
    if not entries:
        problems.append(f"{path}：代码里读了弄坏开关，文件头没有一条「X_BREAK=<值>  <弄坏哪一处>」再下一行起写点名的项的开关")
    for entry, missing in gaps:
        problems.append(f"{path}:{entry['line']} {entry['name']}={entry['value']} 缺{'、'.join(missing)}")
    if written:
        complete.append(path)
for path, number in sorted(exempt.items(), key=lambda item: item[1]):
    if not os.path.isfile(path):
        problems.append(f"{exempt_path}:{number} 指到的 {path} 不在，那一行删掉")
    elif path not in readers:
        problems.append(f"{exempt_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
if not readers and not problems:
    print("  ! scripts/ 与 .claude/hooks/ 里没有读弄坏开关的脚本，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 弄坏开关文件头：{len(problems)} 处不对（读开关的 {len(readers)} 份：写全 {len(complete)} 份、豁免 {len(exempted)} 份）")  # gate-lint:summary
    print("     → 怎么办：照 scripts/mutation/mutate.sh 文件头那张表写：一行「X_BREAK=<值>  <弄坏哪一处>」，下一行起缩进更深写打开后哪几项该红（自证的项名，括号里写失败行句首）；"
          f"一时改不完的存量登记进 {exempt_path}（只缩不涨），改完就删那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 弄坏开关文件头：读开关的 {len(readers)} 份：写全 {len(complete)} 份、豁免 {len(exempted)} 份（{exempt_path}，只缩不涨）")
PY
}

# 开关复跑两格（stage-break-switch-replay、break-switch-replay）的进度间隔：300 秒（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟）；
# 样本根（judged_root_is_sample）里那一格的标记文件（一个正整数秒数）顶替它，样本不必等满 5 分钟。写的不是正整数：打拒绝与出路两行（调用方原样转出）、返回 1，一条都不跑。
switch_replay_progress_seconds() {
  local marker="$1" seconds=300
  if judged_root_is_sample && [[ -f "$marker" ]]; then
    seconds="$(<"$marker")"
    if [[ ! "$seconds" =~ ^[1-9][0-9]*$ ]]; then
      echo "  ✗ 样本根里的 $marker 不是一个正整数秒数，一条都没跑"
      echo "     → 怎么办：那份文件只写一个正整数（进度间隔的秒数）。"
      return 1
    fi
  fi
  printf '%s\n' "$seconds"
}

# 开关复跑两格共用的逐条进度（两格文件头「进度」；不各写一份）。装法照 BREAK_SWITCH_TABLE_LIBRARY：
#   progress_library = types.ModuleType("lib_switch_replay_progress"); exec(compile(<这一段>, "switch-replay-progress-library", "exec"), progress_library.__dict__)
# 只打行、只读 /proc 与本进程的 cgroup 记账，不碰判定：判对没有、收回几条照各格自己的判法。
SWITCH_REPLAY_PROGRESS_LIBRARY="$(cat <<'PY_SWITCH_REPLAY_PROGRESS'
import collections, contextlib, itertools, os, threading, time

MEMORY_SAMPLE_SECONDS = 5      # 报进度的线程每过这么多秒（进度间隔更短时照进度间隔）采一次后代进程的内存，峰值只从这些采样里取
LONGEST_RUNNING_SHOWN = 3      # 每次报总进度列出跑得最久的几趟
KIB_PER_MIB = 1024


def mib(kib):
    return "读不到" if kib is None else f"{kib / KIB_PER_MIB:.0f} MiB"


def proportional_kib(pid):
    """→ 这个进程的 Pss（KiB：共享页按共用它的进程数摊开，各进程相加不重算）；smaps_rollup 读不到退回 VmRSS；进程已经退了 → None。"""
    for path, field in ((f"/proc/{pid}/smaps_rollup", "Pss:"), (f"/proc/{pid}/status", "VmRSS:")):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                for row in handle:
                    if row.startswith(field):
                        return int(row.split()[1])
        except (OSError, ValueError, IndexError):
            continue
    return None


def command_words(pid):
    """→ 这个进程命令行的前三个词（说明峰值是谁占的）；读不到写「（读不到）」。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            words = [word.decode("utf-8", "replace") for word in handle.read().split(b"\0") if word]
    except OSError:
        words = []
    return " ".join(words[:3])[:80] or "（读不到）"


def descendants_of(root_pid):
    """→ root_pid 的全部后代进程号（不含它自己），按 /proc/<进程号>/stat 的父进程号连；读的时候已经退了的不算。"""
    children_of = collections.defaultdict(list)
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", "rb") as handle:
                stat = handle.read()
        except OSError:
            continue
        # 命令名在括号里、自己可以带空格与括号：取最后一个「)」之后的字段，状态之后那一个是父进程号
        fields = stat[stat.rfind(b")") + 1:].split()
        if len(fields) >= 2 and fields[1].isdigit():
            children_of[int(fields[1])].append(int(name))
    found, pending = [], list(children_of.get(root_pid, []))
    while pending:
        pid = pending.pop()
        found.append(pid)
        pending.extend(children_of.get(pid, []))
    return found


def cgroup_charge_kib():
    """→ 本进程所在 cgroup 的 memory.current（KiB，含页缓存与 tmpfs）；读不到 → None。"""
    try:
        with open("/proc/self/cgroup", encoding="utf-8") as handle:
            relative = next(row.rstrip("\n")[3:] for row in handle if row.startswith("0::"))
        with open(f"/sys/fs/cgroup{relative}/memory.current", encoding="utf-8") as handle:
            return int(handle.read()) // 1024
    except (OSError, StopIteration, ValueError):
        return None


class ReplayProgress:
    """开关复跑的逐条进度。item_done：每条判完打一行「<格名> 第 k / N 条<对上没有>：<哪一条>，用了 S 秒」（k 按判完的先后数）；
    start：打开跑那一行，另起一条线程，每过 interval_seconds 秒打一行总进度、跑得最久的几趟、extra_lines 给的行与一行内存，
    每过 MEMORY_SAMPLE_SECONDS 秒（进度间隔更短时照它）采一次后代进程的内存记峰值；close：停线程，打收尾一行（逐条进度几行、派出去几条、内存峰值）。
    silent（弄坏开关 replay-progress-silent）：一行都不打、不起线程（改前的样子）。行都以「·」起头、不带 ✗，打完立即冲出去
    （标准输出接到日志时 python 按块缓冲，不冲就要等整格跑完才落进日志，统一任务服务看日志满 10 分钟没涨就当卡住停掉）。"""

    def __init__(self, cell, total, interval_seconds, silent, extra_lines=None):
        self.cell, self.total, self.interval_seconds, self.silent = cell, total, interval_seconds, silent
        self.extra_lines = extra_lines or (lambda: [])
        self.sample_seconds = min(interval_seconds, MEMORY_SAMPLE_SECONDS)
        self.lock = threading.Lock()          # 管在跑的趟、计数与峰值；拿着它可以再拿 print_lock，反过来不许
        self.print_lock = threading.Lock()
        self.running_since = {}               # 在跑的一趟（自增号）→ (组, 说明, 起跑时刻)
        self.tokens = itertools.count()
        self.reported = 0                     # 打出去的逐条进度行数
        self.samples = 0
        self.peak = None                      # 后代进程 Pss 合计最大的那一次采样：(KiB, 进程数, cgroup 记账 KiB, 在跑的按组计数)
        self.peak_charge = None               # 采到过的 cgroup 记账最大值（KiB）
        self.largest = (0, "")                # 采到过的单个进程最大 Pss：(KiB, 命令前三个词)
        self.stop = threading.Event()
        self.thread = None

    def say(self, text):
        if self.silent:
            return
        with self.print_lock:
            print(f"  · {text}", flush=True)

    @contextlib.contextmanager
    def running(self, group, label):
        """包住起一趟子进程的那几行：跑着时算「在跑」，报总进度时列跑得最久的、采内存时按组记。"""
        token = next(self.tokens)
        with self.lock:
            self.running_since[token] = (group, label, time.monotonic())
        try:
            yield
        finally:
            with self.lock:
                del self.running_since[token]

    def item_done(self, verdict, label, seconds):
        if self.silent:
            return
        with self.lock:
            self.reported += 1
            self.say(f"{self.cell} 第 {self.reported} / {self.total} 条{verdict}：{label}，用了 {seconds:.0f} 秒")

    def start(self, headline):
        if self.silent:
            return
        self.say(headline)
        self.thread = threading.Thread(target=self.beat, daemon=True)
        self.thread.start()

    def beat(self):
        ticks_per_report = max(1, self.interval_seconds // self.sample_seconds)
        tick, report = 0, 0
        while not self.stop.wait(self.sample_seconds):
            tick += 1
            try:
                memory = self.sample_memory()
            except OSError as error:   # 读 /proc 出错只丢这一次采样：报进度的线程一停，日志就不再涨
                memory = f"这一次没采成（{error}）"
            if tick % ticks_per_report:
                continue
            report += 1
            with self.lock:
                done, now = self.reported, time.monotonic()
                running = sorted(self.running_since.values(), key=lambda item: item[2])
            self.say(f"{self.cell} 进度（第 {report} 次报，间隔 {self.interval_seconds} 秒）：{self.total} 条里判完 {done} 条、在跑 {len(running)} 趟")
            for _group, label, since in running[:LONGEST_RUNNING_SHOWN]:
                self.say(f"{self.cell} 跑得最久的一趟：{label}，已跑 {now - since:.0f} 秒")
            for line in self.extra_lines():
                self.say(line)
            self.say(f"{self.cell} 内存：{memory}")

    def sample_memory(self):
        """采一次后代进程的内存，更新峰值 → 报进度用的一句。只在报进度的线程里调。"""
        total, measured = 0, 0
        for pid in descendants_of(os.getpid()):
            kib = proportional_kib(pid)
            if kib is None:
                continue
            measured += 1
            total += kib
            if kib > self.largest[0]:
                self.largest = (kib, command_words(pid))
        charge = cgroup_charge_kib()
        with self.lock:
            groups = collections.Counter(group for group, _label, _since in self.running_since.values())
        self.samples += 1
        if self.peak is None or total > self.peak[0]:
            self.peak = (total, measured, charge, groups)
        if charge is not None and (self.peak_charge is None or charge > self.peak_charge):
            self.peak_charge = charge
        return f"此刻后代进程 {measured} 个、Pss 合计 {mib(total)}、cgroup 记账 {mib(charge)}；到此刻 Pss 合计峰值 {mib(self.peak[0])}、cgroup 记账峰值 {mib(self.peak_charge)}"

    def close(self):
        """停报进度的线程，打收尾一行。逐条进度的行数与派出去的条数相等，才是每条判完都报了一行。"""
        self.stop.set()
        if self.thread is None:
            return
        self.thread.join()
        if self.peak is None:
            memory = f"内存没采到（整格跑得比一次采样间隔 {self.sample_seconds} 秒短）"
        else:
            total, measured, charge, groups = self.peak
            in_flight = "、".join(f"{group} {count} 趟" for group, count in groups.most_common()) or "无"
            memory = (f"内存采了 {self.samples} 次，后代进程 Pss 合计峰值 {mib(total)}（那一刻 {measured} 个进程、cgroup 记账 {mib(charge)}；在跑的按组：{in_flight}），"
                      f"cgroup 记账峰值 {mib(self.peak_charge)}，单个进程 Pss 最大 {mib(self.largest[0])}（{self.largest[1]}）")
        self.say(f"{self.cell} 逐条进度 {self.reported} 行、派出去 {self.total} 条；{memory}")
PY_SWITCH_REPLAY_PROGRESS
)"

# ── 格 stage-break-switch-replay：还 checks-owed.md C639（门禁阶段的弄坏开关没有复跑方）。
# 判据：被判的根里 .claude/gate.d/ 顶层每道有样本目录的阶段，文件头开关表（mutate.sh 写法，读法在 BREAK_SWITCH_TABLE_LIBRARY；点名的项写样本目录名）逐条：
#   在环境里设上那个开关，经这一道所在仓里的 .claude/singlefs-ai-sop/scripts/stage-selftest.sh <根>/.claude/gate.d --item <阶段> 喂这一道的全部样本，
#   判错的样本集合（stage-selftest 报「✗ <阶段> <样本> …」的那几份）要与那一条点名的样本目录名集合相等；不等判红，逐条列多出与少了的。
#   开关表一条都读不出、或有一条没写全的阶段判红。被喂的样本根里都有 .gate-cells 或 .gate-break-sample（C622：开关只在样本根上认）。
#   各条互不依赖、各起各的 stage-selftest（样本各拷各的临时目录），按每台默认重活线程数并行，收回的条数与派出去的对不上整格红。
# 不复跑的（逐条报本次未跑，往 $GATE_NOT_RUN_FILE 报一行）：
#   .claude/hooks/selftest-registry.tsv 里这一道的样本档没登记成 samples 的（样本档在登记表里才复跑）；
#   .claude/gate.d/stage-break-switch-replay-exempt.tsv 登记的（「<阶段文件名><TAB><开关=值，或 * 整道><TAB># 理由>」：头注还不是这种写法，
#   或点名的与实测不等、头注还没改准的那几条）。登记的一条在头注里已经没有了（改了名、删了）判红：那一行删掉。表只缩不涨（不判涨，靠 review）。
# 一道有样本目录的阶段都没有：退 77。真仓上一趟约 700 条，每条把那一道的全部样本串着喂一遍；第二台跑完整格的两趟是 695 条 5409 秒、704 条 2392 秒
#   （任务 3630、3922 的日志），整格要四十分钟到一个半小时。
# 进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟；写法在 SWITCH_REPLAY_PROGRESS_LIBRARY）：开跑打一行「stage-break-switch-replay 开跑：」
#   （条数、阶段数、并行度）；每条判完打一行「stage-break-switch-replay 第 k / N 条<对上、没对上（点名的样本与实测不符）或没跑成（stage-selftest.sh 退 c）>：<阶段> <开关>，用了 S 秒」；
#   另起一条线程每过 300 秒打一行总进度（判完几条、在跑几趟）、跑得最久的三趟与一行内存（后代进程数、Pss 合计、cgroup 记账、到此刻的峰值），
#   每 5 秒采一次内存；收尾打一行「stage-break-switch-replay 逐条进度 R 行、派出去 N 条；」加内存峰值、峰值那一刻在跑的按阶段计数与单个最大的进程。
#   样本根里的 .stage-break-switch-replay-progress-seconds（一个正整数秒数）顶替 300 秒；写的不是正整数判红、一条都不跑（switch_replay_progress_seconds）。
# 样本：stage-break-switch-replay-red（一道小阶段：一条开关点名对、一条点名的与实测不等、一条没写点名的项；豁免表一行指到头注里没有的开关；
#   一道没登记的不跑，逐条点名判红）；stage-break-switch-replay-green（点名都对的两条、豁免一条、没登记的一道本次未跑，
#   报「复跑了 2 条开关（1 道阶段）」）。
#   stage-break-switch-replay-progress-green：样本根里 .stage-break-switch-replay-progress-seconds 写 1；一道小阶段两份样本、喂每份先睡 2 秒，两条开关各弄错一份、点名都对，
#   必须判绿，且输出里有开跑一行、「进度（第 1 次报」、跑得最久的一趟、内存一行、「第 1 / 2 条对上」「第 2 / 2 条对上」与「逐条进度 2 行、派出去 2 条」。
# 弄坏开关 CODE_TOOLING_BREAK=switch-replay-named-unchecked：不比点名的与实测的，stage-break-switch-replay-red 里「点名的与实测判错的不等」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-progress-silent：不打进度、不起报进度的线程（改前的样子），stage-break-switch-replay-progress-green 里那几条进度的 want 找不到
#   （与 break-switch-replay 格同一条开关，各红各的进度样本）。
# 临时服务的二进制：每条开关起的 stage-selftest.sh 都套 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 包装，包装不带二进制又没有任务服务给的编译目录就不现编、退 2。
#   任务服务只给最外层命令编译目录（外层是这一道门禁，不算会编的），所以开跑前取一次（code_tooling_prune_server_binary：PRUNE_SERVER_BINARY 设了用它，没设照 prune-server-selftest 格的编法编进这一格自己的临时目录，跑完删），
#   每条都经 --server-binary 交；起每条前摘掉环境里的 PRUNE_SERVER_BINARY 与 CARGO_TARGET_DIR，二进制只认交的这一份，哪台机器上结局相同。取不到整格判红、一条开关都不起，出路写编译命令。二进制路径不进单元键。
#   样本根里没有 prune-server/，样本档靠外层包装设的 PRUNE_SERVER_BINARY（见「样本根只在临时 prune-server 底下判」那一段，裸跑样本档退 2）。
# 弄坏开关 CODE_TOOLING_BREAK=stage-replay-binary-not-passed：不取、不交二进制（改前的样子），套包装起的每条退 2、「没跑成（stage-selftest.sh 退 2）」，文件头开关表点名的七份样本判错。
# 按条复用（.claude/rules/implementation-workflow.md「复用重型门禁的判定：按每条判定的单元键判」；共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：
#   一条判定 = 一道阶段加它的一条开关，名字「<阶段文件名> <开关名>=<值>」，种类 code-tooling.sh:stage-break-switch-replay（prune-server 那一份库，unit-verdicts 族
#   singlefs-unit-verdicts/code-tooling.sh:stage-break-switch-replay，库根不进键）。键（scripts/gate/admission.py verdict-plan 算，不带路径、行号与限时）由这几样组成：
#   ① 那道阶段的单元闭包（入口 .claude/gate.d/<阶段>，shell 的 script-body 连同它调到、source 到的函数单元，去注释）；
#   ② 判法：这一道所在仓里 stage_break_switch_replay_judge_code 的闭包（令牌 STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN：开关表的读法与 STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY）；
#   ③ line:switch=<开关名>=<值>；④ line:named=<点名的样本，排好用逗号连>（文件头是注释，不进单元指纹，另进这一行）；
#   ⑤ line:samples=<它喂的 fixtures/<阶段>/ 整棵的摘要：每个目录、每份文件的相对路径、执行位与内容、每个链接的指向>；
#   ⑥ line:feeder=<喂样本的 stage-selftest.sh、它旁边的 lib.sh 与临时服务包装三份内容的摘要>。
#   键相同、库里记着绿的不起（复用）；键对不上的、库里记着红的（原因以「上次判红」起头）现判；拒算的（闭包收不全）照起、不记库。
#   每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（种子打进日志；被判的根下有 .unit-verdict-sample/ 的样本不抽）；哨兵被现判、强制现判那一条与库里的不同，交 plan-failure 判红。
#   一条判完就记：判绿记绿，判红照同一把键记红、撤掉原有的绿（verdict-record --red-names）；收尾把这一趟判完的整份再交一次（哨兵在那一趟比）。
#   成功句报「按条复用：现判 N、复用 M、拒算 R」；全部复用时报「全部复用，这一格一条都没起」、退 0。算不出（verdict-plan 退非 0、没打 PLAN）照全部现判跑完、判红；
#   prune-server 不可用（退 75）一条都不起、这一格退 75。
#   逐项点名（这一道「逐项点名」一节）：项是 <阶段文件名> 或 <阶段文件名>:<开关名>=<值>，只算、只起点到的那几条，键与判法照整格。
#   样本：stage-break-switch-replay-reuse-three-questions（两道小阶段三条开关；复用三问：change.sh 改 alpha-stage.sh 的 judged_verdict，affected 是它的两条；四趟都绿、三问都过）；
#     stage-break-switch-replay-samples-in-key（同样两道；change.sh 只给 alpha-stage.sh 的一份样本的 expect 多加一句照样对得上的 want：样本内容进键，alpha 的两条现判）；
#     stage-break-switch-replay-red-over-green（.unit-verdict-sample/prior/ 照判绿记两条；red-over-green 下一律现判，样本根的 .red-switch 让 swallow-red 那一条点错，
#     判红记红、撤绿，再算一次计划：复用 1 条、现判 1 条、原因「上次判红」）；stage-break-switch-replay-item-green、stage-break-switch-replay-item-unknown（逐项点名）。
#   弄坏开关（文件头开关表）CODE_TOOLING_BREAK=replay-plan-ignored、replay-red-not-recorded、stage-replay-samples-unkeyed、stage-replay-key-whole-gate-directory、
#     items-ignored、item-unknown-accepted，各红在开关表点名的样本。
cell_stage_break_switch_replay() {
  [[ -d .claude/gate.d/fixtures ]] || { echo "  ! 没有 .claude/gate.d/fixtures，这一格无对象可判"; exit 77; }
  local jobs
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），复跑的并行度定不了，一条都没跑"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  local stage_replay_progress_seconds
  if ! stage_replay_progress_seconds="$(switch_replay_progress_seconds .stage-break-switch-replay-progress-seconds)"; then
    printf '%s\n' "$stage_replay_progress_seconds"
    exit 1
  fi
  # 每条开关起的 stage-selftest.sh 都套临时服务包装；包装不现编（任务服务只给最外层会编的命令编译目录，门禁在外层），开跑前取一次二进制交给每一条（这一格文件头「临时服务的二进制」）
  local stage_replay_binary="" stage_replay_scratch="" stage_replay_exit
  if [[ -f "$STAGE_REPOSITORY/scripts/prune-server/prune_server_client_selftest.py" && "${CODE_TOOLING_BREAK:-}" != stage-replay-binary-not-passed ]]; then
    stage_replay_scratch="$(mktemp -d)" || { echo "  ✗ 建不了放 prune-server 二进制的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条开关都没起。"; exit 1; }
    if ! stage_replay_binary="$(code_tooling_prune_server_binary "$stage_replay_scratch" 2>"$stage_replay_scratch/server-binary.err")"; then
      echo "  ✗ 取不到 prune-server 二进制（$(tail -n 3 "$stage_replay_scratch/server-binary.err" | tr '\n' ' ' | cut -c1-400)），每条开关起的 stage-selftest.sh 要套在临时服务底下，一条开关都没起"
      echo "     → 怎么办：修好 prune-server/ 的编译（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）再跑这一格；或设 PRUNE_SERVER_BINARY=<编好的二进制的绝对路径>。"
      rm -rf -- "$stage_replay_scratch"
      exit 1
    fi
  fi
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds stage-break-switch-replay stage_break_switch_replay_pass; stage_replay_exit=$?
  else
    stage_break_switch_replay_pass ""; stage_replay_exit=$?
  fi
  [[ -z "$stage_replay_scratch" ]] || rm -rf -- "$stage_replay_scratch"
  return "$stage_replay_exit"
}

# stage_break_switch_replay_pass <各条结局写到哪，空是不写>：这一格的一趟（算键、查库、起键对不上的、记库、判）；jobs 与 stage_replay_progress_seconds 取调用它的那一格的
stage_break_switch_replay_pass() {
  python3 - "$(stage_break_switch_replay_judge_code)" "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$STAGE_REPOSITORY/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" \
    "$ROOT" "$jobs" "$STAGE_NAME" "${CODE_TOOLING_BREAK:-}" "${GATE_NOT_RUN_FILE:-}" "$SWITCH_REPLAY_PROGRESS_LIBRARY" "$stage_replay_progress_seconds" "$STAGE_REPOSITORY" \
    "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:stage-break-switch-replay" "$STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN" "$STAGE_REPOSITORY" \
    "${CODE_TOOLING_PICKED[stage-break-switch-replay]:-}" "$(judged_root_is_sample && echo 1)" "$1" "$stage_replay_binary" <<'PY'
import concurrent.futures, glob, hashlib, importlib.util, os, shutil, subprocess, sys, tempfile, time, types
judge_library = types.ModuleType("lib_stage_break_switch_replay_judge")
exec(compile(sys.argv[1], "stage-break-switch-replay-judge-library", "exec"), judge_library.__dict__)
table = judge_library   # 开关表的读法与判法在同一个模块里（stage_break_switch_replay_judge_code 打的两段）
runs_library_path, stage_selftest, root, jobs, own_stage, broken, not_run_file = sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), sys.argv[6], sys.argv[7], sys.argv[8]
progress_library = types.ModuleType("lib_switch_replay_progress")
exec(compile(sys.argv[9], "switch-replay-progress-library", "exec"), progress_library.__dict__)
progress_seconds = int(sys.argv[10])     # 进度间隔（这一格文件头「进度」）
stage_repo = sys.argv[11]
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[12], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_token, judge_root = sys.argv[13], sys.argv[14], sys.argv[15], sys.argv[16]
picked = [item for item in sys.argv[17].split("\n") if item]      # 逐项点名（这一道「逐项点名」一节）：空是整格
sample, states_path = sys.argv[18] == "1", sys.argv[19]
server_binary = sys.argv[20]     # 临时服务包装起的 prune-server 二进制（这一格开跑前取一次）；空是没取（只在弄坏开关 stage-replay-binary-not-passed 下）
picked_stages, picked_switches = set(), {}
for item in picked:
    item_stage, separator, item_switch = item.partition(":")
    if separator:
        picked_switches.setdefault(item_stage, set()).add(item_switch)
    else:
        picked_stages.add(item_stage)


def stage_picked(stage):
    return not picked or stage in picked_stages or stage in picked_switches


def switch_picked(stage, switch):
    return not picked or stage in picked_stages or switch in picked_switches.get(stage, ())


prune_wrapper = os.path.join(stage_repo, "scripts/prune-server/prune_server_client_selftest.py")
spec = importlib.util.spec_from_file_location("lib_project_code_runs", runs_library_path)
runs_library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runs_library)
EXEMPT = ".claude/gate.d/stage-break-switch-replay-exempt.tsv"
registry, _ = runs_library.read_selftest_registry(os.path.join(root, ".claude/hooks/selftest-registry.tsv"))
problems, not_run, runs = [], [], []
exempt = {}
if os.path.isfile(EXEMPT):
    for number, line in enumerate(open(EXEMPT, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.rstrip("\n").split("\t")
        if len(columns) != 3 or len(columns[2].lstrip("#").strip()) < 4:
            problems.append(f"{EXEMPT}:{number} 不是「<阶段文件名><TAB><开关=值，或 *><TAB># 理由>」（理由至少 4 个字）")
            continue
        exempt[(columns[0], columns[1])] = (number, columns[2].lstrip("#").strip(), [False])
stages = sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh") if os.path.isdir(f".claude/gate.d/fixtures/{os.path.basename(path)}"))
for stage in stages:
    whole = exempt.get((stage, "*"))
    if whole:
        whole[2][0] = True
        if stage_picked(stage):
            not_run.append(f"{stage}：整道登记在豁免表（{whole[1]}）")
        continue
    if not stage_picked(stage):
        for (exempt_stage, _switch), mark in exempt.items():
            if exempt_stage == stage:
                mark[2][0] = True   # 没点到的阶段不判它的豁免行（这一道「逐项点名」：点名单跑只判点到的项）
        continue
    row = registry.get(f".claude/gate.d/{stage}")
    if row is None or row.kind != runs_library.STAGE_SAMPLES:
        not_run.append(f"{stage}：样本档{'没登记' if row is None else '登记成 ' + row.kind}（要登记成 samples 或 samples:<假工具目录>），复跑的范围不含它")
        continue
    samples = sorted(name for name in os.listdir(f".claude/gate.d/fixtures/{stage}") if os.path.isdir(f".claude/gate.d/fixtures/{stage}/{name}"))
    entries = table.switch_entries(open(f".claude/gate.d/{stage}", encoding="utf-8", errors="replace").read())
    if not entries:
        problems.append(f"{stage}：文件头读不出一条「X_BREAK=<值>  <弄坏哪一处>」再下一行起点名样本的开关")
        continue
    for entry in entries:
        switch = f"{entry['name']}={entry['value']}"
        mark = exempt.get((stage, switch))
        if mark:
            mark[2][0] = True
        if not switch_picked(stage, switch):
            continue
        if mark:
            not_run.append(f"{stage} {switch}：登记在豁免表（{mark[1]}）")
            continue
        if table.incomplete(entry):
            problems.append(f"{stage}:{entry['line']} {switch} 缺{'、'.join(table.incomplete(entry))}")
            continue
        runs.append((stage, entry, switch, table.named_tokens(entry, samples), samples))
for (stage, switch), (number, _reason, used) in sorted(exempt.items(), key=lambda item: item[1][0]):
    if not used[0]:
        problems.append(f"{EXEMPT}:{number} {stage} {switch}：那一道的头注里已经没有这一条（或那一道没有样本目录），那一行删掉")



# 按条复用（这一格文件头「按条复用」）：一条判定 = 一道阶段加它的一条开关；键 = 那道阶段的单元闭包（入口 .claude/gate.d/<阶段>）、判法（judge_token）、
# 开关名与值、点名的样本、它喂的样本目录的内容（逐份文件的相对路径、执行位与内容）、喂样本的那几份程序的内容；不带路径、行号与限时
def file_digest(path):
    if not os.path.isfile(path):
        return "不在"
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def samples_digest(stage):
    """fixtures/<阶段>/ 整棵的摘要：每个目录、每份文件（相对路径、执行位、内容）与每个链接（指向）按路径排好连起来取 sha256。"""
    digest, base = hashlib.sha256(), f".claude/gate.d/fixtures/{stage}"
    for directory, subdirectories, files in os.walk(base):
        subdirectories.sort()
        digest.update(f"dir\0{os.path.relpath(directory, base)}\0".encode("utf-8", "surrogateescape"))
        linked = [name for name in subdirectories if os.path.islink(os.path.join(directory, name))]
        for name in sorted(files + linked):
            path = os.path.join(directory, name)
            relative = os.path.relpath(path, base)
            if os.path.islink(path):
                digest.update(f"link\0{relative}\0{os.readlink(path)}\0".encode("utf-8", "surrogateescape"))
            else:
                digest.update(f"file\0{relative}\0{'x' if os.access(path, os.X_OK) else '-'}\0{file_digest(path)}\0".encode("utf-8", "surrogateescape"))
    return digest.hexdigest()


feeder_digest = hashlib.sha256("".join(file_digest(path) for path in (stage_selftest, os.path.join(os.path.dirname(stage_selftest), "lib.sh"), prune_wrapper)).encode()).hexdigest()
digest_of = {stage: samples_digest(stage) for stage in sorted({run[0] for run in runs})}


def item_name(stage, switch):
    return f"{stage} {switch}"


def request_line(run):
    stage, _entry, switch, named, _samples = run
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:switch={switch}", "line:named=" + (",".join(sorted(named)) or "（无）"), f"line:feeder={feeder_digest}"]
    if broken != "stage-replay-samples-unkeyed":
        tokens.append(f"line:samples={digest_of[stage]}")
    if broken == "stage-replay-key-whole-gate-directory":
        tokens += [f"file:.claude/gate.d/{other}" for other in stages]
    return "\t".join([item_name(stage, switch), f".claude/gate.d/{stage}", *tokens])


requests = {}
for run in runs:
    requests[item_name(run[0], run[2])] = request_line(run)
plan_scratch = tempfile.mkdtemp(prefix="stage-break-switch-replay-plan.")
verdicts = rounds.ItemVerdicts(admission, os.getcwd(), kind, plan_scratch, "code-tooling.sh 的 stage-break-switch-replay 格", sample,
                               "bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay（逐条设上开关经 stage-selftest.sh 喂那一道的样本）",
                               broken != "replay-red-not-recorded")
if requests:
    verdicts.plan(list(requests.values()))
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print(f"  ✗ 门禁阶段的弄坏开关复跑这一格没判：prune-server 不可用（上面那一句），{len(requests)} 条一条都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：这一趟每一条都起、一条都不记库")
eligible = runs
if broken != "replay-plan-ignored":
    runs = [run for run in runs if verdicts.decision(item_name(run[0], run[2]))[0] != rounds.REUSE]
judged_count = sum(1 for run in eligible if verdicts.decision(item_name(run[0], run[2]))[0] == rounds.JUDGE)
refused_count = sum(1 for run in eligible if verdicts.decision(item_name(run[0], run[2]))[0] == rounds.REFUSED)
reused_count = len(eligible) - judged_count - refused_count
if eligible:
    print(f"  · 按条复用：{len(eligible)} 条开关（{len({run[0] for run in eligible})} 道阶段），现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}", flush=True)
    shown = 0
    for run in eligible:
        decision, reason = verdicts.decision(item_name(run[0], run[2]))
        if decision == rounds.REUSE:
            continue
        shown += 1
        if shown <= 20:
            print(f"    {'拒算' if decision == rounds.REFUSED else '现判'}「{item_name(run[0], run[2])}」：{reason}")
    if shown > 20:
        print(f"    …现判与拒算另有 {shown - 20} 条没列")
    if reused_count <= 20 or sample:
        for run in eligible:
            decision, reason = verdicts.decision(item_name(run[0], run[2]))
            if decision == rounds.REUSE:
                print(f"    复用「{item_name(run[0], run[2])}」：{reason}")

tracker = progress_library.ReplayProgress("stage-break-switch-replay", len(runs), progress_seconds, broken == "replay-progress-silent")


def replay(run):
    stage, entry, switch, named, samples = run
    environment = {name: value for name, value in os.environ.items()
                   if not name.endswith("_BREAK") and name not in ("GATE_BASE", "GATE_STAGED_FROM", "GATE_DIFF_BASE", "GATE_NOT_RUN_FILE")}
    environment[entry["name"]] = entry["value"]
    started = time.monotonic()
    with tracker.running(stage, f"{stage} {switch}"):
        cmd = ["bash", stage_selftest, os.path.join(root, ".claude/gate.d"), "--item", stage]
        if os.path.isfile(prune_wrapper):
            # 二进制只经 --server-binary 交：环境里带来的 PRUNE_SERVER_BINARY 与任务服务给的编译目录不替它作保，这一格在哪台机器上都判同一个结局
            for ambient in ("PRUNE_SERVER_BINARY", "CARGO_TARGET_DIR"):
                environment.pop(ambient, None)
            cmd = ["python3", prune_wrapper, "--with-temporary-service", *(["--server-binary", server_binary] if server_binary else []), "--"] + cmd
        done = subprocess.run(cmd, cwd=root, env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
    output = done.stdout + done.stderr
    wrong = judge_library.wrong_samples(output, stage, samples)
    finished = judge_library.run_finished(done.returncode, output)
    result = (stage, entry, switch, named, wrong, finished, done.returncode, output)
    problem = judge_library.problem_of(*result, broken)
    verdict = ("对上" if problem is None else "没对上（点名的样本与实测不符）") if finished else f"没跑成（stage-selftest.sh 退 {done.returncode}）"
    tracker.item_done(verdict, f"{stage} {switch}", time.monotonic() - started)
    # 一条判完就记（这一格文件头「按条复用」）：判绿记绿，判红记红、撤同一把键原有的绿；中途被停，判完的那几条都在库里
    name = item_name(stage, switch)
    verdicts.record([] if problem else [name], [name] if problem else [])
    return result

collected = []
if runs:
    tracker.start(f"stage-break-switch-replay 开跑：{len(runs)} 条开关（{len({run[0] for run in runs})} 道阶段），并行 {max(jobs, 1)} 条；每条判完打一行，每过 {progress_seconds} 秒打一行总进度")
with concurrent.futures.ThreadPoolExecutor(max_workers=max(jobs, 1)) as pool:
    collected = list(pool.map(replay, runs))
tracker.close()
if len(collected) != len(runs):
    problems.append(f"派出去 {len(runs)} 条复跑，只收回 {len(collected)} 条")
green_names, red_names = [], []
for result in collected:
    problem = judge_library.problem_of(*result, broken)
    (green_names if problem is None else red_names).append(item_name(result[0], result[2]))
    if problem is not None:
        problems.append(problem)
verdicts.record(green_names, red_names, final=True)
if red_names:
    verdicts.reread_after_red()
verdicts.write_states(states_path, [item_name(run[0], run[2]) for run in eligible])
shutil.rmtree(plan_scratch, ignore_errors=True)
for line in not_run:
    print(f"  ! 本次未跑：{line}")
    if not_run_file and os.path.exists(not_run_file):
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"{own_stage} stage-break-switch-replay：{line}\n")
if not stages:
    print("  ! .claude/gate.d/ 顶层一道有样本目录的阶段都没有，这一格无对象可判")
    sys.exit(77)
replayed_stages = sorted({stage for stage, *_rest in collected})
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 门禁阶段的弄坏开关复跑：{len(problems)} 处不对（复跑了 {len(collected)} 条、{len(replayed_stages)} 道阶段；本次未跑 {len(not_run)} 项）")  # gate-lint:summary
    print("     → 怎么办：点名的与实测不等的，照实测改那一道文件头那一条下面点名的样本（实测是对的才改头注；样本判错是因为判法坏了就修判法），"
          f"一时改不准的登记进 {EXEMPT}（只缩不涨）；头注还不是 mutate.sh 写法的整道登记 *；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ 门禁阶段的弄坏开关复跑：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay")
    sys.exit(75)
if verdicts.red:
    print(f"  ✗ 门禁阶段的弄坏开关复跑判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；起了的 {len(collected)} 条都对上")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
if not eligible:
    print(f"  ! 一条开关都没复跑（本次未跑 {len(not_run)} 项），这一格无对象可判")
    sys.exit(77)
reuse_text = f"按条复用：现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}" if verdicts.ok else "按条复用算不出、全部现判"
if not collected:
    print(f"  ✓ 门禁阶段的弄坏开关复跑：点到的 {len(eligible)} 条都有键相同的判定，全部复用，这一格一条都没起（{reuse_text}）；本次未跑 {len(not_run)} 项（列在上面）")
    sys.exit(0)
print(f"  ✓ 门禁阶段的弄坏开关复跑：复跑了 {len(collected)} 条开关（{len(replayed_stages)} 道阶段：{'、'.join(replayed_stages)}），判错的样本与文件头点名的逐条相等；本次未跑 {len(not_run)} 项（列在上面）；{reuse_text}")
PY
}

# ── 格 break-switch-outside-selftest：还 checks-owed.md C622（门禁的弄坏开关平常跑也认）研究脚本与钩子那一半。
# 判据：被判的根里 scripts/ 与 .claude/hooks/ 下（fixtures 目录除外）的 .sh、.py，代码里读了名字以 _BREAK 结尾的变量的（读法同 break-switch-headers 格，注释行不算），
#   平常跑（不在自证里）时设上这个变量要被拒：脚本退非 0，且输出里有一行点名这个变量、说它只在自证里认（含「自证」或「selftest」）。
#   这一格逐份、逐个变量起一次探针：env <变量>=<文件头开关表里这个变量的第一个值（不取关守卫项，见下），没有表写 probe> bash|python3 <脚本> --force --break-guard-probe-unknown-option，
#   stdin 接空，最多 30 秒；那个选项没有哪份脚本认得，没有守卫的脚本只会照常跑或退用法错，说不出「只在自证里认」。
#   关守卫项（开关表里值是 guard-off 或 honor-outside-selftest、或说明里同时写着「平常跑」与「不拒」的那一项：它弄坏的就是守卫本身，单独设时平常跑照认）
#   只在它是唯一一项时认：这个变量的开关表里有关守卫项的，另起一次探针 env <变量>=<关守卫项>,<第一个值>（两次探针的第一个值都取表里第一条不是关守卫项的值，没有写 probe），
#   照样要被拒并点名（拆分提案第一百二十二节 ②：按逗号拆开、里面有关守卫项就放行的脚本，把关守卫项和别的项同写，别的开关平常跑也借道生效）。
#   还没改成这样的存量逐份登记在 .claude/gate.d/break-switch-outside-selftest-pending.tsv（「<相对仓根的路径><TAB># 理由>」），登记了的不起探针（探针对没有守卫的脚本
#   不保证无害，所以登记着的不探）；一行指到的文件不在、或代码里不读弄坏开关了，判红：那一行删掉。表只缩不涨（不判涨，靠 review）；改好一份删一行。
# 管不到的：守卫放在读完参数之后的脚本（探针的选项在那之前就被当用法错拒了，要先把守卫挪到参数解析之前，或登记成存量）；没有命令行入口、被别的程序 import 的库也算存量。
# 一份读开关的都没有：退 77。
# 样本：break-switch-outside-selftest-red（一份读开关、平常跑照常退 0，一份读开关、退 2 却不点名变量，一份守住的，登记表一行指到不在的文件、一行指到不读开关的，逐条点名判红）；
#   break-switch-outside-selftest-green（一份守住的，一份没守住而登记在存量表里，一份不读开关，报「读开关的 2 份：探了 1 份……待改 1 份」）。
#   break-switch-outside-selftest-guard-combined-red（一份按逗号拆开、里面有 guard-off 就放行，一份只认「平常跑」「不拒」写在说明里的关守卫项、同样拆开放行：
#   第一个值单独设都被拒，「关守卫项,第一个值」两份都没被拒，必须各点名一次）；break-switch-outside-selftest-guard-combined-green（两份关守卫项都只在唯一一项时认，
#   一份关守卫项排在表里第一条：第一个值取它后面那一条，报「关守卫项与第一个值同写的探针 2 次」）。
# 弄坏开关 CODE_TOOLING_BREAK=outside-selftest-unchecked：探针的结果一概当守住，break-switch-outside-selftest-red 与 break-switch-outside-selftest-guard-combined-red 里「没被拒」那几条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=guard-combination-unprobed：不起「关守卫项,第一个值」那一次探针（改前的样子），break-switch-outside-selftest-guard-combined-red 判绿（期望退出 1，实测 0），
#   break-switch-outside-selftest-guard-combined-green 里「关守卫项与第一个值同写的探针 2 次」那条 want 找不到（探了 0 次）。
cell_break_switch_outside_selftest() {
  local pending=".claude/gate.d/break-switch-outside-selftest-pending.tsv"
  python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$pending" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, re, subprocess, sys, types
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
pending_path, broken = sys.argv[2:4]
READ = re.compile(r"\$\{?([A-Z][A-Z0-9_]*_BREAK)\b|environ(?:\.get)?\s*[\(\[]\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']|getenv\(\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']"
                  r"|^\s*[A-Z_][A-Z0-9_]*\s*=\s*[\"']([A-Z][A-Z0-9_]*_BREAK)[\"']", re.M)
PROBE_ARGUMENTS = ["--force", "--break-guard-probe-unknown-option"]
GUARD_OFF_VALUES = ("guard-off", "honor-outside-selftest")


def guard_off_value(text, entry):
    """这一条开关是不是关守卫项：值是 GUARD_OFF_VALUES 之一，或说明里同时写着「平常跑」与「不拒」。"""
    if entry["value"] in GUARD_OFF_VALUES:
        return True
    found = table.SWITCH_ENTRY.match(text.split("\n")[entry["line"] - 1])
    rest = found.group("rest") if found else ""
    return "平常跑" in rest and "不拒" in rest


def probe(path, interpreter, name, value):
    """起一次探针 → (退出码或「超时」, 输出)。"""
    try:
        done = subprocess.run(["env", f"{name}={value}", *interpreter, path, *PROBE_ARGUMENTS], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=30)
        return done.returncode, done.stdout + done.stderr
    except subprocess.TimeoutExpired:
        return "超时", ""


def refused(name, code_out, output):
    named = [row.strip() for row in output.split("\n") if name in row and ("自证" in row or "selftest" in row.lower())]
    return code_out not in (0, "超时") and bool(named)


def why_not(name, code_out, output):
    first = output.strip().split("\n")[0][:100] if output.strip() else "（没有输出）"
    return "退 0，照常跑" if code_out == 0 else ("超过 30 秒没返回" if code_out == "超时" else f"退 {code_out} 但输出里没有点名 {name} 只在自证里认的一行（首行：{first}）")


paths = sorted(path for directory in ("scripts", ".claude/hooks") for extension in ("sh", "py")
               for path in glob.glob(f"{directory}/**/*.{extension}", recursive=True) if "/fixtures/" not in path)
problems, pending = [], {}
if os.path.isfile(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        path, _, reason = line.rstrip("\n").partition("\t")
        if len(reason.lstrip("#").strip()) < 4:
            problems.append(f"{pending_path}:{number} 不是「<路径><TAB># 理由>」（理由至少 4 个字）")
            continue
        pending[path] = number
readers, probed, waiting, combined_probes = {}, [], [], 0
for path in paths:
    text = open(path, encoding="utf-8", errors="replace").read()
    code = "\n".join(row for row in text.split("\n") if not row.lstrip().startswith("#"))
    names = sorted({name for found in READ.findall(code) for name in found if name})
    if names:
        readers[path] = (text, names)
for path, (text, names) in readers.items():
    if path in pending:
        waiting.append(path)
        continue
    interpreter = ["python3", "-B"] if path.endswith(".py") else ["bash"]
    entries = table.switch_entries(text)
    for name in names:
        # 第一个值取表里第一条不是关守卫项的（关守卫项单独设时平常跑照认，拿它探守卫探不出东西），没有表或只有关守卫项写 probe
        guards = [entry["value"] for entry in entries if entry["name"] == name and guard_off_value(text, entry)]
        value = next((entry["value"] for entry in entries if entry["name"] == name and not guard_off_value(text, entry)), "probe")
        code_out, output = probe(path, interpreter, name, value)
        if broken != "outside-selftest-unchecked" and not refused(name, code_out, output):
            problems.append(f"{path} 读了 {name}：平常跑时设上它（env {name}={value} … {' '.join(PROBE_ARGUMENTS)}）没被拒，{why_not(name, code_out, output)}")
        # 关守卫项与别的项同写（这一格文件头「关守卫项」一段）：另起一次探针，照样要被拒
        if not guards or broken == "guard-combination-unprobed":
            continue
        combined = f"{guards[0]},{value}"
        combined_probes += 1
        code_out, output = probe(path, interpreter, name, combined)
        if broken != "outside-selftest-unchecked" and not refused(name, code_out, output):
            problems.append(f"{path} 读了 {name}：关守卫项与别的项同写（env {name}={combined} … {' '.join(PROBE_ARGUMENTS)}）没被拒，{why_not(name, code_out, output)}"
                            f"——关守卫项 {guards[0]} 只在它是唯一一项时认，与别的项同写要照拒并点名")
    probed.append(path)
for path, number in sorted(pending.items(), key=lambda item: item[1]):
    if not os.path.isfile(path):
        problems.append(f"{pending_path}:{number} 指到的 {path} 不在，那一行删掉")
    elif path not in readers:
        problems.append(f"{pending_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
if not readers and not problems:
    print("  ! scripts/ 与 .claude/hooks/ 里没有读弄坏开关的脚本，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 平常跑设弄坏开关：{len(problems)} 处不对（读开关的 {len(readers)} 份：探了 {len(probed)} 份、待改 {len(waiting)} 份）")  # gate-lint:summary
    print("     → 怎么办：让脚本在参数解析之前先看这个变量：平常跑（不是 --selftest，也不是自证起的子进程）设了就退 2，输出里点名变量、说它只在自证里认"
          "（样子照 scripts/experiment-apparatus/e180-reading-shards.sh、scripts/mutation/mutate.sh）；关守卫项（guard-off 这类）整值比，只在它是唯一一项时放行"
          "（样子照 .claude/gate.d/lib/stage-cells.sh 的 break-honored-everywhere），与别的项同写照拒并点名；一时改不完的存量登记进 "
          f"{pending_path}（只缩不涨），改好就删那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 平常跑设弄坏开关：读开关的 {len(readers)} 份：探了 {len(probed)} 份（都被拒并点名变量只在自证里认）、待改 {len(waiting)} 份（{pending_path}，只缩不涨）；关守卫项与第一个值同写的探针 {combined_probes} 次，也都被拒")
PY
}

# ── 格 break-switch-replay：还 checks-owed.md C625（弄坏开关文件头三样没有门禁）复跑方那一半（格式那一半是 break-switch-headers 格）。
# 判据：登记表 .claude/gate.d/break-switch-replay-files.tsv（被判的根里那一份，「<相对仓根的路径><TAB><每条开关最多几秒>[<TAB><基准趟最多几秒>]<TAB># 理由>」）里每份脚本或钩子，
#   要在被判的根里读弄坏开关、文件头开关表每条三样写全（break-switch-headers 格的读法）、又在 .claude/hooks/selftest-registry.tsv 里登记成自证（selftest）：
#   先不设开关起一趟这份的 `--selftest`（基准趟，其余名字以 _BREAK 结尾的变量摘掉），要退 0；它打的失败行（带 ✗ 的行）是自证本来就有的
#   （被测的那一段该打的拒绝行），不算开关弄红的。再对表里每一条开关，在环境里设上它、照样起 `--selftest`：
#   这一趟的失败行减去基准趟的（整行去掉临时路径与数字之后比，按次数抵），剩下的是这条开关弄红的行；
#   点名的项取下一行起 「…」 里的原文（嵌套的括号配平，原文里的 … 是中间省掉的一段，各段按次序出现在同一行里）。
#   点名的格与实际红的格逐格相同：点名的每一项都要在剩下的行里照出来（少红的判「点名的项没在失败行里」）；剩下的行里以 ✗ 起头的每一行都要被某一项照出来
#   （多红的判「失败行没被点名」，逐行列）。不以 ✗ 起头、只在行中间带 ✗ 的行（自证转述被测程序输出的那几行）只拿来照点名的项，不要求被点名。
#   汇总行不要求点名：同一份脚本里红了的开关有一半以上（至少两条）下都多红的同一行（去掉临时路径与数字之后相同），是自证的汇总句（「N 处判错」这类），不当一格。
#   输出按字节收、照 UTF-8 解（stdout 与 stderr 各解各的）；解不了的那一路照 U+FFFD 换掉坏字节接着照判，另记一条「坏字节」红、点名哪一路第几字节；
#   单独的 \r 与 \r\n 都当换行（与按文本收时同一个认法），不留在行里把两行接成一行。
#   结局：点名的与实际红的逐格相同是对；退 2 并点名这个变量只在自证里认的是「自证自带开关轮」（C622 的守卫在外面设不进去，开关轮在自证里面，不在这里复跑）；
#   退 0（开关没弄坏什么）、点名的项里没有可比的「…」原文、少红或多红、超时、有坏字节，都判红；基准趟退非 0 或超时，那一份判红、开关趟不起。
#   不是对的先单独重跑一次再定，防并行下计时类自证偶然红。
#   各条互不依赖，按每台默认重活线程数（至多 8）并行（基准趟先齐，再起开关趟；一份的开关趟齐了就当场判这一份、要重跑的另派，不等别的份），
#   收回的条数与派出去的对不上整格红。
# 只在临时 prune-server 上跑的自证（要临时服务的表 .claude/hooks/selftest-temporary-service.tsv 里登记成 required：裸跑退 2、不写真服务，research-script-selftests 格同一个认法）：
#   基准趟与开关趟都照「$SELFTEST_TEMPORARY_SERVICE_PREFIX」那一句包一层起（--server-binary 给这一格开跑前取的那一份，code_tooling_prune_server_binary；
#   包装起的临时服务一趟一份，各趟互不串），开关照样设在环境里传进去；表里有这种的才取二进制，取不到的那几份判红、开关一条都不起。
#   样本 break-switch-replay-temporary-service-green：样本根自己的要临时服务的表里登记成 required 的一份样本自证（源码里不写那几个字，只靠表认），不在样本的假包装（scripts/prune-server/prune_server_client_selftest.py，
#   只设一个变量再起那条命令）底下就退 2；登记一条开关，必须判绿（二进制取 PRUNE_SERVER_BINARY，样本档在包装底下起时由包装设）。
#   弄坏开关 CODE_TOOLING_BREAK=replay-temporary-service-unwrapped：不包（改前的样子），那一份样本判红（期望退出 0，实测 1：「不设开关的 --selftest 就没过（退 2）」）。
# 两个限时分开读（拆分提案第一百二十二节 ③）：
#   第二列「每条开关最多几秒」只管开关趟（设上一条开关起的那一趟 --selftest）；第三列「基准趟最多几秒」只管基准趟（不设开关起的那一趟），不写就照第二列。
#   自证自带开关轮的那几份（crash-point-replay.sh、crash-amplification.sh 这类，外面设开关被守卫当场拒）：开关趟一起来就退 2，基准趟却要把自证与它自己的
#   整轮开关都跑完，比单条开关趟慢得多，第三列照「单跑一条开关的秒数 ×（1 + 开关条数 ÷ 开关轮并行度，向上取整）」写。
# 同一张文件头开关表的两种读法：这一格读每条开关下一行起「」里的失败行原文（点名的项）；自证自带开关轮的脚本，它自己的开关轮读括号外的格名、不读「」里的原文。
#   两种读法各管各的：这一格对自带开关轮的只核守卫（退 2 并点名只在自证里认），格名对不对归它自己的开关轮。
# 没接复跑的（文件头开关表写全、登记成自证、又不在复跑登记表里的）：成功句逐个列名，往 $GATE_NOT_RUN_FILE 报一行；接进来的办法是往登记表加一行，
#   先拿原型把那份脚本的每一条开关跑一遍，点名的格与实际红的格逐格相同再加。登记表只往里加、不往外删，除非那份脚本不再读开关。
# 管不到的：失败行不带 ✗ 的自证（点名的项只照带 ✗ 的行）；汇总行的认法是按次数推的，一格真被一半以上的开关弄红时它也当汇总行、不要求点名；
#   自证自带开关轮的那一份开关轮判得对不对（归它自己的自证）。
# 进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟；写法在 SWITCH_REPLAY_PROGRESS_LIBRARY）：开跑打一行「break-switch-replay 开跑：」
#   （份数、开关条数、并行度）；每条开关判完打一行「break-switch-replay 第 k / N 条<对上、对上（自证自带开关轮）或没对上（结局）>：<路径> <开关>，用了 S 秒」
#   （S 是这条开关趟加重跑的秒数）；每份的开关轮判完（或基准趟没过、开关一条没起）打一行「break-switch-replay 跑完一份：<路径> 开关 N 条、对上 M 条、用了 S 秒」
#   （S 从这份第一趟起跑算起）；另起一条线程每过 300 秒打一行总进度（判完几条、在跑几趟）、跑得最久的三趟、一行「按份：」（跑完几份、还在跑几份），
#   再给起跑了还没跑完的每份打一行「break-switch-replay 还在跑：<路径> 开关 N 条、已跑完 T 趟、判完 K 条、用了 S 秒」，末了一行内存（后代进程数、Pss 合计、cgroup 记账、到此刻的峰值），
#   每 5 秒采一次内存；收尾打一行「break-switch-replay 逐条进度 R 行、派出去 N 条；」加内存峰值、峰值那一刻在跑的按份计数与单个最大的进程。
#   样本根（judged_root_is_sample）里的 .break-switch-replay-progress-seconds（一个正整数秒数）顶替 300 秒，样本不必等满 5 分钟；写的不是正整数判红、一条都不跑。
#   真仓上一趟 58 份、1386 条开关；并行 8 时第二台 8689 秒跑完 854 趟（任务 3542 的日志，被取消前最后一次报的进度），照这个速度整格约四小时（推的）。
# 登记表空或不在：退 77。
# 样本：break-switch-replay-red（登记表里四份：一份有一条开关点名的项没在失败行里，一份有一条开关设上之后自证照样退 0，一份点名的项里没有「…」原文；
#   另一行指到不在的文件）；break-switch-replay-unnamed-red（一份开关弄红两格、点名一格；一份红的一格前面是单独的 \r 接着的进度行，没点名：两处多红）；
#   break-switch-replay-green（一份全对、一份自证自带开关轮、一份基准趟本来就打一行 ✗ 拒绝行且三条开关各红一格加一句汇总行、一份只有一条开关而基准趟也打那一行，
#   另有一份文件头写全却没登记，报「复跑了 6 条开关……没接复跑的 1 份」）；
#   break-switch-replay-undecodable-red（一份两条开关、一条下自证吐坏字节，一份全对，一份退 0，一份吐坏字节而点名的项也不在失败行里：
#   坏字节、退 0、坏字节加少红各记，报「4 处不对（复跑了 5 条、4 份）」）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-items-unchecked：不比点名的项，break-switch-replay-red、break-switch-replay-undecodable-red 里「点名的项没在失败行里」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-output-strict-decode：退回按文本收输出、不容坏字节（改前的收法），break-switch-replay-undecodable-red 整格抛 UnicodeDecodeError，
#   「坏字节，stdout 里有不合法的 UTF-8」与「4 处不对（复跑了 5 条、4 份）」那几条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-unnamed-rows-unchecked：不查多红（失败行比点名的多不判，改前的判法），break-switch-replay-unnamed-red 判绿（期望退出 1，实测 0）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-baseline-unsubtracted：不起基准趟、不减自证本来就有的失败行，break-switch-replay-green 判红（期望退出 0，实测 1：只有一条开关的那一份，基准趟那一行没被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-summary-rows-required：汇总行也要点名，break-switch-replay-green 判红（期望退出 0，实测 1：「✗ 自检：… 格不对」没被点名）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-carriage-return-kept：单独的 \r 留在行里，break-switch-replay-unnamed-red 里「scripts/gate/carriage.py CARRIAGE_BREAK=progress」那条 want 找不到
#   （回车后面那一格接在进度行后面、不以 ✗ 起头，不要求点名；只剩一处不对）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-bad-bytes-first：有坏字节就只记坏字节、不再比点名（改前的次序），break-switch-replay-undecodable-red 里
#   「scripts/gate/garbled-mismatch.py GARBLED_MISMATCH_BREAK=bytes：不符」与「4 处不对」两条 want 找不到。
# 样本 break-switch-replay-slow-baseline-green：一份自证自带开关轮的脚本（外面设开关当场退 2 并点名；不设开关时自证先睡 3 秒、算它自己那一轮），
#   登记成每条开关 2 秒、基准趟 8 秒，必须判绿（开关趟只要一瞬，基准趟要 3 秒，同用 2 秒就超时）。
# 弄坏开关 CODE_TOOLING_BREAK=replay-baseline-shares-switch-limit：基准趟也照第二列的限时跑（改前的样子），break-switch-replay-slow-baseline-green 判红
#   （期望退出 0，实测 1：「不设开关的 --selftest 就没过（超时：基准趟限时 2 秒）」）。
# 样本 break-switch-replay-progress-green：样本根里 .break-switch-replay-progress-seconds 写 1；一份脚本两条开关，自证每一趟（基准趟与开关趟）都先睡 3 秒，
#   两条开关各红一格、点名都对，必须判绿，且输出里有「break-switch-replay 还在跑：scripts/gate/sleepy.sh」（一趟 3 秒、间隔 1 秒，基准趟里就至少报两次）
#   与「break-switch-replay 跑完一份：scripts/gate/sleepy.sh 开关 2 条、对上 2 条」，另有逐条的「第 1 / 2 条对上」「第 2 / 2 条对上」、内存一行与「逐条进度 2 行、派出去 2 条」。
# 弄坏开关 CODE_TOOLING_BREAK=replay-progress-silent：不打进度、不起报进度的线程（改前的样子），break-switch-replay-progress-green 里那几条进度的 want 找不到
#   （与 stage-break-switch-replay 格同一条开关，各红各的进度样本）。
# 按条复用（.claude/rules/implementation-workflow.md「复用重型门禁的判定：按每条判定的单元键判」；共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：
#   一条判定 = 登记表里的一份脚本加它的一条开关（名字「<路径> <开关名>=<值>」），外加那一份不设开关的基准趟（名字「<路径> 基准趟」）；
#   种类 code-tooling.sh:break-switch-replay（prune-server 那一份库，unit-verdicts 族 singlefs-unit-verdicts/code-tooling.sh:break-switch-replay，库根不进键）。
#   键（scripts/gate/admission.py verdict-plan 算，不带路径、行号与限时）由这几样组成：
#   ① 那份脚本自证的单元闭包（入口就是那份脚本：python 的程序入口、shell 的 script-body，连同 import、source 到的模块与函数、读的数据文件、解释器版本，
#      与 research-script-selftests 格算单元键的是同一路，admission.py 文件头「单元键」）；
#   ② 判法：这一道所在仓里 break_switch_replay_judge_code 的闭包（令牌 BREAK_SWITCH_REPLAY_JUDGE_TOKEN：开关表的读法与 BREAK_SWITCH_REPLAY_JUDGE_LIBRARY）；
#   ③ line:command=<起它的命令样子>；④ line:switch=<开关名>=<值>（基准趟写「基准趟」）；
#   ⑤ 开关那几条另加 line:named-sha256=<那一条点名的项原文的 sha256>（文件头是注释，不进单元指纹，另进这一行）；
#   ⑥ 只在临时 prune-server 上跑的另加 file:scripts/prune-server/prune_server_client_selftest.py 与 line:temporary-service=wrapped。
#   起哪几份：点到的条里有一条要现判（键对不上、库里记着红、拒算、复用哨兵强制现判）的整份照起——基准趟与全部开关趟，不是只起那几条：
#   汇总行要整份的开关趟，只起几条判出来的与整格里的不同；别的条这一趟陪跑，判完照记库、判红照报（成功句与按条复用那一行报陪跑几条）。点到的条都复用的整份跳过。
#   键相同、库里记着绿的复用；每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（种子打进日志；被判的根下有 .unit-verdict-sample/ 的样本不抽），失灵交 plan-failure 判红。
#   一份判完就记：基准趟判绿（没超时、退 0、没有坏字节）、开关对上（对、守卫，没有坏字节）记绿，别的照同一把键记红、撤掉原有的绿（verdict-record --red-names）；
#   基准趟没过、开关一条没起的那一份只记基准趟；收尾把这一趟判完的整份再交一次（哨兵在那一趟比）。
#   成功句报「按条复用：现判 N、复用 M、拒算 R、陪跑 C」；全部复用时报「全部复用，这一格一条都没起」、退 0。算不出（verdict-plan 退非 0、没打 PLAN）照全部现判跑完、判红；
#   prune-server 不可用（退 75）一份都不起、这一格退 75。
#   逐项点名（这一道「逐项点名」一节）：项是 <脚本路径> 或 <脚本路径>:<开关名>=<值>；点一条开关时那一份的基准趟与那几条算点到的，起不起照上面「起哪几份」。
#   样本：break-switch-replay-reuse-three-questions（两份脚本五条；复用三问：change.sh 改 alpha.py 的 expected_cells，affected 是 alpha 的三条；四趟都绿、三问都过）；
#     break-switch-replay-named-in-key（同样两份；change.sh 只改 alpha.py 头注里一条开关点名的项的写法（照样照得出）：那一条现判，同一份别的两条陪跑、仍算复用）；
#     break-switch-replay-red-over-green（.unit-verdict-sample/prior/ 照判绿记四条；red-over-green 下一律现判，样本根的 .red-switch 让 flaky.py 那条开关红在没点名的一格，
#     判红记红、撤绿，再算一次计划：复用 3 条、现判 1 条、原因「上次判红」）；break-switch-replay-item-co-run-green（点 tally.py 的一条开关：那一份照整份起，
#     另两条陪跑，汇总行认得出）、break-switch-replay-item-unknown（项写错）。
#   弄坏开关（文件头开关表）CODE_TOOLING_BREAK=replay-plan-ignored、replay-red-not-recorded、replay-key-whole-registry、replay-named-unkeyed、replay-siblings-not-co-run、
#     items-ignored、item-unknown-accepted，各红在开关表点名的样本。
cell_break_switch_replay() {
  local registry=".claude/gate.d/break-switch-replay-files.tsv" jobs
  [[ -f "$registry" ]] || { echo "  ! 没有 $registry，这一格无对象可判"; exit 77; }
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），复跑的并行度定不了，一条都没跑"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  # 只在临时 prune-server 上跑的自证（这一格文件头那一段）：表里有就开跑前取一次二进制，取不到留空（python 那一侧把那几份判红、点名原因在上一行）
  local replay_path replay_needs_binary=0 replay_binary="" replay_scratch="" replay_exit replay_required=""
  temporary_service_table_load || exit 1
  while IFS=$'\t' read -r replay_path _; do
    if [[ -n "$replay_path" && "$replay_path" != \#* && -f "$replay_path" ]] && temporary_service_is_required "$replay_path"; then
      replay_needs_binary=1
      replay_required+="$replay_path"$'\n'
    fi
  done < "$registry"
  if (( replay_needs_binary )) && [[ "${CODE_TOOLING_BREAK:-}" != replay-temporary-service-unwrapped ]]; then
    replay_scratch="$(mktemp -d)"
    replay_binary="$(code_tooling_prune_server_binary "$replay_scratch")" || replay_binary=""
  fi
  # 进度间隔（这一格文件头「进度」）：样本根里的 .break-switch-replay-progress-seconds 顶替 300 秒
  local replay_progress_seconds
  if ! replay_progress_seconds="$(switch_replay_progress_seconds .break-switch-replay-progress-seconds)"; then
    printf '%s\n' "$replay_progress_seconds"
    [[ -z "$replay_scratch" ]] || rm -rf -- "$replay_scratch"
    exit 1
  fi
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds break-switch-replay break_switch_replay_pass; replay_exit=$?
  else
    break_switch_replay_pass ""; replay_exit=$?
  fi
  [[ -z "$replay_scratch" ]] || rm -rf -- "$replay_scratch"
  return "$replay_exit"
}

# break_switch_replay_pass <各条结局写到哪，空是不写>：这一格的一趟（算键、查库、起键对不上的那几份、记库、判）；
# registry、jobs、replay_binary、replay_required、replay_progress_seconds 取调用它的那一格的
break_switch_replay_pass() {
  python3 - "$(break_switch_replay_judge_code)" "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$registry" "${CODE_TOOLING_BREAK:-}" "$jobs" "${GATE_NOT_RUN_FILE:-}" \
    "$replay_binary" "$replay_required" "$SELFTEST_TEMPORARY_SERVICE_PREFIX" "$replay_progress_seconds" "$SWITCH_REPLAY_PROGRESS_LIBRARY" \
    "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:break-switch-replay" "$BREAK_SWITCH_REPLAY_JUDGE_TOKEN" "$STAGE_REPOSITORY" \
    "${CODE_TOOLING_PICKED[break-switch-replay]:-}" "$(judged_root_is_sample && echo 1)" "$1" <<'PY'
import collections, concurrent.futures, glob, hashlib, importlib.util, os, re, shutil, subprocess, sys, tempfile, threading, time, types
judge_library = types.ModuleType("lib_break_switch_replay_judge")
exec(compile(sys.argv[1], "break-switch-replay-judge-library", "exec"), judge_library.__dict__)
table = judge_library   # 开关表的读法与判法在同一个模块里（break_switch_replay_judge_code 打的两段）
runs_library_path, registry_path, broken, jobs, not_run_file = sys.argv[2], sys.argv[3], sys.argv[4], min(int(sys.argv[5]), 4), sys.argv[6]
server_binary, service_required, service_prefix = sys.argv[7], set(sys.argv[8].split("\n")) - {""}, sys.argv[9].split()
progress_seconds = int(sys.argv[10])     # 进度间隔（这一格文件头「进度」）
progress_library = types.ModuleType("lib_switch_replay_progress")
exec(compile(sys.argv[11], "switch-replay-progress-library", "exec"), progress_library.__dict__)
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[12], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_token, judge_root = sys.argv[13], sys.argv[14], sys.argv[15], sys.argv[16]
picked = [item for item in sys.argv[17].split("\n") if item]      # 逐项点名（这一道「逐项点名」一节）：空是整格
sample, states_path = sys.argv[18] == "1", sys.argv[19]
needs_service = set()   # 只在临时 prune-server 上跑的那几份（要临时服务的表里登记成 required，shell 一侧读好交进来的 service_required）：基准趟与开关趟都包一层起
spec = importlib.util.spec_from_file_location("lib_project_code_runs", runs_library_path)
runs_library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runs_library)
READ = re.compile(r"\$\{?[A-Z][A-Z0-9_]*_BREAK\b|environ(?:\.get)?\s*[\(\[]\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']|getenv\(\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']"
                  r"|^\s*[A-Z_][A-Z0-9_]*\s*=\s*[\"'][A-Z][A-Z0-9_]*_BREAK[\"']", re.M)
selftests, _ = runs_library.read_selftest_registry(".claude/hooks/selftest-registry.tsv")
problems, registered, queue, baseline_seconds_of = [], {}, [], {}
BASELINE = "基准趟"
picked_paths, picked_switches = set(), {}
for item in picked:
    item_path, separator, item_switch = item.partition(":")
    if separator:
        picked_switches.setdefault(item_path, set()).add(item_switch)
    else:
        picked_paths.add(item_path)


def path_picked(path):
    return not picked or path in picked_paths or path in picked_switches


def named_by_item(path, label):
    """这一条是不是点到的：整格、点了整份的，各条都是；点了 <路径>:<开关> 的，那几条与那一份的基准趟是，同一份别的开关趟只陪跑。"""
    return not picked or path in picked_paths or label == BASELINE or label in picked_switches.get(path, ())


def reads_switch(path):
    text = open(path, encoding="utf-8", errors="replace").read()
    code = "\n".join(row for row in text.split("\n") if not row.lstrip().startswith("#"))
    return text, bool(READ.search(code))


registry_rows = [line.rstrip("\n").split("\t")[0] for line in open(registry_path, encoding="utf-8") if line.strip() and not line.startswith("#")]
for number, line in enumerate(open(registry_path, encoding="utf-8"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    columns = line.rstrip("\n").split("\t")
    if not path_picked(columns[0]):
        continue
    baseline_column = columns[2] if len(columns) == 4 else columns[1] if len(columns) == 3 else ""
    if (len(columns) not in (3, 4) or not columns[1].isdigit() or int(columns[1]) < 1 or not baseline_column.isdigit() or int(baseline_column) < 1
            or len(columns[-1].lstrip("#").strip()) < 4):
        problems.append(f"{registry_path}:{number} 不是「<路径><TAB><每条开关最多几秒>[<TAB><基准趟最多几秒>]<TAB># 理由>」（秒数是正整数，理由至少 4 个字）")
        continue
    path, seconds = columns[0], int(columns[1])
    registered[path] = seconds
    baseline_seconds_of[path] = int(baseline_column)
    if not os.path.isfile(path):
        problems.append(f"{registry_path}:{number} 指到的 {path} 不在，那一行删掉")
        continue
    text, reads = reads_switch(path)
    entries = table.switch_entries(text)
    row = selftests.get(path)
    if not reads:
        problems.append(f"{registry_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
    elif not entries or any(table.incomplete(entry) for entry in entries):
        problems.append(f"{registry_path}:{number} {path} 文件头开关表没写全（break-switch-headers 格会点名），先写全再复跑")
    elif row is None or row.kind != runs_library.SELFTEST_KIND:
        problems.append(f"{registry_path}:{number} {path} 在 .claude/hooks/selftest-registry.tsv 里不是自证的登记（{'没登记' if row is None else row.kind}），先登记成 selftest 再复跑")
    elif path in service_required and broken != "replay-temporary-service-unwrapped" and not server_binary:
        problems.append(f"{registry_path}:{number} {path} 只在临时 prune-server 上跑（要临时服务的表里登记成 required），这一趟取不到 prune-server 二进制（原因在上面一行），开关一条都没起")
    else:
        if path in service_required and broken != "replay-temporary-service-unwrapped":
            needs_service.add(path)
        queue.extend((path, seconds, entry) for entry in entries)


# 按条复用（这一格文件头「按条复用」）：一条判定 = 一份脚本加它的一条开关，外加那一份不设开关的基准趟；
# 键 = 那份脚本的单元闭包（入口就是那份脚本，照 research-script-selftests 格的算法）、判法（judge_token）、起它的命令样子、开关名与值、那一条点名的项的原文（文件头是注释、
# 不进单元指纹，另进一行 line:）；只在临时 prune-server 上跑的另加包装那一份；不带路径、行号与限时
def label_of(entry):
    return f"{entry['name']}={entry['value']}"


def item_name(path, label):
    return f"{path} {label}"


def command_text(path):
    return f"{'python3 -B' if path.endswith('.py') else 'bash'} {path} --selftest"


labels_of, entry_of = {}, {}
for path, _seconds, entry in queue:
    labels_of.setdefault(path, [BASELINE])
    if label_of(entry) not in labels_of[path]:
        labels_of[path].append(label_of(entry))
    entry_of[(path, label_of(entry))] = entry
all_paths = list(labels_of)


def request_line(path, label):
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:command={command_text(path)}", f"line:switch={label}"]
    if path in needs_service:
        tokens += ["file:scripts/prune-server/prune_server_client_selftest.py", "line:temporary-service=wrapped"]
    if label != BASELINE and broken != "replay-named-unkeyed":
        tokens.append("line:named-sha256=" + hashlib.sha256(entry_of[(path, label)]["items"].encode("utf-8")).hexdigest())
    if broken == "replay-key-whole-registry":
        tokens += [f"file:{other}" for other in sorted(set(registry_rows)) if os.path.isfile(other)]
    return "\t".join([item_name(path, label), path, *tokens])


request_lines = [request_line(path, label) for path in all_paths for label in labels_of[path]]
plan_scratch = tempfile.mkdtemp(prefix="break-switch-replay-plan.")
verdicts = rounds.ItemVerdicts(admission, os.getcwd(), kind, plan_scratch, "code-tooling.sh 的 break-switch-replay 格", sample,
                               "bash .claude/gate.d/code-tooling.sh --check break-switch-replay（逐条设上开关起那一份的 --selftest）", broken != "replay-red-not-recorded")
if request_lines:
    verdicts.plan(request_lines)
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print(f"  ✗ 脚本与钩子的弄坏开关复跑这一格没判：prune-server 不可用（上面那一句），{len(all_paths)} 份一条都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：这一趟每一份都起、一条都不记库")
named_items = [(path, label) for path in all_paths for label in labels_of[path] if named_by_item(path, label)]


def needs_judging(path, label):
    return verdicts.decision(item_name(path, label))[0] != rounds.REUSE


# 起哪几份：点到的条里有一条要现判（或拒算）的整份照起（基准趟与全部开关趟：汇总行要整份的开关趟，点名单跑一条与整格里判的相同）；点到的条都复用的整份跳过
run_paths = [path for path in all_paths
             if broken == "replay-plan-ignored" or any(needs_judging(path, label) for label in labels_of[path] if named_by_item(path, label))]
decision_count = collections.Counter(verdicts.decision(item_name(path, label))[0] for path, label in named_items)
judged_count, refused_count, reused_count = decision_count[rounds.JUDGE], decision_count[rounds.REFUSED], decision_count[rounds.REUSE]
co_run_count = sum(1 for path in run_paths for label in labels_of[path] if not (named_by_item(path, label) and needs_judging(path, label)))
if request_lines:
    print(f"  · 按条复用：{len(all_paths)} 份脚本、{len(named_items)} 条判定（基准趟 {sum(1 for _path, label in named_items if label == BASELINE)} 条、"
          f"开关 {sum(1 for _path, label in named_items if label != BASELINE)} 条），现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}；"
          f"起 {len(run_paths)} 份（陪跑 {co_run_count} 条：同一份里有条要现判就整份照起，汇总行要整份的开关趟）、跳过 {len(all_paths) - len(run_paths)} 份（点到的条都复用）", flush=True)
    shown = 0
    for path, label in named_items:
        decision, reason = verdicts.decision(item_name(path, label))
        if decision == rounds.REUSE:
            continue
        shown += 1
        if shown <= 20:
            print(f"    {'拒算' if decision == rounds.REFUSED else '现判'}「{item_name(path, label)}」：{reason}")
    if shown > 20:
        print(f"    …现判与拒算另有 {shown - 20} 条没列")
    if reused_count <= 20 or sample:
        for path, label in named_items:
            decision, reason = verdicts.decision(item_name(path, label))
            if decision == rounds.REUSE:
                print(f"    复用「{item_name(path, label)}」：{reason}")
# 弄坏开关 replay-siblings-not-co-run：同一份只起要现判的那几条开关趟（改错的样子：汇总行少了陪跑的开关趟，点名单跑一条与整格里判的不同）
queue = [job for job in queue if job[0] in run_paths
         and (broken != "replay-siblings-not-co-run" or (named_by_item(job[0], label_of(job[2])) and needs_judging(job[0], label_of(job[2]))))]
green_of = {}            # 判完的条 → 判绿没有（记库用；一份判完就记，收尾再整份交一次）


def record_path(path):
    """一份的基准趟与开关趟都判完（或基准趟没过、开关一条没起）：当场照键记这一份判完的条（这一格文件头「按条复用」一份判完就记）。"""
    with progress_lock:
        mine = [(name, green) for name, green in green_of.items() if name.startswith(path + " ")]
    verdicts.record([name for name, green in mine if green], [name for name, green in mine if not green])


progress_lock = threading.Lock()         # 管下面几张按份记的表；拿着它不调 tracker（tracker 自己的锁在里面拿打印锁，不反过来）
started_of = {}                          # 路径 → 这份第一趟起跑的时刻（time.monotonic）
trips_of = collections.Counter()         # 路径 → 跑完的趟数（基准趟、开关趟与重跑都算）
settled_of = collections.Counter()       # 路径 → 判完的开关条数
matched_of = collections.Counter()       # 路径 → 判完而对上的开关条数（对、守卫，且没有坏字节）
finished = set()                         # 开关轮跑完（或基准趟没过、开关一条没起）的路径


def seconds_since(path):
    return int(time.monotonic() - started_of.get(path, time.monotonic()))


def running_paths_lines():
    """报总进度时另打的几行：几份里跑完几份，起跑了还没跑完的每份一行（跑得长的那份中途也有进度）。"""
    with progress_lock:
        running = [path for path in started_of if path not in finished]
        snapshot = [(path, trips_of[path], switch_count_of.get(path, 0), settled_of[path], seconds_since(path)) for path in running]
        done = len(finished)
    return ([f"break-switch-replay 按份：{len(paths)} 份里跑完 {done} 份、还在跑 {len(snapshot)} 份"]
            + [f"break-switch-replay 还在跑：{path} 开关 {switch_count} 条、已跑完 {trips} 趟、判完 {settled} 条、用了 {elapsed} 秒"
               for path, trips, switch_count, settled, elapsed in snapshot])


def finish(path, text):
    """一份的开关轮跑完：打一行（脚本名、开关数、对上几条、用了几秒）。"""
    with progress_lock:
        finished.add(path)
    tracker.say(text)


def run_selftest(path, seconds, entry=None):
    """起一趟这份的 --selftest（entry 是 None 时不设开关，是基准趟）：→ {timeout, code, output, bad, seconds}。"""
    with progress_lock:
        started_of.setdefault(path, time.monotonic())
    started = time.monotonic()
    try:
        with tracker.running(path, f"{path} {'基准趟' if entry is None else entry['name'] + '=' + entry['value']}"):
            record = run_selftest_once(path, seconds, entry)
        record["seconds"] = time.monotonic() - started
        return record
    finally:
        with progress_lock:
            trips_of[path] += 1


def run_selftest_once(path, seconds, entry):
    environment = {name: value for name, value in os.environ.items() if not name.endswith("_BREAK")}
    if entry is not None:
        environment[entry["name"]] = entry["value"]
    interpreter = ["python3", "-B"] if path.endswith(".py") else ["bash"]
    strict = {"text": True} if broken == "replay-output-strict-decode" else {}
    command = [*interpreter, path, "--selftest"]
    if path in needs_service:   # 包装写法：-- 前插 --server-binary <这一趟那一份>（这一格文件头「只在临时 prune-server 上跑的自证」）
        command = [*service_prefix[:-1], "--server-binary", server_binary, "--", *command]
    try:
        done = subprocess.run(command, env=environment, stdin=subprocess.DEVNULL, capture_output=True, timeout=seconds, **strict)
    except subprocess.TimeoutExpired:
        return {"timeout": True, "code": None, "output": "", "bad": []}
    output, bad = judge_library.decoded_output((("stdout", done.stdout), ("stderr", done.stderr)), broken)
    return {"timeout": False, "code": done.returncode, "output": output, "bad": bad}


def judge(entry, record, baseline_rows, summary_keys):
    return judge_library.judge(entry, record, baseline_rows, summary_keys, broken)


def summary_keys_of(path, first):
    """这一份这一趟的各条开关趟（同一把键只留表里排在后面的那一趟）交判法认汇总行。"""
    records = [record for (job_path, _seconds, _switch), record in first.items() if job_path == path]
    return judge_library.summary_keys_of(records, baseline_rows_of[path], broken)


paths = sorted({path for path, _seconds, _entry in queue})
seconds_of = {path: seconds for path, seconds, _entry in queue}
# 基准趟的限时读第三列（这一格文件头「两个限时分开读」）；弄坏开关 replay-baseline-shares-switch-limit 退回改前：基准趟也照第二列
baseline_limit_of = {path: seconds_of[path] if broken == "replay-baseline-shares-switch-limit" else baseline_seconds_of[path] for path in paths}
baseline_rows_of, results = {}, []
switch_count_of = collections.Counter(path for path, _seconds, _entry in queue)
tracker = progress_library.ReplayProgress("break-switch-replay", len(queue), progress_seconds, broken == "replay-progress-silent", running_paths_lines)
if queue:
    tracker.start(f"break-switch-replay 开跑：{len(paths)} 份、{len(queue)} 条开关，并行 {jobs} 条；每条判完打一行、每份开关轮跑完打一行，"
                  f"每过 {progress_seconds} 秒打一行总进度并给还没跑完的每份打一行")


def settled(index, kind, why, record, seconds):
    """第 index 条开关判完：记结局，打逐条进度一行；这一份的最后一条判完时另打「跑完一份」。"""
    path, _seconds, entry = queue[index]
    with progress_lock:
        outcome_of[index] = (kind, why, record["bad"])
        settled_of[path] += 1
        matched_of[path] += kind in ("对", "守卫") and not record["bad"]
        last = settled_of[path] == switch_count_of[path]
        matched = matched_of[path]
        green_of[item_name(path, label_of(entry))] = judge_library.switch_green(kind, record)
    if kind not in ("对", "守卫"):
        verdict = f"没对上（{kind}{'、坏字节' if record['bad'] else ''}）"
    elif record["bad"]:
        verdict = "没对上（坏字节）"
    else:
        verdict = "对上（自证自带开关轮）" if kind == "守卫" else "对上"
    tracker.item_done(verdict, f"{path} {entry['name']}={entry['value']}", seconds)
    if last:
        finish(path, f"break-switch-replay 跑完一份：{path} 开关 {switch_count_of[path]} 条、对上 {matched} 条、用了 {seconds_since(path)} 秒")
        record_path(path)


def rerun_and_settle(index, first_record):
    """不是对的先单独重跑一次再定（这一格文件头「判据」末尾）；用了几秒算两趟的和。"""
    path, seconds, entry = queue[index]
    record = run_selftest(path, seconds, entry)
    kind, why = judge(entry, record, baseline_rows_of[path], summary_of[path])
    settled(index, kind, why, record, first_record["seconds"] + record["seconds"])


outcome_of, summary_of = {}, {}
with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
    # 基准趟：不设开关起一趟，退非 0 或超时的单独重跑一次再定
    if broken != "replay-baseline-unsubtracted":
        baselines = dict(zip(paths, pool.map(lambda path: run_selftest(path, baseline_limit_of[path]), paths)))
        for path in paths:
            record = baselines[path]
            if record["timeout"] or record["code"] != 0:
                record = run_selftest(path, baseline_limit_of[path])
            with progress_lock:
                green_of[item_name(path, BASELINE)] = judge_library.baseline_green(record)
            if record["timeout"] or record["code"] != 0:
                problems.append(f"{path}：不设开关的 --selftest 就没过（{'超时：基准趟限时 ' + str(baseline_limit_of[path]) + ' 秒' if record['timeout'] else '退 ' + str(record['code'])}），开关下的失败行比不出哪些是开关弄红的，这一份的开关一条都没起")
                finish(path, f"break-switch-replay 跑完一份：{path} 开关 {switch_count_of[path]} 条、对上 0 条（基准趟没过，开关一条没起）、用了 {seconds_since(path)} 秒")
                record_path(path)
                continue
            problems.extend(f"{path} 不设开关：坏字节，{why}，点名的项比不准；改那份脚本让它只吐 UTF-8" for why in record["bad"])
            baseline_rows_of[path] = judge_library.failure_rows(record["output"])
    else:
        # 弄坏开关 replay-baseline-unsubtracted：不起基准趟（改前的样子），基准趟那一条这一趟没判、不记库
        baseline_rows_of = {path: [] for path in paths}
    queue = [job for job in queue if job[0] in baseline_rows_of]
    tracker.total = len(queue)
    # 开关趟全部派出去；一份的开关趟齐了就当场判这一份（汇总行要这一份的全部开关趟），不等别的份，逐条进度随判随打。
    # 判法与改前逐条相同：同一把键（路径、限时、开关）只留表里排在后面的那一趟，汇总行与判定都读这一份；要重跑的另派出去。
    key_of = [(path, seconds, entry["name"] + "=" + entry["value"]) for path, seconds, entry in queue]
    indexes_of = collections.defaultdict(list)
    for index, (path, _seconds, _entry) in enumerate(queue):
        indexes_of[path].append(index)
    waiting_of = {path: len(indexes) for path, indexes in indexes_of.items()}
    record_of, first, reruns = {}, {}, []
    trips = {pool.submit(run_selftest, path, seconds, entry): index for index, (path, seconds, entry) in enumerate(queue)}
    for future in concurrent.futures.as_completed(trips):
        index = trips[future]
        record_of[index] = future.result()
        path = queue[index][0]
        waiting_of[path] -= 1
        if waiting_of[path]:
            continue
        for sibling in indexes_of[path]:
            first[key_of[sibling]] = record_of[sibling]
        summary_of[path] = summary_keys_of(path, first)
        for sibling in indexes_of[path]:
            record = first[key_of[sibling]]
            kind, why = judge(queue[sibling][2], record, baseline_rows_of[path], summary_of[path])
            if kind not in ("对", "守卫") or record["bad"]:
                reruns.append(pool.submit(rerun_and_settle, sibling, record))
            else:
                settled(sibling, kind, why, record, record["seconds"])
    for future in concurrent.futures.as_completed(reruns):
        future.result()
    results = [outcome_of[index] for index in range(len(queue)) if index in outcome_of]
tracker.close()
if len(results) != len(queue):
    problems.append(f"派出去 {len(queue)} 条、收回 {len(results)} 条，对不上，这一趟不算数")
for (path, _seconds, entry), (kind, why, bad) in zip(queue, results):
    if kind not in ("对", "守卫"):
        problems.append(f"{path} {entry['name']}={entry['value']}：{kind}，{why}")
    problems.extend(f"{path} {entry['name']}={entry['value']}：坏字节，{reason}，改那份脚本让它在这条开关下只吐 UTF-8" for reason in bad)
# 收尾把这一趟判完的条整份再交一次（同一把键写同样的内容），复用哨兵强制现判的那一条在这一趟比
verdicts.record([name for name, green in green_of.items() if green], [name for name, green in green_of.items() if not green], final=True)
if any(not green for green in green_of.values()):
    verdicts.reread_after_red()
verdicts.write_states(states_path, [item_name(path, label) for path, label in named_items])
shutil.rmtree(plan_scratch, ignore_errors=True)
unregistered = []
selftest_files = [] if picked else sorted(path for directory in ("scripts", ".claude/hooks") for extension in ("sh", "py")
                                          for path in glob.glob(f"{directory}/**/*.{extension}", recursive=True) if "/fixtures/" not in path)
for path in selftest_files:
    if path in registered:
        continue
    text, reads = reads_switch(path)
    entries = table.switch_entries(text) if reads else []
    row = selftests.get(path)
    if entries and not any(table.incomplete(entry) for entry in entries) and row is not None and row.kind == runs_library.SELFTEST_KIND:
        unregistered.append(path)
if unregistered and not_run_file:
    with open(not_run_file, "a", encoding="utf-8") as handle:
        handle.write(f"code-tooling.sh break-switch-replay：{len(unregistered)} 份文件头写全的脚本还没接复跑\n")
guarded = sum(1 for kind, _why, _bad in results if kind == "守卫")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 脚本与钩子的弄坏开关复跑：{len(problems)} 处不对（复跑了 {len(queue)} 条、{len(set(path for path, _s, _e in queue))} 份）")  # gate-lint:summary
    print("     → 怎么办：点名的项没在失败行里、失败行没被点名的，照实测改那份文件头那一条下面「」里的失败行原文，让点名的格与实际红的格逐格相同"
          "（实测是对的才改头注；自证判错是因为判法坏了就修判法）；"
          f"设上之后自证照样退 0 的，开关没弄坏它说的那一处，改开关或改头注；不设开关自证就没过的，先修那份自证；输出带不合法 UTF-8 的（坏字节），改那份脚本让它在那条开关下只吐 UTF-8"
          f"（截断的多字节字符、原样转发的二进制都会这样）；基准趟超时的（自证自带开关轮的最常见），在 {registry_path} 那一行第三列写基准趟限时；"
          f"这份还接不进复跑的，从 {registry_path} 删掉那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ 脚本与钩子的弄坏开关复跑：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay")
    sys.exit(75)
if verdicts.red:
    print(f"  ✗ 脚本与钩子的弄坏开关复跑判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；起了的 {len(queue)} 条开关都对上")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check break-switch-replay；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
if not request_lines:
    print("  ! 登记表里没有一条可复跑的开关，这一格无对象可判")
    sys.exit(77)
reuse_text = (f"按条复用：现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}、陪跑 {co_run_count}" if verdicts.ok else "按条复用算不出、全部现判")
if not paths:
    print(f"  ✓ 脚本与钩子的弄坏开关复跑：点到的 {len(named_items)} 条都有键相同的判定，全部复用，这一格一条都没起（{reuse_text}）；"
          f"没接复跑的 {len(unregistered)} 份文件头写全的脚本：{'、'.join(unregistered) or '（没有）'}")
    sys.exit(0)
print(f"  ✓ 脚本与钩子的弄坏开关复跑：复跑了 {len(queue)} 条开关（{len(paths)} 份），自证红且点名的格与实际红的格逐格相同 {len(queue) - guarded} 条、自证自带开关轮 {guarded} 条；"
      f"没接复跑的 {len(unregistered)} 份文件头写全的脚本：{'、'.join(unregistered) or '（没有）'}；{reuse_text}")
PY
}

# ── 格 kb-directories-write-scope：还拆分提案 records/2026-09-16-subagent拆分提案.md 第九十节 ⑧（候选扫描改读 .claude/kb-directories 之后，写回员的写范围仍是写死的模式，
#   载体再搬家时写范围那一侧照样漏）。
# 判据：被判的根里 .claude/kb-directories 登记的每个 kb 目录（读法是这一道所在仓里 scripts/kb/quote-kb.py 的 kb_candidate_roots，不另写一份：登记是替换不是追加，
#   没登记、或登记的都不在、都被拒就是 .claude/kb），写回员（kb-scribe）在被判的根的 .claude/hooks/agent-write-scope.tsv 里那几行路径模式，要盖住这个目录下一层的 .md
#   与任意一层子目录里的 .md。探两条造出来的路径：<目录>/kb-directories-write-scope-probe.md 与 <目录>/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md，
#   不看目录里此刻有没有子目录（载体搬家之后才长出来的子目录同样要写得到）。哪一条没有一条模式配上，判红、逐个目录点名缺哪一层。
# 「盖住」照写范围闸本身的配法：这一道所在仓里 .claude/hooks/lib_write_scope.py 的 expanded_scope_patterns 与 path_in_scope（写闸 write-guard.sh 与 Bash 检出钩子用的同一份），
#   ** 跨目录（配成 .*）、* 不跨目录（配成 [^/]*）：<目录>/*.md 只盖下一层，子目录里的 .md 写不到；<目录>/**/*.md 只盖子目录，下一层写不到；<目录>/** 两样都盖。
#   以 $SINGLEFS_WORK_ROOT/ 起头的模式只管工作根，不展开、不参与。要子目录也算：候选扫描（quote-kb.py 的 invariant_class_files、门禁 doc-registries 的
#   kb_candidate_md_files）认登记目录下各层子目录里的 .md，写回员要能把去向写回那些文件。别的 agent 的行盖住了不算。
# 被判的根里没有 .claude/hooks/agent-write-scope.tsv：退 77（写范围闸没有表，无对象可判）。表里没有 kb-scribe 的行：每个登记目录都判红。
# 样本：kb-directories-write-scope-red（登记三个目录：.claude/kb 由 ** 盖全；notes/invariants 只有 *.md，子目录写不到；docs/extra 只被 clerk 的 docs/** 盖住、
#   kb-scribe 一条都没有；后两个逐个点名判红）；kb-directories-write-scope-green（三个目录 kb-scribe 都由 ** 盖全，报「盖住登记的 3 个 kb 目录」）。
# 弄坏开关 CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked：只探下一层、不探子目录，kb-directories-write-scope-red 照样退 1（docs/extra 那一个照红），
#   「notes/invariants：写回员（kb-scribe）写不到子目录里的 .md」与「登记的 3 个 kb 目录里 2 个写回员写不全」两条 want 找不到。
cell_kb_directories_write_scope() {
  local table=".claude/hooks/agent-write-scope.tsv"
  [[ -f "$table" ]] || { echo "  ! 被判的根里没有 $table，写范围闸无表可判，这一格无对象可判"; exit 77; }
  python3 - "$STAGE_REPOSITORY/scripts/kb/quote-kb.py" "$STAGE_REPOSITORY/.claude/hooks/lib_write_scope.py" "$table" "$ROOT" "${CODE_TOOLING_BREAK:-}" <<'PY'
import importlib.util, os, sys
sys.dont_write_bytecode = True
quote_kb_path, write_scope_path, table, root, broken = sys.argv[1:6]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


quote_kb = load("quote_kb", quote_kb_path)
write_scope = load("lib_write_scope", write_scope_path)
WRITER = "kb-scribe"
PROBE = "kb-directories-write-scope-probe"
directories = quote_kb.kb_candidate_roots(root)
patterns = write_scope.load_scopes(table).get(WRITER, [])
judged_patterns = write_scope.expanded_scope_patterns(patterns, None)
probes = [("下一层的 .md", f"{PROBE}.md")]
if broken != "kb-scope-subdirectory-unchecked":
    probes.append(("子目录里的 .md", f"{PROBE}/{PROBE}.md"))
problems = []
for directory in directories:
    missing = []
    for what, tail in probes:
        probe = os.path.join(directory, tail)
        if not write_scope.path_in_scope(judged_patterns, os.path.normpath(os.path.join(root, probe)), root):
            missing.append(f"{what}（探 {probe}）")
    if missing:
        problems.append(f"{quote_kb.KB_DIRECTORIES_REGISTRY} 登记的 {directory}：写回员（{WRITER}）写不到{'、'.join(missing)}")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    written = "、".join(patterns) if patterns else f"表里没有 {WRITER} 的行"
    print(f"  ✗ 登记的 {len(directories)} 个 kb 目录里 {len(problems)} 个写回员写不全（{table} 里 {WRITER} 的模式：{written}；** 跨目录、* 不跨目录）")  # gate-lint:summary
    print(f"     → 怎么办：在 {table} 给 {WRITER} 加一行 <目录>/**（只许 .md 的写成 <目录>/*.md 与 <目录>/**/*.md 两行），定义 .claude/agents/{WRITER}.md「写范围」同步改；"
          f"那个目录不该写回的，从 {quote_kb.KB_DIRECTORIES_REGISTRY} 撤掉；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 写回员（{WRITER}）的写范围盖住登记的 {len(directories)} 个 kb 目录（每个探下一层与子目录里的 .md 各一条）：{'、'.join(directories)}")
PY
}

# ── 格 prune-server-selftest：仓根 prune-server/（判定库服务，records/2026-11-01-判定库服务化.md）的自证过。
# 判据：被判的根里 prune-server/Cargo.toml 在，就编 prune-server 的服务二进制、直接起 `prune-server --selftest`（每格起一份真服务与真 RocksDB，约 22 秒；
#   与 `cargo run --release --offline --bin prune-server -- --selftest` 同一个入口），读它的标准输出：恰好一行、且在最后一行的
#   `PRUNE_SERVER_SELFTEST cells=<N> passed=<P> failed=<K> failed_cells=<格名,…|->`，N 大于 0、P+K 等于 N、failed_cells 的个数等于 K，退出码 0 且 K 为 0 才绿；
#   退 1 且 K 大于 0 是自证有格红（逐格点名、判红）；退出码与 K 对不上、末行缺或不在最后、N 为 0、退 2 或别的码（格名或开关写错、没跑成），都是输出认不出，判红。
#   prune-server/Cargo.toml 不在：这一格无对象可判，退 77。每一趟都跑，不读判定库（复用与开关扫一遍要不要进提交那一趟，见交回报告的岔路）。
# 起法：真仓与 --staged 的临时树上，先照这一道所在仓的 scripts/hosts/rocksdb-prebuilt.py entry --root <根> 判 RocksDB 预编库的取值（prune-server 依赖 rocksdb），判红这一格红；再在这一格自己的临时目录里 cargo build --release --offline --manifest-path <根>/prune-server/Cargo.toml --bin prune-server
#   （经 capped.sh default），在 <根>/prune-server 下直接起编出来的二进制，TMPDIR 指到临时目录里（自证的库与 socket 随临时目录一起删），临时目录连同编译目录跑完删（编不过时日志另拷一份留着）。
#   样本根（judged_root_is_sample）里放了 .prune-server-selftest-program 的，起它（python3，假的自证：只认 --selftest，打样本根里 .prune-server-selftest-lines 的内容再退样本根里 .prune-server-selftest-exit 写的码），不编。
# 样本（假的自证照样本根里的两份文件打行）：
#   prune-server-selftest-green（20 格都过，末行 failed=0、退 0）；prune-server-selftest-absent-green（根里没有 prune-server/，无对象退 77）；
#   prune-server-selftest-failed-red（末行 cells=20 passed=19 failed=1 failed_cells=batched-point-read、退 1：验收要的 failed=1 那一份）；
#   prune-server-selftest-empty-red（末行 cells=0 passed=0 failed=0、退 0：一格都没跑）；
#   prune-server-selftest-no-summary-red（只打各格的行、没有末行、退 0）；prune-server-selftest-exit-red（末行 failed=0，却退 1）。
# 弄坏开关 CODE_TOOLING_BREAK=prune-server-failed-ignored：不看末行的 failed= 与退出码，failed-red、exit-red 判绿；
#   CODE_TOOLING_BREAK=prune-server-empty-run-accepted：N 为 0 也算过，empty-red 判绿；
#   CODE_TOOLING_BREAK=prune-server-last-line-unchecked：末行缺或不在最后不判（有末行照旧核，没有就只看退出码），no-summary-red 判绿；
#   CODE_TOOLING_BREAK=prune-server-exit-unchecked：不核退出码与 failed= 对不对得上，exit-red 判绿。
cell_prune_server_selftest() {
local scratch program=() run_exit kept_build_log
[[ -f prune-server/Cargo.toml ]] || { echo "  ! 被判的根里没有 prune-server/Cargo.toml（判定库服务），这一格无对象可判"; exit 77; }
scratch="$(mktemp -d "${TMPDIR:-/tmp}/code-tooling-prune-server-selftest.XXXXXX")" || { echo "  ✗ 建不了这一格的临时目录"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
# shellcheck disable=SC2064  # 路径在这里就展开（%q 过）
trap "rm -rf -- $(printf '%q' "$scratch")" EXIT   # 这一格的编译目录、自证的库与 socket 与输出，收尾删
mkdir "$scratch/tmp" || { echo "  ✗ 建不了 $scratch/tmp"; echo "     → 怎么办：看 \${TMPDIR:-/tmp} 满没满、有没有写权限，修好再跑。"; exit 1; }
if judged_root_is_sample && [[ -f .prune-server-selftest-program ]]; then
  program=(python3 "$ROOT/.prune-server-selftest-program")
else
  if ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT" >"$scratch/rocksdb-entry.txt" 2>&1; then
    sed 's/^/     /' "$scratch/rocksdb-entry.txt" | tail -n 8   # gate-lint:detail
    echo "  ✗ 编 prune-server 之前 RocksDB 预编库的取值没判过（上面是 rocksdb-prebuilt.py entry 的输出尾部），这一格没编、没跑自证"
    echo "     → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑这一格。"
    exit 1
  fi
  if ! CARGO_TARGET_DIR="$scratch/target" \
      cargo build --release --offline --manifest-path "$ROOT/prune-server/Cargo.toml" --bin prune-server >"$scratch/build.log" 2>&1; then
    kept_build_log="$(mktemp "${TMPDIR:-/tmp}/code-tooling-prune-server-selftest-build.XXXXXX.log")" && cp -- "$scratch/build.log" "$kept_build_log" || kept_build_log="（没留下：拷不出来）"
    echo "  ✗ 编不出 prune-server（cargo build --release --manifest-path prune-server/Cargo.toml --bin prune-server，日志留在 $kept_build_log）：$(tail -n 3 "$scratch/build.log" | tr '\n' ' ' | cut -c1-300)"
    echo "     → 怎么办：看那份日志，看完自己删；编译错照报错修 prune-server/（归工具实现员），别的会话正改着的就等它改完再跑这一格。"
    exit 1
  fi
  program=("$scratch/target/release/prune-server")
fi
if (cd "$ROOT/prune-server" 2>/dev/null || cd "$ROOT"; TMPDIR="$scratch/tmp" "${program[@]}" --selftest) >"$scratch/selftest.txt" 2>"$scratch/selftest.err"; then
  run_exit=0
else
  run_exit=$?
fi
python3 - "$scratch/selftest.txt" "$scratch/selftest.err" "$run_exit" "${CODE_TOOLING_BREAK:-}" <<'PY'
import sys
output_file, error_file, run_exit, broken = sys.argv[1:5]
PREFIX = "PRUNE_SERVER_SELFTEST"
lines = [line for line in open(output_file, encoding="utf-8", errors="replace").read().split("\n") if line.strip()]
summaries = [line for line in lines if line.startswith(PREFIX + " ")]
red_lines = [line.strip() for line in lines if line.lstrip().startswith("✗")]
faults = []
fields = {}
if (len(summaries) != 1 or lines[-1] != summaries[0]) and broken != "prune-server-last-line-unchecked":
    faults.append(f"末行不是恰好一行 {PREFIX}（实有 {len(summaries)} 行，最后一行是「{lines[-1].strip()[:120] if lines else '（没有输出）'}」）")
if summaries:
    fields = dict(part.split("=", 1) for part in summaries[-1].split()[1:] if "=" in part)
cells, passed, failed = (int(fields[name]) if fields.get(name, "").isdigit() else None for name in ("cells", "passed", "failed"))
failed_cells = [] if fields.get("failed_cells", "-") == "-" else fields["failed_cells"].split(",")
if summaries and (None in (cells, passed, failed) or "failed_cells" not in fields):
    faults.append(f"末行缺 cells=、passed=、failed=、failed_cells= 之一，或前三个不是整数：{summaries[-1][:200]}")
elif summaries:
    if broken == "prune-server-failed-ignored":
        failed, failed_cells, run_exit = 0, [], "0"
        passed = cells
    if cells == 0 and broken != "prune-server-empty-run-accepted":
        faults.append("末行 cells=0：一格都没跑")
    if passed + failed != cells:
        faults.append(f"末行 passed={passed} 加 failed={failed} 不等于 cells={cells}")
    if failed != len(failed_cells):
        faults.append(f"末行 failed={failed}，failed_cells 却列了 {len(failed_cells)} 格")
if broken != "prune-server-exit-unchecked":
    if run_exit not in ("0", "1"):
        tail = " | ".join(open(error_file, encoding="utf-8", errors="replace").read().strip().split("\n")[-3:])
        faults.append(f"prune-server --selftest 退 {run_exit}（格名或开关写错、起不来或没跑成：{tail[:300]}）")
    elif summaries and failed is not None:
        if run_exit == "0" and failed:
            faults.append(f"退 0，末行却报 failed={failed}")
        elif run_exit == "1" and not failed:
            faults.append("退 1，末行却报 failed=0")
if failed_cells or faults or (failed and failed > 0):
    for cell in failed_cells:
        print(f"  ✗ prune-server 自证的格 {cell} 红了")  # gate-lint:detail
    for line in red_lines[:20]:
        print(f"     {line[:300]}")  # gate-lint:detail
    for fault in faults:
        print(f"  ✗ prune-server --selftest 的输出：{fault}")  # gate-lint:detail
    print(f"  ✗ prune-server 自证没过：{cells if cells is not None else '?'} 格里红 {failed if failed is not None else '?'} 格（{'、'.join(failed_cells) or '无'}）、{len(faults)} 处输出不对（退 {run_exit}）")  # gate-lint:summary
    print("     → 怎么办：单跑看细节：task-server/task submit --cwd <仓根>/prune-server -- cargo run --release --offline --bin prune-server -- --selftest；"
          "只跑红的格：同一条命令改成 task-server/task submit --cwd <仓根>/prune-server -- env PRUNE_SERVER_SELFTEST_CELLS=<格名> cargo run --release --offline --bin prune-server -- --selftest；"
          "格的写法与弄坏开关在 prune-server/src/bin/prune_server.rs 文件头，判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ prune-server 自证：{cells} 格都过（{summaries[-1] if summaries else '无末行'}，退 {run_exit}）")
PY
}

# ── 逐项点名（.claude/rules/verification.md「门禁的结构」的 --list-items 与 --item）：三十七格里只有三格分项，别的格只能整格跑（--check）
#   research-script-selftests   <脚本路径>：runner 表里起这份脚本的那几条自证（按包装 -- 后面那条原命令的第二个词认）
#   stage-break-switch-replay   <阶段文件名>，或 <阶段文件名>:<开关名>=<值>：有样本目录的那几道阶段，与它文件头开关表里的各条
#   break-switch-replay         <脚本路径>，或 <脚本路径>:<开关名>=<值>：复跑登记表里在的那几份，与它文件头开关表里的各条
#                               （点一条开关也照整份起它的基准趟与别的开关趟：汇总行要整份的开关趟；别的开关趟只陪跑，判红照报）
#   --list-items：逐行打「<格名><TAB><项>」（先整份、再它的各条），只读 runner 表、复跑登记表与文件头，不起自证、不起 stage-selftest.sh、不算键、不连 prune-server；
#     给了 --check 只列点名的格里分项的那几格；点名的格里一格分项的都没有：列出分项的三格、退 2。
#   --item <项>：只跑点名的项（给几次取并集）；三格里哪一格认得它就进哪一格（同一个脚本路径两格都认的两格都跑），一项都没点到的格不跑；给了 --check 只认点名的格。
#     点名单跑一项与它在整格里跑的判定相同：同一套键、同一份判定库、同一份判法；复用、现判、记库照整格的走法。项写错（这几格都不认得）：列出可点的项、退 2，一格都不跑。
#   样本根（judged_root_is_sample）里的 .gate-items（一行一项，空行与 # 开头的行不算）当作命令行上的 --item；命令行给了 --item 就不认它，真仓与 --staged 下不认。
#   样本：runner-item-picked、break-switch-replay-item-co-run-green、stage-break-switch-replay-item-green（点名单跑一项，判绿、报只跑了那一项）；
#     runner-item-unknown、break-switch-replay-item-unknown、stage-break-switch-replay-item-unknown（项写错，退 2、点名写错的项、列出可点的项）。
#   弄坏开关 CODE_TOOLING_BREAK=items-ignored：点名不起作用，点到的格照跑整格（三份点名样本判错）；
#     =item-unknown-accepted：写错的项悄悄丢掉、不退 2，一项都没认出就照跑整格（三份写错样本判错）。
CODE_TOOLING_ITEM_CELLS=(research-script-selftests stage-break-switch-replay break-switch-replay)
declare -A CODE_TOOLING_PICKED=()

# code_tooling_item_cells_in_selection：分项的三格里落在这一趟选格里的那几格，按登记次序一行一格；样本根的 .gate-cells 在共用库里才认，这里照同一条判据先认一次
code_tooling_item_cells_in_selection() {
  local name line
  local -A chosen=()
  if ((STAGE_CELLS_CHECK_GIVEN == 0)) && judged_root_is_sample && [[ -f .gate-cells ]]; then
    while IFS= read -r line || [[ -n "$line" ]]; do
      line="${line%%#*}"; line="${line//[[:space:]]/}"
      [[ -n "$line" ]] && chosen[$line]=1
    done < .gate-cells
  else
    for name in "${STAGE_CELLS_SELECTED[@]}"; do chosen[$name]=1; done
  fi
  for name in "${STAGE_CELLS_NAMES[@]}"; do
    [[ -n "${chosen[$name]+set}" ]] || continue
    case " ${CODE_TOOLING_ITEM_CELLS[*]} " in *" $name "*) printf '%s\n' "$name" ;; esac
  done
}

# code_tooling_items_of <格名>：打这一格可点的项，一行一项（只读表与文件头，格的判法不起）
code_tooling_items_of() {
  local runner
  local -a words=()
  local -A seen=()
  case "$1" in
    research-script-selftests)
      selftest_runner_table_load
      for runner in "${runners[@]}"; do
        read -r -a words <<<"$(selftest_inner_command "$runner")"
        [[ -n "${words[1]:-}" && -z "${seen[${words[1]:-}]+set}" ]] || continue
        seen[${words[1]}]=1
        printf '%s\n' "${words[1]}"
      done
      ;;
    stage-break-switch-replay|break-switch-replay)
      python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$1" <<'PY'
import glob, os, sys, types
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
owners = []
if sys.argv[2] == "stage-break-switch-replay":
    owners = [(name, f".claude/gate.d/{name}") for name in sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh"))
              if os.path.isdir(f".claude/gate.d/fixtures/{name}")]
elif os.path.isfile(".claude/gate.d/break-switch-replay-files.tsv"):
    for line in open(".claude/gate.d/break-switch-replay-files.tsv", encoding="utf-8"):
        path = line.rstrip("\n").split("\t")[0]
        if line.strip() and not line.startswith("#") and os.path.isfile(path) and path not in dict(owners):
            owners.append((path, path))
for owner, path in owners:
    printed = [owner]
    for entry in table.switch_entries(open(path, encoding="utf-8", errors="replace").read()):
        item = f"{owner}:{entry['name']}={entry['value']}"
        if item not in printed:
            printed.append(item)
    print("\n".join(printed))
PY
      ;;
  esac
}

# code_tooling_resolve_items：--list-items 打完退 0；--item（或样本根的 .gate-items）认过之后把各格点到的项放进 CODE_TOOLING_PICKED[<格名>]（一行一项），
# 选格换成点到项的那几格；项写错列出可点的项、退 2（这一道「逐项点名」一节）
code_tooling_resolve_items() {
  local cell item line hit source_description="命令行的 --item" summary=""
  local -a cells=() requested=() unknown=() listing=()
  local -A listed=() matched=() picked_count=()
  if ((${#CODE_TOOLING_ITEMS[@]} == 0 && code_tooling_list_items_requested == 0)) && judged_root_is_sample && [[ -f .gate-items ]]; then
    while IFS= read -r line || [[ -n "$line" ]]; do
      [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
      requested+=("$line")
    done < .gate-items
    source_description="样本根的 .gate-items"
  else
    requested=(${CODE_TOOLING_ITEMS[@]+"${CODE_TOOLING_ITEMS[@]}"})
  fi
  ((code_tooling_list_items_requested || ${#requested[@]})) || return 0
  mapfile -t cells < <(code_tooling_item_cells_in_selection)
  if ((${#cells[@]} == 0)); then
    echo "  ✗ 这一趟选的格里没有分项的格（分项的只有 ${CODE_TOOLING_ITEM_CELLS[*]}），--list-items 与 --item 无从认"
    echo "     → 怎么办：--list-items、--item 只配这三格（--check 点它们或不给 --check）；别的格照 --check <格名> 整格跑。"
    exit 2
  fi
  for cell in "${cells[@]}"; do
    while IFS= read -r item; do
      [[ -n "$item" ]] || continue
      listed["$cell"$'\t'"$item"]=1
      listing+=("$cell"$'\t'"$item")
    done < <(code_tooling_items_of "$cell")
  done
  if ((code_tooling_list_items_requested)); then
    printf '%s\n' ${listing[@]+"${listing[@]}"}
    exit 0
  fi
  for item in "${requested[@]}"; do
    hit=0
    for cell in "${cells[@]}"; do
      if [[ -n "${listed["$cell"$'\t'"$item"]+set}" ]]; then
        CODE_TOOLING_PICKED[$cell]+="$item"$'\n'
        matched[$cell]=1
        picked_count[$cell]=$((${picked_count[$cell]:-0} + 1))
        hit=1
      fi
    done
    ((hit)) || unknown+=("$item")
  done
  if ((${#unknown[@]})) && [[ "${CODE_TOOLING_BREAK:-}" != item-unknown-accepted ]]; then
    echo "  ✗ $source_description 点名的项不在可点的项里：$(printf '「%s」' "${unknown[@]}")；一格都没跑"
    echo "     → 怎么办：照下面列的（bash .claude/gate.d/code-tooling.sh --list-items 打的同一份：格名、项）原样写，几项给几次 --item："
    for line in ${listing[@]+"${listing[@]}"}; do printf '       %s  %s\n' "${line%%$'\t'*}" "${line#*$'\t'}"; done   # gate-lint:detail
    exit 2
  fi
  ((${#matched[@]})) || return 0
  if [[ "${CODE_TOOLING_BREAK:-}" == items-ignored ]]; then
    CODE_TOOLING_PICKED=()
    return 0
  fi
  STAGE_CELLS_SELECTED=()
  for cell in "${STAGE_CELLS_NAMES[@]}"; do
    if [[ -n "${matched[$cell]+set}" ]]; then
      STAGE_CELLS_SELECTED+=("$cell")
      summary+="${summary:+、}$cell ${picked_count[$cell]} 项"
    fi
  done
  STAGE_CELLS_SELECTION_SOURCE="$source_description 点名的项"
  echo "  · 只跑 $source_description 点名的项：$summary"
}

# ── 样本根只在临时 prune-server 底下判（判的根是样本，judged_root_is_sample）：
#   不在临时服务底下，判定库的键没有命名空间（scripts/prune-server/prune_server_client.py 的 namespace_of），样本各格按条复用写读的是真服务里与真门禁同一个种类的键：
#   前一份样本或前一趟记下的绿被后一份样本复用，计数句对不上，样本的判定还留在真库里。所以开跑先调客户端的 selftest_service_problem（只看环境，不连服务），
#   不在临时服务底下退 2、一格都不判，拒绝说明给整条写法。真仓与 --staged 的临时树不判这一条（它们本来就写真服务）；--list、--list-items 与写错的项在这之前已经退了。
#   样本根放 .sample-environment-bare 的，这一判按「环境里什么都没设」答（样本在临时服务底下被喂时造出裸跑的样子）。
#   样本：sample-root-bare-red（.gate-cells 点名 stage-order，放 .sample-environment-bare；期望退 2、点名没有临时 prune-server）。
#   弄坏开关 CODE_TOOLING_BREAK=sample-service-unchecked：不核（改前的样子），那份样本照判 stage-order、退 0，退 2 与点名的两条 want 找不到。
code_tooling_sample_service_check() {
  judged_root_is_sample || return 0
  [[ "${CODE_TOOLING_BREAK:-}" == sample-service-unchecked ]] && return 0
  local service_problem
  if ! service_problem="$(python3 -c 'import os, sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem({} if os.path.isfile(sys.argv[2]) else None))' \
        "$STAGE_REPOSITORY/scripts/prune-server" "$ROOT/.sample-environment-bare" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ 判的根是样本（$ROOT），样本的判定不在临时 prune-server 底下就会写进真服务、与真门禁同一个种类：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}；一格都没判"
    echo "     → 怎么办：样本档整条套临时服务起（交任务服务时前面加 task-server/task submit --）：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item code-tooling.sh"
    echo "       单跑某一份样本根时，把 bash .claude/gate.d/code-tooling.sh <样本根> --check <格名> 接在同一个包装的 -- 后面。"
    exit 2
  fi
}

code_tooling_resolve_items
code_tooling_sample_service_check
stage_cells_run "$ROOT"
