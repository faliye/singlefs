# m3-batch23-code-r1 攻方腿（Opus，接手）报告（2026-10-06）

前一条同立场的腿做完了攻击，没写成报告（它会话里的 Bash 被权限层拒了）。这一份由接手的腿写：在自己的仓副本与编译目录上把它的全部打中、对照与随机批从头复跑了两遍（两个空目录），数与它的原始日志逐行相同之后才写。下面的数都是**副本上的数**，不是入库装置上的数。

## 复跑命令与文件 sha256

```
# 冻结副本里没有 research/，包装脚本目录用 SINGLEFS_TOOL 给
export SINGLEFS_REPO=/tmp/claude-1000/m3-batch23-code-r1/tree SINGLEFS_TOOL=<仓>/research/scripts
bash research/prompts/m3-batch23-code-r1-opus-model/rerun.sh <空目录> all        # 估约 10 分钟（按两遍分段的挂钟加的）；或按段：setup build n5 …
bash research/prompts/m3-batch23-code-r1-opus-model/rerun.sh compare <同一目录>  # 不读 SINGLEFS_REPO
```

- 编译经 `run-with-memory-cap.sh 6G`，跑经 `4G`；线程经 `capped.sh 7`（`OPUS_THREADS` 可改）；全部 `nice -n 19`。只编、只跑 `singlefs-harness` 的 `opus_b23_attack` 与 `checker_known_bad_images` 两个目标，GPU 不跑，checker 档一个不跑。
- `all`：setup、build、八个单段依次跑，四个自证段并行，十六个随机批段四段一组并行（每段退出码写 `logs/<段>.rc`，单个 `wait` 等齐后数个数）。
- `compare`：各段日志里的 `RESULT` 行、`test … ok` 行、去掉耗时的 `test result:` 行，加各段退出码，每行带段名，`LC_ALL=C sort` 之后与 `expected-summary.txt` 比。
- 在空目录里从头跑通两次：`/tmp/claude-1000/m3-batch23-code-r1-opus2/rerun-work`（它的摘要就是 `expected-summary.txt`）与 `/tmp/claude-1000/m3-batch23-code-r1-opus2/rerun-work2`（不带 `SINGLEFS_REPO`、`SINGLEFS_TOOL` 跑 compare：`compare: 一致（612 行）`，退 0）。把一份日志里 `versions=11` 改成 `12` 再 compare：`compare: 不一致，差异 8 行`，退 1。
- 跑第二遍时我是按段手动分组起的（每条前台命令要在 240 秒内）：`setup build n5 n5-fixa batch-test fixa-legal n4 n7`、`parallel n6 cliff st-29 st-24 st-18 st-death`、`parallel b1-2a b1-2b b1-1a b1-1b b1-3a b1-3b`、`parallel b2-2a b2-2b b1-4a b1-4b b2-1a b2-1b b2-3a`、`parallel b2-3b b2-4a b2-4b`。`all` 那条路径只用一个打桩的 `parallel_group` 核过分组（四组、每组四段），没有整条跑过。

| 文件（`research/prompts/m3-batch23-code-r1-opus-model/`） | sha256 | 谁写的 |
|---|---|---|
| `opus_b23_attack.rs`（1795 行） | `0041c725b96892f32145d594b5fb9a6e90610a4bf29babb5b12fa71155d341c4` | 前一条腿 |
| `apply_capture_hook.py` | `a4eb4a07cda74d3d4a1ca89412eca501a7be82c2f1c4dbff0e562c924ee7f7fe` | 前一条腿 |
| `apply_fix_a.py` | `2d886c9078dd072ab0fbb47d13c2faa6b857e0ff1433c82e8e8b96c760617ec1` | 前一条腿 |
| `rerun.sh`（186 行） | `c38adf7520e75188999e93fe2aa3570387400fde9a7ccb0a5a39248fe82ecc98` | 接手的腿 |
| `expected-summary.txt`（612 行） | `dcd0fb2ff242db019198aed624047c2c99845ef5fe307b25bbbe066de4a475cd` | 接手的腿（第一遍复跑的摘要） |

- `apply_capture_hook.py` 在副本的 `walk.rs` 里加一个只记数的钩子：记下删除态扫描用的版本清单，以及 I-9.29 那一判滤没滤掉被抛弃的根。判定一处不改。`apply_fix_a.py` 是改法 A，只施加在 `tree-fixa` 上。
- 两个脚本施加在冻结副本上得出的 `walk.rs`，与前一条腿副本里的那份逐字节相同（`cmp`）。
- 冻结副本 13 份被判文件过 `sha256sum -c research/prompts/m3-batch23-code-r1-snapshot/crates-sha256.txt`：13 个 OK。

## 各格判定一览

| 格 | 判定 | 依据（副本上） |
|---|---|---|
| K2 N5：实例表读得出时被误报判不了 | **打中**（三形：N5-a、N5-b、N5-c；最硬的是 N5-a，整张镜像零红） | 第一节 |
| K2 N5：读不出时漏判 | 没打中（读不出时恒报部分没判，按 m3-batch22-code-r1 第五节口径不算漏判）；线索一条（N5-e） | 第二节 |
| K2 N1、N3：CPU 与 GPU 判得不一样 | 没打中（一次抽样）：译本对拍 3093 次判定，对不上 0 次；线索一条（2048 格那张表的悬崖） | 第三节 |
| 第二次抽样 N4（其余几形只红自己） | 没打中（第二次） | 第四节 |
| 第二次抽样 N5、N9（合法流误红） | 没打中（第二次）：710 个合法状态 I-9.29 全部成立 | 第四节 |
| 第二次抽样 N6（产品读法 I-9.21） | 没打中（第二次） | 第四节 |
| 第二次抽样 N7（I-9.24） | 没打中（第二次）；[5, 5] 照上一轮判决第三节第 3 条那一问只记 | 第四节 |

## 一、N5 打中：I-9.29 的「判不了」门与候选集的滤法用的不是同一个读法

正文要攻的格：`research/prompts/_m3-batch23-code-r1-body.md:45`：「N5：实例表读得出时有没有被误报判不了；读不出时有没有漏判。」
条款：`.claude/kb/invariants.md:316`：「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」。

**代码（冻结副本，现查）**
- 候选集用主走读收下的行来滤。
  - 行从这里来：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10246`：「let instance_table_rows = walk.instance_table_rows.clone();」
  - 回退候选集按这张表滤：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10318`：「let abandoned = abandoned_by_the_newest_roots_table(root);」
- 「滤没滤全」那道门用的是另一个读法，只读不判：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10254`：「let abandoned_roots_filtering = match instance_table_rows_of_root_without_judging(」。它交回 `None` 时报判不了：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10259`：「None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,」
- 两个读法的拒收条件至少有三处不同。每一处都是只读不判那个读法拒了，主走读照样收行：
  1. **链指针记录之前有不是 kind 0 的记录。** 只读不判：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:5500`：「if row[0] != 0 {」，接着就 `return None`。主走读只把 kind 0 那几条收进表：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2822`：「self.instance_table_rows.push(parse_instance_table_row(row));」；别的 kind 跳过、不红：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2829`：「_ => chain_record_last_in_this_page = false,」
  2. **链指针记录不合法。** 只读不判：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:5511`：「ChainRecordView::Malformed => return None,」。主走读先收下这一片：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2784`：「pages.push(records);」；再停：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2793`：「ChainRecordView::Malformed => break,」；收下的行照样判：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2799`：「self.judge_instance_table_rows(&pages, mount_root_instance);」
  3. **载荷 CRC 对不上，整单元校验和对得上。** 只读不判：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:5476`：「if !matches!(check_unit(&unit), Ok(3)) {」，`check_unit` 判载荷 CRC：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/lib.rs:444`：「if read_u32(unit, payload_crc_offset) != crc32_castagnoli_bitwise(&unit[header_end..]) {」。主走读只红 I-2.3 就往下用：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:1239-1240`：「if !payload_crc_holds {…return true;」
- 门的注释写了用意：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10252`：「// 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。」三处不同都落在「表读全了，门却说没读全」这一侧。

**镜像与输出**（模型 `n5_the_one_birth_invariant_against_what_the_newest_instance_table_reads_back`）
- 起点：建 5 号 → 那次发布被崩溃恢复抛弃 → 新线重发 5 号。被抛弃的那条根读得出，5 号在它上面是另一个出生代，只靠最新根的实例表滤掉。实例表是一片，两条记录：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:427`：「RESULT N5-setup 最新根实例表 槽50304 记录数2 各条 kind [0, 1]」
- 每形改最新根的实例表，重封，补引用链：
  - 干净：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:416`：「RESULT N5-0 干净镜像 violated=[] I-9.29=成立（） filtered_flag=Some(true) versions=11 (干净镜像 11)」
  - **N5-a**，链指针记录之前多插一条 kind 2：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:417`：「RESULT N5-a 链指针记录之前多一条 kind 2 violated=[] I-9.29=部分没判（最新根的实例表读不出（链上有一片读不出或解不开）） filtered_flag=Some(false) versions=11 (干净镜像 11)」
  - **N5-b**，链指针记录的「有无下一片」写 2：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:418`：「RESULT N5-b 链指针记录有无下一片写 2 violated=["I-3.8"] I-9.29=部分没判」
  - **N5-c**，载荷 CRC 坏、整单元校验和补过：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:419`：「RESULT N5-c 载荷 CRC 对不上、整单元校验和补过 violated=["I-2.3"] I-9.29=部分没判」
  - **对照 N5-d**，两份都读不出（批里自带用例的造法）：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:420`：「RESULT N5-d 两份都读不出（对照） violated=["I-2.1", "I-3.1", "I-4.8", "I-7.2", "I-7.4"] I-9.29=部分没判」，那一行里候选版数是 12。
- 怎么读这几行：`versions` 是钩子记下的删除态扫描版本数，就是滤过之后的回退候选集。N5-a、N5-b、N5-c 都是 11，与干净镜像相同，被抛弃的根已经滤掉了；N5-d 是 12，被抛弃的根留在候选集里。所以 a、b、c 三形上「被崩溃恢复抛弃的根滤不掉」不成立，I-9.29 却报了判不了，一个号都没比。报法见 `/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:9369`：「"最新根的实例表读不出（链上有一片读不出或解不开）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",」
- N5-a 最硬：整张镜像一条都不红，主走读收下了全部 kind 0 行，可 I-9.29 部分没判。N5-c 上 I-2.3 红了，这一片算不算「读得出」可以两说，门与滤法不一致这一点两种读法下都成立。N5-b 见下面改法表那一行。

**四句**
1. 分不分辨臂：这一轮正文没写跑前臂。这一格分得开「门用只读不判那个读法」（今天的代码）与「门看主走读走没走到末片」（改法 A）：N5-a、N5-c 上前者报判不了，后者报成立（下表）。
2. 判别子看不看得到：看得到。主走读在同一个函数里知道自己有没有走到末片（`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:2786`：「ChainRecordView::LastPage => break,」），滤候选集用的又正是它收下的行。
3. 满足的是哪一个分句：是正文 K2 N5 前半句「实例表读得出时有没有被误报判不了」。它不是 m3-batch22-code-r1 第五节的误红（没报不成立），也不是漏判（`research/prompts/_m3-batch22-code-r1-body.md:68`：「配不上的版报部分没判不算漏判」）。kb 那一行「读不出、滤不掉时记判不了」的条件在 a、b、c 三形上都不成立。
4. 跑前改法：正文第五节只写「打中：主 agent 现查坐实，改法进下一批」，没给具体改法，所以没有一个要逐个核的。

**改法**（都是攻方提的，算零轮。改法 A 是前一条腿提的、写成了 `apply_fix_a.py`，我在自己的副本上复跑过，被攻过零轮；改法 B 没实现）

- 改法 A：门不再另读一遍。主走读沿最新根的实例表链读到链指针记录说「没有下一片」，就记「滤全了」；没走到末片就记判不了。
  - 改在 `Walk` 上加一个字段，在 `ChainRecordView::LastPage` 那一支、`mount_root_instance_when_newest` 给了时置真。
  - `walk.rs:10254` 那个 `match` 换成看这个字段。
- 改法 B（推的）：放宽只读不判那个读法，让它与主走读收同一批行。缺点：这个读法还给 I-7.9 与记录核对器用（`walk.rs` 5540、5635、7743 三处调用），改它会动到这两处。

| 镜像 | 今天的代码 | 改法 A（量过，副本 `tree-fixa`） | 改法 B（推的） |
|---|---|---|---|
| N5-0 干净 | 成立 | 成立（`expected-summary.txt:430`） | 成立 |
| N5-a 多一条 kind 2 | 部分没判 | **成立**，`filtered_flag=Some(true)`（`expected-summary.txt:431`） | 成立 |
| N5-b 链指针记录写坏 | 部分没判 | 部分没判（`expected-summary.txt:432`） | 看放宽到哪 |
| N5-c 载荷 CRC 坏 | 部分没判 | **成立**（`expected-summary.txt:433`） | 成立 |
| N5-d 两份都读不出 | 部分没判、12 版 | 部分没判、12 版（`expected-summary.txt:434`） | 部分没判 |
| 批里自带用例 `an_abandoned_root_naming_another_object_birth_…` | ok | ok（`expected-summary.txt:371`） | 推的 ok |
| 合法流 8 段 × 10 步、前缀 reissue，65 个状态 | — | I-9.29 全部成立（`expected-summary.txt:392`） | — |

- 原样输出：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:431`：「RESULT N5-a 链指针记录之前多一条 kind 2 violated=[] I-9.29=成立（） filtered_flag=Some(true) versions=11 (干净镜像 11)」
- 原样输出：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:392`：「RESULT campaign knob=0 fault="" seeds=300..308 steps=10 mutations=0 clean I-9.29=成立 65」
- **N5-b 在改法 A 下仍报判不了，我认为这是对的。** 链指针记录写坏了，主走读不知道后面还有没有片。这一形单片的表其实读全了，但从镜像上分不出「单片」与「后面还有片」。所以 N5-b 只算「门与滤法不一致」的一个例子，改法 A 不修它，也不该修。
- 改法 A 没在带改坏的随机批上跑过，只跑了上面 7 个点。

**反方向**（门说滤全了，候选集其实没滤全）：没找到。
- 最新根先走：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:10212`：「walk.walk_root(&roots[newest_index].2.record_bytes, true);」。走到 `visited_units` 提前停的那一支，最新根这条链碰不上。
- 两边都从位置条目读：主走读用 `image.rs` 的 `read_referenced_unit`，只读不判那一边用 `read_unit_without_judging`，都只取整单元校验和对得上的那一份。
- 两边都判头、身份四元组、记录宽、声明长度。上面三处之外，我看到的不同也是主走读收得更宽：头里 flags 不为 0 时，`check_unit` 拒收，主走读只红 I-1.6、照样往下走。
- 我没逐条列全两边的拒收条件。前一条腿报「其余逐条相同」，我只核了上面这几条。

## 二、N5 读不出时：没打中；线索 N5-e

- 读不出时每一形都报部分没判：N5-d、N5-e1，加上随机批里每一步改坏的状态。按 m3-batch22-code-r1 第五节口径，这不算漏判。
- 随机批里 `unfiltered_states=0`：没有一个改坏的状态让门报判不了。改坏落不到实例表上，这条路没被随机批扫到。
- **线索 N5-e**（攻方推的，交主 agent 定）：
  - 造法：只把最新那一版里 1 号的出生代加一，它就与同一实例（实例 2）更早那一版 txg 10 不同。
  - 表读得出时报不成立：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:422`：「RESULT N5-e0 最新那一版 1 号出生代加一、实例表读得出 violated=["I-9.29"] I-9.29=不成立」
  - 再让实例表两份都读不出，就报部分没判：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:423`：「RESULT N5-e1 同上、实例表两份都读不出 violated=["I-2.1", "I-3.1", "I-4.8", "I-7.2", "I-7.4"] I-9.29=部分没判」
  - 推的理由：被抛弃按「有那个实例的行且 txg 大于行里的 Ti」判。最新根自己那个实例在表里没有行（I-3.8 要求行的实例低于挂载根），所以它那几版不会被抛弃，表读不出时也能互相比。今天的代码一个号都不比。
  - 注意：e0 那一行的说明只列了每个出生代第一次出现的那条根（实例 1 txg 3 与实例 2 txg 11）。「实例 2 txg 10 与 txg 11 不同」是我按造法推的，没单独打出来。

## 三、N1、N3：没打中（一次抽样）；线索一条

**做法**（模型是前一条腿写的，我只复跑，没逐行复核译本与 wgsl 的对应）
- 把扫描内核里墓碑与谱系那一遍，以及删除态扫描的 I-9.24，译成 Rust（模型 `gpu_model`，`research/prompts/m3-batch23-code-r1-opus-model/opus_b23_attack.rs:425` 起）。wgsl 不跑。
- 每个状态上 CPU 池级 checker 旧入口判一次、译本判一次，逐位比。比哪几位：`research/prompts/m3-batch23-code-r1-opus-model/opus_b23_attack.rs:943`：「const TARGETS: [&str; 6] = ["I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.24", "I-9.29"];」。I-9.10 只比墓碑那一个等号。
- CPU 滤不掉被抛弃的根时，I-9.29 不比：`research/prompts/m3-batch23-code-r1-opus-model/opus_b23_attack.rs:946`：「/// I-9.29：CPU 滤不掉被抛弃的根（最新根实例表读不出）时 GPU 那一侧照比，是第 24 批的活，这里不比。」正文把它划到下一批：`research/prompts/_m3-batch23-code-r1-body.md:24`：「GPU 侧 I-9.29…第 24 批（G 交回时 X 还没落地）」。
- 译本读的版本清单是钩子从 CPU 那一遍抄来的。GPU 自己怎么建回退候选集，这里没比。

**随机历史**
- 用户动作放开扫：建 1–3 个号、发意图、走一步、走空、回退到前 20 版里随机一版、重挂、崩溃恢复抛弃最新一次发布，每步随机挑。
- 前缀三种：rich（删 → 回退 → 再删，墓碑、谱系、队列都先有）、reissue（建号 → 被抛弃 → 新线重发同号）、无前缀。
- 删除开关：0（产品容量）、1、2、3。
- 每做成一步，干净镜像比一次；再在这一版的版本清单里随机挑一版、挑一个单元改坏，各比一次。
- 取样：第一批 4 组 × 10 段历史 × 10 步 × 每步改坏 6 次；第二批 4 组 × 6 段 × 20 步 × 2 次。复跑时每组拆成前后两半（`b1-1a` 等 16 段），每段的 `RESULT history` 行与前一条腿未拆的日志逐段相同，计数按组加起来逐键相同。
- 16 段合计（awk 加总 `RESULT campaign` 行）：

| 计数 | 值 |
|---|---|
| 干净镜像判定 `clean_checks` | 710 |
| 改坏镜像判定 `mutated_checks` | 2383 |
| 对不上 `mismatches` | 0 |
| 门报判不了的状态 `unfiltered_states` | 0 |
| 译本判不了的站点 `gpu_model_undecidable_site` 行 | 0 行 |
| 改坏之后 CPU 红的目标位：I-9.21 / I-9.29 / I-9.18 / I-9.20 / I-9.19 / I-9.10 / I-9.24 | 1122 / 307 / 234 / 187 / 77 / 74 / 31 |
| 做成的动作：回退 / 建 / 意图 / 抛弃 / 重挂 / 走一步 / 走空 | 189 / 177 / 174 / 77 / 59 / 23 / 11 |

- 一段的原样：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:57`：「RESULT campaign knob=1 fault="" seeds=100..105 steps=10 mutations=6 clean_checks=45 mutated_checks=244 mismatches=0 unfiltered_states=0」

**译本的自证**：在译本里注入四个故障（`OPUS_GPU_MODEL_FAULT`），同一套随机历史（开关 1，6 段 × 8 步 × 6 次）都抓到了：
- `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:573`：「RESULT campaign knob=1 fault="no-lineage-filter-29" seeds=0..6 steps=8 mutations=6 clean_checks=41 mutated_checks=170 mismatches=46 unfiltered_states=0」
- `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:478`：「RESULT campaign knob=1 fault="no-whole-object-18" seeds=0..6 steps=8 mutations=6 clean_checks=37 mutated_checks=194 mismatches=7 unfiltered_states=0」
- `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:506`：「RESULT campaign knob=1 fault="queue-bound-off-24" seeds=0..6 steps=8 mutations=6 clean_checks=41 mutated_checks=170 mismatches=1 unfiltered_states=0」
- `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:595`：「RESULT campaign knob=1 fault="death-bound-plus-one" seeds=0..6 steps=8 mutations=6 clean_checks=37 mutated_checks=194 mismatches=1 unfiltered_states=0」
- 摘要里 55 行 `MISMATCH` 全在这四个自证段里（46 + 7 + 1 + 1），正式随机批一行都没有。
- I-9.24 队列上界与死亡界那两个故障各只抓到 1 次，说明随机改坏落到这两支的次数少。这两支上「0 次对不上」的分量比 I-9.29、I-9.18 那几支轻。

**读码**：走读内核的 I-9.17 / I-9.23、I-9.4 第四句、N3 那两处条目宽，前一条腿报「与 CPU 和 core 逐支对过，相同」。我没复核这部分，这一格只算它的一次。

**线索（N1，只在译本上量过）：I-9.29 跨版本那张表 2048 格，合法的长删除流删满之后 GPU 每个状态都判不了，整趟放量会停下。**
- 表的上限：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:818`：「const D_MAX_OBJECT_BIRTHS: u32 = 2048u;」
- 满了记站点 59：`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2794`：「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (59u << 8u);」
- 合法流删 2044 个号还判得了，删 2048 个就判不了；CPU 都判成立：
  - `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:379`：「RESULT N6-cliff 删过 2044 个号、活的 1 个 最后一版墓碑容器记录数=[815, 815, 414] violated=[] I-9.20=成立 I-9.21=成立 I-9.29=成立 gpu_undecidable=None mismatches=[]」
  - `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:380`：「RESULT N6-cliff 删过 2048 个号、活的 1 个 最后一版墓碑容器记录数=[815, 815, 418] violated=[] I-9.20=成立 I-9.21=成立 I-9.29=成立 gpu_undecidable=Some(59) mismatches=[]」
- 放量遇到第一个判不了的状态就整趟停下、不退回 CPU：先是 `/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2005`：「work.stop.store(true, Ordering::SeqCst);」；报的原因是 `/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2009`：「配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑」
- 推的：墓碑只追加，谱系上的墓碑每一版都进这张表，所以删过 2048 个号之后，这条流后面的每个崩溃状态都超上限。正文 N1 写了「超上限记判不了」，所以这不是判错，是容量悬崖。今天的层 0 流删不到这个量（推的，没数）。4096、256 那几个上限是不是同一类，没量。

## 四、第二次抽样（m3-batch22-code-r1 第三节第 5 条那几格）：都没打中

- **N4，其余几形只红自己**（`n4_second_sample_of_container_tree_entry_identities`）：
  - 墓碑树根条目身份的出生代、出生树各加一（s1、s2）：只红 I-9.23。
  - 只有旧一版的墓碑树根条目容器号加一（s6）：只红 I-9.23，见 `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:413`：「RESULT N4-s6 只有旧一版（txg 8，根槽 50316，最新根槽 50346）的墓碑树根条目身份容器号加一 violated=["I-9.23"] I-9.23=不成立」
  - 谱系树根条目容器号加一（s3）：只红 I-9.17，见 `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:410`：「RESULT N4-s3 谱系树根类型段 5 条目身份容器号加一 violated=["I-9.17"] I-9.23=成立 I-9.17=不成立」
  - 层级 2 的墓碑树（s4 根类型段 0、s5 内层节点）：红 I-9.21 与 I-9.23。I-9.21 在这张旋钮镜像的干净版上就红（旧入口判产品容量，`/tmp/claude-1000/m3-batch23-code-r1/tree/crates/singlefs-checker/src/walk.rs:9040`：「/// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。」），所以多出来的只有 I-9.23。
  - 每一形扫描那一侧与译本都对得上（`scan_mismatches=[]`）。I-7.2、I-4.8、I-7.4、I-3.1 都没有连带红。
- **N5、N9，合法流误红**（第三节的随机批，干净镜像那一半）：
  - 710 个合法状态 I-9.29 全部成立（各段 `clean I-9.29=成立` 行加起来是 710）。I-9.17 到 I-9.29 里，干净镜像上只有 I-9.21 红过。
  - I-9.21：开关 1–3 的段上 279 + 11 个状态。它只在开关 ≥ 1 的段上出现（逐段看了 `clean_red` 行），是上面那条「旧入口判产品容量」，预期内。
  - I-3.1：12 + 11 个状态。上一轮判决把它划给根环线：`research/prompts/m3-batch22-code-r1-main-verification.md:33`：「附带：崩溃恢复落到前一条根的随机历史里 I-3.1 红」。模型只按段计数、不逐状态记，这 23 个是不是都在做过崩溃恢复的历史里，我没核。
- **N6，产品读法的 I-9.21**（`n6_product_tombstone_containers_second_sample`，产品容量 815 条一片）：
  - 合法的三形都不红：满一片；开第二片那次被崩溃恢复抛弃、新线删别的号；815 + 815 + 1。
    - `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:445`：「RESULT N6-s1 开第二片那次被崩溃恢复抛弃、新线删另一个号 最后一版墓碑容器记录数=[815, 1] violated=[] I-9.20=成立 I-9.21=成立 I-9.29=成立 gpu_undecidable=None mismatches=[]」
    - `research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:447`：「RESULT N6-s2 1631 条（815 + 815 + 1） 最后一版墓碑容器记录数=[815, 815, 1] violated=[] I-9.20=成立 I-9.21=成立 I-9.29=成立 gpu_undecidable=None mismatches=[]」
  - 四种改坏（第一片少一条、容器号重、容器号逆序、最右片多一条）CPU 都红在 I-9.21、I-9.20 或 I-9.23，译本一致。
- **N7，I-9.24**（`n7_second_sample_of_the_deletion_queue_bound`）：
  - 只有旧一版的队列是 {5, 6}（或乱序 [6, 5]）时 I-9.24 红：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:455`：「RESULT N7-s1 只有旧一版的队列是 {5, 6} violated=["I-1.1", "I-9.18", "I-9.24", "I-9.5"] I-9.24=不成立」
  - 同号两条 [5, 5] 时 I-9.24 成立、只红 I-1.1：`research/prompts/m3-batch23-code-r1-opus-model/expected-summary.txt:457`：「RESULT N7-s3 旧一版队列同号两条 [5, 5]（上一轮判决第三节第 3 条那一问，这里只记） violated=["I-1.1"] I-9.24=成立」
  - 这就是 `research/prompts/m3-batch22-code-r1-main-verification.md:41`：「队列同号两条归哪一条管：查清，没有就记欠账。」那一问。这里只记，不算打中。

## 没打中的形状

- **N5 反方向**（门说滤全了，候选集没滤全）：按代码比了两个读法（第一节末），没造出来。
- **N5 读不出时报成立或不适用**：N5-d、N5-e1，加上 2383 个改坏状态，都没有。随机改坏不碰实例表，所以这一侧只有两张手造的镜像。
- **N1、N3 对拍**：
  - 取样：64 段历史、710 个合法状态、2383 个改坏状态；前缀 3 种；开关 0–3。
  - 动作：建、意图、走一步、走空、回退（前 20 版里随机）、重挂、崩溃恢复抛弃。
  - 改坏：墓碑树、谱系树、inode 树、`deleted_inodes` 里任一单元的一个字段，取值从镜像上见过的号与 txg 里挑，落在判据边界上。
- **N6**：产品容量 815、1631、开第二片被抛弃；长流删到 2100 条。
- **N4**：s1–s6 六形。**N7**：s1–s3 三形。

## 这条腿自己的限度

- **崩溃状态一个都没枚举。** 模型直接拿合法流与改坏的镜像判，没调 `enumerate_layer0`。
  - 前一条腿报的理由是「每段历史超过 10⁶」。我没用 `closed_form_state_count` 重算这个数。
  - 所以 m3-batch22-code-r1 N9 那几格里「崩在屏障段界上」那一形，这一轮没扫到。
  - 本腿与前一条腿原型跑的崩溃状态合计：0 个。
- **译本是人译的**，自证四个故障都抓到，但 I-9.24 与死亡界两支各只抓到 1 次。译本用的版本清单抄自 CPU，GPU 建候选集那一段没比。
- **改法 A 只在 7 个点上量过**（上面那张表），没在带改坏的随机批上跑；被攻过零轮。
- **复跑是两次同一份确定性程序**：两遍与前一条腿的日志逐行相同，只说明结果可复现，不是独立的第二条验证路径。
- 走读内核的读码那部分是前一条腿的，我没复核。

## 现场核对与删除

- **接手时核的现场**：
  - 前一条腿的模型三件在 `research/prompts/m3-batch23-code-r1-opus-model/`，`rerun.sh`、`expected-summary.txt` 与报告都不存在。
  - 它副本里的 `opus_b23_attack.rs` 与模型目录那份逐字节相同。
  - 它 `logs/` 里各段的 `.rc` 都是 0。
- **与它原始日志的比对**：
  - n5、n5-fixa、fixa-legal、n4、n6、n7、cliff 的 `RESULT` 行逐行相同。
  - 批里自带用例在两份副本上都是 ok，与它的一样。
  - 8 组随机批：每段历史的 trace 行相同，计数加总逐键相同。
  - 4 个自证段的计数行相同。
- **开跑前看的负载**（`ps -u`）：
  - 有 E171 的 `device_timing.py run`（pid 657309，测装置一趟的挂钟，用来定逐行限时）在跑。它不在共用约束点名的那几种性能测量里，所以我照常 `nice -n 19` 跑了。我的进程只会让它量到的挂钟偏长，偏长的方向是限时放宽。
  - 另有别的会话的 `cargo test -p singlefs-harness` 与 `random_histories` 在跑。
- **删除前 `du -sh`**（删的是仓副本与编译目录，`logs/` 都留着）：

| 目录 | 大小 |
|---|---|
| `/tmp/claude-1000/m3-batch23-code-r1-opus/tree`、`tree-fixa` | 19M、19M |
| `/tmp/claude-1000/m3-batch23-code-r1-opus/target`、`target-fixa` | 614M、615M |
| `/tmp/claude-1000/m3-batch23-code-r1-opus2/tree`、`tree-fixa`、`target`、`target-fixa` | 19M、19M、614M、615M |
| `/tmp/claude-1000/m3-batch23-code-r1-opus2/rerun-work/` 下同名四个 | 19M、19M、614M、615M |
| `/tmp/claude-1000/m3-batch23-code-r1-opus2/rerun-work2/` 下同名四个 | 19M、19M、614M、615M |

- **删完之后**：`/tmp/claude-1000/m3-batch23-code-r1-opus` 剩 252K，有 `logs/`、`run.sh` 与 `.rc`；`/tmp/claude-1000/m3-batch23-code-r1-opus2` 剩 1.0M，有两遍复跑的 `logs/` 与 `summary.txt`、模型目录的拷贝 `model-copy/`、交接摘要。
- 这些 `/tmp` 路径只是告诉主 agent 原始输出在哪。本报告引的数都取自仓里的 `expected-summary.txt`。

## 没做什么

- 没在入库装置上跑：数都是副本上的。改法 A 只施加在副本上，没进代码。
- GPU 不跑，checker 档不跑，崩溃状态不枚举（见上一节限度）。
- 没复核译本与 wgsl 的逐行对应，也没复核走读内核那部分读码：这两样是前一条腿做的。
- 没为 N5 反方向写镜像，只读了代码。N5-e 只造了一形。
- 改法 B 没实现。
- `rerun.sh all` 没有整条跑过：子 agent 的前台命令有 240 秒上限。两遍都是按段分组跑的，分组循环用打桩核过。
- 正推腿的报告与模型、本轮判决没读（禁读）。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-batch23-code-r1-opus-output.md --root /tmp/claude-1000/m3-batch23-code-r1/tree --root . --background research/prompts/_m3-batch23-code-r1-background.md` 末行（退 0）：

```
  ✓ 核了 54 处引文，对上 54 处，没判 0 处（逐处列在上面）
```
