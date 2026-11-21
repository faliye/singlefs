# replay-prefix-backfill-code-r3：攻方腿报告（2026-11-19）

正文 `research/prompts/_replay-prefix-backfill-code-r3-body.md`，开工快照 `research/prompts/replay-prefix-backfill-code-r3-snapshot/`。开工时 `sha256sum -c` 两份清单八个文件都 OK，下面读的就是快照那一刻的树。这条腿没写模型（这一轮只改说明文件与必红表，没有可放进模型的历史或崩溃状态），所以没有 `rerun.sh`、没有模型文件的 sha256；判的数一律读码、`grep`、`awk` 现数，或读第二轮判决第二节第 9 条点名的那一趟 formal-verification 的日志（下称「上一趟 formal-verification 日志」，`task-server/task log <号> --all` 取，号见那一节）。

## 各格判定一览

| 格 | 判定 | 一句 |
|---|---|---|
| 1-a 文件区间 19 项 | 成立（量的） | 按 verus-proof 格的成项规则是 10 个 exec fn 加 9 个 proof fn；上一趟 formal-verification 日志里 `proof:singlefs_replay_prefix::file_range_proofs::` 的项恰 19 个 |
| 1-b README 别处的义务数与必红数 | 打中 | `:119` 的「条义务」是另一种单位且是历史句；`:138` 写 7 个、表里 unlink 12 行；`:148` 写 11 个、表里部分写计划 12 行；`:165` 列举 21 行、表里 overwrite 23 行（这一轮新加的两行没进）；`:117`、`:126` 数对、是按当时那一次记的说法 |
| 2-a 复跑命令的全名与范围数 | 成立（量的） | 82、4、5、2 条测试与 README 的 4 条全在源码里、模块路径对、范围数等于点名数；12 个真卡用例名全对 |
| 2-b 真卡一节有没有照抄会被拒的命令 | 成立 | 那一节不给命令；与 `.claude/rules/verification.md`、`verification-reference.md` 的「接口待定」一致 |
| 2-c `NOT_RUN_NO_GPU_ADAPTER` 那几句 | 打中（轻） | 「没拿到这一台的卡时，每条打一行 NOT_RUN…」在库里已有两边一致结论、或只录那一段时不成立：测试打的是 `PAIRING_CONCLUSION_REUSED` 或 `RECORDED_ONLY`，不打 NOT_RUN |
| 3-a 新加 13 行必红 | 成立（推的） | 13 行改坏的都是名字说的那一条；`gpu-score-theorem-window-may-be-empty` 名字写的是变异为什么红，不是那条 ensures 本身，算说得出 |
| 3-b 第二轮没看的 45 行 | 没打中（推的） | 45 行全读，每行改坏的都是名字说的那条 ensures 或分支 |
| 3-c 7 行条数 | 成立（量的） | 7 行新值与上一趟 formal-verification 日志逐行相同 |
| 3-d 新行连带的说明 | 打中 | `GPU_GC_PROOFS.md:83`「三条 gpu-gc-*witness*」现在这个通配命中 5 行，两行是改值不是注入不可能断言 |
| 4 `verus-pins.tsv` 三行 | 成立 | 三句今天都是真话；`:49` 又写了 214，与 `:48` 改成「见 crate 行」的做法相反，是下一次加证明时同样会过期的形 |
| 5 G11「定义自检」 | 成立，附一条推的 | G11 与图例对得上；图例同样罩住 G10（证明体只有 `reveal_with_fuel`、ensures 是两项求和的展开），G10 只标「模型」 |
| 检查单八格 | 七格成立、一格推不出 | 「带 --item 的命令与项」的 `--list-items` 这条腿没跑，交主 agent |

打中 3 条（1-b、2-c、3-d），另有 1 条推的、没量（5 里的 G10）。都在说明文字上，不碰证明、产品代码与必红行本身。

## 攻击面 1：19 这个数与 README 别处的数

### 1-a 19 成立

成项规则原文：「proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，带值的 const 与 assume_specification 各成一项」（`.claude/gate.d/formal-verification.sh:18`）。

`file_range_proofs.rs` 的 `verus!` 块（第 5 行到第 392 行）里现数（`grep -nE` 列函数头，量的）：exec fn 10 个（第 12、42、55、77、98、134、166、195、212、277 行），proof fn 9 个（第 65、116、125、264、319、333、345、357、369 行），`grep -c decreases` 得 0，没有 `const`。上一趟 formal-verification 日志里 `ITEM_DONE item=proof:singlefs_replay_prefix::file_range_proofs::` 去重后 19 行（量的）。README 把 19 拆成「9 个产品入口、8 条引理、2 个无前提见证」也对得上：9 个产品入口就是文件表第 18 行列的那 9 个，见证是 `witness_file_range_boundaries`（exec）与 `witness_intersection_length_partitions`（proof）。

要说清的一点：README 这一句写的单位是「项」，与 Verus 报的 verified 条数不是一回事。模块里有两处「by (nonlinear_arith)」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:30`、`crates/singlefs-replay-prefix/src/file_range_proofs.rs:273`），登记表把这种子证明记成单独一条：「共 9 项、13 条义务（另 4 条是 nonlinear_arith 的 assert by）」（`.claude/gate.d/verus-pins.tsv:38`）。照这个记法文件区间模块是 19 项、21 条（推的：项数是量的，每处 nonlinear_arith 多一条是照登记表第 38、40、43 行的记法推的，没起 Verus 量这个模块的 verified）。上一趟 formal-verification 日志的整 crate 行是「成项 198 个（proof fn 116、exec fn 50、带 decreases 的 spec fn 16、derive(Clone) 生成的 clone 13、带值的 const 3），各项 verified 之和 214」，差 16 条；全 crate 的 `by (nonlinear_arith)` 现数 7 处（`grep -c` 各文件相加），其余 9 条从哪来这条腿没核。所以「19 项」本身成立，19 前面的「义务」一词在同一节别处指的是条数（下一小节），读者容易把 19 当成 verified 数。

### 1-b README 别处的数：打中

逐句现数（量的，命令都是 `awk -F'\t'` 按第二段或名字前缀数 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 的非注释行）：

| 位置 | 原文 | 现数 | 判 |
|---|---|---|---|
| `:117` | 「另有 7 个必红变异，包含删除两个引理各自的大小顺序前提。」（`crates/singlefs-replay-prefix/src/README.md:117`） | 第 291–297 行恰 7 行（`write-end-*` 两行、`payload-*` 五行） | 数对；「另有」是接在那一次加的东西后面说的，按当时那一次记 |
| `:119` | 「同日继续补齐单元取整：新增三个产品函数与一个单调引理，加上取整边界的 nonlinear_arith 子证明，共 5 条义务。」（`crates/singlefs-replay-prefix/src/README.md:119`） | 3 个 exec 加 1 个 proof 是 4 项；5 是把第 30 行那处 nonlinear_arith 算成一条的条数 | 打中：同一节第 109 行按「项」数、这里按「条」数，19 不等于各段相加；「同日继续补齐」「新增」是经过 |
| `:126` | 「本轮新增 6 个必红变异与 3 个 Rust 回归测试」（`crates/singlefs-replay-prefix/src/README.md:126`） | 第 298–303 行恰 6 行 | 数对；「本轮新增」是经过 |
| `:138` | 「必红世界表另登记 7 个变异（计划的边界、截多一个、续做起点、最后一批留一个、size 换成加法、截口叶误判整片留下、节点存在的边界差一）。」（`crates/singlefs-replay-prefix/src/README.md:138`） | 第二段为 `unlink_proofs.rs` 的 12 行（第 279–290 行）；括号列的是第 279–285 行，第 286–290 行五行（`unlink-plan-shape-skips-the-induction-step` 起）没进 | 打中：7 对不上 12 |
| `:148` | 「本模块新增 6 个被验函数（含一条 nonlinear_arith 子证明，共 7 条义务）与 11 个必红变异。」（`crates/singlefs-replay-prefix/src/README.md:148`） | 部分写计划那一族（名字以 `partial-` 起头）12 行：第 304–314 行 11 行加第 317 行 `partial-touch-needs-nonempty-write`；`partial_write_proofs` 模块此刻 14 项（日志现数） | 打中：11 对不上 12；「本模块新增」是经过，「6 个被验函数（含一条子证明）」把条与函数混在一个数里 |
| `:165` | 「两条改坏 `overwritten_bytes` 定义的反例（写入段整段右移一格、写入段少一格）与两条改坏另两条引理结论的反例」（`crates/singlefs-replay-prefix/src/README.md:165`） | 那一句列的是 8 条契约、6 条删前提、2 条改定义、2 条改别的引理结论、3 条组合错位，共 21；名字以 `overwrite-` 起头的行此刻 23 行 | 打中：这一轮新加的第 510 行 `overwrite-exact-keeps-the-length`、第 511 行 `overwrite-exact-empty-request-keeps-the-file`（改第一条引理的 ensures）没进这句列举 |

四句（`.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：分不分辨臂——这是说明文字对不对得上表，不涉及臂；被判的东西当时看不看得到——看得到，表与 README 在同一棵树、同一个补丁里；满足的是哪一个分句——第二轮判决第三节第 1 条「义务数照 `file_range_proofs.rs` 现有被验函数改」只点了第 109、111 行，`:138`、`:148`、`:165` 是「说明与表对不上」，`:119`、`:126`、`:148` 的「同日继续」「本轮新增」「本模块新增」是 `.claude/agent-common.md`「代码与脚本的注释、说明只写现状，不写历史」射程里的 README；已定改法在打中的格上还中不中——第三节第 1 条只改第 109、111 行，碰不到这几句；第 4 条加了两行 `overwrite-exact-*` 而没回头改 `:165`，`:165` 是这一轮改法自己造出来的漏。

门禁拦不到：`scripts/gate/comment-history-check.py` 的词表里没有「同日」「本轮」「新增」（文件头第 37、38 行列的词），所以 `:119`、`:126`、`:148` 不会被 doc-text 判红（推的，读词表推的，没跑那一格）。

改法（只在我的读码上推过、被攻过零轮）：照第 109 行已经用的写法，把 `:117`、`:126`、`:138`、`:148` 的必红个数换成「必红表里第二段为 `<文件>` 的各行」或「名字以 `<前缀>` 起头的各行」这类指路，不写数；`:119`、`:148` 的义务数要留就统一成「项」并写明成项规则，或写「项」「条」两个数；`:165` 补上第一条引理长度与空请求两条改 ensures 的反例，或同样改成指路；「同日继续补齐」「本轮新增」「本模块新增」删掉经过，只写现状。

## 攻击面 2：`GPU_GC_PROOFS.md` 的复跑命令与真卡一节

### 2-a 全名与范围数：成立

量的（脚本都只读源码，输出在草稿目录，结果抄在这里）：

- `--lib` 那一条：从 `crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md` 第 108–191 行抽出 `<模块>::tests::<名>` 82 个、去重 82 个；四个模块各自的 `#[test]` 数与点名数是 7/7、36/36、23/23、16/16；82 个名字逐个在 `crates/singlefs-background-gpu/src/<模块>.rs` 里找到 `fn <名>(`，都在那份文件 `mod tests {` 之后、前一两行是 `#[test]`（缺 0、错位 0）；四个模块在 `crates/singlefs-background-gpu/src/lib.rs` 第 31、33、35、36 行是顶层 `pub mod`，全名的模块路径对。
- 两份重放前缀测试目标：`#[test]` 各 4、5 个，没有 `mod`，名字与第 104、106 行点名的逐个相同。
- `gpu_gc_formal_contracts_through_the_host`：`#[test]` 2 个，名字与第 193 行相同。
- 范围数：四条 `cargo test` 的范围段第一个数 4、5、82、2，等于各自 `--exact` 前的筛选词个数；两条 formal-verification 的「点名的 4 项」等于各自 `--item` 个数；8 个 `must-red:` 名字与 README 第 170 行的 8 个名字在必红表第一段各恰好一行。判法照 `.claude/hooks/run-scope-guard.sh` 文件头第 102–104 行（`--exact` 时 N 是筛选词个数）与第 114 行（范围里第一个阿拉伯数字等于 N）。
- 会不会被当崩溃枚举目标拒：这几个目标不在 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行里，两个包的源码里 `grep -rl` 两个缩域变量名得 0 份（退 1），照不走写是对的。
- README 第 172 行四个 `partial_write_proofs::tests::` 名字在 `partial_write_proofs.rs` 里各一处，`partial_write_proofs` 是 `lib.rs` 第 39 行的顶层 `pub mod`。

没打中：没找到照抄会被拒或会跑 0 条的命令。没交主 agent 实跑（第二轮判决第一节已记同一形的拒绝在范围声明钩子自证里有一格）。

### 2-b 真卡一节：成立

第 198 行不给命令，写「接口待定」。与规则原文一致：「崩溃判段之外用卡的测试交 GPU 服务代跑（接口待定）」（`.claude/rules/verification.md:50`），「product-gpu-real-card-tests 的对拍那一趟交 GPU 服务代跑（接口待定）」（`.claude/rules/verification-reference.md:21`）。12 个用例名与 `crates/singlefs-background-gpu-tests/tests/` 下两个目标里的 `#[test]` 逐个相同，各 6 个（量的）。

### 2-c `NOT_RUN_NO_GPU_ADAPTER` 那几句：打中（轻）

被判的一句：「没拿到这一台的卡时，每条打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、返回成功」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:203`）。

反例（读码推的，没跑）：这两个目标的每条用例经判定库两段对拍，持卡判的入口（`judged_on_the_cards_that_<用例名>`）只在要现判时才调，开卡与打 NOT_RUN 都在它里面。库里那一把键已有两边一致的结论时：「两边一致的复用、打一行 `PAIRING_CONCLUSION_REUSED test=<用例名>`，`judge_on_the_cards` 不调」（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:190`），「复用的那一趟不领卡、不开卡，找不到显卡也照通过」（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:293`）。只录那一段（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）打「RECORDED_ONLY test={test_name} input_bytes={}」（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:218`）就返回。可达的历史：一台有卡的机器上判过一次、结论照键入库；之后在没卡的机器上照同一份输入再跑，这一条通过、打的是 `PAIRING_CONCLUSION_REUSED`，既没有 NOT_RUN 行、也没有交了卡的那一行。

后果：同一段末句「每条用例在日志里都有交了卡的那一行、没有 `NOT_RUN_NO_GPU_ADAPTER` 与 `GPU_DEVICE_LOST` 行，才是在卡上判过」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:203`）作为「这一趟在卡上判过」的判据仍是对的（偏严），但照前半句读的人会把复用那一条当成「本次未跑」以外的说不清的东西，读不出它是照键复用了以前在卡上的结论。

与规则对不对得上：`.claude/rules/mutation-sampling.md` 那一节判的是变异重放里 `FAILED`/`ok` 加标记行，原文「那一行是 `ok`（带不带 `GPU_DEVICE_LOST` 都照这一条往下判），输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它」（`.claude/rules/mutation-sampling.md:63`）记本次未跑；第 61 行 `FAILED` 加自己那一段的 `GPU_DEVICE_LOST` 也记本次未跑。说明文件没有讲 `FAILED` 加 `GPU_DEVICE_LOST` 怎么读，只讲通过的那几条，不矛盾，只是窄。

四句：不涉及臂；看得到（复用打的那一行就在同一份日志里）；满足的是第二轮判决第三节第 2 条「`NOT_RUN_NO_GPU_ADAPTER` 怎么读的那句现状说明并进来」——并进来的这句漏了复用与只录两种结局；第 2 条的改法就是这句本身，对这一格不起作用。

改法（推的、被攻过零轮）：第 203 行前半句改成按结局分：交了卡现判的打 `GPUS_HANDED_OVER_BY_THE_TASK_SERVER` 与 `PAIRING_CONCLUSION_JUDGED`，没卡又要现判的打 NOT_RUN，库里有两边一致结论的打 `PAIRING_CONCLUSION_REUSED`、不开卡，只录那一段打 `RECORDED_ONLY`。

另记一条，不在这一题改动里、交主 agent：被跑的测试自己的文件头写「用 GPU，经 `task-server/task submit` 交（卡与清场由服务定）」（`crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:5`），与 `.claude/rules/verification.md:50`「统一任务服务只调配 CPU」那一句相反；那两个包别的会话正在拆，这条腿不判归谁改。

## 攻击面 3：新加 13 行、第二轮没看的 45 行、7 行条数

13 行怎么认的：快照 `changes.patch` 里必红表的 `+` 行去掉同名的 `-` 行，得 13 个名字（量的）；13 行的原文在被改文件里各恰好一次（脚本数，量的）。下表「断的是哪条」是读码推的，没起 Verus；这 20 行（13 加 7）每行改坏都报证明错、落点对，是正文「验证读数」那一趟 formal-verification 的读数，这条腿没读那一趟的日志。

| 必红表行 | 名字 | 改了什么 | 断的是哪条（推的） | 名字说得对不对 |
|---|---|---|---|---|
| 414 | write-intersection-empty-has-zero-length | `bytes_in_unit` None 支「== 0」改「== 1」 | 「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`）本身；`checked_bytes_in_indexed_unit` 第 221 行原来靠它推出，跟着断，所以 2 条、落两处 | 对 |
| 415 | indexed-intersection-empty-has-zero-length | 第 221 行「== 0」改「== 1」 | 第 221 行本身 | 对 |
| 416 | write-intersection-one-byte-is-not-dropped | 「if from < until {」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:187`）加 `&& until - from > 1` | 一字节交集时交 None：第 171、172 行两条一起断（两条在 None 支等价，任删一条另一条仍抓它） | 对，名字写的是函数体错在哪 |
| 502 | file-range-witness-is-checked | 见证函数开头注入 `assert(false)` | `witness_file_range_boundaries` 进了验证 | 对 |
| 503 | gpu-score-candidate-window-first-not-after-past | 引理体里 `if past < first {` 那两句提示改成 `if false` | 「first <= past,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:117`）：没有 `records[past]` 这一项，两条下界的量词起不来；第 118、119 行靠第 128–137 行，不受影响。第 117 行被删时这一行会变成不红，所以它也钉住了第 117 行在 | 对 |
| 504 | gpu-score-candidate-window-holds-only-the-device | 「first <= position < past ==> records[position].device == device,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:118`）的 `<` 改 `<=` | 第 118 行本身：`position == past` 可以是下一个盘的记录 | 对 |
| 505 | gpu-score-theorem-first-is-the-candidate-lower-bound | 第 220 行二分的槽从 `earliest_candidate_start(start)` 换成 `start` | 定理第 214 行；第 225 行调用的前提跟着断 | 对 |
| 506 | gpu-score-theorem-past-is-the-end-lower-bound | 第 221 行 `end` 换成 `end - 1` | 第 215 行；起点恰为 `end - 1` 的同盘记录让两个下界不同 | 对 |
| 507 | gpu-score-theorem-window-may-be-empty | 「window.0 <= window.1,」（`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs:216`）改成 `<` | 第 216 行本身：空表时窗口是 `(0, 0)` | 说得出：名字写的是变异为什么红（窗口可以空），钉的是第 216 行 |
| 508 | gpu-gc-witness-record-slice-sum-value | 见证第 345 行的 2 改 3 | 第 345 行：`[9,12)` 对 `[10,20)` 交 2 | 对 |
| 509 | gpu-gc-witness-first-disagreement-count-value | 见证第 346 行的 1 改 2 | 第 346 行：空表上第一次分歧记 1 | 对 |
| 510 | overwrite-exact-keeps-the-length | 「overwritten_bytes(old, start, data).len() == old.len(),」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:123`）加一 | 第 123 行本身 | 对 |
| 511 | overwrite-exact-empty-request-keeps-the-file | 「data.len() == 0 ==> overwritten_bytes(old, start, data) == old,」（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs:131`）的 `== 0` 改 `<= 1` | 第 131 行本身：一字节请求改了文件 | 对 |

第二轮判决第三节第 4 条点名的都有了行：两处 None 支（414、415，另加函数体那一行 416）、候选窗口两条（503、504）、定理三条（505–507）、覆盖写第一条引理两条（510、511）、`witness_file_range_boundaries`（502）；`gpu_gc_proofs.rs` 见证的两个值（508、509）。13 行里 9 行改的是 ensures 本身或见证的值（钉的是「这一条还在、没被放宽」），4 行改函数体或证明提示（416、503、505、506）。没打中。

### 3-b 第二轮没看的 45 行：没打中

45 行怎么认的：第二轮判决说的「新加 75 行」= `git diff HEAD` 里必红表新增的 100 行减去改动的 4 行、交集长度那一路的 8 行（取自第二轮快照 `changes-intersection-length.patch`）、这一轮的 13 行，得 75 个名字；再减第二轮攻方腿报告攻击面 2 表里点名的 30 个（30 个都在 75 里），得 45 个（量的）。45 行逐行读了原文、替换文与被改函数的 ensures（推的，没起 Verus）：

| 组 | 行（必红表行号） | 判 |
|---|---|---|
| G02、G03 五行 | 435–439 | 各改一条 ensures 的真假或计数值，`next_uncertain_count`、`cpu_is_needed` 在那几个具体输入上算得出原值，改后为假；名字逐条对上第 50、51、52、60、61 行 |
| G05 四行、G06 一行 | 441–444、446 | 第 97、98、100、104、122 行各改一处；第 104 行把第二步改成失败批，左边少了第二个条目的更新 |
| G07、G12、G13 六行 | 447、448、451–453、455 | 界收紧一格或加一，各有现成反例（等长区间、两字都取最大、低字和为 2^32 − 1） |
| G14、G16 两行 | 456、459 | 可表示的和交 None，None 支要求溢出，断 |
| G17、G18 五行 | 460、462–465 | 第 294、309、310、314、315 行各收紧或换一边；`held == 0`、`start == total`、恰满容量、两片非空各是反例 |
| G20 常量 | 468 | 常量改 2，规格写死 1，判定字 1 时交 None 而规格要 Some |
| G21–G26 十二行 | 471、472、474、476、478、480–484、486–488 | 函数体删一道界、改一个比较或拒绝支交回值，各断 `chunk_fits` 那一项或 None/Some 支的等式 |
| G28、G29 六行 | 490、491、493、494、496、497 | ensures 收紧或乘数、占位改坏，各断第 146、149、163–167 行 |
| 打分计数下界、覆盖写交换 | 421、428 | `visited == 0` 时和为 0；去掉第一次写之后两种顺序的结果不等 |

45 行里没有一行改坏的不是名字说的那一条（推的）。

### 3-c 7 行条数：成立（量的）

上一趟 formal-verification 日志原样（第 2615、2620–2623、2661、2684 行，草稿目录 `formal-verification-previous-run.log`）：

```text
  · must-red:overwrite-exact-needs-fit：绿，复用（2026-11-18 判绿：落点 lemma_overwrite_changes_exactly_the_requested_bytes；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G01-confirmation-never-asks-cpu：绿，复用（2026-11-18 判绿：落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G01-cpu-judgment-restarts-the-count：绿，复用（2026-11-18 判绿：落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G01-second-disagreement-asks-cpu：绿，复用（2026-11-18 判绿：落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G01-threshold-constant：绿，复用（2026-11-18 判绿：落点 reconcile_gpu_entry；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G18-short-remainder-is-not-overrun：绿，复用（2026-11-18 判绿：落点 next_gpu_record_slice_end；证明错 2 条（条数与登记的 proof_errors=1 不同，条数只报））
  · must-red:gpu-gc-G28-group-count-is-not-truncated：绿，复用（2026-11-18 判绿：落点 gpu_workgroups_for_entries；证明错 3 条（条数与登记的 proof_errors=1 不同，条数只报））
```

必红表此刻的第五段：这 7 行是 2、2、2、2、2、2、3，逐行相同。那一趟日志里「条数与登记的」共 8 行，第 8 行是 `indexed-intersection-does-not-drop-written-bytes`（日志第 2595 行，证明错 2 条），它在必红表第 324 行此刻已是 `proof_errors=2`（第二轮判决第二节第 4 条收下的别的会话的改动），所以「7 行」对。同一份日志的「必红世界：…的 93 行」「按条现判 1、复用 92」也与第二轮判决第二节第 9 条相同（日志第 2695 行，量的）。

### 3-d 新行连带的说明：打中

被判的一句：「三条 `gpu-gc-*witness*` 向见证注入不可能断言」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:83`）。

必红表里名字匹配 `^gpu-gc-.*witness` 的此刻 5 行（`awk -F'\t' '$1 ~ /^gpu-gc-.*witness/'`，量的）：第 358 行 `gpu-gc-witness_gpu_gc_protocol_and_score_premises`、第 359 行 `gpu-gc-witness_gpu_gc_arithmetic_boundaries`、第 376 行 `gpu-gc-capacity-witness` 三行注入 `false`；这一轮新加的第 508、509 行 `gpu-gc-witness-record-slice-sum-value`、`gpu-gc-witness-first-disagreement-count-value` 改的是见证 ensures 里的值，不是注入不可能断言。照这句的通配去数会数出 5 行，与「三条」对不上；这两行也没在别处被说到。

四句：不涉及臂；看得到；满足的是「说明与表对不上」，与 1-b 的 `:165` 同一种：第二轮判决第三节第 4 条加了行，没回头改说明；第 4 条的改法碰不到这一句。

改法（推的、被攻过零轮）：改成「三条 `gpu-gc-*witness_*`／`gpu-gc-capacity-witness` 向见证注入不可能断言，两条 `gpu-gc-witness-*-value` 把协议与打分见证的一个具体值改错」，或改成按名字逐个写。

## 攻击面 4：`verus-pins.tsv` 三行注释

三行原文（`awk 'NR>=47 && NR<=50'`，量的）：

- 「# GPU GC G21–G30：十个核（G30 没有产品调用点）与一个边界见证；模块现量 11 条义务、0 errors。」（`.claude/gate.d/verus-pins.tsv:47`）：`gpu_gc_capacity_proofs.rs` 里 `pub fn` 11 个（10 个核加第 198 行的见证），上一趟 formal-verification 日志里这个模块的项恰 11 个，模块里没有 nonlinear_arith，11 条也对（条数是推的）。`extended_gpu_chunk_input` 在 `crates/` 下除重放前缀包之外的 `.rs` 里 `grep -rn` 零命中（退 1）；G21–G29 九个函数名在 `crates/singlefs-background-gpu/src/` 里各有 1 份文件命中。成立。
- 「# 固定长度覆盖写组合：分段等价、包含写折入请求与构造见证共三条；整 crate 的义务数见 crate 行。」（`.claude/gate.d/verus-pins.tsv:48`）：`partial_write_proofs.rs` 第 217、249、273 行三个 proof fn，成立。
- 「# 交集长度组合：单元分割、请求分割与无前提见证共三条，整 crate 实测 214 verified、0 errors。」（`.claude/gate.d/verus-pins.tsv:49`）：三条是 `file_range_proofs.rs` 第 345、357、369 行；214 与第 50 行 crate 行、正文「验证读数」的「verified 之和 214」相同，没有日期。成立。

附一条推的隐患（不算打中）：第 48 行这一轮刚把 211 改成「见 crate 行」，第 49 行又写了 214。第二轮打中的正是「`verus-pins.tsv:48` 还写 211 verified、crate 行是 214」那一形；下一次加证明改 crate 行时第 49 行会同样过期。改法（推的、被攻过零轮）：第 49 行也写「整 crate 的义务数见 crate 行」。同一段第 31–46 行仍是带日期的逐次记账（含「用户 … 定」这类句子），属于第二轮判决第三节第 6 条那笔欠账（doc-text 的 comment-history 格不读 `.tsv`）的射程，这一轮的改法没碰，不重复报。

## 攻击面 5：G11 的「定义自检」与图例

图例：「“定义自检”表示 `ensures` 是同一文件里 spec 函数定义的展开，只钉住那份定义，对产品代码没有约束。」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:28`）。G11 的三条 ensures（`gpu_gc_proofs.rs` 第 194、195–196、197–198 行）都只说 `record_contribution`（第 157 行）与它用的 `overlap_slots`（第 126 行），两者都在同一文件；第 67 行补的那句写明了「连同它用的 `overlap_slots`」。G11 那一标与图例对得上，成立。

推的一条（没量，不算打中）：照图例的字面，G10 也该标。G10 的 ensures 是「ensures record_slice_sum(seq![record, record], device, start, end, 0, 2)」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:184`）等于两倍 `record_contribution`，证明体只有「reveal_with_fuel(record_slice_sum, 3);」（`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs:187`），是同一文件里 `record_slice_sum`（第 162 行）在两项序列上的展开，性质与 G02–G04、G11 同类；表里 G10 只标「模型，只证重数 2」（`crates/singlefs-replay-prefix/src/GPU_GC_PROOFS.md:41`）。G07、G12（`overlap_slots`、`words_value` 的算术性质，证明体为空）在两可之间：它们是从定义推出的性质而不只是代入展开，标不标要看图例里「展开」取多宽。分辨的办法：给图例补一句判据（例如「ensures 只在具体输入上求值，或只把定义的分支照抄」），再逐行对 G05–G13 判。

## 检查单八格

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 快照 `changes.patch` 的 `diff --git` 只有三份：`GPU_GC_PROOFS.md`、`README.md`、`verus-must-red-worlds.tsv`，没有 `.rs` |
| 类型与名字 | 成立 | 13 个变异名都说得出改坏的是哪一条（攻击面 3 表）；第 507 行名字写的是变异为什么红，算说得出；必红表第一段 `sort \| uniq -d` 为空 |
| 不兼容旧代码与旧格式 | 成立 | 补丁里没有版本号、布局号与认旧分支 |
| 不新建判定库 | 成立 | 补丁只改说明与登记表 |
| 所有任务多机可用 | 成立 | 两份说明文件里 `grep -nE '/home/\|/tmp/'` 零命中（退 1）；复跑命令只用仓根相对路径 |
| 测试优先多线程 | 成立 | 没有新测试 |
| 带 --item 的命令与项 | 推不出 | 说明文件里的命令名字与范围数都核过（攻击面 2）；13 行能不能被 `--list-items` 列出：`.claude/gate.d/formal-verification.sh` 文件头第 52 行写「必红世界每行一项 must-red:<变异名>」，13 行名字唯一、六段齐（`NF != 6` 的非注释行 0 行），推得出会列出，但这条腿没跑 `--list-items`，命令在下面交主 agent |
| 新崩溃流多罩了什么 | 成立 | 没有新崩溃流或崩溃枚举用例 |

## 没打中的形状

- 19：试了「成项规则漏了 nonlinear_arith 子证明」「有带 decreases 的 spec fn 或带值的 const 没数」「见证算不算产品入口」三种，项数都是 19；只剩「义务」一词在同一节里两种单位（记在 1-b）。
- 复跑命令：试了全名不在源码里、模块不在顶层、测试不在 `mod tests` 里、范围数与筛选词个数不等、目标会走崩溃枚举五种，五条 `cargo test`、三条 formal-verification（两份说明文件）都没撞上。
- 13 行：试了「改的不是名字那条」「原文不唯一」「改了 ensures 却在别处先断」「两条等价的 None 支 ensures 只罩一条」四种，没撞上；416 行两条一起断是因为两条在 None 支等价，不是漏罩。
- 45 行：全 45 行（不是抽样）试「改坏的不是名字那条」，没撞上。
- 7 行条数与 93/1/92：逐行与日志比，没撞上。
- G30 没有调用点、G21–G29 有调用点：`grep` 现查，没撞上。

## 这条腿自己的限度

- 「断的是哪条」全部读码推的，没起 Verus；20 行（13 加 7）红不红、落点对不对依赖正文「验证读数」那一趟，这条腿没读它的日志，那 13 行的第五段条数与那一趟实测是否相同没核。
- 文件区间模块 21 条 verified 是推的（19 项是量的）；整 crate 198 项与 214 条之间的 16 条只认出 7 条 nonlinear_arith，余 9 条没核。
- 2-c 的反例（库里有结论时打 `PAIRING_CONCLUSION_REUSED`）是读测试共用模块推的，没在机器上跑出那一行；这两个包别的会话正在拆，判的是今天的树。
- 5 的 G10 是按图例字面推的，图例的「展开」取多宽这条腿定不了。
- 没跑 `--list-items`；没交任何任务；没建仓副本、编译目录与模型目录。

## 要主 agent 跑的命令

1. `task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items .`
   - 在哪个目录起：仓根。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里以 `must-red:` 起头的项行中，下面 13 个名字各恰好一行：`write-intersection-empty-has-zero-length`、`indexed-intersection-empty-has-zero-length`、`write-intersection-one-byte-is-not-dropped`、`file-range-witness-is-checked`、`gpu-score-candidate-window-first-not-after-past`、`gpu-score-candidate-window-holds-only-the-device`、`gpu-score-theorem-first-is-the-candidate-lower-bound`、`gpu-score-theorem-past-is-the-end-lower-bound`、`gpu-score-theorem-window-may-be-empty`、`gpu-gc-witness-record-slice-sum-value`、`gpu-gc-witness-first-disagreement-count-value`、`overwrite-exact-keeps-the-length`、`overwrite-exact-empty-request-keeps-the-file`。这一条判检查单「带 --item 的命令与项」那一格；提交时被拒（第二轮攻方腿记过它要 `--full-gate` 的说法，这条腿没核）就把拒绝原文记进判决，那一格照「推不出」留。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；改法都只在我的读码上推过、被攻过零轮，没在副本上量过。
- 没往统一任务服务交任务；判决要引的数（`--list-items`）没跑，命令在上一节。
- 没写模型（这一轮没有历史与崩溃状态可造），没有 `rerun.sh` 与复跑包装脚本、没有探针的内存与时间。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/replay-prefix-backfill-code-r3-attack` 里只有现数用的文本（名字清单、必红表对 HEAD 的差异、上一趟 formal-verification 日志的拷贝），没有仓副本、编译目录与库拷贝，不用删。
