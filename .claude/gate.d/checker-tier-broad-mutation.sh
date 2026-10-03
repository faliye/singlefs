#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓与广谱变异库目录里那一版的结论，库目录不在 git 树里，上一次的结论不替这一次作保
# run-condition: command python3 git
# gate-stage: 广谱变异（提交门禁：这一次暂存树那一版里每一条变异都有结论、没有一条没红）
# gate-category: checker-tier 类
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 手写的几十行逐条改坏 crates/、自己起 cargo、要点名的测试红；这里不起 cargo，读 cargo-mutants 全量清单在广谱变异库目录里那一版的结论，两张表、两个判据、两个复用对象都不同
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判变异表成形、锚点唯一；这里不读变异表，读驱动 --summary 的判定行与库目录那一版的源码清单
# gate-similar: checker-tier-formal-verification.sh 接法照抄它（判法在研究脚本里、按 stage-inputs.tsv 复用、environment= 记工具版本、判绿写全绿标记）；它起 Verus 判证明，这里只读结论，并进去会让两件不相干的事共用一个复用判定
# gate-similar: checker-tier-layer0-replay.sh 它的快档逐条核崩溃枚举用例的全绿标记、不作数的报本次未跑；这里的结论在驱动的库目录里，不是这一版、有没结论的判红不报本次未跑（用户定提交前跑完才能提交）
#
# gate-cell: library-version-is-staged-tree 广谱变异库目录此刻那一版的源码与被判的暂存树在导出范围上逐文件相同、导出范围与登记的相同、记下的工具链与全局键与此刻的相同
# gate-cell: gate-verdict 驱动 --summary 的 BROAD_MUTATION_GATE 判绿：那一版每一条都有结论（现判或沿用）、没有一条没红，判定行、汇总行与退出码互相对得上
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判据（广谱变异剪枝设计轮判决 research/prompts/tooling-broad-mutation-prune-r3-main-verification.md 第四节、正文 research/prompts/_tooling-broad-mutation-prune-r3-body.md 第三节 C6；
#   治理轮判决 research/prompts/m3-broad-gate-gov-r1-main-verification.md 第二节第 1–10 行）：
#   这次改到的变异每一条都有这一版的结论，沿用的结论都在，有任何一条没红就红。
#   「这一版」是驱动 research/scripts/broad-mutation.py 的库目录（${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}）里
#   tree.source 指的那一版：源码指纹（导出范围逐文件 sha256）、rustc -Vv、判法版本与全局键（驱动文件头「指纹」）。
#   开格之前把 tree.source 拷一份钉住，两格都判钉住的那一版；两格跑完再读一次，与钉住的不同（别的会话这段时间里 --sync-tree 换了版本指针）判红。
#   library-version-is-staged-tree：被判仓的 git 索引（主工作区就是暂存区，--staged 下是临时 worktree 的索引）在导出范围上的逐文件 sha256
#     与那一版的 manifest.tsv 逐文件相同（索引里有合并冲突判红）、逐文件算出的源码指纹等于 tree.source 记的；导出范围按驱动的 export_paths 在被判仓上现算，
#     与 tree.source 记的相同；那一版 toolchain.txt 与此刻 rustc -Vv（在库目录的 tree/ 里起，与驱动同一处）相同；
#     那一版记了全局键（tree.source 的 global_key 与 global-key.txt 是同一次写的），此刻现算的全局键（驱动的 global_key_lines：工具链、Cargo.toml / Cargo.lock / .cargo、
#     构建环境（驱动的 build_environment_of：research/scripts/admission.py 的 build_environment_lines 在库目录的 tree/ 上按此刻的环境算，加文档测试的 RUSTDOCFLAGS）、
#     cargo nextest / cargo mutants / llvm-profdata 的版本、驱动旁边的忽略用例排除表、升档常量、判法版本；限时参数取那一版记的）与它逐行相同；
#     按驱动的 version_key 算出来的指纹等于 tree.source 记的。tree.source.pending 在判红。
#   gate-verdict：驱动 --summary（--home 指到同一个库目录）打恰好一行 BROAD_MUTATION_SUMMARY、一行 BROAD_MUTATION_GATE；
#     verdict=green、missed=0、without_conclusion=0、concluded=total、total>0，退出码 0；verdict=red 而退 8 是驱动判红，照抄它的 NO_CONCLUSION / MISSED 明细；
#     判定行与退出码对不上（verdict 不是 green / red、写 red 而退 0）、计数自相矛盾（判绿而 missed 或 without_conclusion 不是 0，NO_CONCLUSION、MISSED 明细行数与计数不同、清单 0 条）、
#     汇总行与判定行各数对不上（两行的 total、汇总行抓到加没红加无效对判定行的 concluded、汇总行超时加内存撞顶加只过了第一段加驱动错加没跑对 without_conclusion、
#     汇总行八栏之和对它的 total、判定行 carried 加 fresh 对 concluded）、汇总行的 fingerprint 不是钉住那一版的前 16 位，都判红。flipped（里程碑全量翻面的条数）只报不判。
#     超时、内存撞顶照驱动算没结论（用户 2026-10-02 定）；驱动对超时、撞顶的那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到（用户 2026-10-03 定，驱动文件头「升档」）：
#     --summary 逐条打 ESCALATED <TAB> 序号 <TAB> 标签 <TAB> 变异名、判定行带 escalated=<条数>（条数与 ESCALATED 行数不同判红；有 ESCALATED 而判定行没有 escalated= 判红）。
#     这一格逐行复核库目录登记表 escalated-verdicts.tsv（列名、标签、升档常量取此刻驱动里的 ESCALATION_COLUMNS 与四个常量）：
#     按版本键只认 version 是钉住那一版的行（别的版本的行不看，连段数也不核）；每一行要：段数对、标签认得；基线按登记行的 baseline_row（行号）找那一版 baseline.tsv
#     的那一行：成形、步名、结局 ok、线程数都与登记的相同，秒数（超时）或峰值字节（撞顶）与登记的 baseline 相同（基线按线程数分开记、只追加，驱动文件头「基线」）；
#     升档那趟的限时或上限比原来的大、不低于基线的 K 倍；
#     两份日志在 versions/<那一版>/escalation/ 下、sha256 与登记的相同、末尾有驱动写的 BROAD_MUTATION_EVIDENCE 那一行（版本键、序号、步名、哪一趟、限时或上限、退出码 253 / 250）、
#     有包装打的「超过限时 <限时> 秒」或「撞了内存上限 <上限>」；日志不在的是没有驱动写的证据（手写的行）。一行复核不过判红；
#     每一条 ESCALATED 都要有一行复核过的这一版登记行（序号、标签、变异名相同），没有判红。
#     出路：证据不齐、仍是超时或撞顶的（NO_CONCLUSION）加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun；登记行复核不过的照它点的那一样修或 --rerun 重判那一条。
#     结果行的校验列（驱动文件头「结果行的校验列」，用户 2026-10-03 定）：这一格另用同一把密钥逐行核钉住那一版的 results.tsv（密钥位置取驱动的 result_key_path，
#     ${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key；样本里是 .broad-sample/result.key）：每一行十二段、末段是前十一段的 HMAC-SHA256。
#     十一段的（没有校验列）、校验列对不上的、段数不对的、密钥不在或权限不是 600 时的每一行都判红；结果表不在就没有行可核。这一段的算法在这里另写一份，不调驱动的函数。
#     出路：手写或拷来的行照驱动 --summary 的出路删掉、--item <序号> --rerun 重判；密钥不在或每一行都对不上（有校验列之前建的库目录、密钥丢了或换了）就整库重建。
# 查标记（--marker-check，提交流程第 1 步用，.claude/skills/crash-test/commit-stages.md）：先核 stage-inputs.tsv 本阶段那一行的全部路径（含导出范围之外的驱动、
#   mutation-prune.py、排除表、包装脚本，连同登记表与这一道自己）上工作区等于暂存区、没有未跟踪文件，不等退 1、不查标记（标记按工作区算指纹，这一道判的是暂存区；
#   两边相同时按工作区算的就是按暂存区算的）；相同才起 admission.py stage-marker-check：有作数的全绿标记退 0（复用），没有退 3（照第 2b 步起驱动）。不跑格、不起驱动。
# 这一道不起驱动跑变异：跑由主 agent 带前缀起（.claude/skills/crash-test/commit-stages.md 第 2b 步：git commit-tree 包这一次的暂存树、--sync-tree --commit 到它、--changed-since HEAD 只现判改到的），
#   这一道只读结论；只问版本的 rustc -Vv、cargo nextest / cargo mutants / llvm-profdata --version 照起。--summary 只读库目录，不要前缀。
#
# 复用（接法同 checker-tier-formal-verification.sh）：真仓与 --staged 下整道跑时，stage-inputs.tsv 本阶段那一行的路径与 environment=（--tool-versions 那几行）
#   都没变就整道退 77；判绿之后记环境；写全绿标记（admission.py stage-marker-write）之前核登记的输入路径（连同登记表与这一道自己）上工作区等于暂存区、
#   没有未跟踪文件：标记按工作区算指纹，格判的是暂存区，两边不同就不写标记，打「本次未写全绿标记：<原因>」。库目录不在 git 树里，复用期间靠 stage-must-run.sh 的 24 小时上限兜。
#
# 样本（fixtures/checker-tier-broad-mutation.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）：
#   样本根放了标记 .broad-mutation-sample（只供样本的开关：被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认）时，
#   库目录改读 .broad-sample/home/，驱动 --summary 换成 bash .broad-sample/summary.sh（打录好的输出、退录好的码），rustc -Vv 换成 .broad-sample/rustc-vv.txt，
#   工具版本换成 .broad-sample/tool-versions.txt（一行一个），排除表换成 .broad-sample/exclusions.tsv；样本根有 .broad-sample/prepare.py 的，开格之前先起它
#   （python3 .broad-sample/prepare.py <驱动>：照今天驱动的 fingerprint_of_manifest、global_key_lines、version_key 造库目录里的那一版）；
#   有 .broad-sample/write-marker 的，判绿之后对样本根走一遍写全绿标记那一段（标记写进样本自己的 .git）；
#   有 .broad-sample/marker-check 的，开格之前（prepare.py 之后）走查标记那一支（同 --marker-check），不跑格；
#   有 .broad-sample/environment 的（一行一个 名=值，值里的 {root} 换成样本根），全局键里的构建环境与文档测试变量按它算、不按此刻的环境（写死全局键字面值的样本用，
#   样本根的上层目录里不放 .cargo/config 与 rust-toolchain：样本拷进临时目录跑）。
#   library-version-is-staged-tree-literal-global-key-green 不起 prepare.py：库目录那一版的 tree.source、manifest.tsv、toolchain.txt、global-key.txt 与版本指纹由 setup.sh
#   照字面值写（全局键 27 行全文、源码指纹、全局键 sha256、版本指纹都是写死的十六进制，不经驱动的函数算），驱动的 global_key_lines、fingerprint_of_manifest、
#   build_environment_of、version_key 改了写法这一份就红：它是这一格对「驱动改了算法」的钉子（与驱动 --selftest 的 selftest_global_key_class_cells 各管一边）。
#   构建环境里放齐了 admission.py 的 build_environment_lines 认的各类（库目录 tree/ 往上每一层的 .cargo/config.toml、库目录那一层的 rust-toolchain.toml、配置里
#   build.rustc-wrapper 指的程序、CARGO_HOME 下的配置、CARGO_PROFILE_*、CARGO_ENCODED_RUSTFLAGS、RUSTFLAGS、RUSTC_WRAPPER 与 CARGO_TARGET_*_RUNNER 连同它们指的程序、
#   RUSTC 的值与 -V，加文档测试的 RUSTDOCFLAGS），那一份逐类写错也红（治理第三轮攻方 j1c-admission-mutants.py 的 a1–a9 各让它红一次）。
#   查标记那几份（setup.sh 造暂存区，prepare.py 照 admission.py stage-marker-write 写标记）：marker-check-staged-differs-red 照攻方 exp1 的现场——工作区与暂存区相同时
#   写下标记 A，再暂存另一份 B、工作区写回 A，退 1、不查标记；marker-check-clean-reused 两边相同、标记在，退 0；marker-check-clean-unmarked 两边相同、没写标记，退 3。
#   gate-verdict-escalated-* 与 gate-verdict-result-mac-* 那几份（同一份 setup.sh 只换情形：造库目录里那一版的基线表、两趟日志、登记表、带校验列的结果表与样本的密钥，
#   sha256 用 sha256sum、校验列用 python3 的 hmac 现算）：-escalated-green 一条超时、一条撞顶都证据齐，另有一行别的版本的坏行（不看）；
#   -escalated-multiple-red 升档那趟的限时不到基线的 20 倍；-escalated-log-digest-red 升档那趟的日志登记之后又改了；
#   -escalated-old-version-red 第 1 条的登记行是上一版的（证据在上一版的目录里、本身齐），这一版没有；-escalated-handwritten-red 第 1 条的登记行指的日志不在（手写的）；
#   -escalated-count-red 判定行 escalated=3、ESCALATED 只有 2 行；
#   -escalated-threads-rerun-green 攻方 J2-a 现场：同一版换了 --threads 16，16 线程的基线追加在后、第 1 条在 16 线程下又升档登记了一行、8 线程的峰值表被包装改写，
#   旧登记行指的 8 线程那一行没动，两行都复核得过；-escalated-baseline-row-changed-red 第 1 条登记行指的那一行基线被改了线程数；
#   治理第三轮攻方 j2b-inputs 那十份「驱动不会写」的现场各一份：-escalated-baseline-lie-red 登记的基线与它指的那一行不同；-escalated-not-larger-red 两趟限时一样；
#   -escalated-memory-multiple-red 撞顶那一条不到基线峰值的 2 倍；-escalated-no-trailer-red 日志末尾没有驱动那一行；-escalated-trailer-wrong-red 驱动那一行写成别的；
#   -escalated-no-wrapper-red 日志里没有包装那一句；-escalated-columns-red 登记行少一段；-escalated-no-escalated-field-red 判定行没有 escalated=；
#   -escalated-name-differs-red ESCALATED 的变异名与登记的不同；-escalated-logs-elsewhere-red 日志挪到 escalation/ 之外、登记行跟着指过去；
#   -result-mac-forged-red 结果表末尾手追加一行没有校验列的 caught、一行校验列对不上的 caught；-result-mac-key-missing-red 样本的密钥不在。
#   library-version-is-staged-tree 那几份有 setup.sh（git init 再 git add -A 造暂存区；改动前的逐文件 sha256 写进 .broad-sample/manifest.tsv，情形写进 .broad-sample/variant）：
#   -green 原样；-unstaged-change-green 工作区改一个文件不暂存；-new-file-red 另暂存一个那一版没有的文件；-file-differs-red 另暂存改过内容的一个文件；
#   -toolchain-red 那一版记的工具链与 rustc-vv.txt 不同；-export-range-red 那一版的导出范围少了 .cargo；-pending-red 留着 tree.source.pending；
#   -version-key-red 版本指纹照全局键进指纹之前的式子算（全局键本身记得对）；-merge-conflict-red 索引里 lib.rs 在合并冲突（第 3 段与那一版相同）；
#   -source-fingerprint-red tree.source 记的源码指纹与 manifest 不是同一次写的；-global-key-red 那一版记全局键时排除表不在、此刻在（排除表改了而版本没重判）；
#   -global-key-missing-red 那一版是全局键进指纹之前的驱动换的（没有 global_key）；
#   -global-key-file-red global-key.txt 是此刻的全文，tree.source 的 global_key 与版本指纹却照另一份全文算（第二轮攻方 H2-c：两样不是同一次写的）；
#   -marker-clean-written 带 write-marker、登记了这一道，工作区与暂存区相同：写标记；-marker-unstaged-skipped 同上、工作区改了一个输入文件不暂存：判绿、不写标记。
#   gate-verdict 那几份：-green 全有结论（沿用加现判）；-missed-red 一条没红、退 8；-no-conclusion-red 三条没结论、退 8；
#   -silent-driver-red 判定行写 green、退 0，而 missed=1（驱动的判红失灵）；-line-missing-red 只有汇总行、没有判定行、退 0（旧驱动）；
#   -stale-fingerprint-red 汇总行的 fingerprint 不是 tree.source 那一版；-red-exit-zero-red 判定行写 red、计数与明细一致、退 0；
#   -detail-count-red 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION；-empty-list-red 清单 0 条、判定行 green；-unknown-verdict-red verdict=pending、退 8；
#   -totals-mismatch-red 汇总行 total=5091、not_run=5085，判定行 total=6 写 green；-swapped-between-cells-red summary.sh 跑的时候把 tree.source 换成另一版、汇总判的是那一版。
# 弄坏开关 CHECKER_TIER_BROAD_MUTATION_BREAK=<项>（逗号分隔；只给样本证红用），打开后判错的样本：
#   work-tree-read                逐文件清单读工作区不读索引                  library-version-is-staged-tree-unstaged-change-green、-marker-unstaged-skipped（判红）、-merge-conflict-red（判绿）
#   files-unchecked               不比源码清单、源码指纹与版本指纹            library-version-is-staged-tree-file-differs-red、-new-file-red、-version-key-red、-source-fingerprint-red（判绿）
#   toolchain-unchecked           不比工具链（全局键与版本指纹跟着不比）      library-version-is-staged-tree-toolchain-red、-version-key-red、-global-key-red、-global-key-missing-red、
#                                                                             -global-key-file-red（判绿）、-literal-global-key-green（照绿，缺「全局键 27 行逐行相同」的 want）
#   export-range-unchecked        导出范围照 tree.source 记的、不现算          library-version-is-staged-tree-export-range-red（判绿）
#   pending-unchecked             tree.source.pending 在也照判                library-version-is-staged-tree-pending-red（判绿）
#   version-key-unchecked         不按驱动的 version_key 复算版本指纹          library-version-is-staged-tree-version-key-red（判绿）
#   merge-conflict-unchecked      索引里的合并冲突不拒、取最后一段            library-version-is-staged-tree-merge-conflict-red（判绿）
#   source-fingerprint-unchecked  逐文件相同之后不比源码指纹                  library-version-is-staged-tree-source-fingerprint-red（判绿）
#   global-key-unchecked          不核全局键                                  library-version-is-staged-tree-global-key-red、-global-key-file-red（判绿）、-global-key-missing-red（version_key 那一支照红，缺「没记全局键」的 want）、
#                                                                             -literal-global-key-green（照绿，缺「全局键 27 行逐行相同」的 want）
#   marker-unguarded              写全绿标记之前不核工作区等于暂存区          library-version-is-staged-tree-marker-unstaged-skipped（写了标记）
#   summary-red-passes            驱动退 8、判定行写 red 也算绿，判定行与退出码对不上不判
#                                                                             gate-verdict-missed-red、-no-conclusion-red、-red-exit-zero-red（判绿）
#   verdict-exit-unchecked        判定行写 red 而退 0 不判                    gate-verdict-red-exit-zero-red（判绿）
#   verdict-value-unchecked       verdict 不是 green / red 不判               gate-verdict-unknown-verdict-red（判绿）
#   counts-unchecked              不核判定行的计数与明细                      gate-verdict-silent-driver-red、-detail-count-red、-empty-list-red（判绿）
#   detail-count-unchecked        不核明细行数与计数                          gate-verdict-detail-count-red（判绿）
#   empty-list-passes             清单 0 条不判                               gate-verdict-empty-list-red（判绿）
#   summary-gate-totals-unchecked 不核汇总行与判定行的各数                    gate-verdict-totals-mismatch-red（判绿）
#   gate-line-optional            没有判定行而退 0 算绿                       gate-verdict-line-missing-red（判绿）
#   version-unchecked             不比汇总行的 fingerprint 与钉住的那一版      gate-verdict-stale-fingerprint-red（判绿）、-swapped-between-cells-red（跑完再核那一句照红，缺那一格的 want）
#   tree-source-reread            不钉 tree.source：两格各读此刻的、跑完不再核 gate-verdict-swapped-between-cells-red（判绿）
#   marker-check-unguarded        查标记之前不核工作区等于暂存区              marker-check-staged-differs-red（退 0，复用了没判过的那一份）
#   escalated-multiple-unchecked  登记行不核倍数门槛（连同「比原来的大」）    gate-verdict-escalated-multiple-red、-memory-multiple-red、-not-larger-red（判绿）
#   escalated-log-digest-unchecked 登记行不核日志的 sha256                    gate-verdict-escalated-log-digest-red（判绿）
#   escalated-version-unchecked   登记行不按版本键挑，哪一版的都认            gate-verdict-escalated-old-version-red（判绿）、-green（那一行别的版本的坏行被核，判红）
#   escalated-evidence-optional   登记行指的日志不在（或不在 escalation/ 下）也认
#                                                                             gate-verdict-escalated-handwritten-red、-logs-elsewhere-red（判绿）
#   escalated-count-unchecked     不核判定行 escalated= 与 ESCALATED 行数（连同有 ESCALATED 而没有 escalated=）
#                                                                             gate-verdict-escalated-count-red、-no-escalated-field-red（判绿）
#   escalated-baseline-last-row   基线照改前取：这一步最后一行绿记录，不看登记的行号与线程数
#                                                                             gate-verdict-escalated-threads-rerun-green（判红：旧登记行对上了 16 线程那一行）、
#                                                                             -baseline-row-changed-red（判绿：不看那一行的线程数）
#   escalated-baseline-row-identity-unchecked 不核登记行指的那一行的步名、结局与线程数
#                                                                             gate-verdict-escalated-baseline-row-changed-red（判绿）
#   escalated-baseline-unchecked  不比登记的基线与它指的那一行的值            gate-verdict-escalated-baseline-lie-red（判绿）
#   escalated-not-larger-unchecked 不核升档那趟比原来的大                     gate-verdict-escalated-not-larger-red（判绿）
#   escalated-memory-multiple-ignored 撞顶那一边的倍数门槛按 0 算             gate-verdict-escalated-memory-multiple-red（判绿）
#   escalated-trailer-unchecked   不核日志末尾驱动写的那一行                  gate-verdict-escalated-no-trailer-red、-trailer-wrong-red（判绿）
#   escalated-trailer-version-only 驱动那一行只核版本键（不核序号、步名、哪一趟、限时、退出码）
#                                                                             gate-verdict-escalated-trailer-wrong-red（判绿）、-no-trailer-red（照红，缺打全那一行的 want）
#   escalated-wrapper-unchecked   不核日志里包装打的那一句                    gate-verdict-escalated-no-wrapper-red（判绿）
#   escalated-columns-loose       登记行只要不少于 10 段就认                  gate-verdict-escalated-columns-red（判绿）
#   escalated-field-optional      有 ESCALATED 而判定行没有 escalated= 不判   gate-verdict-escalated-no-escalated-field-red（判绿）
#   escalated-sequence-only       ESCALATED 对登记行只比序号（不比标签、变异名）
#                                                                             gate-verdict-escalated-name-differs-red（判绿）
#   escalated-logs-anywhere       日志不在 escalation/ 下也认                 gate-verdict-escalated-logs-elsewhere-red（判绿）
#   result-mac-unchecked          不核结果表的校验列                          gate-verdict-result-mac-forged-red、-result-mac-key-missing-red（判绿）、
#                                                                             gate-verdict-escalated-green、-escalated-threads-rerun-green（照绿，缺「结果表 N 行逐行核过」的 want）
#   global-key-file-unchecked     不核 global-key.txt 与 tree.source 的 global_key 是同一次写的
#                                                                             library-version-is-staged-tree-global-key-file-red（判绿）
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本的格按绿记、整道退 0；gate-verdict-swapped-between-cells-red 的跑完再核那一句在格外头，照红退 1）。
#
#   bash .claude/gate.d/checker-tier-broad-mutation.sh [项目根]
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --list              逐行打格名与判什么，不起驱动、不起 rustc
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --check <格名>      只跑点名的格；格名写错退 2
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --tool-versions     打 rustc -Vv、cargo nextest、cargo mutants 与 llvm-tools 的 llvm-profdata 版本（stage-inputs.tsv 的 environment= 用；不跑变异）
#   bash .claude/gate.d/checker-tier-broad-mutation.sh --marker-check [项目根]  查标记（见上「查标记」）：退 0 复用、退 3 照第 2b 步跑、退 1 输入路径上工作区与暂存区不同（不查标记）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
DRIVER="$STAGE_REPOSITORY/research/scripts/broad-mutation.py"
ADMISSION_MODULE="$STAGE_REPOSITORY/research/scripts/admission.py"
SAMPLE_MARKER=".broad-mutation-sample"
SAMPLE_DIRECTORY=".broad-sample"
RUN_DRIVER_HOWTO="主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步起驱动：先 --sync-tree --commit 到包着这一次暂存树的提交，再 --changed-since HEAD 现判改到的、沿用其余的，跑完再跑这一道"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell library-version-is-staged-tree cell_library_version_is_staged_tree "广谱变异库目录此刻那一版的源码与被判的暂存树在导出范围上逐文件相同、导出范围与登记的相同、记下的工具链与全局键与此刻的相同" \
  "库目录不是这一次暂存树那一版：$RUN_DRIVER_HOWTO；暂存区之后又变了就重来一遍；工具链或全局键（排除表、工具版本、构建环境、升档常量）变了就在新的下照样重来（旧版的结论留在它自己的版本目录里）；会话 shell 里设的编译变量照驱动 --unit-environment 带进起驱动的单元"
stage_cell gate-verdict cell_gate_verdict "驱动 --summary 的 BROAD_MUTATION_GATE 判绿：那一版每一条都有结论（现判或沿用）、没有一条没红，判定行、汇总行与退出码互相对得上" \
  "没有结论的：$RUN_DRIVER_HOWTO（--changed-since 也重跑这一版里超时、内存撞顶的条，那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到；证据不齐、仍是这两种的加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun）；没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（整版重判）；升档登记行复核不过的照它点的那一样修，或 --item <序号> --rerun 重判那一条；--summary 退 1（栏数或结果行对不上）照驱动打的 → 修结果表"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
marker_check_requested=0
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    --marker-check) marker_check_requested=1; shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--tool-versions、--marker-check 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"
BROKEN=",${CHECKER_TIER_BROAD_MUTATION_BREAK:-},"

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
LIBRARY_HOME="${SINGLEFS_BROAD_MUTATION_HOME:-$HOME/.local/share/singlefs/broad-mutation}"
summary_command=(python3 "$DRIVER" --summary --home "$LIBRARY_HOME")
rustc_override=""
sample_tools=""
sample_exclusions=""
sample_environment=""
sample_result_key=""
if ((sample_mode)); then
  LIBRARY_HOME="$ROOT/$SAMPLE_DIRECTORY/home"
  sample_result_key="$ROOT/$SAMPLE_DIRECTORY/result.key"
  summary_command=(bash "$ROOT/$SAMPLE_DIRECTORY/summary.sh")
  [[ -f "$SAMPLE_DIRECTORY/rustc-vv.txt" ]] && rustc_override="$ROOT/$SAMPLE_DIRECTORY/rustc-vv.txt"
  [[ -f "$SAMPLE_DIRECTORY/tool-versions.txt" ]] && sample_tools="$ROOT/$SAMPLE_DIRECTORY/tool-versions.txt"
  [[ -f "$SAMPLE_DIRECTORY/exclusions.tsv" ]] && sample_exclusions="$ROOT/$SAMPLE_DIRECTORY/exclusions.tsv"
  [[ -f "$SAMPLE_DIRECTORY/environment" ]] && sample_environment="$ROOT/$SAMPLE_DIRECTORY/environment"
  [[ -f "$SAMPLE_DIRECTORY/marker-check" ]] && marker_check_requested=1
  if [[ -f "$SAMPLE_DIRECTORY/prepare.py" ]] && ! env -u BROAD_MUTATION_BREAK python3 "$SAMPLE_DIRECTORY/prepare.py" "$DRIVER"; then
    echo "  ✗ 样本现场的后半没造成：$ROOT/$SAMPLE_DIRECTORY/prepare.py 退非 0"
    echo "     → 怎么办：它载入 $DRIVER 调 fingerprint_of_manifest、build_environment_of、global_key_lines、version_key 造库目录（查标记那几份载入 admission.py 写标记）；看上面的报错，驱动里这几个函数改了名或参数就照着改样本的 setup.sh。"
    exit 1
  fi
fi

# ── 格 library-version-is-staged-tree ──
cell_library_version_is_staged_tree() {
env -u BROAD_MUTATION_BREAK python3 - "$ROOT" "$LIBRARY_HOME" "$DRIVER" "$rustc_override" "${CHECKER_TIER_BROAD_MUTATION_BREAK:-}" "$TREE_SOURCE_READ" "$sample_tools" "$sample_exclusions" "$sample_environment" <<'PY'
import hashlib, importlib.util, json, os, subprocess, sys, types
sys.dont_write_bytecode = True
root, home, driver_path, rustc_override, broken_text, source_read_path, sample_tools, sample_exclusions, sample_environment = sys.argv[1:10]
broken = set(filter(None, broken_text.split(",")))
howto_run = "主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步：--sync-tree --commit 到包着这一次暂存树的提交，再 --changed-since HEAD，跑完再跑这一道"
specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_gate", driver_path)
driver = importlib.util.module_from_spec(specification)
try:
    specification.loader.exec_module(driver)
except Exception as error:  # 驱动被删、改坏
    print(f"  ✗ 读不进广谱变异的驱动 {driver_path}：{error}")
    print("     → 怎么办：它随仓走，被删了或改坏了从 git 找回来；导出范围、源码指纹、全局键与版本指纹的算法只认它一份，这一格不另写。")
    sys.exit(1)
source_path = os.path.join(home, "tree.source")
if "pending-unchecked" not in broken and os.path.exists(os.path.join(home, "tree.source.pending")):
    print(f"  ✗ {home} 里有 tree.source.pending：上一次 --sync-tree 没做完，库目录那一版判不了")
    print(f"     → 怎么办：{howto_run}（--sync-tree 重做一遍就把它收掉）")
    sys.exit(1)
try:
    with open(source_read_path, encoding="utf-8") as handle:
        source = json.load(handle)
    fingerprint = source["fingerprint"]
    recorded_paths = list(source["export_paths"])
except (OSError, ValueError, KeyError, TypeError) as error:
    print(f"  ✗ 读不出广谱变异库目录的版本指针 {source_path}：{error}")
    print(f"     → 怎么办：库目录换了地方就设 SINGLEFS_BROAD_MUTATION_HOME；还没建过就 {howto_run}")
    sys.exit(1)
version_directory = os.path.join(home, "versions", fingerprint)
problems = []
# 导出范围：驱动的 export_paths 在被判仓上现算（它读 REPOSITORY 下的登记表，这里把 REPOSITORY 换成被判仓）
if "export-range-unchecked" in broken:
    export_paths = recorded_paths
else:
    driver.REPOSITORY = root
    try:
        export_paths = driver.export_paths()
    except BaseException as error:  # 驱动拒绝时抛 Refusal 或 SystemExit
        print(f"  ✗ 在 {root} 上算不出导出范围（驱动的 export_paths）：{error}")
        print("     → 怎么办：看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行还在不在、成不成形；导出范围就取它。")
        sys.exit(1)
    if sorted(export_paths) != sorted(recorded_paths):
        problems.append(f"导出范围不同：库目录那一版记的是 {' '.join(recorded_paths)}，被判仓登记的是 {' '.join(export_paths)}")
# 被判那棵树在导出范围上的逐文件 sha256：读 git 索引（暂存区），不读工作区；忽略的目录名与符号链接照驱动的 manifest_of_tree
environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
ignored = set(driver.IGNORED_DIRECTORY_NAMES)
def ignored_path(path):
    return any(part in ignored for part in path.split("/")[:-1])
staged = {}
if "work-tree-read" in broken:
    for export_path in export_paths:
        full = os.path.join(root, export_path)
        if os.path.isfile(full):
            staged[export_path] = driver.file_digest(full)
        elif os.path.isdir(full):
            for relative, digest in driver.manifest_of_tree(full).items():
                staged[os.path.join(export_path, relative)] = digest
else:
    listed = subprocess.run(["git", "-C", root, "ls-files", "-s", "-z", "--", *export_paths], capture_output=True, env=environment)
    if listed.returncode != 0:
        print(f"  ✗ 读不了 {root} 的 git 索引（git ls-files 退 {listed.returncode}：{listed.stderr.decode(errors='replace').strip()[:200]}）")
        print("     → 怎么办：这一道判的是暂存树，被判的根要是 git 工作树（主工作区或 gate.sh --staged 的临时 worktree）。")
        sys.exit(1)
    objects = {}
    for record in listed.stdout.split(b"\0"):
        if not record:
            continue
        metadata, _, path = record.partition(b"\t")
        mode, object_name, stage_number = metadata.decode().split()
        name = os.fsdecode(path)
        if stage_number != "0" and "merge-conflict-unchecked" not in broken:
            print(f"  ✗ 索引里 {name} 在合并冲突里，暂存树判不了")
            print("     → 怎么办：先解决冲突并暂存，再按这一道的出路让驱动判这一次的暂存树。")
            sys.exit(1)
        if mode in ("120000", "160000") or ignored_path(name):
            continue
        objects[name] = object_name
    if objects:
        request = "".join(f"{object_name}\n" for object_name in objects.values()).encode()
        blobs = subprocess.run(["git", "-C", root, "cat-file", "--batch"], input=request, capture_output=True, env=environment)
        if blobs.returncode != 0:
            print(f"  ✗ git cat-file --batch 退 {blobs.returncode}，读不出索引里的文件内容")
            print("     → 怎么办：看 git 的报错（对象库坏了先 git fsck），修好再跑这一格。")
            sys.exit(1)
        output, position = blobs.stdout, 0
        for name, object_name in objects.items():
            end = output.index(b"\n", position)
            header = output[position:end].decode().split()
            size = int(header[2])
            staged[name] = hashlib.sha256(output[end + 1:end + 1 + size]).hexdigest()
            position = end + 1 + size + 1
if not staged:
    print(f"  ✗ 被判的暂存树在导出范围（{' '.join(export_paths)}）里一个文件都没有")
    print("     → 怎么办：看被判的根对不对（第一个参数）、导出范围登记得对不对；一个文件都没有就没有可判的变异。")
    sys.exit(1)
try:
    recorded = driver.read_manifest(os.path.join(version_directory, "manifest.tsv"))
except OSError as error:
    print(f"  ✗ 读不出库目录那一版的源码清单 {version_directory}/manifest.tsv：{error}")
    print(f"     → 怎么办：{howto_run}")
    sys.exit(1)
staged_source_fingerprint = driver.fingerprint_of_manifest(staged)
if "files-unchecked" not in broken:
    only_staged = sorted(set(staged) - set(recorded))
    only_recorded = sorted(set(recorded) - set(staged))
    differing = sorted(path for path in set(staged) & set(recorded) if staged[path] != recorded[path])
    if only_staged or only_recorded or differing:
        listing = [f"内容不同 {path}" for path in differing] + [f"暂存树有、那一版没有 {path}" for path in only_staged] + [f"那一版有、暂存树没有 {path}" for path in only_recorded]
        problems.append(f"源码与库目录那一版不同（{len(listing)} 个文件）：" + "；".join(listing[:20]) + ("；……" if len(listing) > 20 else ""))
    if not problems and "source-fingerprint-unchecked" not in broken and staged_source_fingerprint != source.get("source_fingerprint"):
        problems.append(f"逐文件相同而源码指纹不同（暂存树 {staged_source_fingerprint[:16]}，tree.source 记的 {str(source.get('source_fingerprint'))[:16]}）：manifest.tsv 与 tree.source 不是同一次写的")
toolchain_path = os.path.join(version_directory, "toolchain.txt")
current_toolchain = None
toolchain_matches = False
if "toolchain-unchecked" not in broken:
    if rustc_override:
        with open(rustc_override, encoding="utf-8") as handle:
            current_toolchain = handle.read().strip() + "\n"
    else:
        tree_directory = os.path.join(home, "tree")
        try:
            found = subprocess.run(["rustc", "-Vv"], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   cwd=tree_directory if os.path.isdir(tree_directory) else root)
            current_toolchain = found.stdout.strip() + "\n" if found.returncode == 0 and found.stdout.strip() else None
        except OSError:
            current_toolchain = None
        if current_toolchain is None:
            problems.append("起不了 rustc -Vv：此刻的工具链判不了")
    if current_toolchain is not None:
        if not os.path.isfile(toolchain_path):
            problems.append(f"那一版没记工具链（{toolchain_path} 不在）")
        else:
            with open(toolchain_path, encoding="utf-8") as handle:
                recorded_toolchain = handle.read()
            if recorded_toolchain != current_toolchain:
                problems.append(f"工具链不同：那一版记的是「{recorded_toolchain.splitlines()[0] if recorded_toolchain else '空'}」，此刻是「{current_toolchain.splitlines()[0]}」")
            else:
                toolchain_matches = True
# 全局键（E4）：那一版 --sync-tree 时记的全文与此刻现算的逐行相同；限时参数取那一版记的（门禁不知道主 agent 跑 ③ 给的限时参数，驱动真跑时自己核）
global_digest = source.get("global_key")
global_line_count = None
if toolchain_matches and not problems and "global-key-unchecked" not in broken:
    global_path = os.path.join(version_directory, "global-key.txt")
    if not global_digest or not os.path.isfile(global_path):
        problems.append(f"那一版没记全局键（tree.source 里{'有' if global_digest else '没有'} global_key，{global_path} {'在' if os.path.isfile(global_path) else '不在'}）：是全局键进版本指纹之前的驱动换的那一版")
    else:
        with open(global_path, encoding="utf-8") as handle:
            recorded_global = handle.read()
        if hashlib.sha256(recorded_global.encode()).hexdigest() != global_digest and "global-key-file-unchecked" not in broken:
            problems.append(f"{global_path} 与 tree.source 记的 global_key 不是同一次写的")
        else:
            if sample_tools:
                with open(sample_tools, encoding="utf-8") as handle:
                    tools = [line.strip() for line in handle if line.strip()]
            else:
                tools = driver.tool_versions(types.SimpleNamespace(llvm_bin=None))
            limits = next((line for line in recorded_global.splitlines() if line.startswith("limits ")), "limits （那一版没记）")
            # 构建环境与驱动同一处算：库目录的 tree/（没有就是被判的根），此刻的环境；样本给了 .broad-sample/environment 的按它算（写死字面值的样本用）
            build_root = os.path.join(home, "tree") if os.path.isdir(os.path.join(home, "tree")) else root
            key_environment = os.environ
            if sample_environment:
                with open(sample_environment, encoding="utf-8") as handle:
                    key_environment = dict(line.rstrip("\n").replace("{root}", root).split("=", 1) for line in handle if "=" in line and not line.startswith("#"))
            try:
                build_environment = driver.build_environment_of(build_root, key_environment)
            except Exception as error:  # 驱动拒绝时抛 Refusal：配置文件读不了、RUSTC 的 -V 跑不出
                problems.append(f"算不出此刻的构建环境（驱动的 build_environment_of，在 {build_root} 上）：{error}")
                build_environment = None
            if build_environment is not None:
                current_global = driver.global_key_lines(driver.JUDGE_RULES_VERSION, current_toolchain, staged, build_environment, tools,
                                                         driver.exclusions_digest_of(sample_exclusions or driver.EXCLUSIONS_FILE), limits)
                if current_global != recorded_global:
                    recorded_lines, current_lines = recorded_global.splitlines(), current_global.splitlines()
                    changed = [f"那一版记的「{line}」" for line in recorded_lines if line not in current_lines] + [f"此刻是「{line}」" for line in current_lines if line not in recorded_lines]
                    problems.append(f"全局键与那一版记的不同（排除表、工具版本、构建环境、升档常量或判法版本变了，那一版的结论不按此刻的全局键重判）："
                                    + "；".join(changed[:8] or ["逐行相同的行一样、次序不同"]))
                else:
                    global_line_count = len(current_global.splitlines())
if (toolchain_matches and not problems and "files-unchecked" not in broken and "version-key-unchecked" not in broken
        and driver.version_key(home, staged_source_fingerprint, current_toolchain, global_digest) != fingerprint):
    problems.append(f"按驱动的 version_key 算出的版本指纹不是 tree.source 记的 {fingerprint[:16]}")
if problems:
    print(f"  ✗ 广谱变异库目录 {home} 此刻那一版（{fingerprint[:16]}，导出自 {str(source.get('commit'))[:12]}）不是被判的这棵暂存树：")  # gate-lint:summary
    for problem in problems:
        print(f"     {problem}")  # gate-lint:detail
    print(f"     → 怎么办：{howto_run}；暂存区之后又变了就重来一遍")
    sys.exit(1)
print(f"  ✓ 广谱变异库目录那一版（{fingerprint[:16]}，导出自 {str(source.get('commit'))[:12]}）就是被判的暂存树：导出范围 {' '.join(export_paths)} 里 {len(staged)} 个文件逐个相同，工具链相同"
      + (f"，全局键 {global_line_count} 行逐行相同" if global_line_count is not None else ""))
PY
}

# ── 格 gate-verdict ──
cell_gate_verdict() {
local output
output="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
if env -u BROAD_MUTATION_BREAK "${summary_command[@]}" > "$output" 2>&1; then summary_exit=0; else summary_exit=$?; fi
env -u BROAD_MUTATION_BREAK python3 - "$output" "$summary_exit" "$LIBRARY_HOME" "${CHECKER_TIER_BROAD_MUTATION_BREAK:-}" "$TREE_SOURCE_READ" "$DRIVER" "$sample_result_key" <<'PY'
import hashlib, hmac, importlib.util, json, os, re, sys
sys.dont_write_bytecode = True
output_path, exit_text, home, broken_text, source_read_path, driver_path, sample_result_key = sys.argv[1:8]
broken = set(filter(None, broken_text.split(",")))
summary_exit = int(exit_text)
with open(output_path, encoding="utf-8", errors="replace") as handle:
    lines = handle.read().splitlines()
howto_run = "主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步：--sync-tree --commit 到包着这一次暂存树的提交，再 --changed-since HEAD，跑完再跑这一道"
def fields_of(prefix):
    found = [line for line in lines if line.startswith(prefix + " ")]
    return [dict(token.split("=", 1) for token in line.split()[1:] if "=" in token) for line in found]
def tail():
    return "\n".join("       | " + line for line in lines[-12:])
summaries, gates = fields_of("BROAD_MUTATION_SUMMARY"), fields_of("BROAD_MUTATION_GATE")
no_conclusion = [line for line in lines if line.startswith("NO_CONCLUSION\t")]
missed_lines = [line for line in lines if line.startswith("MISSED\t")]
if summary_exit not in (0, 8):
    print(f"  ✗ 驱动 --summary 退 {summary_exit}（不是判绿的 0、也不是判红的 8）：结果表对不上或驱动没跑成；它的输出末几行：")
    print(tail())
    print("     → 怎么办：照上面驱动打的 → 办（退 1 是栏数或结果行对不上，退 2 是用法或库目录不对）；修好再跑这一格。")
    sys.exit(1)
if len(summaries) != 1 or (len(gates) != 1 and not ("gate-line-optional" in broken and not gates and summary_exit == 0)):
    print(f"  ✗ 驱动 --summary 打了 {len(summaries)} 行 BROAD_MUTATION_SUMMARY、{len(gates)} 行 BROAD_MUTATION_GATE，各该恰好一行：判不了提交门禁；输出末几行：")
    print(tail())
    print("     → 怎么办：没有判定行是驱动还是加剪枝之前的版本：research/scripts/broad-mutation.py 从 git 取今天的那一份；行数多了是输出被拼进了别的东西。")
    sys.exit(1)
summary = summaries[0]
gate = gates[0] if gates else {"verdict": "green", "total": summary.get("total", "0"), "concluded": summary.get("total", "0"),
                                "missed": "0", "without_conclusion": "0", "carried": "?", "fresh": "?", "flipped": "0"}
try:
    with open(source_read_path, encoding="utf-8") as handle:
        version = json.load(handle)["fingerprint"]
except (OSError, ValueError, KeyError, TypeError) as error:
    print(f"  ✗ 读不出广谱变异库目录的版本指针 {home}/tree.source：{error}")
    print(f"     → 怎么办：库目录换了地方就设 SINGLEFS_BROAD_MUTATION_HOME；还没建过就 {howto_run}")
    sys.exit(1)
if "version-unchecked" not in broken and summary.get("fingerprint") != version[:16]:
    print(f"  ✗ 汇总行的 fingerprint={summary.get('fingerprint')} 不是库目录 tree.source 那一版（{version[:16]}）：判的不是同一版")
    print("     → 怎么办：跑这一道的时候别的会话在换库目录的树（--sync-tree）；等它停下，再照这一道的出路让驱动判这一次的暂存树。")
    sys.exit(1)
def number(key, source=gate):
    try:
        return int(source.get(key, ""))
    except ValueError:
        return None
total, concluded, missed, without = number("total"), number("concluded"), number("missed"), number("without_conclusion")
if None in (total, concluded, missed, without):
    print(f"  ✗ 判定行的计数读不出：{gate}")
    print("     → 怎么办：驱动打的判定行坏了，research/scripts/broad-mutation.py 从 git 取今天的那一份再跑。")
    sys.exit(1)
verdict = gate.get("verdict")
if verdict == "red" and summary_exit == 8 and "summary-red-passes" not in broken:
    print(f"  ✗ 广谱变异提交门禁判红：这一版 {total} 条里 {without} 条没有结论、{missed} 条没红")  # gate-lint:summary
    for line in (missed_lines + no_conclusion)[:30]:
        print("     " + line.replace("\t", " "))  # gate-lint:detail
    if len(missed_lines) + len(no_conclusion) > 30:
        print(f"     ……另有 {len(missed_lines) + len(no_conclusion) - 30} 行，全文在驱动 --summary 的输出里")  # gate-lint:detail
    print(f"     → 怎么办：没有结论的：{howto_run}（超时、内存撞顶的 --changed-since 也重跑，那一步自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到；"
          "证据不齐、仍是这两种的加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun）；"
          "没红的：加一条能杀它的测试（改 crates/ 归实现员）再让驱动判那一条，确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（整版重判）。")
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
    if concluded + without != total:
        inconsistent.append(f"concluded={concluded} 加 without_conclusion={without} 不等于 total={total}")
    if "detail-count-unchecked" not in broken and (len(no_conclusion) != without or len(missed_lines) != missed or number("missed", summary) != missed):
        inconsistent.append(f"明细与计数不同：NO_CONCLUSION {len(no_conclusion)} 行、MISSED {len(missed_lines)} 行，判定行 without_conclusion={without}、missed={missed}，汇总行 missed={summary.get('missed')}")
    if total <= 0 and "empty-list-passes" not in broken:
        inconsistent.append("这一版清单 0 条：没有可判的变异，不是通过")
if "summary-gate-totals-unchecked" not in broken:
    columns = ("total", "caught", "missed", "unviable", "timeout", "memory_cap", "stage_one_only", "driver_error", "not_run")
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
        carried, fresh = number("carried"), number("fresh")
        if carried is not None and fresh is not None and carried + fresh != concluded:
            inconsistent.append(f"判定行 carried={carried} 加 fresh={fresh} 不等于 concluded={concluded}")
escalated_lines = [line.split("\t") for line in lines if line.startswith("ESCALATED\t")]
if "escalated-count-unchecked" not in broken:
    if "escalated" in gate and number("escalated") != len(escalated_lines):
        inconsistent.append(f"判定行 escalated={gate.get('escalated')}，ESCALATED 明细 {len(escalated_lines)} 行")
    elif "escalated" not in gate and escalated_lines and "escalated-field-optional" not in broken:
        inconsistent.append(f"有 {len(escalated_lines)} 行 ESCALATED 明细，判定行却没有 escalated=")
if inconsistent:
    print("  ✗ 驱动 --summary 的判定行、汇总行、计数、明细与退出码对不上（驱动的提交门禁判法失灵）：")  # gate-lint:summary
    for entry in inconsistent:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：这是驱动的毛病，不是这一版的结论：交工具实现员查 research/scripts/broad-mutation.py 的 summarise（它的 --selftest 有 summary-gate-silent 那一格）；修好前不提交。")
    sys.exit(1)
# 升档判的抓到（驱动文件头「升档」）：逐行复核库目录登记表里这一版的行，每一条 ESCALATED 都要有一行复核过的这一版登记行
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
    result_columns, signed_columns = len(driver.RESULT_COLUMNS), driver.RESULT_SIGNED_COLUMNS
    result_key_path = sample_result_key or driver.result_key_path()
except Exception as error:  # 驱动被删、改坏、少了升档那几个常量
    print(f"  ✗ 读不进广谱变异的驱动 {driver_path} 里升档与结果表的常量：{error}")
    print("     → 怎么办：它随仓走，被删了或改坏了从 git 找回来；登记表与基线表的列、标签、倍数门槛、结果表的列与密钥的位置只认它一份。")
    sys.exit(1)
# 结果行的校验列（驱动文件头「结果行的校验列」）：这一版结果表的每一行都要带用这台机器的密钥算的 HMAC-SHA256，这里另算一遍、逐行比
mac_problems, mac_rows = [], 0
results_path = os.path.join(home, "versions", version, "results.tsv")
if "result-mac-unchecked" not in broken and os.path.exists(results_path):
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
    with open(results_path, "rb") as handle:
        result_lines = handle.read().decode("utf-8", errors="surrogateescape").split("\n")
    for line_number, line in enumerate(result_lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        where = f"结果表第 {line_number} 行（第 {fields[0]} 条，结局 {fields[3] if len(fields) > 3 else '?'}）"
        if len(fields) == signed_columns:
            mac_problems.append(f"{where}没有校验列（{signed_columns} 段）")
        elif len(fields) != result_columns:
            mac_problems.append(f"{where}段数是 {len(fields)}，不是 {result_columns}")
        elif key is None:
            mac_problems.append(f"{where}校验不了：{key_problem}")
        elif not hmac.compare_digest(hmac.new(key, "\t".join(fields[:signed_columns]).encode("utf-8", "surrogateescape"), hashlib.sha256).hexdigest(), fields[-1]):
            mac_problems.append(f"{where}的校验列对不上")
        else:
            mac_rows += 1
if mac_problems:
    print(f"  ✗ 广谱变异库目录那一版（{version[:16]}）的结果表里有 {len(mac_problems)} 行不是驱动用这台机器的密钥写的（{results_path}）：")  # gate-lint:summary
    for entry in mac_problems[:20]:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：没有校验列或校验列对不上的行是手写或拷来的：照驱动 --summary 打的出路把结果表拷一份留底、删掉点到的那几行，再让驱动点名重判那几条"
          "（--item <序号> --rerun，前缀与起法照 .claude/skills/crash-test/commit-stages.md 第 2b 步 ③），跑完再跑这一道；"
          "密钥不在、权限不对，或每一行都校验不过（库目录是有校验列之前建的、密钥丢了或换了），照 research/scripts/broad-mutation.py 文件头「结果行的校验列」整库重建")
    sys.exit(1)
cap_form, cap_units = re.compile(r"^([1-9][0-9]*)([KMGT])$"), {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}

def amount_of(label, text):
    """登记行里的限时（整数秒）或上限（run-with-memory-cap.sh 的写法）→ 数；读不出交 None。"""
    if label == timeout_label:
        return int(text) if re.fullmatch(r"[1-9][0-9]*", text or "") else None
    found = cap_form.match(text or "")
    return int(found.group(1)) * cap_units[found.group(2)] if found else None

def referenced_baseline(version_directory, row):
    """登记行 baseline_row 指的那一行基线（驱动文件头「升档」①）：交 (那一行的值, 不对的地方)。值是秒数（超时）或峰值字节（撞顶）；
    那一行要是这一版 baseline.tsv 里成形的一行，步名、结局（ok）、线程数都与登记的相同。没有那一行交 (None, [为什么])。"""
    value_column = "seconds" if row["label"] == timeout_label else "peak_bytes"
    path = os.path.join(version_directory, "baseline.tsv")
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")[:-1]
    except OSError:
        return None, [f"那一版没有 baseline.tsv（{row['step']} 的基线）"]
    if "escalated-baseline-last-row" in broken:   # 改前的取法：这一步最后一行绿记录，不看行号与线程数
        found = None
        for line in lines:
            fields = line.split("\t")
            if not line.startswith("#") and len(fields) == len(baseline_columns) and fields[0] == row["step"] and fields[1] == "ok":
                found = dict(zip(baseline_columns, fields))[value_column]
        return found, ([] if found is not None else [f"那一版没记 {row['step']} 的基线"])
    number = row.get("baseline_row", "")
    if not number.isdigit() or not 1 <= int(number) <= len(lines):
        return None, [f"登记的基线行号「{number}」在那一版 baseline.tsv 里没有（它有 {len(lines)} 行）"]
    line = lines[int(number) - 1]
    fields = line.split("\t")
    if line.startswith("#") or len(fields) != len(baseline_columns):
        return None, [f"那一版 baseline.tsv 第 {number} 行不是一行基线（{len(fields)} 段）"]
    entry = dict(zip(baseline_columns, fields))
    identity = []
    if "escalated-baseline-row-identity-unchecked" not in broken:
        for column, wanted in (("step", row["step"]), ("outcome", "ok"), ("threads", row["threads"])):
            if entry[column] != wanted:
                identity.append(f"那一版 baseline.tsv 第 {number} 行的 {column} 是「{entry[column]}」，登记的是「{wanted}」")
    return entry[value_column], identity

def row_problems(row):
    """一行登记的证据缺哪几样（空表是复核过了）。"""
    found, label = [], row["label"]
    if label not in multiples:
        return [f"标签「{label}」认不出（只认 {timeout_label}、{memory_label}）"]
    version_directory = os.path.join(home, "versions", row["version"])
    baseline_text, baseline_identity = referenced_baseline(version_directory, row)
    found += baseline_identity    # 没有那一行时为什么也在里面
    if baseline_text is not None and baseline_text != row["baseline"] and "escalated-baseline-unchecked" not in broken:
        found.append(f"登记的基线 {row['baseline']} 与那一版 baseline.tsv 第 {row.get('baseline_row', '?')} 行记的 {baseline_text or '空'} 不同")
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
        path = os.path.normpath(os.path.join(version_directory, relative))
        outside = not path.startswith(os.path.join(version_directory, evidence_directory) + os.sep) and "escalated-logs-anywhere" not in broken
        if outside or not os.path.isfile(path):
            if "escalated-evidence-optional" not in broken:
                found.append(f"{attempt} 那趟的日志 {relative} 不在 versions/<那一版>/{evidence_directory}/ 下：没有驱动写的证据（手写的行）")
            continue
        with open(path, "rb") as handle:
            content = handle.read()
        if "escalated-log-digest-unchecked" not in broken and hashlib.sha256(content).hexdigest() != row[f"{attempt}_log_sha256"]:
            found.append(f"{attempt} 那趟的日志 {relative} 的 sha256 与登记的不同（登记之后改过）")
        text = content.decode("utf-8", errors="replace")
        trailer = f"BROAD_MUTATION_EVIDENCE version={row['version']} sequence={row['sequence']} step={row['step']} attempt={attempt} limit={limit} exit={hit_exit}"
        if "escalated-trailer-version-only" in broken:
            trailer = f"BROAD_MUTATION_EVIDENCE version={row['version']}"
        if trailer not in text and "escalated-trailer-unchecked" not in broken:
            found.append(f"{attempt} 那趟的日志末尾没有驱动写的那一行（{trailer}）")
        wrapper = f"超过限时 {limit} 秒" if label == timeout_label else f"撞了内存上限 {limit}"
        if wrapper not in text and "escalated-wrapper-unchecked" not in broken:
            found.append(f"{attempt} 那趟的日志里没有包装打的「{wrapper}」")
    return found

registry_problems, verified = [], set()
if os.path.exists(registry_path):
    with open(registry_path, encoding="utf-8", errors="replace") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip() or line.startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t")
            if fields[0] != version and "escalated-version-unchecked" not in broken:
                continue   # 按版本键只认这一版的行（别的版本的行不看，连段数也不核：它们可能是改列之前写的）
            if (len(fields) < 10) if "escalated-columns-loose" in broken else (len(fields) != len(columns)):
                registry_problems.append(f"登记表第 {line_number} 行段数是 {len(fields)}，不是 {len(columns)}")
                continue
            row = dict(zip(columns, fields))
            found = row_problems(row)
            if found:
                registry_problems.append(f"登记表第 {line_number} 行（第 {row['sequence']} 条 {row['label']}，版本 {row['version'][:16]}）：" + "；".join(found))
            else:
                verified.add((row["sequence"], row["label"], row["name"]))
for fields in escalated_lines:
    if "escalated-sequence-only" in broken:
        matched = len(fields) == 4 and fields[1] in {entry[0] for entry in verified}
    else:
        matched = len(fields) == 4 and (fields[1], fields[2], fields[3]) in verified
    if not matched:
        registry_problems.append(f"--summary 说第 {fields[1] if len(fields) > 1 else '?'} 条是升档判的 {fields[2] if len(fields) > 2 else '?'}，"
                                 f"登记表里没有这一版（{version[:16]}）复核得过的那一行")
if registry_problems:
    print(f"  ✗ 升档判的抓到复核不过（{len(registry_problems)} 处；登记表 {registry_path}）：")  # gate-lint:summary
    for entry in registry_problems[:20]:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：登记行与日志只由驱动写，手改、拷来的都不算；照点的那一样查（日志被删或改了、基线对不上），"
          "再让驱动点名重判那几条（--item <序号> --rerun，前缀与起法照 .claude/skills/crash-test/commit-stages.md 第 2b 步 ③），跑完再跑这一道；"
          "倍数不够的加大 --timeout-floor / --memory-cap 再 --rerun")
    sys.exit(1)
print(f"  ✓ 广谱变异提交门禁判绿：这一版（{version[:16]}）{total} 条每一条都有结论（沿用 {gate.get('carried')}、现判 {gate.get('fresh')}），没红 0 条；"
      f"升档判的抓到 {len(escalated_lines)} 条、登记行逐行复核过；里程碑全量翻面 {gate.get('flipped')} 条（只报不判）；"
      f"结果表 {mac_rows} 行逐行核过带密钥的校验列")
PY
python_exit=$?
rm -f "${output:?}"
exit "$python_exit"
}

# 写全绿标记之前：登记的输入路径（连同登记表与这一道自己）上工作区等于暂存区、没有未跟踪文件（标记按工作区算指纹，格判的是暂存区）。
# 查标记之前（--marker-check）同一核。第二个参数是 write（写标记，默认）或 check（查标记）：只换原因那一句的后半。
# 干净退 0；不干净退 1，stdout 打一句原因。弄坏开关 marker-unguarded 下写标记那一支不核，marker-check-unguarded 下查标记那一支不核。
marker_inputs_clean() {
python3 - "$1" "$ADMISSION_MODULE" "$(basename "$0")" "${CHECKER_TIER_BROAD_MUTATION_BREAK:-}" "${2:-write}" <<'PY'
import importlib.util, os, subprocess, sys
sys.dont_write_bytecode = True
root, admission_path, stage, broken_text, purpose = sys.argv[1:6]
if ("marker-unguarded" if purpose == "write" else "marker-check-unguarded") in broken_text.split(","):
    sys.exit(0)
specification = importlib.util.spec_from_file_location("admission_for_broad_mutation_gate", admission_path)
admission = importlib.util.module_from_spec(specification)
specification.loader.exec_module(admission)
rows = admission.rows_of_key(admission.read_registration_rows(root), stage)
paths = [path for row in rows for path in row.input_paths]
if not paths:
    print(f"{admission.REGISTRATION_TABLE} 里没有 {stage} 这一行，判不了输入路径上工作区与暂存区同不同")
    sys.exit(1)
paths += [admission.REGISTRATION_TABLE, f".claude/gate.d/{stage}"]
environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
changed = subprocess.run(["git", "-C", root, "diff", "--name-only", "-z", "--", *paths], capture_output=True, env=environment)
untracked = subprocess.run(["git", "-C", root, "ls-files", "--others", "--exclude-standard", "-z", "--", *paths], capture_output=True, env=environment)
if changed.returncode != 0 or untracked.returncode != 0:
    print(f"git diff 退 {changed.returncode}、git ls-files 退 {untracked.returncode}，判不了输入路径上工作区与暂存区同不同")
    sys.exit(1)
changed_names = [os.fsdecode(name) for name in changed.stdout.split(b"\0") if name]
untracked_names = [os.fsdecode(name) for name in untracked.stdout.split(b"\0") if name]
if changed_names or untracked_names:
    listing = [f"没暂存的改动 {name}" for name in changed_names] + [f"没跟踪的文件 {name}" for name in untracked_names]
    tail = ("标记按工作区算指纹、这一道判的是暂存区，写了会让没判过的那一份被复用；暂存或撤掉那几处、重跑这一道才写" if purpose == "write" else
            "标记按工作区算指纹、这一道判的是暂存区，此刻查到的标记说明不了要提交的那一批；驱动、剪枝规则、排除表读的也是工作区那一份")
    print(f"登记的输入路径上工作区与暂存区不同（{len(listing)} 处：{'、'.join(listing[:5])}{' 等' if len(listing) > 5 else ''}），{tail}")
    sys.exit(1)
PY
}

# 查标记（--marker-check、样本放了 .broad-sample/marker-check）：登记的输入路径上工作区等于暂存区、没有未跟踪文件，才起 admission.py stage-marker-check。
# 退 0 有作数的全绿标记（复用）；退 3 两边相同而没有作数的标记（照第 2b 步跑）；退 1 两边不同（不查标记、第 2b 步 ② 也不起）或判不了。
marker_check() {
  local marker_root="$1" reason check_exit
  if ! reason="$(marker_inputs_clean "$marker_root" check)"; then
    echo "  ✗ 不查全绿标记：${reason:-判不了输入路径上工作区与暂存区同不同}"
    echo "     → 怎么办：照 .claude/main-agent.md「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再查；在那之前 .claude/skills/crash-test/commit-stages.md 第 2b 步 ② 也不起"
    return 1
  fi
  if python3 "$ADMISSION_MODULE" stage-marker-check "$marker_root" "$(basename "$0")"; then
    echo "  ✓ 登记的输入路径上工作区等于暂存区，这一批输入有作数的全绿标记（上一行）：这一道记「复用」，第 2b 步不起"
    return 0
  else
    check_exit=$?
  fi
  if ((check_exit == 1)); then
    echo "  · 登记的输入路径上工作区等于暂存区，这一批输入没有作数的全绿标记（上一行是原因）：照 .claude/skills/crash-test/commit-stages.md 第 2b 步起驱动、再跑这一道"
    return 3
  fi
  echo "  ✗ admission.py stage-marker-check 退 $check_exit：判不了这一道的全绿标记（上一行是原因）"
  echo "     → 怎么办：看 .claude/gate.d/stage-inputs.tsv 里这一道那一行还在不在、cargo -V 与 rustc -V 跑不跑得出，修好再查"
  return 1
}

write_marker_if_clean() {
  local marker_root="$1" reason marker_path
  if ! reason="$(marker_inputs_clean "$marker_root")"; then
    echo "  ! 本次未写全绿标记：${reason:-判不了输入路径上工作区与暂存区同不同}"
    return 0
  fi
  if marker_path="$(python3 "$ADMISSION_MODULE" stage-marker-write "$marker_root" "$(basename "$0")")"; then
    echo "  · 写了这一道的全绿标记：$marker_path"
  else
    echo "  ! 没写成这一道的全绿标记（admission.py stage-marker-write 没成：$marker_path）：下一趟这一道照跑，不会复用"
  fi
}

# 查标记那一支（文件头「查标记」）：不跑格、不起驱动
if ((marker_check_requested)); then
  if marker_check "$ROOT"; then exit 0; else exit "$?"; fi
fi

# 复用（接法同 checker-tier-formal-verification.sh）：真仓与 --staged 下整道跑时，stage-inputs.tsv 本阶段那一行的路径与 environment=（--tool-versions 那几行）都没变就整道退 77；判定只认 1 为可跳过，别的非 0 判红
if [[ "$(pwd -P)" == "$(cd "$STAGE_REPOSITORY" && pwd -P)" || -n "${GATE_STAGED_FROM:-}" ]] && ((STAGE_CELLS_CHECK_GIVEN == 0)); then
  real_run=1
else
  real_run=0
fi
if ((real_run)); then
  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  if ( stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
         "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径、工具版本怎么进判定见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")" ); then
    reuse_exit=0
  else
    reuse_exit=$?
  fi
  if ((reuse_exit == 77)); then
    [[ -n "${GATE_NOT_RUN_FILE:-}" ]] && printf '%s 整道复用上一次整轮全绿的判定（输入与工具版本都没变），本次未跑\n' "$(basename "$0")" >> "$GATE_NOT_RUN_FILE"
    exit 77
  fi
  ((reuse_exit == 0)) || exit "$reuse_exit"
fi

# 钉住版本指针：两格都判开格之前这一刻的 tree.source，跑完再核一次（弄坏开关 tree-source-reread 下两格各读此刻的、不再核）
LIVE_TREE_SOURCE="$LIBRARY_HOME/tree.source"
pinned_tree_source=""
if [[ "$BROKEN" != *,tree-source-reread,* ]]; then
  pinned_tree_source="$(mktemp)" || { echo "  ✗ 建不了钉住版本指针的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  if [[ -f "$LIVE_TREE_SOURCE" ]]; then
    cp "$LIVE_TREE_SOURCE" "$pinned_tree_source" || { rm -f "${pinned_tree_source:?}"; echo "  ✗ 拷不出版本指针 $LIVE_TREE_SOURCE"; echo "     → 怎么办：看那个文件的权限，修好再跑。"; exit 1; }
  else
    rm -f "${pinned_tree_source:?}"
  fi
  TREE_SOURCE_READ="$pinned_tree_source"
else
  TREE_SOURCE_READ="$LIVE_TREE_SOURCE"
fi


if ( stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
if [[ -n "$pinned_tree_source" ]]; then
  pointer_moved=0
  if [[ -f "$pinned_tree_source" ]]; then
    cmp -s "$pinned_tree_source" "$LIVE_TREE_SOURCE" || pointer_moved=1
  elif [[ -f "$LIVE_TREE_SOURCE" ]]; then
    pointer_moved=1
  fi
  rm -f "${pinned_tree_source:?}"
  if ((pointer_moved)); then
    echo "  ✗ 库目录的版本指针 $LIVE_TREE_SOURCE 在这一道判的时候换了（开格之前钉住的与两格跑完再读的不同）：两格判的那一版已经不是库目录此刻那一版"
    echo "     → 怎么办：别的会话在跑驱动的 --sync-tree；等它停下（ps -o pid,args -u \"\$(id -u)\" 看驱动的进程行），再照这一道的出路让驱动判这一次的暂存树，跑完再跑这一道。"
    stage_exit=1
  fi
fi
if ((stage_exit == 0 && real_run)); then
  # 写不成只报不红：后果只是下一趟多跑
  if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0")"; then
    echo "  ! 没记下这一次的工具版本（admission.py gate-record-environment 没成）：下一趟整轮这一道照跑，不会复用"
  fi
  write_marker_if_clean "$ROOT"
elif ((stage_exit == 0 && sample_mode)) && [[ -f "$SAMPLE_DIRECTORY/write-marker" ]]; then
  write_marker_if_clean "$ROOT"
fi
exit "$stage_exit"
