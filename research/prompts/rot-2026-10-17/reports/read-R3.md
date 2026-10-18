# read-R3：总览文档通读（CLAUDE.md、三语 README、kb 总览与说明文件）

日期：2026-10-17。批次：`records/2026-10-17-全仓知识腐烂治理.md#批6`。只读，不改任何文件；没建副本、没起编译、没跑重型测试。
读了 10 份：`CLAUDE.md`、`README.md`、`README.zh.md`、`README.ja.md`、`.claude/kb/INDEX.md`、`.claude/kb/tooling.md`、`.claude/kb/verification-build.md`、`.claude/kb/decisions.md`（只判索引表之外的说明段）、`.claude/kb/experiments.md`（索引表逐行对实验页标题）、`crates/singlefs-checker/invariants/README.md`。

判定取值：要改（给改后整句）、要补、要人看（拿不准）；不改的行只在汇总里报数。

## 逐行判定表（只列要改、要补、要人看）

| 文件:行 | 原文（截 120 字） | 判定 | 改后句或理由（引原话，带现查命令） |
|---|---|---|---|
| CLAUDE.md:112 | 崩溃点重放（门禁 checker-tier-crash-point-replay）的负载是两条流：新池新建文件，以及固定脚本到 E； | 要改 | 现查：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 输出 14；`grep -vc '^#' .claude/gate.d/crash-point-replay-items.tsv` 是 14 项、一项一条流（新池新建文件、固定脚本到回退、到卸载、多记录发布、删除、树分裂、位置寻址树、实例表清扫等）；放量 `bash scripts/crash/crash-amplification.sh --list-items` 列 8 项。「两条流」已不是真话。改后句：崩溃点重放（门禁 checker-tier-crash-point-replay）登记 14 条崩溃枚举用例（数它的命令：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`），合表 `.claude/gate.d/crash-point-replay-items.tsv` 一项一条流，放量分项 `bash scripts/crash/crash-amplification.sh --list-items` 列； |
| CLAUDE.md:112 | 门禁全绿只构成新池新建文件在模型层的崩溃一致性证据；里程碑「覆盖写、释放、回退与复用」步 0 那条固定脚本……的崩溃点重放结果不作数，重写与重跑归崩溃点重放规模那一轮与实六 | 要人看 | 范围词「新池新建文件」和「步 0 固定脚本……不作数」是 2026-09-26 的判断（`records/2026-09-24-里程碑二收尾调度.md` 第 182 行「实三交回」：`step_zero_layer0` 下游语义断了）。之后固定脚本流已重写并全量：`grep -n 'FULL_STATES_THROUGH_THE_UNMOUNT' crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 得 1_678_377_204；`.claude/kb/milestone/02-second-txn.md:227` 写 2026-09-17 两条流全量零违例；`.claude/kb/milestone/03-third-txn.md` 段 3 写多记录、删除流放量 0 红。`README.md` 同位置只写「only evidence of crash consistency at the model level」，没有「新池新建文件」限定。是否改成「只构成模型层的崩溃一致性证据」并删「不作数」半句，要主 agent 定（这是政策口径，不是我能判的事实） |
| CLAUDE.md:112 | 哪几条按回退候选集判、候选集的下界怎么取，见 `crates/singlefs-checker/invariants/README.md`「## 状态」那一段与各条的状态列 | 要人看 | 现查 `grep -n '候选集' crates/singlefs-checker/invariants/README.md`：只有第 33 行（违例位那一段，说哪几个下标「在回退候选集的版本都走完之后」判），「下界怎么取」不在「## 状态」里，在 04、05、06、07、09 各文件条目的状态列（`grep -c '候选集' crates/singlefs-checker/invariants/0*.md` 各 2、5、2、2、11）。指针说「与各条的状态列」，半真；要不要改成「见该 README 第 33 行那一段与 04 到 09 各条的状态列」由主 agent 定 |
| .claude/kb/tooling.md:198 | doc-registries 的 segment-registry 格（段序列登记表） \| `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，历史版本里写同一串就红；只有一段的路径（取号 `2`）正则不认 \| 历史里用「4 + 1 + 1 + 1 + 4」带空格的写法；把 `check-segment-registry.py` 的正则改成 `*` | 要改 | 「正则不认」「把正则改成 `*`」已做完：`grep -n 'SEGMENT_SEQUENCE_PATTERN' scripts/crash/check-segment-registry.py` 第 71 行 `(?:\+[0-9]+)*` 且带注释「一条路径可以只有一段（取号那一行是 `2`），所以 + 与 * 之间取 *」。另 `grep -n '^## 历史版本' .claude/kb/layout/01-first-txn.md` 零命中、`.claude/history-carriers` 不登记 layout，所谓「历史版本里写同一串」今天没有这一节。改后行：`\| doc-registries 的 segment-registry 格（段序列登记表） \| `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，同一串在别处再写一遍就红 \| 别处引用写成「4 + 1 + 1 + 1 + 4」带空格的写法 \|` |
| .claude/kb/verification-build.md:5 | 门禁的未实现清单剩 QEMU 崩溃注入与 shell 命名纪律两条；崩溃点重放由 checker-tier-crash-point-replay 覆盖，射程是那两条流； | 要改 | 现查：`cat .claude/gate-not-implemented.tsv` 登记「崩溃点重放」「模型对拍」两键；`sed -n 574p .claude/singlefs-ai-sop/scripts/gate.sh` 共享键是 `("最终判据" "命名纪律（shell）")`；`grep -n '^# gate-covers' .claude/gate.d/*.sh` 只有崩溃点重放与模型对拍两条覆盖声明；全仓无「QEMU 崩溃注入」这一键（`grep -rl 'QEMU 崩溃注入' .claude scripts` 只中 CHANGELOG 与本文件）。「射程是那两条流」同 CLAUDE.md:112，现为 14 条用例（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`）。改后句：门禁末尾的未实现清单有四项：共享的「最终判据」「命名纪律（shell）」，与 `.claude/gate-not-implemented.tsv` 登记的「崩溃点重放」「模型对拍」（后两项由门禁 checker-tier-crash-point-replay 与 harness-model-differential-and-scenarios 头部的 `gate-covers` 覆盖，覆盖后换成「由哪个阶段覆盖」）；崩溃点重放的射程是登记的 14 条崩溃枚举用例（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 数出）； |
| .claude/kb/verification-build.md:39 | 2026-09-14 用户定案加四个字段 ⇒ **481 字节、512 槽余 31**，不再挡 | 要人看 | 带日期的事件句，不算错；但「481 字节」今天不是现值：`grep -n 'SYSTEM_CONFIGURATION_BYTES' crates/singlefs-format/src/lib.rs` 得 489、`SYSTEM_CONFIGURATION_SLOT_BYTES` 得 4096（余 3607，`.claude/kb/layout/01-first-txn.md:156`「合计 **489 字节**」）。读者易把 481 当现值。主 agent 定要不要在句末补「（2026-09-26 加回退下界 F 8 字节，现行 489 字节，槽宽 4096、余 3607）」 |
| .claude/kb/verification-build.md:76 | `crates/singlefs-core/src/write_accounting.rs`，12 种 | 要改 | 现查：`grep -n 'IN_REPORT_ORDER' crates/singlefs-core/src/write_accounting.rs` 第 40 行 `[WrittenStructureKind; 16]`。改后句：`crates/singlefs-core/src/write_accounting.rs`，16 种（`WrittenStructureKind::IN_REPORT_ORDER` 的长度） |
| .claude/kb/verification-build.md:154 | 2026-09-14 现状：「harness 自己要证明会红的用例」那张表的第一条……已做成用例……；其余七条要的覆盖写、释放、多次挂载、回退还不在崩溃点重放的负载里。（2026-09-17 起覆盖写、释放、多次挂载、回退都已进崩溃点重放覆盖写固定脚本流…） | 要改 | 「现状」冒号后的分句去掉日期读起来是在说今天，又被自己的括号推翻（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 已有全量与残留记录正例，`.claude/kb/milestone/02-second-txn.md:227` 写必红八条到齐）。改成事件句：「2026-09-14 当天的现状：……（两个用例名照旧）；其余七条要的覆盖写、释放、多次挂载、回退当天还不在崩溃点重放的负载里。2026-09-17 起覆盖写、释放、多次挂载、回退都已进崩溃点重放覆盖写固定脚本流，每条必红用例的现状见 `.claude/kb/milestone/02-second-txn.md` 步 0 与增补 2 收口表第 22 行。」两个用例函数现查都在：`grep -n 'fn removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker' crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`（第 1095 行）与 `grep -n 'named_unit_verification_keeps_a_damaged_transaction_out_of_the_rebuilt_root' crates/singlefs-harness/tests/recovery_reads_the_created_file_after_reopen.rs`（第 338 行） |
| .claude/kb/verification-build.md:301 | 只取改坏对象在 `crates/singlefs-core/src/` 下的行，每个测试目标一行，是 171 行（登记表 `.claude/gate.d/crates-mutation-light-rows.tsv`；用户弹窗选「只取核心代码 171 行」） | 要改 | 现查：`grep -vc '^#\|^$' .claude/gate.d/crates-mutation-light-rows.tsv` 输出 166（登记表文件头「2026-10-10 照挑法生成……171 行」是生成当天的数）。改后句：……每个测试目标一行，现有 166 行（`grep -vc '^#\|^$' .claude/gate.d/crates-mutation-light-rows.tsv` 现数；2026-10-10 生成时 171 行，用户弹窗选「只取核心代码」）；可变部分用 `--quick --item` 加 |
| .claude/kb/experiments.md:19 | ⚠️ **「结论」列一律 50 字以内**，写那个实验**测出了什么**，不写它为什么这么测、也不写口径。 | 要改 | 现查：`python3 -I` 数索引表 177 行的第三列字符数，107 行超过 50、最长 3030、中位 67（`python3 -I /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-R3/count-experiment-conclusions.py`，在仓根跑，输出 177 107 3030 67）；`grep -n 'gate-cell' .claude/gate.d/doc-experiments.sh` 没有宽度格（决策索引有 `decision-summary-width`，实验索引没有）。「一律 50 字」不是真话。改后句（保守版）：「结论」列写那个实验**测出了什么**，不写它为什么这么测、也不写口径；目标 50 字以内，现有 107 / 177 行超出，门禁不判宽度。主 agent 也可选另一条路：把 107 行压到 50 字、再加宽度格；这是改行还是改句的取舍，请定 |
| .claude/kb/experiments.md:24 | E1（节点大小 vs 可恢复性） \| 部分已跑 \| 爆炸半径那一半已答……；恢复时间那一半仍欠 checker | 要人看 | 前置「欠 checker」是现状句：`ls -d crates/singlefs-checker` 存在，池级 checker 判 92 条（`grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md \| awk '{s+=$1} END{print s}'` 得 92）。实验页首 `.claude/kb/experiments/01-节点大小vs可恢复性.md` 写「前置是 mkfs、checker、定点破坏工具」，定点破坏工具有没有我没查到。若三样都有，改成「恢复时间那一半未跑」并去掉「仍欠 checker」；请主 agent 定 |
| .claude/kb/experiments.md:26 | E3（意图日志的断电语义） \| 未跑 \| 意图日志的断电语义，欠事务层与 checker | 要人看 | 事务层与 checker 今天都有：`ls -d crates/singlefs-core crates/singlefs-checker`；删除意图已写（`.claude/kb/milestone/03-third-txn.md` 三·2：`crates/singlefs-core/src/unlink.rs`、`transaction.rs` 的 `publish_unlink_intent`），并有崩溃放量流 `unlink-after-the-multi-record-publish`（`bash scripts/crash/crash-amplification.sh --list-items`）。「欠事务层与 checker」不再是真话；E3 页首 `未跑（依赖事务层与 checker）` 同。E3 判据③「续做幂等」要两镜像比，是否已被删除流覆盖我没判，请主 agent 定 |
| .claude/kb/experiments.md:28 | E5（nonce 唯一性 × 崩溃点重放） \| 未跑 \| nonce 唯一性 × 崩溃点重放，欠事务层、加密与 checker | 要人看 | 加密仍没有：`grep -n '^| D9' .claude/kb/decisions.md` 进度「已实现 4 项 / 未实现 10 项」；事务层与 checker 已有（同上）。「欠事务层、加密与 checker」应收窄为「欠加密」；页首 `未跑（依赖事务层、加密实现与 checker）` 同 |
| .claude/kb/experiments.md:33 | E10（老化：结构抗老化 vs 事后整理） \| 未跑 \| 老化：结构抗老化 vs 事后整理，欠文件系统；……格式臂不再是候选 | 要人看 | 「欠文件系统」：mkfs、写、挂载、回退、删除的文件系统已有（`ls -d crates/singlefs-core`），后台整理没有（`grep -n '^| D26' .claude/kb/decisions.md` 进度「已实现 0 项」）。该写「欠后台整理」还是别的，由主 agent 定 |
| crates/singlefs-checker/invariants/README.md:36 | 这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。 | 要改 | 现查：`grep -n '历史版本' crates/singlefs-checker/invariants/README.md` 只中本行；`.claude/history-carriers` 不登记这份 README（只登记 `.claude/kb/decisions/`、`.claude/kb/experiments/` 和门禁样本），文件里没有「## 历史版本」节。数本身对：`grep -h '^| I-' crates/singlefs-checker/invariants/0*.md \| wc -l` 得 121，带「已停用」的 6 行（I-3.2、I-3.3、I-4.5、I-6.10、I-7.10、I-7.11），121 − 6 = 115。改后句：现共 115 条在用（编号至 I-9.37（测试镜像里谱系与退役容器不超旋钮值），另有六条退役，编号不回收）。这一句是当前条数的权威登记位。 |
| crates/singlefs-checker/invariants/README.md:40-46 | 以下类别在对应决策定案前无法写出：……反向索引一致性（D1 定案后）……快照可见性（D6 定案后）……索引结构自身的结构不变量，如树的有序性、平衡性（D8 定案后） | 要改 | 现查 `sed -n 36,66p .claude/kb/decisions.md`：D1「已定 8 项 / 未定 0 项」、D6「已定 10 项 / 未定 0 项」、D8「已定 16 项 / 未定 0 项」、D2「已定 21 项 / 未定 1 项」，「定案前」的前置在 D1、D6、D8 上已过时；树的有序性已有 I-9.12（分隔 key 落在孩子区间之外）与 I-9.16（树表条目按树 ID 严格升序……）（`grep -n 'I-9.12\|I-9.16' crates/singlefs-checker/invariants/09-inode树结构.md`），所以第 45 行「如树的有序性」也不再是「无法写出」。改后（第 40 行）：以下类别还没有不变量；对应决策的现状：D1、D6、D8 已定，D2 已定 21 项、未定 1 项；第 46 行（密钥层级与换钥）我没判，D9 索引行是「已定 17 项 / 未定 0 项」，是否还有没定的部分请核 `grep -n '主密钥槽\|换钥' .claude/kb/decisions/09-加密.md`。（第 42-46 行）逐条去掉「定案后」「细化后」，第 45 行改成「索引结构自身的结构不变量里 inode 树以外的树（有序性已由 I-9.12、I-9.16 罩住 inode 树与树表）」。哪几类今天仍写不出，我没有逐类判，请主 agent 或 kb-scribe 逐类核 |
| crates/singlefs-checker/invariants/README.md:48-52 | `---` / 空行 / 空行 / `---` | 要改 | 文件末尾两道空的水平线，`tail -8 crates/singlefs-checker/invariants/README.md \| cat -A` 见第 48 行与第 51 行是 `---`、中间与其后是空行，不围着任何内容（「## 历史版本」被删之后的残留）。改后：删去第 47-52 行，文件以第 46 行收尾 |

## 没有判成要改的几处（备查，不改）

| 文件:行 | 原文（截 120 字） | 为什么不改 |
|---|---|---|
| CLAUDE.md:8 | 代码在 `crates/` 下八个 crate | `ls -d crates/*/` 输出 8 个目录，与句中列的八样一一对上 |
| CLAUDE.md:112 | checker 判 92 条不变量（数它的命令……） | `grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md \| awk '{s+=$1} END{print s}'` 得 92；`grep -n 'pub const IMPLEMENTED_INVARIANTS' crates/singlefs-checker/src/image.rs` 是 `[&str; 92]` |
| CLAUDE.md:74-101 | 项目本地事实表 | 表里 `.claude/kb/`、`scripts/`、`records/` 等 40 多个路径逐个 `[ -e ]` 现查，全在；门禁格名逐个 `grep -l 'gate-cell: <格名> '` 都对上 |
| README.md:57（三语同行） | The 92 invariants marked implemented | 同上 92；三语逐行对：数字、路径、反引号串、表格列数一致（用 `grep -o` 抽出后比对，仅 `qemu.env` 等几处顺序不同） |
| README 三语 | Mounting starts by reading two 4096-byte slots……489-byte structure……389 / 4 / 36 / 60 | `grep -n 'SYSTEM_CONFIGURATION_BYTES' crates/singlefs-format/src/lib.rs` 489 与 4096；`.claude/kb/decisions/22-单元原子性怎么合成.md:229-232` 389、4、36、60 |
| README 三语 | Small-object packing has been decided by D27 but is not implemented yet | `grep -n '^| D27' .claude/kb/decisions.md` 已定 13 项；`grep -rli '打包容器' crates/singlefs-core/src` 无命中（`build_packed_unit` 是码 3 单元类，不是 D27 的共享容器）。这一条只查了字面，没读 D27 全部分项的进度，见「要主 agent 现查」 |
| .claude/kb/INDEX.md | 全文 | 表里 19 个指向逐个存在；`ls .claude/kb/` 的 19 个文件与目录都列了；「只剩 T1 一问没定」对上 verification-build.md 第六节；「按类拆成同目录 9 份」对上 `ls crates/singlefs-checker/invariants/` 的 9 个 `0*.md` |
| .claude/kb/tooling.md | 其余各节 | 现查全对：`rustc --version` / `cargo --version` 1.98.0；`rustup target list --installed` 有 `x86_64-unknown-linux-musl`；`~/kbuild/linux-om` 1.8G、`Makefile` 为 7.2.0、`fs/bcachefs` 不在；`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` 只有 Kconfig 与 Makefile；`uname -r` 为 6.17.0-1028-oem；`lsblk` 的 nvme0n1 3.6T 512/512 ROTA 0；`/boot/config-6.17.0-1028-oem` 四个配置项一致；`scripts/devices/vm-bench.sh` 默认 2048/4/4096/900；`Cargo.toml:21` 与 `scripts/experiments/Cargo.toml:750` 有 `overflow-checks = true`；第 194-206 行各门禁格名、`.claude/naming-lint-exclude`、`.claude/format-constants-exclude`、`scripts/kb/replace-batch.py`、`history-consolidate.py loss-check` 都在；`scripts/agents/ask-local.sh` 等已删（`ls` 无） |
| .claude/kb/verification-build.md | 其余各节 | 现查对：八个 crate；14 条 `crash-case:`；`FULL_STATES` 16_777_260 / `FULL_STATES_WITH_TWO_STATES_PER_WRITE` 16_777_240 / `FULL_STATES_THROUGH_THE_UNMOUNT` 1_678_377_204；`DATA_UNIT_HEADER_BYTES` 105、`ROOT_RECORD_BYTES` 457、`JOURNAL_HEADER_BYTES` 487、`JOURNAL_HEADER_TEN_FIELD_BYTES` 78；`count_key_order_mismatches`、`available_on_each_device`、`reclaim_released_up_to` 都在；`.claude/gate.d/gpu-exemptions.tsv` 10 行；第八节所有 `crash_*.rs`、`verdict_store.rs`、两份对拍测试文件、`two_stages`、`RunStage`、`StageOutcome`、`CrashLibraryClass`、`ComputedOn`、`judge_crash_image_against_the_version_table` 都在；`grep -l O_DIRECT scripts/experiments/*.rs \| wc -l` 得 14；`gpu-cards.env.example` 已无 |
| .claude/kb/decisions.md:1-93 | 说明段 | `.claude/scripts/gen-decision-items.py` 在；格名 `decision-summary-width`、`decision-items-sync`、`kb-shape`、`item-ref-status`（doc-decisions.sh）、`decision-links`（doc-experiments.sh）都在；「结论列 200 字以内」：28 行最长 181；`ls .claude/kb/decisions \| wc -l` 为 28，首行都是 `## D<n> 简称 —— 状态` |
| .claude/kb/experiments.md 索引表 | 177 行 | `ls .claude/kb/experiments \| wc -l` 177，行数对上；用 `/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-R3/check-experiment-index-status.py`（在仓根 `python3 -I` 跑）比每行状态首词与实验页首个 `## E<n>` 标题的状态首词，只有 E2、E16、E21、E34、E74 五行措辞不同（语义一致），其余同；E175 行链接存在（正则没抓到是因为状态里带括号） |

## 汇总

行数用 `wc -l` 数；现状句数是我逐句读时手数的约数（一个格里并列几个断言算几句），不是命令数出的；要改、要补、要人看数按上面「逐行判定表」的行数（`grep -c` 表里判定列）。

| 文件 | 行数 | 现状句数（约） | 要改 | 要补 | 要人看 |
|---|---|---|---|---|---|
| CLAUDE.md | 112 | 60 | 1 | 0 | 2 |
| README.md | 154 | 45 | 0 | 0 | 0 |
| README.zh.md | 154 | 45 | 0 | 0 | 0 |
| README.ja.md | 154 | 45 | 0 | 0 | 0 |
| .claude/kb/INDEX.md | 39 | 25 | 0 | 0 | 0 |
| .claude/kb/tooling.md | 217 | 75 | 1 | 0 | 0 |
| .claude/kb/verification-build.md | 306 | 130 | 4 | 0 | 1 |
| .claude/kb/decisions.md（只说明段第 1-93 行） | 478 | 20 | 0 | 0 | 0 |
| .claude/kb/experiments.md（说明段加 177 行索引表） | 200 | 185 | 1 | 0 | 4 |
| crates/singlefs-checker/invariants/README.md | 52 | 50 | 3 | 0 | 0 |
| 合计 | 1866 | 约 680 | 10 | 0 | 7 |

核对数行：`wc -l CLAUDE.md README.md README.zh.md README.ja.md .claude/kb/INDEX.md .claude/kb/tooling.md .claude/kb/verification-build.md .claude/kb/decisions.md .claude/kb/experiments.md crates/singlefs-checker/invariants/README.md` 末行 1866。

三语 README 的做法：英文版逐句对仓，中文、日文版用 `grep -o` 抽数字、反引号串、链接目标与表格列数同英文版比，结构与数值一致；没发现要改的现状句，所以没有「三处都写」的改后句。译文用词是否照 `.claude/kb/translation-glossary.md` 我没逐词核。

## 要主 agent 现查（我查不了或没查全）

1. `.claude/kb/verification-build.md:201`（环境现查表）「herd7 \| 7.58」：我起 herd7 被 `.claude/hooks/heavy-test-guard.sh` 拒了（`herd7 -version` 算重型），没核。该表头写「环境现查（2026-09-03）」，是带日期的事件，可不改；要核请主 agent 跑 `herd7 -version`。
2. `.claude/kb/tooling.md:12-16`（本地网关 `http://127.0.0.1:8200/v1/chat/completions`、模型名 `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`、`max_model_len` 262144）：`curl -s -m 5 http://127.0.0.1:8200/v1/models` 连不上（curl 退出码 7）。我只核了 `~/code/ai-center/.env.tenants` 里有键 `AI_CENTER_KEY_VSCODE_CHAT`、`~/code/ai-center/.claude/kb/token-budget.md` 在；服务此刻没起，模型名与长度没法对。本节开头已写三方论证不用它，不影响别处。
3. `.claude/kb/verification-build.md:69`「整理落点那一行还没有（整理还没进代码，C146 ① 仍开着）」与「收口表第 58 行」：我只核了 C146 在 `.claude/kb/checks-owed.md` 里有行，没读状态列与收口表第 58 行。
4. README 三语「Small-object packing ……is not implemented yet」：只核了 `crates/singlefs-core/src` 里没有 D27 共享容器的字样，没读 D27 十三个分项的「进度」列（`grep -n '^| D27' .claude/kb/decisions.md` 进度列被截断在我的输出里）。
5. `.claude/kb/experiments.md` 177 行的「结论」列内容与实验页正文是否一致：我只比了状态首词与标题，没逐行比结论文字。
6. `.claude/kb/decisions.md` 索引表 28 行的「结论」文字：按派发说明没判（生成器出状态与进度列，结论列是手写的），没核。
7. `CLAUDE.md:112` 两处要人看（见上表）和 `.claude/kb/experiments.md` 四行（E1、E3、E5、E10）的取舍。
8. 交叉：`CLAUDE.md:112` 与 `.claude/kb/verification-build.md:5` 都写了「崩溃点重放的射程是两条流」，改的时候两处同改；`.claude/kb/milestone/02-second-txn.md:227` 开头也有「崩溃点重放的负载是两条流」的现状句（带 2026-09-17 日期，那份不在我的范围，没判）。

## 没做什么

- 没改任何文件，没建副本，没起编译，没跑重型测试，没跑 `crash-point-replay.sh`（读合表 `.claude/gate.d/crash-point-replay-items.tsv` 代替）；跑过的只读命令有 `crash-amplification.sh --list-items`（输出 8 行）。
- 没核 `.claude/kb/decisions.md` 478 行里分项清单（自动生成段第 70-478 行）。
- 没逐词核三语译文。
- 现状句数是约数；「不改」的行没有逐行列出。
- 工作目录认领：`bash scripts/hosts/work-directory-ownership.sh claim …/rot-2026-10-17-R3 --purpose …` 已跑，认领的是已有目录；目录里留了两个辅助脚本（`count-experiment-conclusions.py`、`check-experiment-index-status.py`），没有仓副本和编译目录。
