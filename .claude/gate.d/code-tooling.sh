#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录、跑研究脚本与共用库自己的自证、起各道门禁的 --list；research-gate-lint 格调的上游 gate-lint.sh、shell-lint.sh 要 gawk，缺了由它们自己拒绝（退 78），那一格按红记，不交给 gate.sh 预判；function-units-two-implementations-agree 格编 unit-graph 要的 cargo 缺了那一格自己判红
# gate-stage: 工具层自检（四十五格：研究脚本的自证都过、有阶段在跑；带判定库的脚本自证里复用三问都过；项目规则清单（@ 引用加 read-once 登记）；阶段归属表按格登记；钩子注册与写范围表与定义头；定义的形状（model 与 tools 写明、tools 不给 Agent、四节齐、不抄规则的长行、调度表与定义两向相等）；改动范围只许一份取法；研究脚本与钩子的拒绝出路与 shell 纪律；阶段声称的样本与编造的日期；带编号登记位的样本都排在 doc-lint 之外；kb 登记表；门禁结构；环境检查排在跑测试的阶段之后；目录形态；crates/ 下各包登记了角色、按角色放该放的东西；提交里没有超过单文件上限的文件；阶段在 cd 之前取好自己的位置；并行度默认值只经每台默认重活线程数的登记位取；跑法不写死内存上限与台数；函数单元两份实现对拍；同一个 Rust 编译目标里不同模块的条目不撞名；变异门禁的 --list-items 在被判的仓上跑得通；门禁阶段的弄坏开关只在样本根上认；入口脚本带列项、自证与每项一行；弄坏开关文件头三样写全；门禁阶段的弄坏开关逐条复跑、判错的样本与头注点名的相等；研究脚本与钩子平常跑时设了弄坏开关要被拒；登记的脚本与钩子的弄坏开关逐条复跑、点名的项都在失败行里；kb 目录登记位里的每个目录连同子目录都在写回员的写范围里；脚本不经 ~ 与 $HOME 取数据根下共享的那几项；CLAUDE.md 登记的规则都被定义点名；runner 表里套临时服务包装的自证与要临时服务的表里登记成 required 的两边对齐；要临时服务的自证的整条写法在登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、门禁的包装前缀常量这七处逐字相同、包装的解析收得下、不带尖括号占位的 --server-binary；建仓副本的写法都经共用库算闭包、不手抄拷贝清单；登记的自证成功句报并行度或文件头写明不能并行；跑过两小时也不拆的豁免表每一行写对：理由、出处、不重复、项名不是目录与整道门禁）
# gate-fixture: yes
# gate-category: 代码类
# 不写 gate-covers：.claude/gate-not-implemented.tsv 里的两项判的是被测代码的行为，这一道只判工具层自己。
# gate-similar: research-build-and-replay.sh 它编 research/ 的装置、跑它们的单测，不跑研究脚本的自证（stage-mine.py、relabel-item.py、claim-experiment.sh 这几份的自证不要 research 工作区先编好，同别的研究脚本一样在这里的 runner 表里跑）；这一道的 research-script-selftests 格跑 runner 表里研究脚本的自证、research-script-selftest-coverage 格核每份实现了 --selftest 的都有阶段在跑，不编译
# gate-similar: model-differential-and-scenarios.sh 它在真仓上判 harness 用例一条一个场景，crash-case-check.py 的判别力由这一道 research-script-selftests 格跑它的 --selftest；判的对象一个是用例源码，一个是脚本的自证
# gate-similar: doc-text.sh 也扫全仓文件，但判的是文本里登记过的禁用写法；这一道 fixture-claims 格判文件名与样本目录里编出来的日期、阶段头部声称的样本在不在
# gate-similar: code-source-discipline.sh 它判实验二进制的变异表、绝对值断言与读数纪律，对象是实验源码；crates-mutation-rows.py 与 check-segment-registry.py 这类脚本的自证由这一道 research-script-selftests 格替它们跑
# gate-similar: doc-process-records.sh 它是改动范围共用取法的使用者之一，判三方与同步留下的记录；这一道 change-range-single-source 格判每个阶段只经那一份取法、列路径带 quotepath，对象是门禁阶段的源码
# gate-similar: doc-experiments.sh 它也是改动范围共用取法的使用者，判实验页与产物；这一道只判工具层自己（脚本、钩子、登记表、定义头、门禁结构与次序），不读实验页
# gate-similar: doc-registries.sh 它判欠账表与里程碑收口表这类 kb 登记表；这一道判工具层自己的登记（阶段归属表、写范围表、kb 登记表对目录）与自证
# gate-similar: test-environment.sh 它判这台机器此刻有没有残留设备、挂载、盘满；这一道 stage-order 格只判它的文件名排在跑测试的阶段之后，不读机器，放进它里面就成了它判自己
# gate-similar: hooks-registered.sh 上游这一道判 settings.json 里注册的钩子在、hook-events 挂全、带 --selftest 的自检通过；这一道 agent-write-scope 格判的是项目点名的那几个钩子挂在要的 matcher 上、写范围表与定义双向一致、定义头，钩子的自证只归它跑
# gate-similar: manifest.sh 上游这一道判规范包自己的 CLAUDE.md 与 rules/ 对得上；这一道 rules-manifest 格判项目本地 .claude/rules/ 与项目 CLAUDE.md 的 @ 引用，射程不重叠
# gate-similar: doc-lint.sh 上游这一道的 J 段判排除表自己的形状（理由、目录在、下面有 .md）；这一道 fixture-registrations-excluded 格判样本目录里有登记位的阶段都在排除表里、排除表没有指向 fixtures 下不存在的阶段，对象是样本与表的对账，不并进 doc-lint
# gate-similar: doc-process-records.sh 它的 crates-adversarial-review、agent-def-adversarial-review 两格判按内容哈希登记的豁免表（.claude/crates-review-exempt、.claude/agent-def-review-exempt）；这一道 long-run-split-exempt 格判按项名登记、理由带 records 出处的 .claude/long-run-split-exempt，键与对象都不同，没有可并进它们的判据；本道里别的排除表格（heavy-thread-defaults、hardcoded-quota、shared-data-root-literals、repository-copy-closure、selftest-parallelism）各绑着自己的扫描判法，这张表没有扫描对象，不并进它们
# gate-similar: stage-selftest.sh 上游这一道拿 fixtures 下的红绿样本证明本地阶段会红；这一道 fixture-claims 格只判阶段头部的声称与样本目录对不对得上，gate-structure 格只判结构（格名表、--list、--check），都判不了样本本身有没有判别力
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就四十五格按下面的次序全跑，格名写错退 2）：
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
# gate-cell: crate-roles crates/ 下每个包在登记表里恰有一行角色，实验装置之外的二进制目标都登了放行表或滞后表、空库带测试的只许测试支撑、测试支撑至少两个无条件的依赖者、产品包不经 [dependencies]、[build-dependencies] 依赖测试支撑包、没有包依赖实验装置包
# gate-cell: committed-file-size 被提交的东西里没有超过 100 MiB 的文件：HEAD 加暂存区里跟踪的每个文件，与还没推上去的每笔提交里新增或改过的 blob
# gate-cell: git-hooks-installed 进仓的 git 钩子装上了：core.hooksPath 是主工作树钩子目录的绝对路径、pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 都在、有执行位、进仓的模式是 100755（提交、合并、git am 与推送前的大文件判法靠它起作用）
# gate-cell: stage-location-before-cd 门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname "$0" 找旁边的文件
# gate-cell: heavy-thread-defaults 脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨
# gate-cell: hardcoded-quota 脚本、门禁与钩子里不定内存上限与台数（上限、机器由 task-server 按记录定），排除表指得到、只缩不涨
# gate-cell: memory-bounded-parallelism 按内存上限定开几个进程或线程的入口都登记在 .claude/memory-bounded-parallelism.tsv，表每行两列（入口、理由）成形、入口在，登记的入口代码里还有这类写法
# gate-cell: function-units-two-implementations-agree 函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）原样与产品行为视图两趟：dump 逐单元相同，登记入口的闭包与键逐个相同
# gate-cell: rust-name-collisions 同一个 Rust 编译目标里不同模块各自定义的同名条目（fn、struct、enum、union、const、static、type、trait，私有的与只在测试里的也算）判红，逐处列出文件:行
# gate-cell: mutation-list-items 变异门禁的 --list-items 在被判的仓上跑通：退 0、报出键对不上的条数，没有「没算出来」、回溯与 PicklingError
# gate-cell: break-switches-sample-only 门禁阶段读弄坏开关都经共用库那一判（开关只在样本根上认），共用库的 stage_cells_run 开头调它
# gate-cell: entry-scripts 登记表 entry-scripts.tsv 里的每份入口脚本都带 --list-items、--selftest 与带齐字段的每项一行（或末行总计）
# gate-cell: break-switch-headers 研究脚本与钩子读了弄坏开关的，文件头开关表三样写全（名字与值、弄坏哪一处、点名的项），存量豁免表指得到、只缩不涨
# gate-cell: stage-break-switch-replay 门禁阶段文件头的每条弄坏开关逐条设上、经 stage-selftest.sh 喂样本，判错的样本与点名的相等
# gate-cell: break-switch-outside-selftest 研究脚本与钩子读了弄坏开关的，平常跑时设上要被拒并点名只在自证里认，存量待改表指得到、只缩不涨
# gate-cell: break-switch-replay 登记表里的脚本与钩子，文件头每条弄坏开关逐条设上跑它的 --selftest，点名的格与实际红的格逐格相同
# gate-cell: kb-directories-write-scope .claude/kb-directories 登记的每个 kb 目录，下一层与子目录里的 .md 都在写回员（kb-scribe）的写范围里
# gate-cell: prune-server-selftest 判定库服务 prune-server/ 的自证过：编服务二进制、起 --selftest，末行 failed=0、cells 大于 0、退出码对得上（目录不在退 77）；判的是判定库服务自己，不入库
# gate-cell: shared-data-root-literals 脚本、钩子、门禁与服务不经 ~ 与 $HOME 取数据根下共享的那几项（经 lib_work_root.py 的 shared_data_root() 取），排除表指得到、只缩不涨
# gate-cell: cp-preserve-mode 项目脚本、钩子、门禁、样本根的搭建脚本与服务里的 cp 不带保留权限的选项（-a、-p、--archive、不带值或带 mode / all 的 --preserve）
# gate-cell: peer-launch-direct 项目脚本与门禁不自己起 multi-host-run.sh --peer 往第二台起活（照服务的分片协议，派哪台由服务定）
# gate-cell: verdict-library-code 登记表 verdict-library-scripts.tsv 写了判定库的行第三列都是 prune-server，点名的脚本里没有取不到库照跑的分支、prune-server 行没有本地库读写
# gate-cell: temporary-service-runners runner 表里套了临时服务包装的自证，与要临时服务的表 selftest-temporary-service.tsv 里登记成 required 的两边对齐（裸写、缺条、包装了表里没有的都点名；own-server 不要求包装）
# gate-cell: temporary-service-wrapper-form 要临时服务的自证的整条写法在七处逐字相同（登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、包装前缀常量）、包装的解析收得下、整条里没有带尖括号占位的 --server-binary
# gate-cell: repository-copy-closure scripts/、.claude/gate.d/、.claude/hooks/ 里建仓副本的写法都经 scripts/hosts/lib_repository_copy.py 算闭包（拷源点名仓根条目的 cp、rsync、git archive、shutil.copy 一族与手抄的拷贝清单字面量判红），排除表一行一处、带理由、只缩不涨
# gate-cell: selftest-cells-alone 有选格手段的自证（写着「<名字>_SELFTEST_CELLS=<格名…」的）逐格单跑与整份跑判定相同：单跑红而整份里绿、单跑绿而整份里红的格点名判红
# gate-cell: selftest-parallelism 登记表 selftest-registry.tsv 里登记成 selftest 的每份脚本，成功句报并行度或文件头写明不能并行（# selftest-serial:），排除表指得到、只缩不涨
# gate-cell: long-run-split-exempt 跑过两小时也不拆的豁免表 .claude/long-run-split-exempt 的每一行写对：有 #、理由至少 8 个字、理由里有存在的 records/<文件>.md:<行>（行号不超过文件行数）、项名不重复、不是目录也不是门禁文件名
#
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码经共用库 lib/stage-cells.sh（写法在它的文件头），这里只登记格、解析项目根。
#
# 格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的九份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析、逐项点名与汇总。格报错出路里写的「code-tooling.sh 里这一格函数上方的注释」，就在下面对应的那份库里：
# 库 lib/code-tooling-shared-libraries.sh：（共用的判法库变量与取表、记库的辅助）
# 库 lib/code-tooling-selftest-cells.sh：research-script-selftests、research-script-selftest-coverage、temporary-service-runners
# 库 lib/code-tooling-selftest-cells.sh：temporary-service-wrapper-form
# 库 lib/code-tooling-rules-and-agents-cells.sh：rules-manifest、rules-named-by-definitions、stage-owners、agent-write-scope、agent-definition-shape
# 库 lib/code-tooling-rules-and-agents-cells.sh：change-range-single-source、research-gate-lint
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：fixture-claims、fixture-registrations-excluded、kb-registry、gate-structure、stage-order
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：directory-layout、crate-roles、committed-file-size、git-hooks-installed、stage-location-before-cd
# 库 lib/code-tooling-source-scan-cells.sh：heavy-thread-defaults、hardcoded-quota、memory-bounded-parallelism、shared-data-root-literals、cp-preserve-mode
# 库 lib/code-tooling-source-scan-cells.sh：peer-launch-direct、verdict-library-code
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：reuse-selftests、function-units-two-implementations-agree、rust-name-collisions
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：mutation-list-items、break-switches-sample-only、entry-scripts、break-switch-headers
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：stage-break-switch-replay、break-switch-outside-selftest
# 库 lib/code-tooling-break-switch-replay-cells.sh：break-switch-replay、kb-directories-write-scope、prune-server-selftest
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：repository-copy-closure、selftest-cells-alone、selftest-parallelism、long-run-split-exempt
#
# 判别力：fixtures/code-tooling.sh/ 下每一格至少一红一绿，每份样本根放 .gate-cells 只点名它判的那一格：
#   research-script-selftests          runner-red、runner-green、runner-guard-red、runner-guard-green、runner-unit-key、runner-plan-exit-red、runner-plan-sentinel-red、runner-red-over-green、
#                                      runner-red-record-failed、runner-not-run-green、runner-all-not-run、runner-recorded-before-the-run-ends、
#                                      runner-bare-temporary-service-red、runner-plan-unavailable、runner-item-picked、runner-item-unknown、
#                                      runner-judge-entries-unreadable-red（样本根的 .selftest-judge-guard.py 顶替守卫，里面没有判法名单那一句）
#                                      runner-memory-bound-batches-green、runner-memory-bound-unlisted-red（样本根的 .selftest-memory-bound 顶替登记的大自证）
#                                      runner-record-skips-forwarded-green（收尾记库的「!」行转进日志）
#   research-script-selftest-coverage  red、green、coverage-claims-red、coverage-claims-green、coverage-nested-red、coverage-suffix-red
#   reuse-selftests                    reuse-selftests-red、reuse-selftests-green、reuse-selftests-reuse-three-questions、reuse-selftests-red-over-green、
#                                      reuse-selftests-red-record-failed（按条复用，setup.sh 都 git init）
#   rules-manifest                     rules-manifest-red、rules-manifest-green、rules-manifest-read-once-red、rules-manifest-read-once-green
#   rules-named-by-definitions         rules-named-by-definitions-red、rules-named-by-definitions-green、rules-named-by-definitions-main-only-green、
#                                      rules-named-by-definitions-main-only-unregistered-red、rules-named-by-definitions-main-only-named-red、
#                                      rules-named-by-definitions-main-only-reason-red（样本根自己的 .claude/gate.d/rules-main-agent-only.tsv）
#   stage-owners                       stage-owners-red、stage-owners-green
#   agent-write-scope                  agent-write-scope-red、agent-write-scope-multiedit-red、agent-write-scope-timer-red、agent-write-scope-run-scope-red、
#                                      agent-write-scope-find-timestamp-red、agent-write-scope-readers-red、agent-write-scope-placement-red、
#                                      agent-write-scope-session-number-red、agent-write-scope-parts-red、agent-write-scope-name-like-red、agent-write-scope-comment-named-red、
#                                      agent-write-scope-loaded-shared-red、agent-write-scope-green、agent-write-scope-import-module-green
#   agent-definition-shape             agent-definition-shape-model-red、agent-definition-shape-tools-red、agent-definition-shape-agent-tool-red、agent-definition-shape-sections-red、
#                                      agent-definition-shape-rule-copy-red、agent-definition-shape-table-red、agent-definition-shape-green
#   change-range-single-source         change-range-single-source-red、change-range-single-source-green、change-range-single-source-path-moves-red、change-range-single-source-path-moves-green、
#                                      change-range-single-source-path-moves-evidence-red、change-range-single-source-path-moves-evidence-green、
#                                      change-range-single-source-path-moves-committed-green、change-range-single-source-path-moves-base-row-green、
#                                      change-range-single-source-prose-green、change-range-single-source-split-gate-red、change-range-single-source-split-gate-green
#   research-gate-lint                 research-gate-lint-red、research-gate-lint-green、research-gate-lint-arithmetic-shift-red、research-gate-lint-pending-outside-green、research-gate-lint-pending-stale-red
#   fixture-claims                     fixture-claims-red、fixture-claims-unnumbered-red、fixture-claims-cells-red、fixture-claims-unmarked-red、
#                                      fixture-claims-marker-malformed-red、fixture-claims-none-with-directory-red、fixture-claims-exempt-red、fixture-claims-green
#   fixture-registrations-excluded     fixture-registrations-excluded-red、fixture-registrations-excluded-stale-red、fixture-registrations-excluded-green
#   kb-registry                        kb-registry-red、kb-registry-green
#   gate-structure                     gate-structure-red、gate-structure-green、preflight-guarded-red、preflight-guarded-green
#   stage-order                        stage-order-red、stage-order-green
#   directory-layout                   directory-layout-red、directory-layout-green
#   crate-roles                        crate-roles-green、crate-roles-empty-library-tests-red、crate-roles-product-experiment-binary-red、
#                                      crate-roles-product-depends-on-experiment-red、crate-roles-lag-grown-red（setup.sh 在样本根 git init、提交一版少一行的滞后表）
#   committed-file-size                committed-file-size-red、committed-file-size-unpushed-red、committed-file-size-detached-red、committed-file-size-green、committed-file-size-no-upstream-green
#   git-hooks-installed                git-hooks-installed-green、git-hooks-installed-unset-red、git-hooks-installed-wrong-path-red、git-hooks-installed-relative-path-red、
#                                      git-hooks-installed-hook-missing-red、git-hooks-installed-hook-no-exec-red、git-hooks-installed-tracked-mode-red
#                                      （setup.sh 在样本根里建小 git 仓与五个桩钩子，格里起 scripts/gate/git-hooks-install.py check，只起 git）
#   stage-location-before-cd           stage-location-before-cd-red、stage-location-before-cd-green
#   heavy-thread-defaults              heavy-thread-defaults-red、heavy-thread-defaults-cfg-test-red、heavy-thread-defaults-cfg-test-module-file-red、heavy-thread-defaults-capped-elsewhere-red、heavy-thread-defaults-quoted-name-red、heavy-thread-defaults-per-line-red、heavy-thread-defaults-wide-cost-red、heavy-thread-defaults-var-source-red、heavy-thread-defaults-var-source-forms-red、heavy-thread-defaults-per-line-green、heavy-thread-defaults-var-source-green、heavy-thread-defaults-green、heavy-thread-defaults-moved-red、heavy-thread-defaults-moved-green、heavy-thread-defaults-exempt-family-red、heavy-thread-defaults-exempt-family-green、heavy-thread-defaults-sample-inputs-red、heavy-thread-defaults-sample-inputs-green、heavy-thread-defaults-split-locals-red、heavy-thread-defaults-split-locals-green、heavy-thread-defaults-shell-selftest-samples-red、heavy-thread-defaults-shell-selftest-samples-green、heavy-thread-defaults-registry-wrapper-red、heavy-thread-defaults-registry-wrapper-green、heavy-thread-defaults-registered-split-red、heavy-thread-defaults-registered-split-green、heavy-thread-defaults-exempt-hits-grown-red、heavy-thread-defaults-exempt-hits-split-green
#   hardcoded-quota                    hardcoded-quota-red、hardcoded-quota-green、hardcoded-quota-exempt-family-red、hardcoded-quota-exempt-family-green（后两份 setup.sh 都 git init）
#   memory-bounded-parallelism         memory-bounded-parallelism-red、memory-bounded-parallelism-green
#   function-units-two-implementations-agree  function-units-two-implementations-agree-green、-dump-red、-closure-red、-key-red、-product-view-red、-data-directory-green、-doctest-green、-cfg-one-side-red、
#                                      -reuse-three-questions、-red-over-green（Rust 那一侧是录好的输出，不编、不起 unit-graph；setup.sh 都 git init）
#   rust-name-collisions               rust-name-collisions-red、rust-name-collisions-green（样本根是一个小工作区，判法只读 .rs 与 Cargo.toml，不编）
#   mutation-list-items                mutation-list-items-red、mutation-list-items-green、mutation-list-items-reuse-three-questions、mutation-list-items-red-over-green、
#                                      -build-environment-reuse-three-questions（setup.sh 现写小工作区、git init，--list-items 只起 git）
#   break-switches-sample-only         break-switches-sample-only-red、break-switches-sample-only-green
#   entry-scripts                      entry-scripts-red、entry-scripts-green、entry-scripts-reuse-three-questions、entry-scripts-red-over-green（setup.sh 都 git init）
#   break-switch-headers               break-switch-headers-red、break-switch-headers-green、break-switch-headers-quote-unbalanced-red（点名的项里「」没配平：一份少收尾、一份多收尾，加一份成对的）
#                                      break-switch-headers-split-family-green、break-switch-headers-split-family-red、break-switch-headers-split-family-table-in-part-green
#                                      （拆分族：入口自己不读开关、部分文件读，开关表在入口或部分文件的文件头；部分文件读的开关整族的表里没有判红；同目录同名起头而入口没载入的脚本照单份判）
#   stage-break-switch-replay          stage-break-switch-replay-red、stage-break-switch-replay-green、stage-break-switch-replay-progress-green（样本里的小阶段只读一份文件，进度那份另先睡 2 秒，经这一道所在仓的 stage-selftest.sh 喂）、
#                                      stage-break-switch-replay-reuse-three-questions、stage-break-switch-replay-samples-in-key、stage-break-switch-replay-red-over-green、
#                                      stage-break-switch-replay-item-green、stage-break-switch-replay-item-unknown（按条复用与逐项点名，setup.sh 都 git init）
#                                      stage-break-switch-replay-interrupted-resume（被打断之后第二趟只补没判的）、stage-break-switch-replay-unread-row-red、stage-break-switch-replay-unread-row-green
#                                      stage-break-switch-replay-jobs-registered-green、stage-break-switch-replay-jobs-registered-red、stage-break-switch-replay-threads-split-green、
#                                      stage-break-switch-replay-memory-knob-red（并行度：样本根的 .stage-break-switch-replay-memory 顶替内存、线程与峰值）
#                                      stage-break-switch-replay-key-changed-red（一道小阶段每喂一次往自己的样本目录里追加一个文件：跑的过程中键变了，点名点错的那一条失败行带「跑的过程中键变了」）
#   break-switch-outside-selftest      break-switch-outside-selftest-red、break-switch-outside-selftest-green、break-switch-outside-selftest-guard-combined-red、
#                                      break-switch-outside-selftest-guard-combined-green、break-switch-outside-selftest-split-family-green
#                                      （拆分族：探入口、不探部分文件；同目录同名起头而入口没载入的脚本照单份探）
#   break-switch-replay                break-switch-replay-red、break-switch-replay-unnamed-red、break-switch-replay-undecodable-red、break-switch-replay-green、break-switch-replay-slow-baseline-green、
#                                      break-switch-replay-temporary-service-green、break-switch-replay-progress-green、
#                                      break-switch-replay-reuse-three-questions、break-switch-replay-named-in-key、break-switch-replay-red-over-green、
#                                      break-switch-replay-item-co-run-green、break-switch-replay-item-unknown（按条复用与逐项点名，setup.sh 都 git init）
#                                      break-switch-replay-item-single-switch-green、break-switch-replay-item-single-summary-unresolved-red、break-switch-replay-item-unknown-switch
#                                      （只点单条开关：只起基准趟与点到的那几条、汇总行认不出的判不定不记库、写错的开关名退 2 并列出各条开关项；setup.sh 都 git init）
#                                      break-switch-replay-item-single-summary-declared-green、break-switch-replay-item-single-summary-declared-wrong-red、break-switch-replay-summary-declared-in-key
#                                      （文件头声明汇总行：只点单条也照声明认得出、声明对不上判不符并点名声明、声明进每条开关的键；setup.sh 都 git init）
#                                      break-switch-replay-admission-stall-red（admission.py 的子命令整组不涨 CPU 到限时就停整组、判红；setup.sh 都 git init）
#                                      break-switch-replay-interrupted-resume（被打断之后第二趟只补没落盘的）、break-switch-replay-forced-trip-rerun-green（强制现判那一条不取回）
#                                      break-switch-replay-resumed-bad-trips-parallel-green（落盘的开关趟里有被杀的：续跑时取回的判得不是对的，与没落盘的一起按外层路数并行重起、不落进一次一条的重跑；setup.sh 都 git init）
#                                      break-switch-replay-scoped-cells-green、break-switch-replay-whole-scope-red、break-switch-replay-scoped-named-quiet-red、
#                                      break-switch-replay-scoped-key-separate-green（指定范围只起点名的格、不指定范围起整份、点名的格没红判红、两种范围的判定不共用单元键）
#                                      break-switch-replay-jobs-registered-green、break-switch-replay-jobs-registered-red、break-switch-replay-jobs-threads-capped-green、
#                                      break-switch-replay-jobs-inner-threads-green、break-switch-replay-jobs-baseline-threads-by-count-green
#                                      （并行度取登记的每台默认重活线程数、不按内存上限收，基准趟与开关趟两段各分各的里层线程数：样本根的 .break-switch-replay-jobs 给内存上限与线程数，
#                                      registered 那两份一份脚本 32 条开关、内存上限 1536 MiB、线程 8）
#                                      break-switch-replay-refused-red（按条复用的共用段开跑前有拒算就判红、一条都不起）
#                                      break-switch-replay-trip-group-stopped-green、break-switch-replay-reruns-serial-green、break-switch-replay-rerun-threads-full-green
#                                      （每趟自成一组、到点停整组核清才还名额；要重跑的别的趟收完之后一次一条、拿满登记的线程数）
#                                      break-switch-replay-baseline-split-timeout-rerun-green、break-switch-replay-baseline-full-timeout-red
#                                      （基准趟分过而到点的拿满登记的线程数、同一个限时重跑；已拿满还到点的不重跑、判红；setup.sh 都 git init）
#                                      break-switch-replay-mismatch-detail-red（一份开关红五格、点名一格加一格没红：日志只列前三行，完整的失败行与点名的项落到明细文件、回读核过；setup.sh 都 git init）
#                                      break-switch-replay-key-changed-red（基准趟往自己读的表末尾加一行，开关那一条判成不符：跑的过程中键变了，那一条的失败行带「跑的过程中键变了」与变了的数据文件；setup.sh 都 git init）
#                                      break-switch-replay-split-family-green、break-switch-replay-split-family-unrelated-red、break-switch-replay-split-family-item-green
#                                      （拆分族：登记的入口自己不读开关、读开关的代码与开关表在部分文件里也算读，--item 能点部分文件里的开关；同目录同名起头而入口没载入的脚本不算部分文件；setup.sh 都 git init）
#   kb-directories-write-scope         kb-directories-write-scope-red、kb-directories-write-scope-green
#   prune-server-selftest              prune-server-selftest-green、prune-server-selftest-absent-green、prune-server-selftest-failed-red、prune-server-selftest-empty-red、prune-server-selftest-no-summary-red、prune-server-selftest-exit-red（假的自证是样本根里的 .prune-server-selftest-program，不编）
#   shared-data-root-literals          shared-data-root-literals-red、shared-data-root-literals-data-root-red、shared-data-root-literals-green
#   cp-preserve-mode                   cp-preserve-mode-red、cp-preserve-mode-green
#   peer-launch-direct                 peer-launch-direct-red、peer-launch-direct-green
#   verdict-library-code               verdict-library-code-red、verdict-library-code-green
#   temporary-service-runners          temporary-service-runners-red、temporary-service-runners-green（样本根自己的要临时服务的表与 .selftest-runners，不起自证）
#   temporary-service-wrapper-form     temporary-service-wrapper-form-green、temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red、temporary-service-wrapper-form-place-missing-red（样本根放齐七处里的文件，不起自证）
#   repository-copy-closure            repository-copy-closure-red、repository-copy-closure-green、repository-copy-closure-exempt-family-red、repository-copy-closure-exempt-family-green（后两份 setup.sh 都 git init）
#   selftest-cells-alone               selftest-cells-alone-red、selftest-cells-alone-green、selftest-cells-alone-reuse-three-questions、selftest-cells-alone-threads-split-green、
#                                      selftest-cells-alone-jobs-registered-green、selftest-cells-alone-jobs-registered-red（样本的小自证只在内存里判，setup.sh 都 git init）
#   selftest-parallelism               selftest-parallelism-red、selftest-parallelism-green（样本根自己的登记表、排除表与几份小脚本，只读文本）
#   long-run-split-exempt              long-run-split-exempt-red、long-run-split-exempt-green、long-run-split-exempt-empty（样本根自己的豁免表、records 文件与门禁文件，只读文本）
#   break-switch-replay、selftest-cells-alone  prune-server-shared-build-green（两格一起跑：样本根的 sample-prune-server-build 顶替 cargo build，第二次被起就退 1；两格取 prune-server 二进制共用同一趟编出的那一份，只起一次，两格都判绿；setup.sh 都 git init）
#   break-switch-headers、break-switch-replay  break-switch-header-gap-red（两格一起跑：一份复跑登记表点名的文件头，点名行缩进不够，两格都红在同一条开关；复跑格的 setup.sh 都 git init）
#   （不属于哪一格）                   sample-root-bare-red（判的根是样本却不在临时 prune-server 底下：退 2、一格不判；样本根放 .sample-environment-bare 造出裸跑的样子，其余样本都是在临时服务底下被喂的，它们就是这一判的绿侧；写法在文件末尾 code_tooling_sample_service_check 上方）
# 每格的样本放了什么写在那一格的注释里。各份绿样本里 vm-bench.sh、stage-owners.tsv、定义、settings.json 这副架子是并道时为八格全跑补的，留着不碍事。
# 弄坏开关的判法代码按格分在 lib/ 下的库里；下面每条 CODE_TOOLING_BREAK 开关的值，在哪份库的代码里被读，按库列在这里（共用库 lib/stage-cells.sh 的开关归它的文件头）：
# 库 lib/code-tooling-shared-libraries.sh：red-record-failure-kept-green、record-skip-lines-dropped、key-change-not-rechecked、key-change-noted-always
# 库 lib/code-tooling-shared-libraries.sh：switch-replay-named-unchecked、replay-items-unchecked、replay-unnamed-rows-unchecked
# 库 lib/code-tooling-shared-libraries.sh：replay-summary-rows-required、replay-carriage-return-kept、replay-bad-bytes-first、refused-items-run-anyway
# 库 lib/code-tooling-shared-libraries.sh：admission-run-unbounded
# 库 lib/code-tooling-selftest-cells.sh：runner-exit-swallowed、selftest-not-run-as-pass、guard-skipped、selftest-key-whole-runner-table
# 库 lib/code-tooling-selftest-cells.sh：selftest-judge-whole-guard、selftest-judge-entries-unread-ignored、red-not-recorded
# 库 lib/code-tooling-selftest-cells.sh：red-record-failure-kept-green、selftest-recorded-at-end、record-skip-lines-dropped、selftest-memory-bound-ignored
# 库 lib/code-tooling-selftest-cells.sh：selftest-memory-bound-unlisted-accepted、bare-temporary-service-selftest-accepted
# 库 lib/code-tooling-selftest-cells.sh：temporary-service-table-ignored、temporary-service-alignment-unchecked、temporary-service-form-unchecked
# 库 lib/code-tooling-selftest-cells.sh：claims-any-file、coverage-scans-build-output、selftest-coverage-top-level-only、selftest-coverage-name-substring
# 库 lib/code-tooling-selftest-cells.sh：selftest-coverage-trailing-comments-kept、numbered-stages-only、prune-server-build-not-shared
# 库 lib/code-tooling-rules-and-agents-cells.sh：owner-rows-ignored、safety-top-level-only、bare-cd-unchecked、heredoc-in-arithmetic
# 库 lib/code-tooling-rules-and-agents-cells.sh：bare-cd-condition-flagged、syntax-unchecked、path-moves-unchecked、change-range-docstrings-judged
# 库 lib/code-tooling-rules-and-agents-cells.sh：change-range-selftest-bodies-judged、change-range-exempt-marker-ignored、definition-model-unchecked
# 库 lib/code-tooling-rules-and-agents-cells.sh：change-range-libraries-by-name、change-range-own-cell-in-entry-only、change-range-names-by-file
# 库 lib/code-tooling-rules-and-agents-cells.sh：definition-tools-unchecked、definition-agent-tool-allowed、definition-sections-unchecked
# 库 lib/code-tooling-rules-and-agents-cells.sh：definition-rule-copies-unchecked、definition-table-unchecked、rules-read-once-ignored
# 库 lib/code-tooling-rules-and-agents-cells.sh：rules-named-read-once-skipped、rules-named-by-basename、rules-named-main-only-ignored
# 库 lib/code-tooling-rules-and-agents-cells.sh：rules-named-main-only-unregistered-unchecked、rules-named-main-only-named-unchecked
# 库 lib/code-tooling-rules-and-agents-cells.sh：rules-named-main-only-reason-unchecked
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：numbered-stages-only、tool-version-dates-judged、date-judge-drifted、bare-red-green-only
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：claims-by-prose、unmarked-stage-passes、structure-exit-ignored、stage-order-ignored
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：dirname-after-cd-ignored、registrations-unscanned、git-hooks-path-unchecked
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：git-hooks-relative-path-accepted、git-hooks-exec-bit-unchecked、git-hooks-tracked-mode-unchecked
# 库 lib/code-tooling-fixtures-and-layout-cells.sh：git-hooks-added-unchecked、crate-roles-baseline-unpassed
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-strings-scanned、thread-default-half-unchecked、thread-default-exempt-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-exempt-growth-unchecked、thread-default-timeout-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-own-variable-unchecked、thread-default-capped-list-ignored
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-shell-local-unchecked、thread-default-test-module-unskipped
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-cfg-test-without-mod、thread-default-fallback-by-file、thread-default-statement-single-line
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-subscript-write-counted、thread-default-shell-inline-assignment-unseen
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-quoted-name-needs-environment-read、thread-default-conditional-assignment-unpinned
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-external-mod-skipped、thread-default-test-module-file-scanned、thread-default-exempt-by-file
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-conditional-assignment-literal-only、thread-default-printenv-unseen
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-conditional-variable-source-ignored、thread-default-break-guarded-counted
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-registry-substring-accepted、thread-default-defaulted-variable-accepted
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-other-assignment-forms-unseen、thread-default-sourcing-ignored
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-selftest-samples-counted、thread-default-core-report-counted
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-name-fragment-counted、thread-default-path-variable-counted、quota-strings-scanned
# 库 lib/code-tooling-source-scan-cells.sh：quota-sentence-samples-counted、quota-memory-cap-option-unchecked、quota-memory-variable-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：quota-host-count-unchecked、quota-host-computed-unseen、quota-auto-allowed、quota-exempt-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：quota-exempt-growth-unchecked、memory-table-columns-unchecked、memory-table-entry-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：memory-table-reason-unchecked、memory-bounded-unregistered-accepted、memory-bounded-stale-accepted
# 库 lib/code-tooling-source-scan-cells.sh：memory-bounded-prefilter-memory-only、memory-table-directory-target-main-only、shared-literals-prose-scanned
# 库 lib/code-tooling-source-scan-cells.sh：shared-literals-join-form-unseen、shared-literals-braced-home-unseen、shared-literals-per-host-judged
# 库 lib/code-tooling-source-scan-cells.sh：shared-literals-definition-file-judged、shared-literals-selftest-bodies-judged
# 库 lib/code-tooling-source-scan-cells.sh：shared-literals-exempt-unchecked、shared-literals-exempt-growth-unchecked、shared-literals-data-root-unseen
# 库 lib/code-tooling-source-scan-cells.sh：shared-literals-run-records-unseen、cp-preserve-short-cluster-unseen、cp-preserve-python-unseen
# 库 lib/code-tooling-source-scan-cells.sh：cp-preserve-prose-scanned、cp-preserve-fixture-setups-unscanned、peer-launch-variable-unseen
# 库 lib/code-tooling-source-scan-cells.sh：peer-launch-launcher-scanned、peer-launch-prose-scanned、library-code-third-column-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：library-code-silent-run-unseen、library-code-local-library-unseen、library-code-unregistered-rows-scanned
# 库 lib/code-tooling-source-scan-cells.sh：library-code-explanations-scanned、library-code-break-guards-scanned、library-code-guard-names-unchecked
# 库 lib/code-tooling-source-scan-cells.sh：library-code-else-branches-released
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-exempt-family-unrecognized、quota-exempt-family-unrecognized
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-shell-local-by-file、memory-bounded-family-unrecognized
# 库 lib/code-tooling-source-scan-cells.sh：memory-table-bin-directory-target-main-only、peer-launch-launcher-parts-scanned
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-shell-selftest-samples-counted、thread-default-registry-wrapper-unrecognized
# 库 lib/code-tooling-source-scan-cells.sh：thread-default-registered-split-unrecognized、thread-default-exempt-growth-rows-counted
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：reuse-selftest-line-unchecked、reuse-selftests-red-not-recorded、reuse-selftests-plan-ignored
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：function-units-dump-unchecked、function-units-closures-unchecked、function-units-keys-unchecked
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：function-units-product-view-unchecked、function-units-data-kind-ignored
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：function-units-doctest-target-named、function-units-red-not-recorded、function-units-plan-ignored
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：list-items-exit-only、mutation-list-items-red-not-recorded、mutation-list-items-plan-ignored、mutation-list-items-build-environment-keyed
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：break-switch-guard-unchecked、entry-item-fields-unchecked、entry-scripts-red-not-recorded
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：entry-scripts-plan-ignored、switch-items-unchecked、headers-quotes-unpaired-unchecked
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：headers-replay-files-unread、headers-split-family-unread、headers-split-family-table-entry-only
# 库 lib/code-tooling-unit-verdict-and-entry-cells.sh：headers-split-family-by-name
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：stage-replay-binary-not-passed、stage-replay-samples-unkeyed
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：stage-replay-key-whole-gate-directory、stage-replay-recorded-at-end
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：stage-unread-rows-unchecked、stage-replay-jobs-by-memory、stage-replay-inner-jobs-unset
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：stage-replay-inner-threads-uncapped、stage-replay-memory-knob-unchecked、replay-plan-ignored
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：replay-red-not-recorded、outside-selftest-unchecked、guard-combination-unprobed
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：replay-progress-silent、replay-cell-name-colon-only、replay-jobs-by-memory
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：replay-trip-leader-only-stopped、cells-alone-jobs-by-memory
# 库 lib/code-tooling-stage-break-switch-replay-cells.sh：outside-selftest-split-family-unread、outside-selftest-split-family-by-name
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-header-gaps-unjudged、replay-plan-ignored、replay-red-not-recorded、replay-items-unchecked
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-output-strict-decode、replay-unnamed-rows-unchecked、replay-mismatch-detail-truncated
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-baseline-unsubtracted、replay-summary-rows-required、replay-baseline-shares-switch-limit
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-temporary-service-unwrapped、replay-progress-silent、replay-key-whole-registry
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-named-unkeyed、replay-siblings-not-co-run、replay-trips-not-stored、replay-trips-not-loaded
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-forced-trip-loaded、replay-scope-ignored、replay-whole-scoped、replay-scoped-key-shared
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-jobs-by-memory、replay-inner-threads-uncapped、replay-baseline-threads-split
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-baseline-timeout-rerun-same-threads、replay-machine-progress-silent
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-trip-leader-only-stopped、replay-reruns-batched、replay-rerun-threads-split
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-resumed-bad-trips-serial
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-split-family-unread、replay-split-family-table-entry-only、replay-split-family-by-name
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-single-picks-co-run、replay-single-pick-summary-guessed、replay-summary-declaration-ignored
# 库 lib/code-tooling-break-switch-replay-cells.sh：replay-summary-declaration-unkeyed、kb-scope-subdirectory-unchecked、prune-server-failed-ignored
# 库 lib/code-tooling-break-switch-replay-cells.sh：prune-server-empty-run-accepted、prune-server-last-line-unchecked、prune-server-exit-unchecked
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：replay-trip-leader-only-stopped、cells-alone-whole-only
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：cells-alone-inner-threads-uncapped、cells-alone-jobs-by-memory
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：repository-copy-sources-unscanned、repository-copy-hand-lists-unscanned
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：repository-copy-exempt-growth-unchecked、selftest-parallelism-report-anywhere
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：repository-copy-exempt-family-unrecognized
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：selftest-parallelism-serial-reason-unchecked、selftest-parallelism-exempt-unchecked
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：long-run-exempt-form-unchecked、long-run-exempt-reason-unchecked
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：long-run-exempt-citation-unchecked、long-run-exempt-duplicate-unchecked
# 库 lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：long-run-exempt-scope-unchecked、long-run-exempt-empty-accepted
# 入口里读的：sample-service-unchecked、items-ignored、item-unknown-accepted、list-items-split-family-unread
# 代码里拼出名字读的（不带字面值）：committed-size-tracked-half-skipped、committed-size-unpushed-half-skipped、committed-size-limit-inclusive
# 代码里拼出名字读的（不带字面值）：committed-size-no-upstream-as-pass、committed-size-detached-unresolved
# 弄坏开关（样本自检时设进环境；每条一行「开关=值  弄坏哪一处」，下面逐行点名打开后判错的样本，fixtures/code-tooling.sh/ 下的目录名；
# 这一道的 stage-break-switch-replay 格逐条复跑，比点名的与实测判错的。每格的样本放了什么、红在哪一句，写在那一格的注释里）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0（共用库别的项见那份文件头）
#     agent-write-scope-multiedit-red、agent-write-scope-red、agent-write-scope-run-scope-red、agent-write-scope-timer-red、agent-write-scope-find-timestamp-red、agent-write-scope-readers-red、agent-write-scope-session-number-red
#     preflight-guarded-red、break-switch-headers-red、break-switch-outside-selftest-red、break-switch-replay-red
#     break-switch-replay-undecodable-red、break-switch-replay-unnamed-red、break-switches-sample-only-red、change-range-single-source-path-moves-evidence-red
#     change-range-single-source-path-moves-red、change-range-single-source-red、committed-file-size-detached-red、committed-file-size-red
#     committed-file-size-unpushed-red、coverage-claims-red、coverage-nested-red、coverage-suffix-red
#     directory-layout-red、entry-scripts-red、fixture-claims-cells-red、fixture-claims-exempt-red
#     crate-roles-empty-library-tests-red、crate-roles-product-experiment-binary-red、crate-roles-product-depends-on-experiment-red、crate-roles-lag-grown-red
#     agent-write-scope-placement-red、agent-write-scope-parts-red、agent-write-scope-name-like-red、agent-write-scope-comment-named-red、agent-write-scope-loaded-shared-red
#     break-switch-headers-split-family-red、break-switch-replay-split-family-unrelated-red
#     fixture-claims-marker-malformed-red、fixture-claims-none-with-directory-red、fixture-claims-red、fixture-claims-unmarked-red
#     fixture-claims-unnumbered-red、fixture-registrations-excluded-red、fixture-registrations-excluded-stale-red、function-units-two-implementations-agree-cfg-one-side-red、function-units-two-implementations-agree-closure-red
#     function-units-two-implementations-agree-dump-red、function-units-two-implementations-agree-key-red、function-units-two-implementations-agree-product-view-red
#     gate-structure-red、heavy-thread-defaults-capped-elsewhere-red
#     heavy-thread-defaults-cfg-test-red、heavy-thread-defaults-cfg-test-module-file-red、heavy-thread-defaults-moved-red、heavy-thread-defaults-per-line-red、heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red、heavy-thread-defaults-var-source-forms-red、heavy-thread-defaults-var-source-red、heavy-thread-defaults-wide-cost-red、heavy-thread-defaults-sample-inputs-red
#     hardcoded-quota-red、memory-bounded-parallelism-red
#     heavy-thread-defaults-exempt-family-red、hardcoded-quota-exempt-family-red、repository-copy-closure-exempt-family-red
#     rust-name-collisions-red
#     kb-directories-write-scope-red、kb-registry-red
#     mutation-list-items-red、red、research-gate-lint-arithmetic-shift-red、research-gate-lint-pending-stale-red
#     research-gate-lint-red、reuse-selftests-red、rules-manifest-red、runner-guard-red
#     runner-plan-exit-red、runner-plan-sentinel-red、runner-red、runner-red-over-green、runner-red-record-failed
#     stage-break-switch-replay-red、stage-location-before-cd-red、stage-order-red、stage-owners-red
#     agent-definition-shape-agent-tool-red、agent-definition-shape-model-red、agent-definition-shape-rule-copy-red、agent-definition-shape-sections-red
#     agent-definition-shape-table-red、agent-definition-shape-tools-red、break-switch-outside-selftest-guard-combined-red
#     rules-manifest-read-once-red
#     prune-server-selftest-empty-red、prune-server-selftest-exit-red、prune-server-selftest-failed-red、prune-server-selftest-no-summary-red
#     shared-data-root-literals-red、shared-data-root-literals-data-root-red
#     cp-preserve-mode-red
#     peer-launch-direct-red
#     change-range-single-source-split-gate-red、heavy-thread-defaults-split-locals-red
#     verdict-library-code-red
#     rules-named-by-definitions-red、rules-named-by-definitions-main-only-unregistered-red
#     rules-named-by-definitions-main-only-named-red、rules-named-by-definitions-main-only-reason-red
#     runner-bare-temporary-service-red、temporary-service-runners-red
#     temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-place-missing-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red
#     break-switch-replay-red-over-green、stage-break-switch-replay-red-over-green
#     break-switch-replay-scoped-named-quiet-red、break-switch-replay-whole-scope-red
#     break-switch-replay-mismatch-detail-red、break-switch-headers-quote-unbalanced-red
#     break-switch-replay-key-changed-red、stage-break-switch-replay-key-changed-red
#     repository-copy-closure-red
#     git-hooks-installed-unset-red、git-hooks-installed-wrong-path-red、git-hooks-installed-relative-path-red
#     git-hooks-installed-hook-missing-red、git-hooks-installed-hook-no-exec-red、git-hooks-installed-tracked-mode-red
#     reuse-selftests-red-over-green、function-units-two-implementations-agree-red-over-green、mutation-list-items-red-over-green、entry-scripts-red-over-green
#     runner-red-record-failed、reuse-selftests-red-record-failed
#     runner-judge-entries-unreadable-red
#     runner-memory-bound-unlisted-red、stage-break-switch-replay-unread-row-red
#     break-switch-replay-refused-red、selftest-cells-alone-red
#     break-switch-replay-jobs-registered-red、stage-break-switch-replay-jobs-registered-red、selftest-cells-alone-jobs-registered-red
#     selftest-parallelism-red
#     long-run-split-exempt-red、break-switch-header-gap-red
#     stage-break-switch-replay-memory-knob-red
#     break-switch-replay-baseline-full-timeout-red、break-switch-replay-item-single-summary-unresolved-red、break-switch-replay-admission-stall-red
#     break-switch-replay-item-single-summary-declared-wrong-red
#     heavy-thread-defaults-shell-selftest-samples-red、heavy-thread-defaults-registry-wrapper-red、heavy-thread-defaults-registered-split-red、heavy-thread-defaults-exempt-hits-grown-red
#   CODE_TOOLING_BREAK=runner-exit-swallowed  research-script-selftests 回读时把每条的退出码都当 0
#     runner-all-not-run
#     runner-guard-red
#     runner-not-run-green
#     runner-red
#     runner-red-over-green
#     runner-red-record-failed
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
#     runner-red-record-failed（两条都拒算：没有键，不记红、没有记红没成那一句）
#   CODE_TOOLING_BREAK=selftest-judge-entries-unread-ignored  research-script-selftests 读不出守卫里的判法名单不判红、名单空着往下算（selftest_judge_entries_load）
#     runner-judge-entries-unreadable-red（期望退出 1，实测 0：三条照判绿）
#   CODE_TOOLING_BREAK=red-not-recorded  research-script-selftests 判红时不交 verdict-record --red-names（只记绿，判红的不记红、不撤同一把键原有的绿）
#     runner-red-over-green
#     runner-red-record-failed（不交红的就没有记红没成那一句、不撤绿：再算一次计划答「复用 2 条」）
#   CODE_TOOLING_BREAK=red-record-failure-kept-green  research-script-selftests 与按条复用的共用段（UNIT_VERDICT_ROUNDS_LIBRARY 的 ItemVerdicts.record）记红没成只打一行「!」、
#                                       不交 plan-failure（同一把键原有的绿留着，下一趟照复用）
#     runner-red-record-failed（输出里找不到「同一把键原有的绿已撤掉 1 条」，再算一次计划答「复用 2 条」）
#     reuse-selftests-red-record-failed（同一句找不到，再算一次计划答「复用 2 条」）
#   CODE_TOOLING_BREAK=selftest-recorded-at-end  research-script-selftests 每条跑完不当场记，只在整格收尾一次记（中途被停，判完的全丢）
#     runner-recorded-before-the-run-ends
#   CODE_TOOLING_BREAK=record-skip-lines-dropped  收尾那一趟 verdict-record 退 0 时它打的「!」行（跑的过程中键变了、不记的）不转进日志，只剩 RECORD 行
#                                       （research-script-selftests 的 selftest_record_skips_report 与按条复用的共用段 ItemVerdicts.report_record_skips 两处）
#     runner-record-skips-forwarded-green（「--selftest：跑的过程中键变了（开跑」与「另有 1 条没转进日志」两条 want 找不到）
#     break-switch-replay-record-skips-forwarded-green（同两条 want 找不到）
#   CODE_TOOLING_BREAK=key-change-not-rechecked  stage-break-switch-replay 与 break-switch-replay 判成「没对上（不符）」时不重算那一条的键（ItemVerdicts.key_changes）：
#                                       跑的过程中键变了，那一条的失败行也不写「跑的过程中键变了」，日志里没有「重算键」那一行
#     break-switch-replay-key-changed-red（「· 重算键：没对上的 1 条照开跑前同一份请求重算，键变了 1 条」与「跑的过程中键变了（开跑前算的键」两条 want 找不到）
#     stage-break-switch-replay-key-changed-red（同两条 want 找不到）
#     break-switch-replay-mismatch-detail-red（「键变了 0 条」那条 want 找不到：没有重算，日志里没有那一行）
#     stage-break-switch-replay-red（同一条 want 找不到）
#   CODE_TOOLING_BREAK=key-change-noted-always  同两格重算出来的一概当键变了：键没变的那一条也接「跑的过程中键变了」那一句，日志里写「键变了 1 条」
#     break-switch-replay-mismatch-detail-red（「键变了 0 条」那条 want 找不到）
#     stage-break-switch-replay-red（同一条 want 找不到）
#   CODE_TOOLING_BREAK=selftest-memory-bound-ignored  research-script-selftests 登记的大自证照别的一样进批（几份大自证叠在一批，整格的内存峰值是它们的和）
#     runner-memory-bound-batches-green
#   CODE_TOOLING_BREAK=selftest-memory-bound-unlisted-accepted  research-script-selftests 不核登记的大自证在不在 runner 表里（runner 表里改了名就悄悄少分一批）
#     runner-memory-bound-unlisted-red
#   CODE_TOOLING_BREAK=bare-temporary-service-selftest-accepted  research-script-selftests 不核只在临时 prune-server 上跑的自证写没写成包装写法（裸写照起、退 2）
#     runner-bare-temporary-service-red
#   CODE_TOOLING_BREAK=temporary-service-table-ignored  认「这份自证要不要临时服务」看源码里有没有 selftest_service_problem、不读要临时服务的表（own-server 的被要求包装，表里 required 而源码没写那几个字的认不出；四个格共用 temporary_service_is_required）
#     break-switch-replay-temporary-service-green（样本脚本源码里不写那几个字，不包起就退 2）、temporary-service-runners-green（own-server 裸写那份被点名）、temporary-service-runners-red（「该写成」那条 want 找不到）
#   CODE_TOOLING_BREAK=temporary-service-alignment-unchecked  temporary-service-runners 不判 runner 表与要临时服务的表两边对不对得上（只有 research-script-selftests 格看裸写）
#     temporary-service-runners-red
#   CODE_TOOLING_BREAK=temporary-service-form-unchecked  temporary-service-wrapper-form 有对不上也不判红（没有检查绑住要临时服务的自证的整条写法）
#     temporary-service-wrapper-form-binary-placeholder-red、temporary-service-wrapper-form-place-missing-red、temporary-service-wrapper-form-prefix-differs-red、temporary-service-wrapper-form-unparsable-red
#   CODE_TOOLING_BREAK=stage-replay-binary-not-passed  stage-break-switch-replay 开跑前不取 prune-server 二进制、套包装起 stage-selftest.sh 时不带 --server-binary（包装不现编，退 2、每条开关都没跑成）
#     stage-break-switch-replay-green、stage-break-switch-replay-item-green、stage-break-switch-replay-progress-green、stage-break-switch-replay-red
#     stage-break-switch-replay-red-over-green、stage-break-switch-replay-reuse-three-questions、stage-break-switch-replay-samples-in-key
#     stage-break-switch-replay-interrupted-resume、stage-break-switch-replay-unread-row-green、stage-break-switch-replay-key-changed-red
#     stage-break-switch-replay-jobs-registered-green、stage-break-switch-replay-jobs-registered-red、stage-break-switch-replay-threads-split-green
#   CODE_TOOLING_BREAK=sample-service-unchecked  判的根是样本时不核在不在临时 prune-server 底下（裸跑也判，样本的判定写进真服务）
#     sample-root-bare-red
#   ADMISSION_BREAK=plan-unavailable-as-failure  归 scripts/gate/admission.py：research-script-selftests 计划退 75 照复用失灵判红、整道退 1
#     runner-plan-unavailable
#   ADMISSION_BREAK=plan-failure-not-red  归 scripts/gate/admission.py：research-script-selftests 按条复用失灵不判红（退回全部现判）
#     runner-plan-exit-red
#     runner-plan-sentinel-red
#   ADMISSION_BREAK=red-record-failure-swallowed  归 scripts/gate/admission.py：记红没成只打一行 !、照退 0，plan-failure 不撤绿
#     runner-red-record-failed
#     reuse-selftests-red-record-failed
#   CODE_TOOLING_BREAK=reuse-selftest-line-unchecked  reuse-selftests 不看规范输出那一行、自证退 0 就算过（折成 REUSE_CHECKS_BREAK=audit-line-unchecked）
#     reuse-selftests-red
#     reuse-selftests-red-over-green
#     reuse-selftests-red-record-failed
#   CODE_TOOLING_BREAK=reuse-selftests-red-not-recorded  reuse-selftests 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     reuse-selftests-red-over-green
#     reuse-selftests-red-record-failed
#   CODE_TOOLING_BREAK=reuse-selftests-plan-ignored  reuse-selftests 不按计划跳过复用的行，要现跑的自证全部现跑（每趟整份重跑）
#     reuse-selftests-reuse-three-questions
#   CODE_TOOLING_BREAK=claims-any-file  research-script-selftest-coverage 退回「全文出现 --selftest 就算声称有」
#     coverage-claims-green
#   CODE_TOOLING_BREAK=coverage-scans-build-output  research-script-selftest-coverage 不跳过 scripts 下叫 target 的编译产物目录（里面代码写着 --selftest 的 .sh 也算声称有自证、没人跑）
#     coverage-claims-green
#   CODE_TOOLING_BREAK=selftest-coverage-top-level-only  research-script-selftest-coverage 只扫 scripts/<类>/ 那一层、不进装置目录
#     coverage-claims-green
#     coverage-nested-red
#     green
#   CODE_TOOLING_BREAK=selftest-coverage-name-substring  research-script-selftest-coverage 按文件名当子串认、左边不设边界
#     coverage-suffix-red
#   CODE_TOOLING_BREAK=selftest-coverage-trailing-comments-kept  research-script-selftest-coverage 不去行尾注释（行尾注释里的 --selftest 当实现了）
#     coverage-claims-green
#   CODE_TOOLING_BREAK=numbered-stages-only  research-script-selftest-coverage 与 fixture-claims 只认两位数开头的阶段
#     coverage-claims-green
#     fixture-claims-cells-red
#     fixture-claims-green
#     fixture-claims-unnumbered-red
#   CODE_TOOLING_BREAK=tool-version-dates-judged  fixture-claims 第 ④ 条把工具版本串「(<提交的十六进制> <年-月-日>)」里的日期也当样本正文的日期判
#     fixture-claims-green
#   CODE_TOOLING_BREAK=date-judge-drifted  fixture-claims 第 ④ 条在进程里判日期的下界多算一天（与上游 date_out_of_range 的答案分叉：边界日期对不上就报判据失效）
#     fixture-claims-green
#     fixture-claims-red
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
#     gate-structure-red、preflight-guarded-red
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
#   CODE_TOOLING_BREAK=heredoc-in-arithmetic  research-gate-lint 裸 cd 那一项把算术里的 << 也当 heredoc 起始
#     research-gate-lint-arithmetic-shift-red
#   CODE_TOOLING_BREAK=bare-cd-condition-flagged  research-gate-lint 裸 cd 那一项把 if / while 条件里收尾的 cd（`if ! cd <目录>; then`）也当裸 cd
#     research-gate-lint-green
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
#   CODE_TOOLING_BREAK=git-hooks-path-unchecked  git-hooks-installed 只看 core.hooksPath 设没设、不看它指到哪（折成 scripts/gate/git-hooks-install.py 的 GIT_HOOKS_INSTALL_BREAK=check-ignores-path）
#     git-hooks-installed-wrong-path-red
#   CODE_TOOLING_BREAK=git-hooks-relative-path-accepted  git-hooks-installed 不报相对路径的 core.hooksPath（折成 GIT_HOOKS_INSTALL_BREAK=check-accepts-relative-path）
#     git-hooks-installed-relative-path-red
#   CODE_TOOLING_BREAK=git-hooks-exec-bit-unchecked  git-hooks-installed 不看钩子文件有没有执行位（折成 GIT_HOOKS_INSTALL_BREAK=check-ignores-exec-bit）
#     git-hooks-installed-hook-no-exec-red
#   CODE_TOOLING_BREAK=git-hooks-tracked-mode-unchecked  git-hooks-installed 不看进仓的文件模式（折成 GIT_HOOKS_INSTALL_BREAK=check-ignores-tracked-mode）
#     git-hooks-installed-tracked-mode-red
#   CODE_TOOLING_BREAK=git-hooks-added-unchecked  git-hooks-installed 只核 pre-commit 与 pre-push，不核另外三份钩子（折成 GIT_HOOKS_INSTALL_BREAK=check-skips-added-hooks）
#     git-hooks-installed-hook-missing-red、git-hooks-installed-hook-no-exec-red、git-hooks-installed-tracked-mode-red
#   CODE_TOOLING_BREAK=thread-default-strings-scanned  heavy-thread-defaults 把 python 与 rust 的字面量与文档串照代码扫
#     heavy-thread-defaults-green
#     heavy-thread-defaults-red
#     heavy-thread-defaults-wide-cost-red
#   CODE_TOOLING_BREAK=thread-default-half-unchecked  heavy-thread-defaults 不认核数的一半
#     heavy-thread-defaults-red
#     heavy-thread-defaults-registered-split-red
#   CODE_TOOLING_BREAK=thread-default-exempt-unchecked  heavy-thread-defaults 的排除表不核文件在不在、锚串命不命中
#     heavy-thread-defaults-red
#   CODE_TOOLING_BREAK=thread-default-exempt-growth-unchecked  heavy-thread-defaults 的排除表不比基准
#     heavy-thread-defaults-exempt-family-green
#     heavy-thread-defaults-exempt-family-red
#     heavy-thread-defaults-moved-green
#     heavy-thread-defaults-moved-red
#     heavy-thread-defaults-red
#     heavy-thread-defaults-exempt-hits-grown-red
#     heavy-thread-defaults-exempt-hits-split-green
#   CODE_TOOLING_BREAK=thread-default-exempt-family-unrecognized  heavy-thread-defaults 的排除表比基准时不认拆分族：拆出来的新路径都当族外
#     heavy-thread-defaults-exempt-family-green
#     heavy-thread-defaults-exempt-hits-split-green
#   CODE_TOOLING_BREAK=thread-default-timeout-unchecked  heavy-thread-defaults 把超时回退值也按线程数判
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-own-variable-unchecked  heavy-thread-defaults 不判装置读自己的线程变量
#     heavy-thread-defaults-capped-elsewhere-red
#     heavy-thread-defaults-cfg-test-module-file-red
#     heavy-thread-defaults-cfg-test-red
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-per-line-red
#     heavy-thread-defaults-quoted-name-red
#     heavy-thread-defaults-red
#     heavy-thread-defaults-sample-inputs-red
#     heavy-thread-defaults-split-locals-red
#     heavy-thread-defaults-wide-cost-red
#     heavy-thread-defaults-shell-selftest-samples-red
#     heavy-thread-defaults-registry-wrapper-red
#   CODE_TOOLING_BREAK=thread-default-capped-list-ignored  heavy-thread-defaults 读了 capped.sh 的清单不认，每个线程变量都算清单外
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-shell-local-unchecked  heavy-thread-defaults 把 shell 里文件自己赋了值的变量也算读环境
#     heavy-thread-defaults-green
#     heavy-thread-defaults-per-line-green
#     heavy-thread-defaults-red
#     heavy-thread-defaults-split-locals-green
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
#     heavy-thread-defaults-sample-inputs-red
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
#   CODE_TOOLING_BREAK=thread-default-test-module-file-scanned  heavy-thread-defaults 把整份算测试的 rust 文件（父文件里 #[cfg(test)] mod 声明引进来的、祖先整份算测试的、带 #![cfg(test)] 的）也照产品代码扫
#     heavy-thread-defaults-cfg-test-module-file-red
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
#   CODE_TOOLING_BREAK=thread-default-break-guarded-counted  heavy-thread-defaults 把弄坏开关护着的 `break_is <项> && X=$变量` 也算钉数
#     heavy-thread-defaults-green
#   CODE_TOOLING_BREAK=thread-default-registry-substring-accepted  heavy-thread-defaults 的 registry_sourced_variables ① 下右边含登记位读法、去掉之后没有别的数字就算
#     heavy-thread-defaults-var-source-forms-red
#     heavy-thread-defaults-var-source-green
#   CODE_TOOLING_BREAK=thread-default-defaulted-variable-accepted  heavy-thread-defaults 不看 ${名:-…}、${名:=…}
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=thread-default-other-assignment-forms-unseen  heavy-thread-defaults 不看别的赋值写法，只认「名=值」
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=thread-default-sourcing-ignored  heavy-thread-defaults 不看文件 source 了别的文件
#     heavy-thread-defaults-var-source-forms-red
#   CODE_TOOLING_BREAK=thread-default-selftest-samples-counted  heavy-thread-defaults 把 python 自证函数与单测类里字典键给的数、capped.sh 后面的数也算
#     heavy-thread-defaults-sample-inputs-green
#   CODE_TOOLING_BREAK=thread-default-core-report-counted  heavy-thread-defaults 把只记进报告的核数（字典里核数那一键的值）也算
#     heavy-thread-defaults-sample-inputs-green
#   CODE_TOOLING_BREAK=thread-default-name-fragment-counted  heavy-thread-defaults 把引号里只有线程字样本身的名字片段也算读
#     heavy-thread-defaults-sample-inputs-green
#   CODE_TOOLING_BREAK=thread-default-path-variable-counted  heavy-thread-defaults 把名字以 _FILE、_PATH、_DIRECTORY、_DIR 收尾的路径变量也算线程变量
#     heavy-thread-defaults-sample-inputs-green
#   CODE_TOOLING_BREAK=thread-default-shell-local-by-file  heavy-thread-defaults 的 shell 局部变量只认读它的那一份文件里的赋值，不认同一拆分族别的文件里的
#     heavy-thread-defaults-split-locals-green
#   CODE_TOOLING_BREAK=thread-default-shell-selftest-samples-counted  heavy-thread-defaults 把 shell 自证函数里喂给样本的环境词与单引号里的 $变量 也算
#     heavy-thread-defaults-shell-selftest-samples-green
#   CODE_TOOLING_BREAK=thread-default-registry-wrapper-unrecognized  heavy-thread-defaults 不认登记位包装函数（$(<包装>) 不算回退到登记值）
#     heavy-thread-defaults-registry-wrapper-green
#   CODE_TOOLING_BREAK=thread-default-registered-split-unrecognized  heavy-thread-defaults 不认从登记位取好的线程数再分（rust 的核数的一半照点名）
#     heavy-thread-defaults-registered-split-green
#   CODE_TOOLING_BREAK=thread-default-exempt-growth-rows-counted  heavy-thread-defaults 的排除表比基准照旧数行，不数放行的被判行
#     heavy-thread-defaults-exempt-hits-grown-red
#     heavy-thread-defaults-exempt-hits-split-green
#   CODE_TOOLING_BREAK=quota-strings-scanned  hardcoded-quota 把 python 与 rust 的字面量与文档串照代码扫
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-sentence-samples-counted  hardcoded-quota 把引号里整句的样本也当跑法
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-memory-cap-option-unchecked  hardcoded-quota 不认 --memory-cap <常量>
#     hardcoded-quota-green
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-memory-variable-unchecked  hardcoded-quota 不认写死的内存上限默认
#     hardcoded-quota-exempt-family-green
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-host-count-unchecked  hardcoded-quota 不认写死的台数
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-host-computed-unseen  hardcoded-quota 台数变量另有现算的赋值也照判它的整数初值
#     hardcoded-quota-green
#   CODE_TOOLING_BREAK=quota-auto-allowed  hardcoded-quota 不把 auto 当写死的上限
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-exempt-unchecked  hardcoded-quota 的排除表不核文件在不在、锚串命不命中
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-exempt-growth-unchecked  hardcoded-quota 的排除表不比基准
#     hardcoded-quota-exempt-family-green
#     hardcoded-quota-exempt-family-red
#     hardcoded-quota-red
#   CODE_TOOLING_BREAK=quota-exempt-family-unrecognized  hardcoded-quota 的排除表比基准时不认拆分族：拆出来的新路径都当族外
#     hardcoded-quota-exempt-family-green
#   CODE_TOOLING_BREAK=memory-table-columns-unchecked  memory-bounded-parallelism 不核登记表每行两列
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-table-entry-unchecked  memory-bounded-parallelism 不核入口在不在、登没登两次
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-table-reason-unchecked  memory-bounded-parallelism 不核理由够不够长
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-bounded-unregistered-accepted  memory-bounded-parallelism 代码里按内存定并发的写法不核所在入口登没登记
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-bounded-stale-accepted  memory-bounded-parallelism 登记的入口不核代码里还有没有这类写法
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-bounded-prefilter-memory-only  memory-bounded-parallelism 的预筛只认 memory、不认 bytes（只带 gibibytes、free_bytes 的写法被漏掉）
#     memory-bounded-parallelism-green
#     memory-bounded-parallelism-red
#   CODE_TOOLING_BREAK=memory-table-directory-target-main-only  memory-bounded-parallelism 入口是目录式测试目标（tests/<目标>/main.rs）时只收 main.rs，不收它 mod 声明到的子模块文件
#     memory-bounded-parallelism-green
#   CODE_TOOLING_BREAK=memory-table-bin-directory-target-main-only  memory-bounded-parallelism 入口是目录式二进制目标（src/bin/<目标>/main.rs）时只收 main.rs，不收它 mod 声明到的子模块文件
#     memory-bounded-parallelism-green
#   CODE_TOOLING_BREAK=memory-bounded-family-unrecognized  memory-bounded-parallelism 写成路径的入口与代码行里点名的文件只认它们自己，不认拆分族里按拆法命名的部分文件
#     memory-bounded-parallelism-green
#   CODE_TOOLING_BREAK=path-moves-unchecked  change-range-single-source 不判搬迁登记表（写法、出处在不在、此刻生效的行串没串、有没有搬迁实据）
#     change-range-single-source-path-moves-base-row-green
#     change-range-single-source-path-moves-committed-green
#     change-range-single-source-path-moves-evidence-green
#     change-range-single-source-path-moves-evidence-red
#     change-range-single-source-path-moves-green
#     change-range-single-source-path-moves-red
#   CODE_TOOLING_BREAK=change-range-docstrings-judged  change-range-single-source 把 python 的文档串照代码判
#     change-range-single-source-prose-green
#   CODE_TOOLING_BREAK=change-range-selftest-bodies-judged  change-range-single-source 把 python 自证函数体里造的输入照代码判
#     change-range-single-source-prose-green
#   CODE_TOOLING_BREAK=change-range-exempt-marker-ignored  change-range-single-source 不认行尾的 # change-range:exempt <理由>
#     change-range-single-source-prose-green
#   CODE_TOOLING_BREAK=change-range-libraries-by-name  change-range-single-source 门禁拆出的库只按三个库名的通配扫，不照拆分族归到每道门禁
#     change-range-single-source-split-gate-red
#     change-range-single-source-split-gate-green
#   CODE_TOOLING_BREAK=change-range-own-cell-in-entry-only  change-range-single-source 这一格自己的那一段只在入口里除掉，拆出的库里的不除
#     change-range-single-source-split-gate-green
#   CODE_TOOLING_BREAK=change-range-names-by-file  change-range-single-source 带 quotepath 的变量与包装函数只认调用那一份文件自己里的，不认同一道门禁的族
#     change-range-single-source-split-gate-green
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
#     function-units-two-implementations-agree-red-over-green
#   CODE_TOOLING_BREAK=function-units-keys-unchecked  function-units-two-implementations-agree 闭包相同就不比键
#     function-units-two-implementations-agree-key-red
#     function-units-two-implementations-agree-red-over-green
#   CODE_TOOLING_BREAK=function-units-product-view-unchecked  function-units-two-implementations-agree 不跑产品行为视图那一趟
#     function-units-two-implementations-agree-data-directory-green
#     function-units-two-implementations-agree-doctest-green
#     function-units-two-implementations-agree-green
#     function-units-two-implementations-agree-product-view-red
#   CODE_TOOLING_BREAK=function-units-data-kind-ignored  function-units-two-implementations-agree 闭包的数据行一律写 data（读目录的也不写 data-directory）
#     function-units-two-implementations-agree-data-directory-green
#   CODE_TOOLING_BREAK=function-units-doctest-target-named  function-units-two-implementations-agree 给文档测试入口写 --target doctest:<名>（unit-graph 不认）
#     function-units-two-implementations-agree-doctest-green
#   CODE_TOOLING_BREAK=function-units-red-not-recorded  function-units-two-implementations-agree 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     function-units-two-implementations-agree-red-over-green
#   CODE_TOOLING_BREAK=function-units-plan-ignored  function-units-two-implementations-agree 不按计划跳过复用的那一半、每趟两半都比
#     function-units-two-implementations-agree-reuse-three-questions
#   CODE_TOOLING_BREAK=list-items-exit-only  mutation-list-items 只看 --list-items 的退出码
#     mutation-list-items-red
#     mutation-list-items-red-over-green
#   CODE_TOOLING_BREAK=mutation-list-items-red-not-recorded  mutation-list-items 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     mutation-list-items-red-over-green
#   CODE_TOOLING_BREAK=mutation-list-items-plan-ignored  mutation-list-items 不按计划跳过、每趟都起 --list-items
#     mutation-list-items-reuse-three-questions
#   CODE_TOOLING_BREAK=mutation-list-items-build-environment-keyed  mutation-list-items 的键另收 .cargo/、rust-toolchain(.toml)、Cargo.lock 与 rustc、cargo 的版本
#     mutation-list-items-build-environment-reuse-three-questions
#   CODE_TOOLING_BREAK=break-switch-guard-unchecked  break-switches-sample-only 不判
#     break-switches-sample-only-red
#   CODE_TOOLING_BREAK=entry-item-fields-unchecked  entry-scripts 不核每项那一句的字段
#     entry-scripts-red
#     entry-scripts-red-over-green
#   CODE_TOOLING_BREAK=entry-scripts-red-not-recorded  entry-scripts 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     entry-scripts-red-over-green
#   CODE_TOOLING_BREAK=entry-scripts-plan-ignored  entry-scripts 不按计划跳过复用的行、每份都读
#     entry-scripts-reuse-three-questions
#   CODE_TOOLING_BREAK=switch-items-unchecked  break-switch-headers 不判点名的项
#     break-switch-headers-red
#     break-switch-header-gap-red
#   CODE_TOOLING_BREAK=headers-quotes-unpaired-unchecked  break-switch-headers 不判点名的项里「」配不配平
#     break-switch-headers-quote-unbalanced-red
#   CODE_TOOLING_BREAK=headers-replay-files-unread  break-switch-headers 不读复跑登记表（只判 scripts/ 与 .claude/hooks/ 下的，登记表点名的别处文件不判、豁免照算）
#     break-switch-headers-green
#     break-switch-headers-red
#     break-switch-header-gap-red
#   CODE_TOOLING_BREAK=headers-split-family-unread  break-switch-headers 不认拆分族：部分文件各算一份、各判各的文件头
#     break-switch-headers-split-family-green
#     break-switch-headers-split-family-red
#     break-switch-headers-split-family-table-in-part-green
#   CODE_TOOLING_BREAK=headers-split-family-table-entry-only  break-switch-headers 读开关按整族、开关表只读入口那一份的文件头
#     break-switch-headers-split-family-table-in-part-green
#   CODE_TOOLING_BREAK=headers-split-family-by-name  break-switch-headers 部分文件只按命名认、不核入口的代码提没提到它
#     break-switch-headers-split-family-green
#   CODE_TOOLING_BREAK=replay-header-gaps-unjudged  break-switch-replay 复跑开跑前不判登记的文件头开关表写没写全（缩进不够读不出点名的项的，照样复跑）
#     break-switch-header-gap-red
#   CODE_TOOLING_BREAK=replay-split-family-unread  break-switch-replay 只读登记路径那一份的代码与文件头、不认部分文件
#     break-switch-replay-split-family-green
#   CODE_TOOLING_BREAK=replay-split-family-table-entry-only  break-switch-replay 读开关按整族、开关表与汇总行声明只读入口那一份的文件头
#     break-switch-replay-split-family-green
#   CODE_TOOLING_BREAK=replay-split-family-by-name  break-switch-replay 部分文件只按命名认、不核入口的代码提没提到它
#     break-switch-replay-split-family-unrelated-red
#   CODE_TOOLING_BREAK=switch-replay-named-unchecked  stage-break-switch-replay 不比点名的与实测判错的
#     stage-break-switch-replay-red
#     stage-break-switch-replay-jobs-registered-red（点名少写的那一条不算不符，退 0）
#     stage-break-switch-replay-red-over-green
#     stage-break-switch-replay-key-changed-red（点名点错的那一条判不成「不符」，没有「键变了」那句）
#   CODE_TOOLING_BREAK=stage-replay-samples-unkeyed  stage-break-switch-replay 每条的键不带它喂的样本目录的内容（改错的样子：样本改了照复用）
#     stage-break-switch-replay-samples-in-key
#     stage-break-switch-replay-key-changed-red（样本目录跑的过程中变了，键不跟着变，没有「键变了」那句）
#   CODE_TOOLING_BREAK=stage-replay-key-whole-gate-directory  stage-break-switch-replay 每条的键另加有样本目录的每一道阶段的整份文件（改一道、注释也算，全部作废）
#     stage-break-switch-replay-reuse-three-questions
#     stage-break-switch-replay-samples-in-key
#   CODE_TOOLING_BREAK=stage-replay-recorded-at-end  stage-break-switch-replay 一条判完不当场记，只在整格收尾一次记（中途被停，判完的全丢）
#     stage-break-switch-replay-interrupted-resume
#   CODE_TOOLING_BREAK=stage-unread-rows-unchecked  stage-break-switch-replay 不判头注里写了 X_BREAK= 却读不出的行（那一条悄悄不复跑）
#     stage-break-switch-replay-unread-row-red
#   CODE_TOOLING_BREAK=stage-replay-jobs-by-memory  stage-break-switch-replay 同时跑的条数再照内存份额（这一台能给的与这一条拿到的上限里小的、扣掉本进程用量）÷ 那一道阶段每条开关开满时的峰值收
#     stage-break-switch-replay-jobs-registered-green、stage-break-switch-replay-jobs-registered-red（份额约 2.5 GiB、峰值 1 GiB：同时最多 2 条，不是登记的线程数 3 条）
#   CODE_TOOLING_BREAK=stage-replay-inner-jobs-unset  stage-break-switch-replay 起 stage-selftest.sh 不带 --jobs（喂样本的并行度照它自己的核数）
#     stage-break-switch-replay-jobs-registered-green、stage-break-switch-replay-threads-split-green
#   CODE_TOOLING_BREAK=stage-replay-inner-threads-uncapped  stage-break-switch-replay 每条 stage-selftest 的 --jobs 仍取每台默认重活线程数、不按外层路数分（外层路数 × 里层 --jobs 超过线程数）
#     stage-break-switch-replay-jobs-registered-green、stage-break-switch-replay-jobs-registered-red、stage-break-switch-replay-threads-split-green
#   CODE_TOOLING_BREAK=stage-replay-memory-knob-unchecked  stage-break-switch-replay 样本根 .stage-break-switch-replay-memory 里读不出的行悄悄跳过
#     stage-break-switch-replay-memory-knob-red
#   CODE_TOOLING_BREAK=replay-plan-ignored  break-switch-replay 与 stage-break-switch-replay 不按计划跳过复用的条，点到的全部起（每趟整格重跑）
#     break-switch-replay-reuse-three-questions
#     break-switch-replay-named-in-key
#     break-switch-replay-summary-declared-in-key
#     break-switch-replay-item-co-run-green（second 那一趟照起四条，「全部复用，这一格一条都没起」那条 want 找不到）
#     break-switch-replay-interrupted-resume
#     break-switch-replay-forced-trip-rerun-green
#     break-switch-replay-resumed-bad-trips-parallel-green（第三趟照起四条，「全部复用，这一格一条都没起」那条 want 找不到）
#     stage-break-switch-replay-reuse-three-questions
#     stage-break-switch-replay-samples-in-key
#     stage-break-switch-replay-interrupted-resume
#   CODE_TOOLING_BREAK=replay-red-not-recorded  break-switch-replay 与 stage-break-switch-replay 判红时不交 verdict-record --red-names（只记绿，判红的不撤同一把键原有的绿）
#     break-switch-replay-red-over-green
#     stage-break-switch-replay-red-over-green
#   CODE_TOOLING_BREAK=items-ignored  逐项点名不起作用，点到的格照跑整格
#     runner-item-picked
#     break-switch-replay-item-co-run-green
#     stage-break-switch-replay-item-green
#     break-switch-replay-item-single-switch-green（整份起，没点的 TRIO_BREAK=gamma 判不符）
#     break-switch-replay-item-single-summary-unresolved-red（整份起，汇总行认得出，判绿）
#     break-switch-replay-item-single-summary-declared-green（整份起，「只跑 样本根的 .gate-items 点名的项」那条 want 找不到）
#     break-switch-replay-item-single-summary-declared-wrong-red（整份起，三条开关都判不符，「只跑 样本根的 .gate-items 点名的项」那条 want 找不到）
#     break-switch-replay-split-family-item-green（整份起，「只跑 样本根的 .gate-items 点名的项」那条 want 找不到）
#   CODE_TOOLING_BREAK=list-items-split-family-unread  break-switch-replay 的可点的项只读登记路径那一份的文件头、不认部分文件里的开关表
#     break-switch-replay-split-family-item-green
#   CODE_TOOLING_BREAK=item-unknown-accepted  写错的项悄悄丢掉、不退 2，一项都没认出就照跑整格
#     runner-item-unknown
#     break-switch-replay-item-unknown
#     stage-break-switch-replay-item-unknown
#     break-switch-replay-item-unknown-switch（不退 2，整格起 trio.py 三条开关，TRIO_BREAK=gamma 判不符、退 1）
#   CODE_TOOLING_BREAK=outside-selftest-unchecked  break-switch-outside-selftest 把探针的结果一概当守住
#     break-switch-outside-selftest-guard-combined-red
#     break-switch-outside-selftest-red
#   CODE_TOOLING_BREAK=guard-combination-unprobed  break-switch-outside-selftest 不另起「关守卫项,第一个值」那一次探针
#     break-switch-outside-selftest-guard-combined-green
#     break-switch-outside-selftest-guard-combined-red
#   CODE_TOOLING_BREAK=outside-selftest-split-family-unread  break-switch-outside-selftest 不认拆分族：部分文件也当脚本起来探
#     break-switch-outside-selftest-split-family-green
#   CODE_TOOLING_BREAK=outside-selftest-split-family-by-name  break-switch-outside-selftest 部分文件只按命名认、不核入口的代码提没提到它
#     break-switch-outside-selftest-split-family-green
#   CODE_TOOLING_BREAK=replay-items-unchecked  break-switch-replay 不比点名的项
#     break-switch-replay-red
#     break-switch-replay-undecodable-red
#     break-switch-replay-scoped-named-quiet-red（指定范围、点名的格没红：脚本退 1 却只在转述行里带不一样的 ✗，判绿）
#     break-switch-replay-mismatch-detail-red（点名的一格没红这一条 want 找不到）
#     break-switch-replay-resumed-bad-trips-parallel-green（取回的退 137、不打失败行的两趟判成对上，不并行重起，「取回的落盘趟里 2 趟判得不是对的」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-output-strict-decode  break-switch-replay 按文本收自证输出、不容坏字节
#     break-switch-replay-undecodable-red
#   CODE_TOOLING_BREAK=replay-unnamed-rows-unchecked  break-switch-replay 不查多红（失败行比点名的多不判）
#     break-switch-replay-unnamed-red
#     break-switch-replay-whole-scope-red（不指定范围起整份，没点名的两格红了不判，判绿）
#     break-switch-replay-mismatch-detail-red（没被点名的四行多红这一条 want 找不到）
#     break-switch-replay-item-single-summary-unresolved-red（汇总行那一行不查，判绿）
#     break-switch-replay-item-single-summary-declared-wrong-red（汇总行那一行不查，判绿，「不符，失败行没被点名」那条 want 找不到）
#     break-switch-replay-key-changed-red（多红不查，「失败行没被点名：」那一条 want 找不到）
#   CODE_TOOLING_BREAK=replay-mismatch-detail-truncated  break-switch-replay 判「不符」的明细文件照日志的样子截：只写前三行失败行，每行只写前八十个字，点名的项只写前三项、每项只写前六十个字
#     break-switch-replay-mismatch-detail-red
#   CODE_TOOLING_BREAK=replay-baseline-unsubtracted  break-switch-replay 不起基准趟、不减自证本来就有的失败行
#     break-switch-replay-key-changed-red（基准趟不起，没人往表里加那一行，键不变，没有「键变了」那句）
#     break-switch-replay-green
#     break-switch-replay-reuse-three-questions（基准趟那一条不判、不记库，下一趟它照现判：三问 ① 红）
#     break-switch-replay-named-in-key
#     break-switch-replay-summary-declared-in-key
#     break-switch-replay-item-co-run-green（基准趟那一条不判、不记库，second 那一趟它照现判：三问 ① 红，「全部复用」那条 want 找不到）
#     break-switch-replay-interrupted-resume（基准趟不起，判完的趟数与取回的趟数都对不上）
#     break-switch-replay-forced-trip-rerun-green（基准趟那一条没记库，第二趟现判的多一条）
#     break-switch-replay-resumed-bad-trips-parallel-green（基准趟不起，第一趟落盘的是三条开关趟：续跑时现起的 LANE_BREAK=stem 只剩它一条、起跑后等不到别的趟，多打一行没点名的 ✗ 判不符，重起的也是 3 趟、不是 2 趟）
#     break-switch-replay-scoped-key-separate-green（基准趟不起，两趟现跑的都只有 2 趟）
#     break-switch-replay-baseline-split-timeout-rerun-green（基准趟不起，没有「基准趟拿满线程重跑」那一行）
#     break-switch-replay-baseline-full-timeout-red（基准趟不起，开关那一条对上，判绿）
#     break-switch-replay-progress-green（基准趟不起，机读进度只数两趟开关趟，四条「TASK_SERVER_PROGRESS done=… total=3」的 want 找不到）
#     break-switch-replay-record-skips-forwarded-green（基准趟不起，没人往 rewritten.tsv 加那一行，键不变：「RECORD recorded=0 changed=2」「：跑的过程中键变了（开跑」「另有 1 条没转进日志」三条 want 找不到）
#   CODE_TOOLING_BREAK=replay-summary-rows-required  break-switch-replay 汇总行也要点名
#     break-switch-replay-green
#     break-switch-replay-item-co-run-green
#     break-switch-replay-item-single-summary-declared-green（声明认出的汇总行也要点名，判不符）
#     break-switch-replay-interrupted-resume
#   CODE_TOOLING_BREAK=replay-carriage-return-kept  break-switch-replay 把单独的 \r 留在行里
#     break-switch-replay-unnamed-red
#   CODE_TOOLING_BREAK=replay-bad-bytes-first  break-switch-replay 有坏字节就只记坏字节、不再比点名
#     break-switch-replay-undecodable-red
#   CODE_TOOLING_BREAK=replay-baseline-shares-switch-limit  break-switch-replay 的基准趟也照每条开关的限时跑、不读基准趟那一列
#     break-switch-replay-slow-baseline-green
#   CODE_TOOLING_BREAK=replay-temporary-service-unwrapped  break-switch-replay 起只在临时 prune-server 上跑的自证也不包一层（裸跑退 2，基准趟就没过）
#     break-switch-replay-temporary-service-green
#   CODE_TOOLING_BREAK=replay-progress-silent  break-switch-replay 与 stage-break-switch-replay 不打进度、不起报进度的线程（整格跑完之前日志里一行不涨）
#     break-switch-replay-progress-green
#     stage-break-switch-replay-progress-green
#     break-switch-replay-jobs-registered-green、break-switch-replay-jobs-registered-red、break-switch-replay-jobs-threads-capped-green、break-switch-replay-jobs-inner-threads-green、
#     break-switch-replay-jobs-baseline-threads-by-count-green（开跑那一行不打）
#   CODE_TOOLING_BREAK=replay-key-whole-registry  break-switch-replay 每条的键另加复跑登记表里每一份脚本的整份文件（改一份、注释也算，全部作废）
#     break-switch-replay-reuse-three-questions
#     break-switch-replay-named-in-key
#     break-switch-replay-summary-declared-in-key
#     break-switch-replay-item-co-run-green（unrelated.sh 只改 tally.py 一行说明，四条的键都变：三问 ② 红，「现判 1、复用 3」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-named-unkeyed  break-switch-replay 每条开关的键不带那一条点名的项的原文（改错的样子：头注改了照复用）
#     break-switch-replay-named-in-key
#     break-switch-replay-item-co-run-green（change.sh 改了 prefix 那一条点名的项的写法，键不变，unit 那一趟全部复用、一条都没起：三问 ③ 红）
#   CODE_TOOLING_BREAK=replay-siblings-not-co-run  break-switch-replay 同一份只起要现判的那几条开关趟、别的不陪跑（改错的样子：汇总行少了陪跑的开关趟）
#     break-switch-replay-item-co-run-green（unit 那一趟只起 prefix 那一条，汇总行认不出，判不符）
#   CODE_TOOLING_BREAK=replay-trips-not-stored  break-switch-replay 开关趟跑完不落盘（一份没跑齐就被打断，跑完的那几趟全丢）
#     break-switch-replay-interrupted-resume
#     break-switch-replay-forced-trip-rerun-green
#     break-switch-replay-resumed-bad-trips-parallel-green（第二趟取回 0 趟、现起 5 趟，「取回 3 趟」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-trips-not-loaded  break-switch-replay 不取回落了盘的开关趟，要起的整份照起
#     break-switch-replay-interrupted-resume
#     break-switch-replay-forced-trip-rerun-green
#     break-switch-replay-resumed-bad-trips-parallel-green（第二趟四条开关趟都现起，没有取回的，「取回 3 趟」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-forced-trip-loaded  break-switch-replay 复用哨兵抽中强制现判的那一条也取回（强制现判没真起，判定稳不稳比不出）
#     break-switch-replay-forced-trip-rerun-green
#   CODE_TOOLING_BREAK=replay-scope-ignored  break-switch-replay 指定范围时也不选格，每条开关趟与基准趟照整份起（键里也不带选格那一行）
#     break-switch-replay-scoped-cells-green（没点名的两格也红了，逐条判红）
#     break-switch-replay-scoped-named-quiet-red（范围那一行写的是只起点名的格 0 条）
#     break-switch-replay-scoped-key-separate-green（两趟键相同，第二趟复用第一趟的）
#   CODE_TOOLING_BREAK=replay-cell-name-colon-only  break-switch-replay 指定范围时点名的项只认「自证：<格名>：」，「✗ 自证 <格名>：」读不出格名，那条开关与基准趟整份起
#     break-switch-replay-scoped-cells-space-form-green（整份起，没点名的两格也红了，逐条判红）
#   CODE_TOOLING_BREAK=replay-whole-scoped  break-switch-replay 不指定范围时也只起点名的格
#     break-switch-replay-whole-scope-red（没点名的格不起，判绿）
#     break-switch-replay-scoped-key-separate-green（第二趟写的是指定范围，键与第一趟相同、复用）
#   CODE_TOOLING_BREAK=replay-scoped-key-shared  break-switch-replay 只起点名的格的那几趟，键里不另带选格那一行（与不指定范围共用同一把键）
#     break-switch-replay-scoped-key-separate-green（第二趟复用第一趟的判定）
#   CODE_TOOLING_BREAK=replay-jobs-by-memory  break-switch-replay 并行条数照这一趟的内存上限算（⌊上限的份额 ÷ 每趟内存⌋、不超过线程数，parallel_library.jobs_for），在跑各趟的内存估计之和不超过份额
#     break-switch-replay-jobs-registered-green（上限 1536 MiB、每趟 4096 MiB：并行 1 条，开关趟一条接一条，等不到同时 8 趟在跑，每条开关多红一行没点名的）
#     break-switch-replay-jobs-registered-red（开跑那一行写的是并行 1 条）
#     break-switch-replay-jobs-threads-capped-green（开跑那一行括号里写的是内存上限、每趟与份额，不是登记的线程数）
#   CODE_TOOLING_BREAK=replay-inner-threads-uncapped  break-switch-replay 不把每趟里层的线程数收到「登记的线程数 ÷ 外层路数的上限」：不设 SINGLEFS_THREAD_CAP，里层仍是每台默认重活线程数（外层路数 × 里层线程数超过登记的线程数）
#     break-switch-replay-jobs-threads-capped-green、break-switch-replay-jobs-inner-threads-green、break-switch-replay-jobs-baseline-threads-by-count-green
#     break-switch-replay-baseline-split-timeout-rerun-green（slow.sh 的基准趟读不到 SINGLEFS_THREAD_CAP，打一行没点名的 ✗ 退 1，基准趟没过）
#     break-switch-replay-rerun-threads-full-green（again.sh 的基准趟读不到 SINGLEFS_THREAD_CAP，打一行没点名的 ✗ 退 1，基准趟没过）
#   CODE_TOOLING_BREAK=replay-baseline-threads-split  break-switch-replay 的基准趟那一段也拿开关趟那一段分过的里层线程数（登记的线程数 ÷ 开关趟那一段外层路数的上限），不按同时起的基准趟条数分
#     break-switch-replay-jobs-threads-capped-green（开跑那一行基准趟写的是外层 1 条 × 里层 1）
#     break-switch-replay-jobs-inner-threads-green（基准趟拿到的 SINGLEFS_THREAD_CAP 是 2、不是 8，基准趟没过）
#     break-switch-replay-jobs-baseline-threads-by-count-green（基准趟拿到的是 2、不是 4，基准趟没过）
#     break-switch-replay-baseline-split-timeout-rerun-green（slow.sh 的头一趟拿到的是 2、不是 4，「基准趟拿满线程重跑」那一行写的是头一趟里层 2 个线程）
#     break-switch-replay-baseline-full-timeout-red（基准趟拿到的是 2、不是登记的 4，当成分过的拿满重跑，重跑那一趟看到记号就过，判绿）
#     break-switch-replay-rerun-threads-full-green（again.sh 的基准趟拿到的是 2、不是 8，基准趟没过）
#   CODE_TOOLING_BREAK=replay-baseline-timeout-rerun-same-threads  break-switch-replay 到点停整组的基准趟一律照头一趟那一段的线程数重跑一次：分过而到点的不拿满登记的线程数，拿满了还到点的也再跑一趟
#     break-switch-replay-baseline-split-timeout-rerun-green（slow.sh 重跑那一趟仍拿 4 个线程、又到点，基准趟没过）
#     break-switch-replay-baseline-full-timeout-red（拿满了还到点的那一份又跑了一趟、看到记号就过，判绿）
#   CODE_TOOLING_BREAK=replay-machine-progress-silent  break-switch-replay 不打行首顶格的 TASK_SERVER_PROGRESS 机读进度行（别的进度行照打）
#     break-switch-replay-progress-green
#   CODE_TOOLING_BREAK=replay-trip-leader-only-stopped  break-switch-replay 与 selftest-cells-alone 每趟不开新会话，到点只 kill 直接起的那一个、不收整组、不核组里留没留下进程，名额当场还
#     break-switch-replay-trip-group-stopped-green
#   CODE_TOOLING_BREAK=replay-reruns-batched  break-switch-replay 一份的开关趟齐了、要重跑的当场派进池子，与还在跑的趟、同一批要重跑的一起起
#     break-switch-replay-reruns-serial-green
#     break-switch-replay-rerun-threads-full-green（重跑那一趟在池子里起、拿开关趟那一段分过的 2 个线程，again.sh 此后多打一行没点名的 ✗，判不符）
#   CODE_TOOLING_BREAK=replay-resumed-bad-trips-serial  break-switch-replay 取回的落盘趟判得不是对的不挑出来并行重起，照头一趟判、进全部开关趟收完之后一次一条的重跑
#     break-switch-replay-resumed-bad-trips-parallel-green（取回的两趟一次一条重跑、起跑后 6 秒内等不到别的趟，lane.sh 多打一行没点名的 ✗，判不符）
#   CODE_TOOLING_BREAK=replay-rerun-threads-split  break-switch-replay 全部开关趟收完之后一次一条的重跑仍拿开关趟那一段分过的里层线程数，不拿满登记的线程数
#     break-switch-replay-rerun-threads-full-green（重跑那一趟核到的 SINGLEFS_THREAD_CAP 是 2、不是 8，again.sh 此后多打一行没点名的 ✗，判不符）
#   CODE_TOOLING_BREAK=replay-single-picks-co-run  break-switch-replay 只点了一份的单条开关时也照整份起，没点的开关陪跑
#     break-switch-replay-item-single-switch-green（没点的 TRIO_BREAK=gamma 也起了，判不符；「复跑了 1 条开关」与「只点了单条的 1 份」两条 want 找不到）
#     break-switch-replay-item-single-summary-unresolved-red（三条都起，汇总行认得出，判绿）
#     break-switch-replay-item-single-summary-declared-green（三条都起，「按条复用：1 份脚本、2 条判定」那条 want 找不到）
#     break-switch-replay-item-single-summary-declared-wrong-red（三条都起，三条都判不符，「按条复用：1 份脚本、2 条判定」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-single-pick-summary-guessed  break-switch-replay 只点单条的那几份照点到的几条认汇总行、多出来的失败行照常判不符并记库（不判「认不出汇总行」）
#     break-switch-replay-item-single-summary-unresolved-red（判成不符，「认不出汇总行」那条 want 找不到）
#   CODE_TOOLING_BREAK=replay-summary-declaration-ignored  break-switch-replay 不读被测脚本文件头 break-summary-rows 的汇总行声明，汇总行照整份统计认、只点单条的认不出（键里也不带声明）
#     break-switch-replay-item-single-summary-declared-green（认不出汇总行，判红）
#     break-switch-replay-item-single-summary-declared-wrong-red（认不出汇总行，「不符，失败行没被点名」那条 want 找不到）
#     break-switch-replay-summary-declared-in-key（声明不进键，改声明那一趟 alpha 的两条照复用，三问 ③ 红）
#   CODE_TOOLING_BREAK=replay-summary-declaration-unkeyed  break-switch-replay 每条开关的键不带声明的汇总行原文（改错的样子：声明改了照复用）
#     break-switch-replay-summary-declared-in-key（change.sh 只改了声明的写法，键不变，unit 那一趟全部复用、一条都没起：三问 ③ 红）
#   CODE_TOOLING_BREAK=refused-items-run-anyway  按条复用的共用段（UNIT_VERDICT_ROUNDS_LIBRARY 的 ItemVerdicts.stop_on_refusals）开跑前有拒算也不判红，拒算的条照跑到底、不入库
#     break-switch-replay-refused-red
#   CODE_TOOLING_BREAK=admission-run-unbounded  按条复用的共用段（UNIT_VERDICT_ROUNDS_LIBRARY 的 admission_run）起 admission.py 的子命令不盯整组的 CPU、不设卡住的限时，卡住就一直等
#     break-switch-replay-admission-stall-red（样本那一趟睡满 20 秒才退、没打 PLAN，照失灵判红，「admission.py verdict-plan 卡住」那条 want 找不到）
#   CODE_TOOLING_BREAK=cells-alone-whole-only  selftest-cells-alone 不单跑，每格照整份那一趟里红没红记
#     selftest-cells-alone-green selftest-cells-alone-red selftest-cells-alone-threads-split-green（只起整份那一趟，外层 1 条、里层拿满线程数，样本自证核到的 SINGLEFS_THREAD_CAP 不是 2）
#     selftest-cells-alone-jobs-registered-green selftest-cells-alone-jobs-registered-red（同上）
#   CODE_TOOLING_BREAK=cells-alone-inner-threads-uncapped  selftest-cells-alone 不把每趟里层的线程数收到「登记的线程数 ÷ 外层路数的上限」：SINGLEFS_THREAD_CAP 仍交每台默认重活线程数（外层路数 × 里层线程数超过登记的线程数）
#     selftest-cells-alone-threads-split-green、selftest-cells-alone-jobs-registered-green、selftest-cells-alone-jobs-registered-red
#   CODE_TOOLING_BREAK=cells-alone-jobs-by-memory  selftest-cells-alone 并行条数照这一趟的内存上限算（⌊上限的份额 ÷ 每趟内存⌋、不超过线程数，parallel_library.jobs_for）
#     selftest-cells-alone-jobs-registered-green、selftest-cells-alone-jobs-registered-red（上限 1536 MiB、每趟 4096 MiB：并行 1 趟，外层 1 条、里层拿满线程数 8，样本自证核到的 SINGLEFS_THREAD_CAP 不是 2）
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
#   AGENT_WRITE_SCOPE_BREAK=find-timestamp-unchecked  agent-write-scope 不判 find 时刻写法闸注册在 Bash 的 matcher 上
#     agent-write-scope-find-timestamp-red
#   AGENT_WRITE_SCOPE_BREAK=session-checks-unchecked  agent-write-scope 不判会话检测钩子注册着、matcher 覆盖四种 source
#     agent-write-scope-red、agent-write-scope-timer-red
#   AGENT_WRITE_SCOPE_BREAK=readers-unchecked  agent-write-scope 只要求有 Write 或 Edit 的定义在写范围表里登记，只带 Read、Bash 这类的定义没有行也不判
#     agent-write-scope-readers-red
#   AGENT_WRITE_SCOPE_BREAK=placement-guard-unchecked  agent-write-scope 不判放置类弹窗闸注册在 AskUserQuestion 的 matcher 上
#     agent-write-scope-placement-red
#   AGENT_WRITE_SCOPE_BREAK=session-number-stop-unchecked  agent-write-scope 不判会话编号收工检查注册在 Stop 上
#     agent-write-scope-session-number-red
#   AGENT_WRITE_SCOPE_BREAK=family-parts-unread  agent-write-scope 查手抄的函数名只取共用模块入口文件里的顶层函数、不取同层部分文件里的
#     agent-write-scope-red（手抄的 shell_tokens、judged_by_name 都住在部分文件里）、agent-write-scope-parts-red
#   AGENT_WRITE_SCOPE_BREAK=family-by-name  agent-write-scope 只按名字认共用模块的族：<模块名>_<部分>.py 的文件都算族、不看入口载没载入，被判仓里名字像部分文件而入口没载入的文件被跳过
#     agent-write-scope-name-like-red（lib_shell_words_shadow.py 手抄的 commands_at_command_position 被当成族里的、不报）、agent-write-scope-comment-named-red（同一份影子文件，入口只在注释与文档串里点名它）
#   AGENT_WRITE_SCOPE_BREAK=family-by-loading  agent-write-scope 只按载入认共用模块的族：入口 import 的同目录文件都算族、不看名字像不像部分文件，被判仓里被入口 import 的共用库被跳过
#     agent-write-scope-loaded-shared-red（shared_text_tools.py 手抄的 commands_at_command_position 被当成族里的、不报）
#   AGENT_WRITE_SCOPE_BREAK=family-by-text  agent-write-scope 认共用模块的族时载入按文本行扫：注释、文档串里写的文件名与 import 行也算载入，字面量参数的 importlib.import_module 认不出
#     agent-write-scope-comment-named-red（入口注释里点名的 lib_shell_words_shadow.py 被当成族里的、手抄的函数不报）、agent-write-scope-import-module-green（按 import_module 载入的 lib_shell_words_core.py 不在族里、里面的 brace_expand 被报成手抄）
#   CODE_TOOLING_BREAK=crate-roles-baseline-unpassed  crate-roles 不取基准那一版滞后表，一律交 --lag-baseline-absent（只缩不涨不比）
#     crate-roles-lag-grown-red
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
#   CODE_TOOLING_BREAK=rules-read-once-ignored  rules-manifest 不认 CLAUDE.md 的 read-once 登记
#     rules-manifest-read-once-green
#     rules-manifest-read-once-red
#   CODE_TOOLING_BREAK=rules-named-read-once-skipped  rules-named-by-definitions 只判 @ 规则，read-once 登记的不判
#     rules-named-by-definitions-green
#     rules-named-by-definitions-red
#   CODE_TOOLING_BREAK=rules-named-by-basename  rules-named-by-definitions 只写文件名、不带目录的也算点名
#     rules-named-by-definitions-red
#   CODE_TOOLING_BREAK=rules-named-main-only-ignored  rules-named-by-definitions 不认只管主 agent 的登记表，登记了的规则照样要求有定义点名
#     rules-named-by-definitions-main-only-green
#   CODE_TOOLING_BREAK=rules-named-main-only-unregistered-unchecked  rules-named-by-definitions 不核登记表里的路径在不在 CLAUDE.md 的登记里
#     rules-named-by-definitions-main-only-unregistered-red
#   CODE_TOOLING_BREAK=rules-named-main-only-named-unchecked  rules-named-by-definitions 不核登记表里的规则是不是其实已经有定义点名
#     rules-named-by-definitions-main-only-named-red
#   CODE_TOOLING_BREAK=rules-named-main-only-reason-unchecked  rules-named-by-definitions 不核登记表里的理由够不够 8 个字
#     rules-named-by-definitions-main-only-reason-red
#   CODE_TOOLING_BREAK=prune-server-build-not-shared  各格取 prune-server 二进制不共用本趟编出的那一份、各编各的（样本的假编译第二次被起就退 1）
#     prune-server-shared-build-green
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
#   CODE_TOOLING_BREAK=peer-launch-launcher-scanned  peer-launch-direct 连 scripts/hosts/multi-host-run.sh 自己与它的部分文件也扫
#     peer-launch-direct-green
#   CODE_TOOLING_BREAK=peer-launch-launcher-parts-scanned  peer-launch-direct 只跳过 scripts/hosts/multi-host-run.sh 本身，它的拆分族里的部分文件（自证）照扫
#     peer-launch-direct-green
#   CODE_TOOLING_BREAK=peer-launch-prose-scanned  peer-launch-direct 把注释与带空白的字面量也照代码扫
#     peer-launch-direct-green
#   CODE_TOOLING_BREAK=library-code-third-column-unchecked  verdict-library-code 不判第三列
#     verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-silent-run-unseen  verdict-library-code 不搜取不到库照跑的写法
#     verdict-library-code-green、verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-local-library-unseen  verdict-library-code 不搜本地库读写的写法
#     verdict-library-code-green、verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-unregistered-rows-scanned  verdict-library-code 连「不算：」「无判定库：」行点名的脚本也搜
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-explanations-scanned  verdict-library-code 连注释与文档串也照判
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-break-guards-scanned  verdict-library-code 连弄坏开关护着的分支也照判
#     verdict-library-code-green
#   CODE_TOOLING_BREAK=library-code-guard-names-unchecked  verdict-library-code 包着的块不按文件头开关表认开关名，任何带连字符的词都算
#     verdict-library-code-red
#   CODE_TOOLING_BREAK=library-code-else-branches-released  verdict-library-code 往上找包着的块时缩进相同的行也算（else 那一支跟着 if 放过）
#     verdict-library-code-red
#   CODE_TOOLING_BREAK=repository-copy-sources-unscanned  repository-copy-closure 拷源与手抄清单一概不认（手抄的拷贝清单照过）
#     repository-copy-closure-exempt-family-green
#     repository-copy-closure-green
#     repository-copy-closure-red
#   CODE_TOOLING_BREAK=repository-copy-hand-lists-unscanned  repository-copy-closure 不认手抄的拷贝清单常量，只认拷法那一行
#     repository-copy-closure-red
#   CODE_TOOLING_BREAK=repository-copy-exempt-growth-unchecked  repository-copy-closure 的排除表不比基准
#     repository-copy-closure-exempt-family-green
#     repository-copy-closure-exempt-family-red
#     repository-copy-closure-red
#   CODE_TOOLING_BREAK=repository-copy-exempt-family-unrecognized  repository-copy-closure 的排除表比基准时不认拆分族：拆出来的新路径都当族外
#     repository-copy-closure-exempt-family-green
#   CODE_TOOLING_BREAK=selftest-parallelism-report-anywhere  selftest-parallelism 把整行注释里的「并行 N」也算报了并行度
#     selftest-parallelism-red
#   CODE_TOOLING_BREAK=selftest-parallelism-serial-reason-unchecked  selftest-parallelism 的 # selftest-serial: 理由不看长短
#     selftest-parallelism-red
#   CODE_TOOLING_BREAK=selftest-parallelism-exempt-unchecked  selftest-parallelism 的排除表行不核指向（不在的文件、已经达标的那一份照放行）
#     selftest-parallelism-red
#   CODE_TOOLING_BREAK=long-run-exempt-form-unchecked  long-run-split-exempt 不判一行有没有 # 和 # 左边有没有项名
#     long-run-split-exempt-red
#   CODE_TOOLING_BREAK=long-run-exempt-reason-unchecked  long-run-split-exempt 不判理由去掉空白之后够不够 8 个字
#     long-run-split-exempt-red
#   CODE_TOOLING_BREAK=long-run-exempt-citation-unchecked  long-run-split-exempt 不判理由里的 records/<文件>.md:<行> 出处（有没有、文件在不在、行号在不在文件行数以内）
#     long-run-split-exempt-red
#   CODE_TOOLING_BREAK=long-run-exempt-duplicate-unchecked  long-run-split-exempt 不判项名重复
#     long-run-split-exempt-red
#   CODE_TOOLING_BREAK=long-run-exempt-scope-unchecked  long-run-split-exempt 不判项名是目录或整道门禁的文件名
#     long-run-split-exempt-red
#   CODE_TOOLING_BREAK=long-run-exempt-empty-accepted  long-run-split-exempt 登记表一行都没有也报绿（不退 77）
#     long-run-split-exempt-empty
#
#   bash .claude/gate.d/code-tooling.sh [项目根]                                四十五格都跑
#   bash .claude/gate.d/code-tooling.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-tooling.sh --check <格名>[,<格名>…] [项目根]       只跑点名的格
#   bash .claude/gate.d/code-tooling.sh --list-items [--check <格名>…] [项目根]  逐行打分项的三格可点的项（格名<TAB>项），不起自证、不算键（这一道「逐项点名」一节）
#   bash .claude/gate.d/code-tooling.sh --item <项> [--item …] [项目根]         只跑点名的项，给几次取并集；项写错列出可点的项、退 2
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ code-tooling.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi

STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd -P)"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
# research-script-selftests 格分批的并行度取每台默认重活线程数（heavy_thread_default，用这一道所在仓里那一份）
source "$STAGE_REPOSITORY/scripts/hosts/heavy-threads.sh"
# 开关复跑两格的判法入口（单元键里的 judge，两格文件头「按条复用」）：令牌照字面写，scripts/gate/lib_function_units.py 的 registered_judging_entries 从这里现取、它的自证逐个收闭包
BREAK_SWITCH_REPLAY_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-shared-libraries.sh::break_switch_replay_judge_code"
STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-shared-libraries.sh::stage_break_switch_replay_judge_code"
# selftest-cells-alone 格的判法入口（那一格文件头「按条复用」），写法同上
SELFTEST_CELLS_ALONE_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh::selftest_cells_alone_judge_code"
# 另四格的判法入口，写法同上；格函数在库里经变量取它们
FUNCTION_UNITS_AGREE_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh::function_units_agree_judge_code"
REUSE_SELFTESTS_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh::reuse_selftests_judge_code"
MUTATION_LIST_ITEMS_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh::mutation_list_items_judge_code"
ENTRY_SCRIPTS_JUDGE_TOKEN="judge:.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh::entry_scripts_judge_code"

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
  "给排在环境检查后面的那几道跑测试的阶段改名，让它们按 gate.sh 的 find … | sort 排到 test-environment.sh 前面；环境检查那一道改了名就同步改这一格的 ENVIRONMENT_STAGE"
stage_cell directory-layout cell_directory_layout "research/ 与 scripts/ 的顶层、说明文件、层级与不放产物合目录规矩" \
  "照 .claude/rules/directory-layout.md 挪文件、补说明文件；判法与弄坏开关在 scripts/gate/directory-layout-check.py 文件头"
stage_cell crate-roles cell_crate_roles "crates/ 下每个包在登记表里恰有一行角色，实验装置之外的二进制目标都登了放行表或滞后表、空库带测试的只许测试支撑、测试支撑至少两个无条件的依赖者、产品包不经 [dependencies]、[build-dependencies] 依赖测试支撑包、没有包依赖实验装置包" \
  "照 .claude/rules/directory-layout.md「crates/：各包放什么」改：新包在 .claude/gate.d/crate-roles.tsv 登一行，二进制与测试挪到红行点名的位置；门禁与崩溃入口起的工具、产品自己的可执行入口逐个登 .claude/gate.d/crate-roles-allowed.tsv，一时改不完的存量逐个目标登 .claude/gate.d/crate-roles-lag.tsv（只缩不涨，整包行不收）；判法与弄坏开关在 scripts/gate/crate-roles-check.py 文件头，改完单跑 bash .claude/gate.d/code-tooling.sh --check crate-roles"
stage_cell committed-file-size cell_committed_file_size "被提交的东西里没有超过 100 MiB 的文件：HEAD 加暂存区里跟踪的每个文件，与还没推上去的每笔提交里新增或改过的 blob" \
  "超限的文件 gzip -n -9 <文件> 压成 .gz 入库、读它的地方改成认 .gz（例：scripts/experiment-apparatus/replay.sh 的 read_stored），引用照 .claude/rules/path-moves.md 全仓改名；已经进了没推的提交的，推之前重做那几笔提交；判法与弄坏开关在 scripts/gate/large-file-check.py 文件头"
stage_cell git-hooks-installed cell_git_hooks_installed "进仓的 git 钩子装上了：core.hooksPath 是主工作树钩子目录的绝对路径、pre-commit、pre-merge-commit、prepare-commit-msg、pre-applypatch、pre-push 都在、有执行位、进仓的模式是 100755（提交、合并、git am 与推送前的大文件判法靠它起作用）" \
  "python3 scripts/gate/git-hooks-install.py install 写 core.hooksPath 再核一遍（共用的 .git/config，两台读同一份；经任务服务交）；钩子文件没有执行位的 chmod +x，进仓的模式不对的 git add --chmod=+x；主工作树检出了不带钩子目录的老提交的，先 git switch 回带它的分支；判法与弄坏开关在 scripts/gate/git-hooks-install.py 文件头"
stage_cell stage-location-before-cd cell_stage_location_before_cd "门禁阶段在 cd 到项目根之前取好自己的位置，cd 之后不再用 dirname \"\$0\" 找旁边的文件" \
  "在 cd 之前取阶段自己的位置存进变量，cd 之后用那个变量；点名的文件:行改完再单跑这一格"
stage_cell heavy-thread-defaults cell_heavy_thread_defaults "脚本、门禁、钩子与 crates 里自己定并行度默认值的写法都经每台默认重活线程数的登记位取，排除表指得到、只缩不涨" \
  "shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default、python 经 scripts/hosts/heavy_threads.py 的 default_heavy_threads 取，起命令里不写线程（整条经 task-server/task submit 交，线程由服务定）；不是重活的登记进 .claude/heavy-thread-exempt（<路径>:<锚串>  # 理由），排除只缩不涨"
stage_cell hardcoded-quota cell_hardcoded_quota "脚本、门禁与钩子里不定内存上限与台数（上限、机器由 task-server 按记录定），排除表指得到、只缩不涨" \
  "删掉脚本里定的上限与台数，整条经 task-server/task submit -- <命令> 交（服务按记录给上限、撞了放大重排，机器与分片也归它）；自证造的假值与这一轮不改的文件，在排除表 .claude/gate.d/hardcoded-quota-exempt.tsv 登记理由（<路径>:<锚串>  # 理由），排除只缩不涨；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell memory-bounded-parallelism cell_memory_bounded_parallelism "按内存上限定开几个进程或线程的入口都登记在 .claude/memory-bounded-parallelism.tsv，表每行两列（入口、理由）成形、入口在，登记的入口代码里还有这类写法" \
  "确是每件占一大块内存、31 份同时开会撞上限的，把入口登进 .claude/memory-bounded-parallelism.tsv（入口、理由两列，写法在它文件头）；不是的改成照每台默认重活线程数开（.claude/heavy-thread-default）；登记了而代码里已经没有的删掉那一行；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell function-units-two-implementations-agree cell_function_units_two_implementations_agree "函数单元的两份实现（crates/singlefs-unit-graph 与 scripts/gate/lib_function_units.py）原样与产品行为视图两趟：dump 逐单元相同，登记入口的闭包与键逐个相同" \
  "照点名的单元查两边的切分与闭包规则，以 crates/singlefs-unit-graph 为准改 python 那一份；判法与样本写在这一格的注释里"
stage_cell rust-name-collisions cell_rust_name_collisions "同一个 Rust 编译目标里不同模块各自定义的同名条目（fn、struct、enum、union、const、static、type、trait，私有的与只在测试里的也算）判红，逐处列出文件:行" \
  "给撞名的几份各起一个说出它是什么、做什么的名字，改到这个目标里只剩一份叫这个名字，调用处一起改；不登豁免表；判法、算不算撞名的依据与弄坏开关在 scripts/gate/rust-name-collisions.py 文件头"
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
stage_cell prune-server-selftest cell_prune_server_selftest "判定库服务 prune-server/ 的自证过：编服务二进制、起 --selftest，末行 failed=0、cells 大于 0、退出码对得上（目录不在退 77）；判的是判定库服务自己，不入库" \
  "红的格照输出里逐格点名的修 prune-server/（归工具实现员），单跑看细节：task-server/task submit --cwd <仓根>/prune-server -- cargo run --release --offline --bin prune-server -- --selftest；预编库取值没判过的先照出路修环境；判据在 code-tooling.sh 里这一格函数上方的注释"
stage_cell shared-data-root-literals cell_shared_data_root_literals "脚本、钩子、门禁与服务不经 ~ 与 \$HOME 取数据根下共享的那几项（经 lib_work_root.py 的 shared_data_root() 取），排除表指得到、只缩不涨" \
  "经 lib_work_root.py 的 shared_data_root() 取：python 载入 .claude/hooks/lib_work_root.py 调 shared_data_path(<名>)，shell 写 python3 <仓根>/.claude/hooks/lib_work_root.py shared <名>（工作根不带参数），取不到就停；确实不是取目录的，在排除表 .claude/gate.d/shared-data-root-literals-exempt.tsv 登记理由（<路径>:<锚串>  # 理由），排除只缩不涨；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell cp-preserve-mode cell_cp_preserve_mode "项目脚本、钩子、门禁、样本根的搭建脚本与服务里的 cp 不带保留权限的选项（-a、-p、--archive、不带值或带 mode / all 的 --preserve）" \
  "要时间戳的写 --preserve=timestamps、要链接的再加 links（整棵树 cp -dR --preserve=timestamps,links），都不要的写 cp -R；python 改用 shutil.copy2 / copytree"
stage_cell peer-launch-direct cell_peer_launch_direct "项目脚本与门禁不自己起 multi-host-run.sh --peer 往第二台起活（照服务的分片协议，派哪台由服务定）" \
  "照 task-server/task_server_lib.py 文件头「分片」改：文件头写「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell verdict-library-code cell_verdict_library_code "登记表 verdict-library-scripts.tsv 写了判定库的行第三列都是 prune-server，点名的脚本里没有取不到库照跑的分支、prune-server 行没有本地库读写" \
  "取不到库名、服务连不上一律退出（连不上退 75），判定只经 prune-server 的客户端读写；一族接完（导进服务、--verify、删旧目录）再翻第三列；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell temporary-service-runners cell_temporary_service_runners "runner 表里套了临时服务包装的自证，与要临时服务的表 selftest-temporary-service.tsv 里登记成 required 的两边对齐（裸写、缺条、包装了表里没有的都点名；own-server 不要求包装）" \
  "照逐条的出路改这一道的 SELFTEST_RUNNERS：表里 required 的写成「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest」，缺的补上；表里没有的包装要么先在 .claude/hooks/selftest-registry.tsv 登记再往 .claude/hooks/selftest-temporary-service.tsv 加一行，要么去掉；own-server 的不要求包装；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell temporary-service-wrapper-form cell_temporary_service_wrapper_form "要临时服务的自证的整条写法在七处逐字相同（登记表文件头、拒绝说明、骨架、客户端文件头、自证登记表、runner 表、包装前缀常量）、包装的解析收得下、整条里没有带尖括号占位的 --server-binary" \
  "照逐处的出路把点名的那一处改回包装用法行给的整条（不带 --server-binary、不写尖括号占位，二进制写在整条之外）；包装改了用法就各处一起改；判法与取的几处在 code-tooling.sh 里这一格函数上方的注释"
stage_cell repository-copy-closure cell_repository_copy_closure "scripts/、.claude/gate.d/、.claude/hooks/ 里建仓副本的写法都经 scripts/hosts/lib_repository_copy.py 算闭包（拷源点名仓根条目的 cp、rsync、git archive、shutil.copy 一族与手抄的拷贝清单字面量判红），排除表一行一处、带理由、只缩不涨" \
  "python 调 lib_repository_copy.copy_closure 再 copy_entries，shell 起 python3 scripts/hosts/minimal-copy.py <仓根> <副本> --with <条目>=<为什么>…；确实不是建仓副本的登记进 .claude/gate.d/repository-copy-closure-exempt.tsv（只缩不涨）；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell selftest-cells-alone cell_selftest_cells_alone "有选格手段的自证（写着「<名字>_SELFTEST_CELLS=<格名…」的）逐格单跑与整份跑判定相同：单跑红而整份里绿、单跑绿而整份里红的格点名判红" \
  "让点名的那一格自己建它要读的状态（变量、文件、环境），不读前面的格留下的，前面的格收拾干净自己留下的（.claude/rules/test-script-discipline.md「用例原子化：每一条都能单独跑」）；列不出格名的照出路改那份自证写错格名时的输出"
stage_cell selftest-parallelism cell_selftest_parallelism "登记表 selftest-registry.tsv 里登记成 selftest 的每份脚本，成功句报并行度或文件头写明不能并行（# selftest-serial:），排除表指得到、只缩不涨" \
  "照 .claude/rules/test-script-discipline.md「编译与算法都要快」：互不依赖的格与开关轮按每台默认重活线程数并行跑、成功句报「并行 N 路」；确实不能并行的在文件头写 # selftest-serial: <为什么，至少 8 个字>；存量登记进 .claude/gate.d/selftest-parallelism-exempt.tsv（只缩不涨）；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
stage_cell long-run-split-exempt cell_long_run_split_exempt "跑过两小时也不拆的豁免表 .claude/long-run-split-exempt 的每一行写对：有 #、理由至少 8 个字、理由里有存在的 records/<文件>.md:<行>（行号不超过文件行数）、项名不重复、不是目录也不是门禁文件名" \
  "每行写成「<项名>  # <理由，至少 8 个字，带用户那句话的出处 records/<文件>.md:<行>>」，项名取入口 --list-items 列的那一个，一项一行、不写目录与门禁文件名（.claude/rules/long-run-split.md「豁免」）；项拆开或不再超时的，那一行同一次删掉；判法与样本在 code-tooling.sh 里这一格函数上方的注释"
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

# 按格组 source 九份库；它们读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 都在上面设好
source "$STAGE_DIRECTORY/lib/code-tooling-shared-libraries.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-selftest-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-rules-and-agents-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-fixtures-and-layout-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-source-scan-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-unit-verdict-and-entry-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-stage-break-switch-replay-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-break-switch-replay-cells.sh"
source "$STAGE_DIRECTORY/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh"


# ── 逐项点名（.claude/rules/verification-reference.md「门禁的结构」的 --list-items 与 --item）：四十五格里只有三格分项，别的格只能整格跑（--check）
#   research-script-selftests   <脚本路径>：runner 表里起这份脚本的那几条自证（按包装 -- 后面那条原命令的第二个词认）
#   stage-break-switch-replay   <阶段文件名>，或 <阶段文件名>:<开关名>=<值>：有样本目录的那几道阶段，与它文件头开关表里的各条
#   break-switch-replay         <脚本路径>，或 <脚本路径>:<开关名>=<值>：复跑登记表里在的那几份，与它文件头开关表里的各条
#                               （只点单条的那一份只起基准趟与点到的那几条，没点的不算判过；同一份也点了整份的按整份；这一格文件头「按条复用」的「逐项点名」）
#   --list-items：逐行打「<格名><TAB><项>」（先整份、再它的各条），只读 runner 表、复跑登记表与文件头，不起自证、不起 stage-selftest.sh、不算键、不连 prune-server；
#     给了 --check 只列点名的格里分项的那几格；点名的格里一格分项的都没有：列出分项的三格、退 2。
#   --item <项>：只跑点名的项（给几次取并集）；三格里哪一格认得它就进哪一格（同一个脚本路径两格都认的两格都跑），一项都没点到的格不跑；给了 --check 只认点名的格。
#     点名单跑一项与它在整格里跑的判定相同：同一套键、同一份判定库、同一份判法；复用、现判、记库照整格的走法。项写错（这几格都不认得）：列出可点的项、退 2，一格都不跑。
#   样本根（judged_root_is_sample）里的 .gate-items（一行一项，空行与 # 开头的行不算）当作命令行上的 --item；命令行给了 --item 就不认它，真仓与 --staged 下不认。
#   样本：runner-item-picked、break-switch-replay-item-co-run-green、break-switch-replay-item-single-switch-green、break-switch-replay-item-single-summary-unresolved-red、
#     break-switch-replay-item-single-summary-declared-green、break-switch-replay-item-single-summary-declared-wrong-red、
#     stage-break-switch-replay-item-green（点名单跑一项，报只跑了那一项）；
#     runner-item-unknown、break-switch-replay-item-unknown、break-switch-replay-item-unknown-switch、stage-break-switch-replay-item-unknown（项写错，退 2、点名写错的项、列出可点的项）。
#   弄坏开关 CODE_TOOLING_BREAK=items-ignored：点名不起作用，点到的格照跑整格（点名样本判错）；
#     =item-unknown-accepted：写错的项悄悄丢掉、不退 2，一项都没认出就照跑整格（写错样本判错）。
#   复跑格的项按拆分族列（break-switch-replay 格文件头「拆分族」）：登记路径加它的部分文件里文件头开关表的每条开关都是 <脚本路径>:<开关名>=<值> 一项；
#   样本 break-switch-replay-split-family-item-green（开关表写在部分文件里，--item 点它的开关，照整格报点了 1 / 1 条）。
#     弄坏开关 CODE_TOOLING_BREAK=list-items-split-family-unread：复跑格的项只读登记路径那一份的文件头，那一项写成不认得、退 2。
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
        selftest_inner_into "$runner"; read -r -a words <<<"$selftest_inner"
        [[ -n "${words[1]:-}" && -z "${seen[${words[1]:-}]+set}" ]] || continue
        seen[${words[1]}]=1
        printf '%s\n' "${words[1]}"
      done
      ;;
    stage-break-switch-replay|break-switch-replay)
      python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$1" "$BREAK_SWITCH_HEADER_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" "${CODE_TOOLING_BREAK:-}" <<'PY'
import glob, os, sys, types
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
exec(compile(sys.argv[3], "break-switch-header-library", "exec"), table.__dict__)
owners = []
if sys.argv[2] == "stage-break-switch-replay":
    owners = [(name, f".claude/gate.d/{name}") for name in sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh"))
              if os.path.isdir(f".claude/gate.d/fixtures/{name}")]
elif os.path.isfile(".claude/gate.d/break-switch-replay-files.tsv"):
    for line in open(".claude/gate.d/break-switch-replay-files.tsv", encoding="utf-8"):
        path = line.rstrip("\n").split("\t")[0]
        if line.strip() and not line.startswith("#") and os.path.isfile(path) and path not in dict(owners):
            owners.append((path, path))
# 复跑格的项是登记路径那一份与它的开关：开关表可以写在入口拆出的部分文件的文件头（break-switch-replay 格文件头「拆分族」），整族的各条都可点；
# 门禁阶段的开关表只读阶段文件的文件头。弄坏开关 list-items-split-family-unread：复跑格也只读登记路径那一份的文件头
families = table.SplitFamilies(table.load_split_family(sys.argv[4]), independent=[path for _owner, path in owners], alone=sys.argv[5] == "list-items-split-family-unread")
for owner, path in owners:
    printed = [owner]
    for entry in (families.family_entries(path) if sys.argv[2] == "break-switch-replay" else table.switch_entries(open(path, encoding="utf-8", errors="replace").read())):
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
#   弄坏开关 CODE_TOOLING_BREAK=sample-service-unchecked：不核，那份样本照判 stage-order、退 0，退 2 与点名的两条 want 找不到。
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
# 本趟各格共用的临时目录：prune-server 二进制只编一次（code_tooling_prune_server_binary）、要临时服务的自证登记表只读一次（temporary_service_table_load）、
# gate 基准只算一次（exempt_table_baseline）；格是各自的子 shell，经这个目录里的文件共用。建不成就不设，各格照自己的办法取。收尾随这一道删
if CODE_TOOLING_RUN_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/code-tooling-run.XXXXXX")"; then
  export CODE_TOOLING_RUN_SCRATCH
  trap 'rm -rf -- "${CODE_TOOLING_RUN_SCRATCH:?}"' EXIT
else
  CODE_TOOLING_RUN_SCRATCH=""
fi
stage_cells_run "$ROOT"
