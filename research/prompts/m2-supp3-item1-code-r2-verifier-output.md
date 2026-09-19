# 核查员报告：m2-supp3-item1-code-r2

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

时刻：核查开工与结束均在 2026-09-18 UTC 当天完成。快照：`research/prompts/m2-supp3-item1-code-r2-start-snapshot.sha256`；`.claude/kb/milestone/02-second-txn.md` 在腿开工之后被另一会话改过五处（步 1、步 5、步 7、增补 1 各一处，定点同行替换、不改变行数），核对方式与依据见下方「关于 02-second-txn.md 的核法」一节。其余 11 个文件现查与快照一致，对这些文件的引用直接核主树。

## 判别力自证（先做）

挑的引用：Sonnet 报告二.1 节「`crates/singlefs-core/src/allocator.rs:651` `pub fn records(&self) -> &[AllocationRecord]`」。

1. 先核原样：`sed -n '651p' crates/singlefs-core/src/allocator.rs` → `    pub fn records(&self) -> &[AllocationRecord] {`，与引用逐字相同，本应判 ✓。
2. 自证：把该文件拷进草稿目录（`/tmp/claude-1000/m2-supp3-item1-r2-verifier/draft/allocator.rs.selftest`），在文件最前面插入一个空行，使原第 651 行整体下移到第 652 行。
3. 按同样的核法（`sed -n '651p'` 副本）取到的是 `    #[must_use]`，与引用「`pub fn records(&self) -> &[AllocationRecord] {`」不同。
4. 判定：**✗**（对不上）。自证方法能分辨「引对」与「引错」两种情形，往下按同样的核法继续。

## 关于 02-second-txn.md 的核法

`diff /tmp/claude-1000/m2-supp3-item1-r2-verifier/snapshot-copies/02-second-txn.md .claude/kb/milestone/02-second-txn.md`：两份文件均 591 行，只有 5 处 `NcN`（同行替换，不增删行）：113、213、218、252、344 行，分别在步 1、步 5（两处）、步 7、增补 1 附近，均不在两条腿引用过的行号（收口表第 311、358 行；增补 3 标题 383 与其现状段 383–431；「验收标准」标题各行）范围内。故这些引用行号在快照与主树上取值相同，以下直接核主树，不算「分不清」。

## 一、Opus 攻方腿（`m2-supp3-item1-code-r2-opus-output.md`，sha256 `6dd49fd6…`，现查与交回一致）

| # | 引用 / 复跑项 | 核的结果 | 命令 |
|---|---|---|---|
| 1 | `history.rs:934`「`&& observation.raised_floor_lands_only_on_abandoned_roots == Some(true)`」 | ✓ 逐字同 | `sed -n '934p' crates/singlefs-harness/src/history.rs` |
| 2 | `history.rs:2027-2030`（`readable_roots`/`choose_root`/`instance_table_of_root`、`checker` 不对外解实例表那句） | ✓ 逐字同（含 `Walk::instance_table_rows` 那句括注） | `sed -n '2025,2032p' crates/singlefs-harness/src/history.rs` |
| 3 | `history.rs:1775`「挂载写出的写行与暖机……这里不比分配记录」 | ✓ 逐字同 | `sed -n '1775p' crates/singlefs-harness/src/history.rs` |
| 4 | `mount.rs:158,163,165`（`effective_root`/`row_publish`/`warm_up_publishes` 三个公开字段） | ✓ 三处行号与字段名逐字同 | `grep -n "pub effective_root\|pub row_publish\|pub warm_up_publishes" crates/singlefs-core/src/mount.rs` |
| 5 | `recovery.rs:378`「`pub fn allocation_records_under_root(`」 | ✓ 逐字同 | `sed -n '378p' crates/singlefs-core/src/recovery.rs` |
| 6 | 「`grep -rn should_panic crates/singlefs-harness/` 只命中 1 行，是 `crash.rs` 第 1577 行一条注释，不是属性」 | ✓ 命中数、行号、注释/属性判断都对 | `grep -rn should_panic crates/singlefs-harness/` |
| 7 | `crates/mutations.tsv` 第 131–138 行（层 0 并行 8 条变异，点名两条集成用例） | ✓ 8 行内容与两条测试名逐字同 | `sed -n '131,138p' crates/mutations.tsv` |
| 8 | `opus-model/mutants.tsv`（Y1a/b/c/d/e、A1/2/5/6/8/9/10、Y2d 各条改法描述） | ✓ 与报告表格（第 79–90 行）描述逐条一致 | `cat .../opus-model/mutants.tsv` |
| 9 | 报告「文件的 sha256（`SHA256SUMS` 同文）」内联 11 行 vs 模型目录 `SHA256SUMS` | **✗** 模型目录 `SHA256SUMS` 实为 12 行，多出 `y5-panic-probe.diff`（`95461ebf…`，报告正文的「工作线程 panic」一节确实引用了这个 diff 文件），报告正文内联的哈希块只抄了 11 行，不是「同文」 | `wc -l SHA256SUMS`；`sha256sum -c SHA256SUMS` |
| 10 | 复跑：Y2 基线 off/a/b/e 四模式（`y2-patch.py`+`y2e-patch.py`+`opus_r2_y2.rs`），两条入库用例 `--nocapture` | ✓ 四模式与 `off` 去掉用时行后逐字节相同（`diff` 空）；两条「历史 N 段：跑完…」行与报告第 158、160 行逐字相同 | 见下方「复跑记录」 |
| 11 | 复跑：`opus_r2_y2_mount_only_generation`（`off` 模式两档计数） | ✓ `mount_rewrites=154/368`、`other_rewrites=242/450`、`mount_records=4129/2568`，与报告第 149–150 行逐字相同 | 同上 |
| 12 | 复跑：模式 `c`（对照，只在挂载外生效）新发现数 | ✓ 快档 17、专门取样点（reuse）28，与报告「对照 c……快档 17 段、专门取样点 28 段新发现」一致 | 同上 |
| 13 | 复跑：改法（`y2-proposal-patch.py`）模式 a/b/c/off 的新发现数 | ✓ a=7/12、b=72/47、c=17/28（不变）、off=0/0 且与打改法前的 `off` 输出逐字节相同（0 处 `HJ`），与报告表格（第 178–183 行）逐字对应 | 同上 |
| 14 | 复跑：改法下模式 a 快档首个 `HJ` 行 | ✓ `HJ seed=7 pos=Operation(23) AllocationGenerationIsNotThePublishTxg { records: [AllocationRecord { device: DeviceIdentity(0), slot: SlotNumber(50240), span_slots: 2, generation: CheckpointTxg(4), is_released: false }, …` 与报告第 187 行逐字（到报告截断处为止）相同 | 同上 |

**复跑记录**（草稿目录 `/tmp/claude-1000/m2-supp3-item1-r2-verifier/`，仓副本在 `repo-copy/`、`slots/SLOT2`、`slots/SLOT2P`；日志 `slot2-*.log`、`slot2p-*.log`）：

```
# 建副本、打 Y2 变异（off/a/b/e）
rsync -a --exclude target --exclude .git <仓>/ repo-copy/ && cp -r repo-copy slots/SLOT2
cd slots/SLOT2 && python3 opus-model/y2-patch.py . && python3 opus-model/y2e-patch.py .
cp opus-model/opus_r2_y2.rs crates/singlefs-harness/tests/
nice -n 19 cargo build --release -p singlefs-harness --tests   # Finished in 9.01s，0 处编译错误
for m in off a b e; do OPUS_Y2=$m nice -n 19 cargo test --release -p singlefs-harness \
  --test second_transaction_supplement_three_random_history -- --exact \
  random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation \
  reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms --nocapture; done
diff <(grep -v -E "用时|finished in" slot2-mode-off.log) <(grep -v -E "用时|finished in" slot2-mode-a.log)   # 空
OPUS_Y2=off nice -n 19 cargo test --release -p singlefs-harness --test opus_r2_y2 -- --ignored --exact opus_r2_y2_mount_only_generation --nocapture
  # → Y2 mode=off tier=fast new_findings=0 mount_rewrites=154 other_rewrites=242 mount_records=4129
  # → Y2 mode=off tier=reuse new_findings=0 mount_rewrites=368 other_rewrites=450 mount_records=2568
# 模式 c（对照，手工定点替换）
python3 -c '...old→new 见 opus 报告第 44–45 行...'  # 命中 1 处
OPUS_Y2=c nice -n 19 cargo test ... # 快档(96段)新发现17，专门取样点(48段)新发现28
# 改法：SLOT2 拷成 SLOT2P，打 y2-proposal-patch.py
cp -r slots/SLOT2 slots/SLOT2P && cd slots/SLOT2P && python3 opus-model/y2-proposal-patch.py .
nice -n 19 cargo build --release -p singlefs-harness --tests   # Finished in 8.18s，0 处编译错误
for m in off a b c; do OPUS_Y2=$m nice -n 19 cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --exact ... --nocapture; done
  # off: 快档新发现0、reuse新发现0（与打补丁前逐字节相同，0 处 HJ）
  # a:   快档新发现7、reuse新发现12
  # b:   快档新发现72、reuse新发现47
  # c:   快档新发现17、reuse新发现28（不变）
cp opus-model/opus_r2_gap.rs crates/singlefs-harness/tests/ && nice -n 19 cargo build --release -p singlefs-harness --tests
OPUS_Y2=a OPUS_FIRST=0 OPUS_SEEDS=96 OPUS_OPS=30 OPUS_WEIGHTS=broad nice -n 19 cargo test --release -p singlefs-harness \
  --test opus_r2_gap -- --ignored --exact opus_r2_gap_probe --nocapture
  # → HJ seed=7 pos=Operation(23) AllocationGenerationIsNotThePublishTxg { records: [... SlotNumber(50240) ...
  # SUMMARY ... "new:HarnessJudgement { judgement: \"改写或新增的分配记录的代不是这次发布的 txg\" }": 7
```

**计数**：核了 14 处，✓ 13 处，✗ 1 处。

## 二、Sonnet 辩方腿（`m2-supp3-item1-code-r2-sonnet-output.md`，sha256 `f6fa6000…`，现查与交回一致）

本腿没有产物、没有复跑命令（自陈「本轮全程 Read/Bash 现查」），核的都是文件引用。

| # | 引用 | 核的结果 | 命令 |
|---|---|---|---|
| 1 | `allocator.rs:651`「`pub fn records(&self) -> &[AllocationRecord]`」 | ✓（判别力自证同一条） | 见上 |
| 2 | `allocator.rs:574`（文档注释「已回收、还没被复用的落点……只住内存。」） | ✓ 逐字同 | `sed -n '574p' crates/singlefs-core/src/allocator.rs` |
| 3 | `allocator.rs:574`（同一处又引作字段声明 `reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,` 的行号） | **✗** 该字段声明实际在第 **575** 行；574 行是它上面的文档注释（第 2 条已核对） | `grep -n "reclaimed: BTreeSet" crates/singlefs-core/src/allocator.rs` → `575:` |
| 4 | `history.rs:1472`「`let records_before = session.allocator.records().to_vec();`」 | ✓ 逐字同 | `sed -n '1472p' crates/singlefs-harness/src/history.rs` |
| 5 | `history.rs:1485-1493`（`Ok(output)` 分支到 `harness_judgement` 调用） | ✓ 1485 行为 `Ok(output) => {`，1493 行为该调用的收尾 `);` | `awk 'NR==1485{print} NR==1493{print}' history.rs` |
| 6 | `crash.rs:142`（`MemoryPool` 结构体声明，行文用来支撑「P1 读的是这份镜像字节」） | ✓ 142 行为 `pub struct MemoryPool {`；`image_before_raising` 参数本身声明在 `history.rs:2031`，报告未把 142 当参数声明行引，读法不冲突 | `sed -n '140,145p' crash.rs`；`grep -n image_before_raising history.rs` |
| 7 | `history.rs:2027-2028`（P1 读法引文，二.2 节） | ✓ 与第 2 条同一区间逐字同（含省略号处的括注在原文里确实存在） | 同上 |
| 8 | `m2-supp3-item1-implementer-report.md:390,393,394`（O/U/N 三类计数表与「只数了槽、没核机理」） | ✓ 三行内容与报告转述一致 | `sed -n '385,394p' <implementer 报告>` |
| 9 | `.claude/kb/milestone/02-second-txn.md:311`（收口表第 ② 行，二.4/四.2 两处共引） | ✓ 整行与「生成器已知红第 0 条按今天的宽度接走…只被还在根环里的被抛弃根引用」一致 | `sed -n '311p' 02-second-txn.md` |
| 10 | `implementer-report.md:436-445`（窗口判出率表，48 那一行「至少 2、判出 0 个的窗 0」） | ✓ 逐字同 | `sed -n '434,447p' <implementer 报告>` |
| 11 | `second_transaction_supplement_three_random_history.rs:25-27,177`（种子区间/步数常量、`REUSE_AFTER_RAISING_THE_FLOOR`） | ✓ 逐字同 | `sed -n '25,27p;177p' <测试文件>` |
| 12 | `implementer-report.md:422`（B1 快档红、debug/release 各跑一次） | ✓ 逐字同 | `sed -n '422p' <implementer 报告>` |
| 13 | `implementer-report.md:507`（13.8 节 B1 那一行） | ✓ 逐字同 | `sed -n '505,510p' <implementer 报告>` |
| 14 | `02-second-txn.md:383` 及边界「383-431」（增补 3 标题与现状段范围） | ✓ 383 为标题行，432 为下一个 `##` 标题，边界确为 383–431 | `grep -n "^## 增补 3\|^## 与主线并行" 02-second-txn.md` |
| 15 | `02-second-txn.md`「验收标准段（第 195-207 行区间，`grep -n '^\*\*验收标准\*\*'` 现查在第 195 行）」 | **✗** 该 grep 在全文的命中行号是 50/78/105/129/154/181/208/233/256/292/375/**419**/451/475/497/518，**没有 195**；而 Sonnet 转述的「模型接上之后……这四条变异加 N2、B6 都要判红」那句话，逐字实际在第 **403** 行（「## 增补 3」下「设想实现」第 2 件正文里），根本不在任何一处「验收标准」标题之下 | `grep -n "^\*\*验收标准\*\*" 02-second-txn.md`；`sed -n '403p' 02-second-txn.md` |
| 16 | `02-second-txn.md:358`「按状态序号切 512 片」 | ✓ 整行逐字同（含后半句「512 只是这台测试机…的产物」这一判断，`crash.rs` 常量与 `state_slices` 现查确认片数按线程数算，非写死 512） | `sed -n '358p' 02-second-txn.md`；`sed -n '890,894p;1056,1062p' crash.rs` |
| 17 | `crash.rs:890,892`（两个层 0 切片常量） | ✓ 逐字同 | `sed -n '890p;892p' crash.rs` |
| 18 | `02-second-txn.md:358`「两条流的……与单线程逐字相同」 vs `implementation-workflow.md:47`「并且在同一份代码上核过一次」 | ✓ 两处引文均逐字同；四.4 节据此指出第二条流用的是旧提交单线程日志、非「同一份代码上」重跑，推翻条件写清楚 | 同上；`sed -n '47p' implementation-workflow.md` |
| 19 | `implementation-workflow.md:46,48,49,51`（写法/进度/不能并行/门禁 54 号四处引文） | ✓ 四行内容与转述逐字对应 | `sed -n '42,52p' implementation-workflow.md` |
| 20 | `m2-layer0-parallel-implementer-report.md:224,84,11,139,149`（第二条流单线程未重跑、对照日志出处、三行逐字相同、判红那支只拿合成日志核过） | ✓ 五处均逐字同 | `sed -n '224p;84p;11p;139p;149p' <该报告>` |
| 21 | `crash.rs:1252,943,568,1296`（`std::thread::scope`、`from_environment`、`absorb_following_slice`、`LAYER0_PROGRESS` 打印） | ✓ 四处均逐字或到函数签名行同 | `sed -n '1252p;943p;568p;1296,1297p' crash.rs` |
| 22 | `.claude/gate.d/54-layer0-replay.sh:47,86,112`（函数签名、两条流调用点） | ✓ 三处逐字同 | `sed -n '47p;86p;112p' 54-layer0-replay.sh` |
| 23 | `54-layer0-replay.sh:61-62`（引作 `worker_threads_are_acceptable` 分支 b 判红条件 `"$2" == 1 && ...` 所在行） | **✗** 61、62 行实际是函数收尾的 `}` 与空行；该判红条件真正所在的行是 **54 行** | `sed -n '47,63p' 54-layer0-replay.sh` |

**计数**：核了 23 处（含合并计数的复合行），✓ 20 处，✗ 3 处。

## 三、本地攻方腿

产物：提示 `m2-supp3-item1-code-r2-local-attack.md`；样本 `-output-s1.md`（sha256 `49f183aa…`，与交回一致）、`-output-s2.md`（sha256 `a807d381…`，与交回一致）；转述核对表 `-local-attack-translation-audit.md`；运行记录 `-local-attack-runlog.md`。

### 3.1 产物与门禁复跑

| # | 检查项 | 核的结果 | 命令 |
|---|---|---|---|
| 1 | s1/s2 的 sha256 | ✓ 均与交回一致 | `sha256sum research/prompts/m2-supp3-item1-code-r2-local-attack-output-s{1,2}.md` |
| 2 | 运行记录称两份样本词数分别为 353、290 | ✓ 逐字同 | `wc -w` 两份样本 |
| 3 | 运行记录称 `corruption-check.py`+`oov-check.py` 两份都判绿、退出码 0 | ✓ 重跑得同样结果（`cjk=0 fffd=0 …零` / `生词=0 拼接=0`，两处退出码均 0） | `nice -n 19 python3 research/scripts/corruption-check.py <样本>`；同 `oov-check.py` |

这一步不核 Q1–Q18 各题答案本身的算术对不对——那是本地腿的结论（推理），不归核查员核，见「没做什么」。

### 3.2 转述核对表逐行核（`research/prompts/m2-supp3-item1-code-r2-local-attack-translation-audit.md`）

| # | 表里「原文文件:行」 | 核的结果 |
|---|---|---|
| 1 | `crash.rs:1055`「把 [0, state_count) 按序号切成首尾相接的区间。」 | ✓ 逐字同 |
| 2 | `crash.rs:1061-1066`（scaled 模式两行公式）；对照 `crash.rs:923` | ✓ 逐字同，提示确实译的是代码本身（含 `ceil_div`），非只按注释简化 |
| 3 | `crash.rs:1072-1074`（`.saturating_add(...).min(state_count)`） | ✓ 逐字同 |
| 4 | `crash.rs:589-597`；注释 `crash.rs:566`「计数逐项相加」 | ✓ 逐字同 |
| 5 | `crash.rs:598-600`；注释 `crash.rs:566`「『第一处』只在前面各片都没有时取这一片的」 | ✓ 逐字同 |
| 6 | `crash.rs:613-616`（`or_insert`，按 invariant 独立判定） | ✓ 逐字同 |
| 7 | `crash.rs:566`「各片按状态序号从小到大并」vs 提示用 `slice_index` | ✓ 提示 42 行确已显式建立「slice_index 递增 ⇔ 状态序号递增」等价关系，未凭空替换限定词 |
| 8 | `.claude/gate.d/54-layer0-replay.sh:42`（两处 `*`＝zero or more） | ✓ 首稿的「one or more」误译已在定稿改成「zero or more」，与脚本 `sed` 的 `*` 语义一致 |
| 9 | `54-layer0-replay.sh:43`（同上） | ✓ 同上 |
| 10 | `54-layer0-replay.sh:76→81→86`（判空→判 exhaustive→调 `worker_threads_are_acceptable` 的次序） | ✓ 三行次序现查确为 76 < 81 < 86，定稿已按此次序改写提示步骤 5–9 |
| 11 | `54-layer0-replay.sh:73`（`grep -A1` 取「每一处命中」非仅第一处） | ✓ 逐字同 |
| 12 | `54-layer0-replay.sh:22`（`-n` 判非空） | ✓ 逐字同 |
| 13 | `54-layer0-replay.sh:54`（分支 b 完整布尔条件） | ✓ 逐字同 |
| 14 | `crash.rs:890,892,894`（三个常量的数值） | ✓ 逐字同 |
| 15 | （未译入）`crash.rs:567`「按字段拆开写全：新加一个字段而这里没并，编译不过。」 | ✓ 逐字同，登记「不适用」的理由（实现动机非可计算规则）成立 |
| 16 | （未译入）`54-layer0-replay.sh:30, 32-36`（`LAYER0_PROGRESS` 边跑边转发） | ✓ 逐字同，登记「不适用」的理由（旁路、不影响判红判绿）成立 |

「多出来的限定词」一节称「未发现」——核对表本身列出的加法只有第 7 行的 `slice_index` 说法，而它已在提示里被显式接回「状态序号」的等价关系，不算凭空加宽，与核对表结论一致。

**计数**：3.1 核了 3 处，✓ 3 处；3.2 核了 16 处，✓ 16 处，✗ 0 处。

## 四、总计

三条腿合计核了 56 行（Opus 14 行、Sonnet 23 行、本地攻方 19 行 = 3.1 节 3 行 + 3.2 节 16 行），✗ 4 行：Opus 1 处（`SHA256SUMS` 与报告内联哈希块不同文，漏了 `y5-panic-probe.diff`）、Sonnet 3 处（`allocator.rs` 字段声明行号 574→应为 575；`54-layer0-replay.sh` 判红条件行号 61-62→应为 54；`02-second-txn.md` 「验收标准段第 195 行」不存在，被转述的内容实际在第 403 行、且不在任何「验收标准」标题之下）。Opus 报告里明确要求复跑的 Y2 第二分句两种形态（模式 a/b）与它量过的改法（`y2-proposal-patch.py`），在仓副本上逐项重跑，包括一处具体的 `HJ` 行文本，全部与报告逐字或逐数字吻合，判 ✓。

## 五、没做什么

- 不判三条腿的攻防结论本身成不成立、该不该采纳；三条腿之间谁的判断更有道理，是主 agent 的事。
- 未独立复跑 Opus 报告 Y1（13 条变异 × 3 档规模）与 Y5（60 趟差分、20 趟自证、10 趟 panic 注入）的数字——派发提示明确点名只需复跑「Y2 第二分句两种形态与它量过的改法」，Y1/Y5 只核了它引用的代码行号与 `mutations.tsv`/`mutants.tsv` 的文字内容，没有重新跑那些变异或差分探针。
- 复跑 Y2 时只核了两条入库的两档（快档 96 段、专门取样点 48 段）用例，没有复跑 Opus 报告提到但没有展示完整原样输出的 broad 3000×40 / reuse 3000×30 大档「基线误红」claim（报告称量过、留在 `logs/Y2P-off-*.log`，我没有去它自己的 `/tmp` 目录里核那两份日志，因为派发的范围是「在仓的副本上复跑」，不是去核对方 `/tmp` 里的原始日志）。
- 不核 Q1–Q18 各题计算本身对不对（state_slices 切片是否真的无缝、merge 规则算出的值、gate 脚本判红判绿的推演）——这是本地攻方腿的推理结论，核查员只核转述核对表（原文文件:行、译文有没有丢限定词或多加限定词），不核算术本身；主 agent 若要用这些答案去核 Opus/Sonnet 对同一段代码的描述，需要自己或另一条腿重新核这十八题的答案。
- 未跑任何门禁阶段：`awk -F'\t' -v me=three-way-verifier '...' .claude/gate.d/stage-owners.tsv` 数出 0 个登记给核查员的阶段。
- 未编译或跑主工作区任何文件；本轮所有 `cargo build`/`cargo test` 都在草稿目录的仓副本（`/tmp/claude-1000/m2-supp3-item1-r2-verifier/repo-copy`、`slots/SLOT2`、`slots/SLOT2P`）上跑，主工作区 `crates/`、`.claude/gate.d/` 一字未改。
- 未清理 `/tmp/claude-1000/m2-supp3-item1-r2-verifier/` 下的仓副本与编译产物（约数百 MB，含 `target/`），留作复核证据；未拷进 `research/results/`（本轮是核查记录不是新实验，产物性质与「跑前登记的实验」不同，是否需要额外落盘由主 agent 定）。
