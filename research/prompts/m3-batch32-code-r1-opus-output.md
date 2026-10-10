# m3-batch32-code-r1 云端攻方（Opus）报告（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 P1 P2 P3 P4 P5 P6 P7 F-KEY F-ID-b F-L0 F-W F-IT O3 A3 -->

攻击面：正文第三节 K2，重点 P1（F-KEY，含 I-4.9 那一形）、P4（F-零槽）；另扫 P2 的副本组合与 CPU / GPU 分叉。立场：假设实现与实现员的结论句是错的，造反例。


## 复跑

模型目录 `research/prompts/m3-batch32-code-r1-opus-model/`。在仓根下起（前台有时限时分两步，`run` 可续）：

```
W=<一个空目录>
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-batch32-code-r1/tree SINGLEFS_TOOLS=<仓根> THREADS=4 \
  bash research/prompts/m3-batch32-code-r1-opus-model/rerun.sh "$W" build
SINGLEFS_TOOLS=<仓根> bash research/prompts/m3-batch32-code-r1-opus-model/rerun.sh "$W" run
bash research/prompts/m3-batch32-code-r1-opus-model/compare.sh "$W/summary.txt"
```

交回前在空目录 `~/.local/share/singlefs/work/m3-batch32-code-r1-opus/rerun4` 里从头跑过一次：build 44 秒、run 8 秒，`compare.sh` 原样输出「SAME 213 行」。编译经 `run-with-memory-cap.sh 6G`，每件探针经 `run-with-memory-cap.sh 4G`、单线程，同批至多 4 件并行；用工作目录自己的编译目录。

| 文件 | sha256 |
|---|---|
| compare.sh | fca1ea1dbbc528cfe4a3b8de97d1e08ccb6aaafc093c1655655c7851bd320ee2 |
| expected-summary.txt | 623056407138dbd2c762d27e278aae70535b771b50e12ddeb420400f3a74b7ff |
| patch_walk_arms.py | 34450d989d53948b6e5675932095d92fb5d9da691bd45ba1b1dcadde2c2d794b |
| patch_walk_arms_2.py | 29a05b67521d6dfa1c85823021b33974397c31b1c83ce4296277c0843cb12764 |
| probe_tail.rs | c268fbac47f8af1eb0768479ef35cc4de44880c656fc9a4cf48a1f6a95a04194 |
| probe_tail_p4p2.rs | af28401b8f1e343f2bd75c92898b442d6aa379d31dd1a7b8a0676f21993a9f2f |
| rerun.sh | c234a0a0589fe5a94d2a2c6b8f8a24472fbaa08b574bff42f13599e95ba0725b |
| run_batch.sh | 1fe7dced9e7c7051e82b06b04b4d27f4ea4aad992f05402f42039d497631f92f |
| summarize.py | 6af493846a376565e5e202d913f517c6d1b1439ab3126b7a854abba28b321826 |

模型怎么搭：冻结副本拷一份，`walk.rs` 副本上加臂开关（`patch_walk_arms.py`、`patch_walk_arms_2.py`），把 `checker_known_bad_images.rs` 原样拷成新目标 `opus_b32_probe.rs` 再接上探针（借它现成的造镜像函数），起点镜像每个只建一次、各形从它 clone 出来改，同一进程里逐条换臂判同一批镜像。七条臂：

| 臂 | 一个单元的身份按哪个落点认 |
|---|---|
| today | 冻结副本原样：「走过」按读回那一份；诞生代号表、合并、父单元、叶序、I-3.5 的折叠表按第 0 条位置条目 |
| b31 | F-KEY 之前：全按第 0 条 |
| birthrb | 实现员报告「停下」第 1 条提的候选：诞生代号按读回那一份记，合并照旧按第 0 条 |
| allrb | F-KEY 那几处：走过、诞生代号、合并、父单元、叶序都按读回那一份；I-3.5 的折叠表不动 |
| allrbmap | allrb 之外，读回那一份的落点在 I-3.5 的折叠表里记成它自己（只在它此刻记的是这条指针第 0 条落点时改） |
| both | 诞生代号按读回那一份记，第 0 条落点还没记过时也记一份，合并照旧按第 0 条 |
| gpuemu | 仿 GPU 今天的写法：诞生代号按第 0 条记，第 0 条落点不在走过表里时记不下 |

除 today 与 b31 外都是我提的，只在我的模型上量过、被攻过零轮。gpuemu 只仿诞生代号记不记得下这一处，不是 GPU；GPU 那一侧的结论都是读 wgsl 推的。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 F-KEY 多红 I-4.9 的射程 | **打中（量过，CPU）** | 不止谱系那一形：位置条目 0 撞到「被更早的候选根共享的节点」就多红，墓碑撞 extent 树根（意图镜像）同样多红；实现员报告说墓碑没有，是因为那条用例撞的是 inode 树根（不共享） |
| P1 诞生代号与合并按哪个落点认 | **给出候选与结局（量过，零轮）** | 七条臂里只有 allrbmap 在全部形上两种盘序同判、分叉形 I-4.9 / I-3.5 都红；实现员提的 birthrb 在「直接共享」分叉形上漏 I-4.9、两种分叉形上都漏 I-3.5 |
| P1 I-3.5 的折叠表也按第 0 条认 | **打中（量过，CPU）** | 修掉多红的 I-4.9 之后（birthrb / allrb / both），谱系撞 inode / extent 树根那四形改红 I-3.5，而 I-3.5 那一行按落点判是成立的；today 上这一格被多红 I-4.9 那个错数遮住了 |
| P1 CPU 与 GPU 分叉：第 0 条那一份对不上、落点没有别的单元 | **打中（CPU 量过，GPU 推的）** | 回退之后共享的叶容器诞生代号抬到 D 的 txg、盘 0 那份坏：CPU 红 I-4.9 与 I-3.5，GPU 两条都记不下诞生代号、都判不到（实现员推的那一形，量到了，并且多一条 I-3.5） |
| P1 合法状态多红 / 漏判 | 没打中（读码，推的） | 「走过」的键只在第 0 条那一份校验和对不上时变，那一刻同一遍走读已判 I-2.1 红；判定丢掉的两遍另起走读里的撞键要两盘不同槽，造不出合法形 |
| P4 F-零槽：GPU 交回全零与 CPU 读全零一一对应吗 | **打中（CPU 量过，GPU 推的）** | 没写过的槽 g 后一槽写过、校验和是 32 KiB 全零的 CRC：CPU 按 32 KiB 读到「16 KiB 零 + 下一槽的字节」、I-2.1 红；GPU 按 16 KiB 读回的全零切片带着 32 KiB 全零的检查字，F-α 那一处按它比对得上、I-2.1 成立。F-零槽之前两边都红，这一形是 F-零槽引入的 |
| P4 表里两条全零单元的检查字 | 没打中（读码） | 32 KiB 那条检查字是 [16 KiB 全零的 CRC, 32 KiB 全零的 CRC, 标志带位 4, 布局字]，16 KiB 那条第二个检查字为 0、标志不带位 4，都由 `version_checks` 按单元槽算 |
| P2 两份副本组合两两扫、两种盘序 | 没打中（量过，CPU） | 墓碑层级 1 与层级 0，四种副本 16 对：I-9.23 判定与红的集合换盘都相同 |
| K3 | 不归这条腿 | — |

## 一、P1 F-KEY：I-4.9 那一形，诞生代号与合并该按哪个落点认

### 1.1 造法

O3 族（同实现员的 `image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`，多一个开关）：种类 `kind` 那棵树根第一条条目子指针的位置条目 i 改指「这一遍先走过的码 2 节点 X」（种类 `walked` 那棵树的根）、校验和翻几位；另一条位置条目指的容器照旧。`variant=bad` 容器记录宽减 1（I-1.7 违例），`variant=good` 不改记录宽。三张起点镜像 × 先走过的节点取三种（含自己那棵树的根）× 两种 variant × 两种盘序，共 32 形，每形七条臂。

分叉族：回退到 A 之后的镜像（A txg 3、D txg 9，D 照抄 A 的 inode 树），inode 树根 X 第一条条目指的叶容器 L（槽 50242）。三个开关：`raise_birth` 把 L 头里的诞生代号抬到 9（重封、沿引用链补到根）；`corrupt_loc0` 把 L 在盘 0 那一份写成 0xa5（不补校验和，那个槽上没有别的单元）；`direct` 把 X 原样另写到一个没写过的槽、D 的树表改指它（D 与 A 各走自己的树根，L 由两版直接引用，A 在叶容器那一处「走过」命中；不 `direct` 时 A 在 X 上就命中、按 X 下面记的落点合并）。8 形 × 七条臂。

### 1.2 多红 I-4.9 的射程比实现员报告说的宽（打中，量过）

today 臂上位置条目 0 那一形与位置条目 1 那一形红的集合不同的有 6 对：谱系撞 inode 树根、谱系撞 extent 树根、墓碑撞 extent 树根，各 bad / good 两种。原样（`runs` 的探针输出，rerun4 那一次）：

```
PROBE p1o3 arm=today base=tomb walked=extent variant=good collide=loc0 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-4.9,I-5.1,I-7.4] i1_7=H i4_9=V<别的版本先走过的盘 0 槽 50254 的单元：诞生代号 5 大于引用它的候选根的 txg 4>
PROBE p1o3 arm=birthrb base=tomb walked=extent variant=good collide=loc0 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i1_7=H i4_9=H
PROBE p1o3 arm=today base=lineage walked=inode variant=good collide=loc0 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-4.9,I-5.1,I-7.4] i1_7=H i4_9=V<别的版本先走过的盘 0 槽 50244 的单元：诞生代号 9 大于引用它的候选根的 txg 3> i3_5=H
```

机制（读码）：容器从位置条目 1 读回、照判之后，诞生代号按第 0 条落点记进表里，盖掉 X 自己的那一条——「self.birth_txg_of_walked_units.insert(」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1021`），键是下一行的「(pointer.locations[0].device, pointer.locations[0].slot),」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1022`）；之后更早的候选根走到 X 时在合并里按这个数判——「if let Some(birth_txg) = self.birth_txg_of_walked_units.get(&unit).copied() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1285`）。条件是 X 被一条更早的候选根共享：意图镜像里 extent 树根被 txg 4 那条根共享，inode 树根在意图那一版改写过、不共享，所以墓碑撞 inode 树根（实现员那条用例，「/// F-KEY 墓碑那一形：三个单元的文件发完意图之后，墓碑容器记录宽 40 → 39，墓碑树根第一条条目子指针的一条位置条目指着 inode 树根（先走过）。」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13397`））不多红，换成 extent 树根就多红。GPU 同形：诞生代号按第 0 条落点找走过表那一行记——「let visited_index = w_visited_index_in_the_main_walk(p.device0, p.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1421`），X 在表里，照样盖掉；gpuemu 臂与 today 在这 6 对上逐行相同。

判据字面：I-4.9 那一行「回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。」（`.claude/kb/invariants.md:185`）。更早那条根引用的是 X，X 落点上的头记的诞生代号不晚于它；报出来的 5 / 9 是那片容器的（另一个落点上的头）。字面成立而报红，违例说明点名的单元也是错的。镜像本来就坏（I-2.1、I-5.1 照红），按 m3-batch22-code-r1 正文第五节的口径不算误红，算坏镜像上的错判；后果是两种盘序判得不同，实现员把 I-4.9 放进谱系那条用例的「两种盘序允许不同」（「&[UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT],」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:13450`）），墓碑撞 extent 那一形没有用例。

### 1.3 I-3.5 的折叠表同样按第 0 条认，today 上被 1.2 那个错数遮住（打中，量过）

把多红的 I-4.9 修掉的三条臂（birthrb、allrb、both）在谱系撞 inode / extent 树根那四形上改红 I-3.5：

```
PROBE p1o3 arm=both base=lineage walked=inode variant=good collide=loc0 red=[I-2.1,I-3.1,I-3.11,I-3.5,I-4.8,I-5.1,I-7.4] i1_7=H i4_9=H i3_5=V<相邻两版 (实例 {2}、txg 8) → (实例 2、txg 9)（NoHole）：后一版引用、前一版不引用的单元盘 0 槽 50244（用户可见树 None）诞生代号 3 不大于前一�
PROBE p1o3 arm=allrbmap base=lineage walked=inode variant=good collide=loc0 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4] i1_7=H i4_9=H i3_5=H
```

机制（读码）：走读跟着指针记落点时，两条位置条目的落点都折成「第 0 条落点那个单元」——「let unit = (pointer.locations[0].device, pointer.locations[0].slot);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1311`）、「self.unit_of_each_placement.entry(placement).or_insert(unit);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:1315`）。谱系容器在盘 1 那个落点（D 引用、C 不引用，不在用户可见树里）被折成单元 X，按 X 的诞生代号 3 判 (a) 不成立、(b) 不适用，红。today 上 X 的诞生代号已被容器的 9 盖掉，(a) 恰好成立，红被遮住。

判据字面：I-3.5 按落点判——「R 引用而 R⁻ 不引用的每个落点 b」（`.claude/kb/invariants.md:146`），「`birth(b)` 取 b 的单元头里的诞生代号」（`.claude/kb/invariants.md:146`）。盘 1 那个落点上是容器，头里 9 > 8，(a) 成立；盘 0 那个落点是 X，在 D 的 inode 树里，(b) 成立。字面成立。所以 birthrb / allrb / both 在这四形上是坏镜像上的错判；只有连折叠表一起改的 allrbmap 不红。b31 上这四形本来也红 I-3.5（第 31 批就在），F-KEY 之后被 1.2 的错数遮住，不是修好了。

### 1.4 第 0 条那一份对不上、落点没有别的单元：CPU 与 GPU 分叉（打中，CPU 量过，GPU 推的）

`raise_birth=true corrupt_loc0=true` 两形（A txg 3 引用的 L 头里诞生代号 9；L 盘 0 那份坏、那个槽上没有别的单元）：

```
PROBE p1split arm=today direct=false raise_birth=true corrupt_loc0=true i4_9=V<别的版本先走过的盘 0 槽 50242 的单元：诞生代号 9 大于引用它的候选根的 txg 3> i3_5=V<相邻两版 (实例 {2}、txg 8) → (实例 2、tx…>
PROBE p1split arm=today direct=true raise_birth=true corrupt_loc0=true i4_9=V<别的版本先走过的盘 0 槽 50242 的单元：诞生代号 9 大于引用它的候选根的 txg 3> i3_5=V<相邻两版 (实例 {2}、txg 8) → (实例 2、tx…>
PROBE p1split arm=gpuemu direct=false raise_birth=true corrupt_loc0=true i4_9=H i3_5=H
PROBE p1split arm=gpuemu direct=true raise_birth=true corrupt_loc0=true i4_9=H i3_5=H
```

（`PROBE` 行里 `leaf_slot=50242 a_txg=3 d_txg=9` 与 `red=[…]` 两段这里删了，全文在 `expected-summary.txt` 的 `SPLIT` 行。）同形不坏盘 0 那份（`corrupt_loc0=false`）时七条臂 I-4.9、I-3.5 都红，分叉只来自盘 0 那份坏。

GPU 那一侧（读码推的）：D 走到 L 时「走过」按读回那一份（盘 1 槽 50242）进走过表（「fn w_visit_the_copy_read_back(p: Ptr) -> bool {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1318`）），诞生代号却按盘 0 槽 50242 找那一行（`crash_judge_checker.wgsl:1421`，同 1.2），表里没有、记不下。之后：
- I-4.9：A 合并时按第 0 条落点找（直接共享那一形「w_merge_below(child.device0, child.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2752`）；经 X 合并那一形 X 下面两个落点都展开，盘 0 那个不在走过表、盘 1 那一行没有诞生代号），都判不到。CPU 的诞生代号表不看走过表、照记（`walk.rs:1021`），判红。
- I-3.5：GPU 的折叠表同样把两个落点折成第 0 条那个单元（「appearance_note_placement(p.device1, p.slot1, p.device0, p.slot0);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1254`）），取诞生代号又按那个单元在走过表里的行，L 的诞生代号不知道，「读不出头的单元不计入」，X 那一格 (b) 成立、不红。CPU 红。

判据字面：L 在盘 1 那个落点上头校验和过、诞生代号 9 > 3，A 引用它，I-4.9 不成立；I-3.5 那一行 (b) 要「那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg」（`.claude/kb/invariants.md:146`），L 读得出、9 大于，X 那一格不成立。CPU 两条都对，GPU（推的）两条都漏判。这就是实现员报告「停下」第 1 条第二段推的那一形，量到了，并且多一条 I-3.5。

### 1.5 改法候选在各形上的结局

格里「量过」的贴的是 `expected-summary.txt` 里的行（摘要见上两节）；「推的」没实现没跑。O3 族那一列数的是「位置条目 0 与 1 红的集合不同」的对数（32 形 16 对，每对一条臂上判一次）。

| 臂 | O3 族换盘不同的对数 | 不同在哪 | 分叉形经 X 合并：I-4.9 / I-3.5 | 分叉形直接共享：I-4.9 / I-3.5 | 来源 |
|---|---|---|---|---|---|
| b31 | 10 | 8 对 bad 形位置条目 0 漏 I-1.7（第 31 批 O3）；谱系撞 inode / extent 树根 4 对（bad、good 各两对）位置条目 0 红 I-3.5（1.3），其中两对 good 形只差这一条 | 红 / 红 | 红 / 红 | 量过 |
| today | 6 | 墓碑、谱系撞共享节点多红 I-4.9（1.2） | 红 / 红 | 红 / 红 | 量过 |
| GPU 今天（gpuemu 仿） | 6 | 同 today | **漏 / 漏** | **漏 / 漏** | 量过（仿）；GPU 本身推的 |
| birthrb（实现员提的） | 4 | 谱系 4 对错红 I-3.5 | 红 / **漏** | **漏 / 漏** | 量过 |
| allrb | 4 | 谱系 4 对错红 I-3.5 | 红 / **漏** | 红 / **漏** | 量过 |
| both | 4 | 谱系 4 对错红 I-3.5 | 红 / 红 | 红 / 红 | 量过 |
| allrbmap | 0 | — | 红 / 红 | 红 / 红 | 量过 |
| GPU 照 allrbmap 改 | — | — | — | — | 推的：诞生代号记进 `w_visit_the_copy_read_back` 插的那一行、合并与叶序缓存按读回那一份的落点找、`appearance_note_placement` 把读回那一份的落点记成它自己；没写没跑 |

对「一个单元的身份是哪个落点」的回答（零轮）：在 F-KEY 那三处，读回那一份的落点就是这个单元；走过、诞生代号、合并、父单元、叶序与 I-3.5 的折叠表要按同一个落点认，少改一张表就有一组形翻面。只改诞生代号（birthrb）不够：直接共享的单元在「走过」命中之后按第 0 条合并，诞生代号在另一个键上，I-4.9 判不到（`direct=true` 那一行）。`read_index_node` 那几处（码 2 树、树表、inode 树根）这一轮不判（正文第一节），allrbmap 没动它们，那几处与 F-KEY 三处之间的键仍不一致。

### 1.6 F-KEY 让合法状态多红或漏判吗（没打中，读码推的）

- 主走读：「走过」的键只在第 0 条那一份校验和对不上时与第 0 条不同；对不上的那一刻同一个函数已经判了 I-2.1——「judgements.judge("I-2.1", matches, || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/image.rs:1530`）。所以键变了的状态 I-2.1 已经红；合法状态上键不变，F-KEY 改不了它的判定。
- 判定丢掉的另起走读（隔离用的收位置项、F 之下各根）：那里 I-2.1 不报，撞键会改变收到的引用。撞键要一个单元的第 0 条落点上住着另一版走到的别的单元、第 1 条那份完好；core 两盘同槽写（「/// 两盘同槽（D2（RAID 条带策略） 已定项 10）：`mutate_unit_of` 读第一块盘那一份、两盘写同一个槽，靠的就是这一条。」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:3711`），这是测试里的说明，core 那一侧我没现查），复用盘 0 那一槽同时盖掉盘 1 那一槽，两份一起坏。造不出合法形（推的，没跑层 0）。

## 二、P4 F-零槽：GPU 交回全零之后与 CPU 逐一对

### 2.1 造法与 CPU 的数（量过）

三个单元的文件发完意图之后的镜像，墓碑树根第一条条目类型段 1 改成 7（不认得）或 0（层级 1，F-L0 那一支），位置条目 1 改指盘 1 上一个没写过的槽 g、校验和换成三种之一：16 KiB 全零的 CRC（`zero16`）、32 KiB 全零的 CRC（`zero32`）、「16 KiB 零 + 下一槽前 16 KiB」的 CRC（`zero16+next`）。g 取两种：下一槽 g+1 写过（g = 50179，g+1 = 50180）与 g+1 没写过（g = 50290）。两条类型段都走「走读不往下读的孩子每一份照判 I-2.1」那一支。原样（`expected-summary.txt` 的 `P4` 行）：

```
P4 p4const crc_zero16=0x94640b85 crc_zero32=0xbc43baad
P4 p4 type=7 next_written=true sum=zero16(0x94640b85) gap_slot=50179 red=[I-3.1,I-3.11,I-5.1,I-9.23] i2_1=H
P4 p4 type=7 next_written=true sum=zero32(0xbc43baad) gap_slot=50179 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=V
P4 p4 type=7 next_written=true sum=zero16+next(0x3c20a2fa) gap_slot=50179 red=[I-3.1,I-3.11,I-5.1,I-9.23] i2_1=H
P4 p4 type=7 next_written=false sum=zero32(0xbc43baad) gap_slot=50290 red=[I-3.1,I-3.11,I-9.23] i2_1=H
P4 p4 type=0 next_written=true sum=zero32(0xbc43baad) gap_slot=50179 red=[I-2.1,I-3.1,I-3.11,I-4.8,I-5.1,I-7.4,I-9.23] i2_1=V
```

`zero32`、g+1 写过的那两形 I-2.1 违例说明（探针日志原样）：「墓碑树内部条目的子指针（走读不往下读的孩子）在盘 1 槽 50179 的那一份按头里那一类的长度（没登记的 16 KiB、32 KiB 都试）与位置条目里的校验和对不上」。

### 2.2 GPU 会给什么（读码推的）

- `read_at` 起点不是落点起点时交全零单元：「version = zero_unit_version_over_a_slot_nothing_wrote(location, offset, length);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:481`），切片长度「unit_length = DATA_UNIT_BYTES >> (version & 1u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:482`）。
- 只按 16 KiB 读时，空段只要求装得下一槽——「|| location_word(before, 10u) < first_slot + ((length + NODE_BYTES - 1u) >> SLOT_SHIFT)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:506`）；选哪一条全零单元只看盘尾——「return select(0u, 1u, first_slot >= location_word(before, 11u));」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:509`）。g = 50179 时空段止于下一个落点 50180（`crash_judge_tables.rs` 第 1150 行「.map_or(device_end_slot, |next| next.offset / slot_bytes())」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:1150`）），装得下一槽，交第 0 条：32 KiB 全零，检查字里带 32 KiB 全零的 CRC 与「有 32 KiB」那一位（「flags |= 4;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_tables.rs:852`））。
- F-α 那一处拿这份按 16 KiB 读回的切片去比 32 KiB：「return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3125`），切片是「read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES), child.sum1);」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3138`），比的是「return s.c1 == checksum;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:608`）。c1 是 32 KiB 全零的 CRC 0xbc43baad，等于位置条目里的校验和，判对得上。
- CPU 那一边另读一次 32 KiB——「None => vec![node_bytes(), data_unit_bytes()],」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker/src/walk.rs:4567`）——读到的是「g 的 16 KiB 零 + g+1 上的字节」，CRC 不是 0xbc43baad，判对不上。

所以这两形 CPU 红 I-2.1、I-4.8、I-7.4，GPU（推的）这一份判对得上、三条都不红；别的红项两边同（同形 `zero16` 的 CPU 红集合就是 GPU 这一形该有的那一批：`[I-3.1,I-3.11,I-5.1,I-9.23]`）。F-零槽之前 GPU 在这里读不到、判对不上，与 CPU 同红：这一处分叉是 F-零槽引入的。实现员报告「`read_at` 改了之后三个内核里受影响的判定」表里走读内核那一行写「同第二行」、即与 CPU 稀疏读者读全零之后同判，这一形推翻它。反过来那一形（`zero16+next`：CPU 按 32 KiB 对得上、I-2.1 成立，GPU 用 32 KiB 全零的 CRC 比、对不上）是实现员列过的「改了之后还留着的分叉」，F-零槽之前就在，不算新打中。

判据字面：I-2.1 那一行「类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配」（`.claude/kb/invariants.md:130`）。盘 1 槽 50179 起 32 KiB 的真实字节不是全零，两种长度都对不上，不匹配；CPU 对，GPU 漏判。

只有坏流造得出：合法流里没有指针指向从没写过的槽。三个内核里另外几处按什么长度读就按什么长度比（`slice_crc_matches` 其余调用点读的长度与比的长度相同，读码），只有 F-α 这一处是「按 16 KiB 读、按 32 KiB 比」。

### 2.3 改法（推的，零轮，GPU 没跑）

| 改法 | 修哪一格 | `zero32` 且 g+1 写过 | `zero16+next` 且 g+1 写过 | 来源 |
|---|---|---|---|---|
| Z1：`zero_unit_version_over_a_slot_nothing_wrote` 在空段只装得下一槽（第 10 字 < 起点槽 + 2）时交 16 KiB 那一条 | 新引入的分叉 | 与 CPU 同红 | 仍分叉（GPU 红、CPU 不红） | 推的 |
| Z3：F-α 那一处比 32 KiB 时另按 32 KiB 读一次 | 同 Z1 | 与 CPU 同红 | 仍分叉 | 推的 |
| Z2：比 32 KiB 时用「16 KiB 全零的 CRC」与下一个落点那一版前 16 KiB 的 CRC 拼接（CRC32C 的 combine） | 两形都修 | 同 CPU | 同 CPU | 推的，工作量最大 |

## 三、P2 F-ID-b：两份副本组合两两扫（没打中，量过）

墓碑树根层级 1 与层级 0，第一条条目类型段 1 → 0、身份三段清零，两条位置条目各放一种副本：`node`（这棵树根的 16 KiB 原样、校验和按它，自证成码 2）、`c3`（容器原样，按 32 KiB 自证成码 3）、`c16`（容器原样，校验和换成前 16 KiB 的 CRC）、`bad`（校验和翻几位）。2 层级 × 16 个有序对，32 形。拿脚本逐对比 `(c0, c1)` 与 `(c1, c0)` 的 I-9.23 判定与红的集合：「pairs 32 asym 0」。有一份 `node` 而另一份不是 `c3` 时 I-9.23 成立；有 `c3` 时红（F-W 那一支先判红）；两份都没自证过（`bad`/`c16` 组合）时照改之前的读法，`c16` 读成码 3、红，两份都 `bad` 读不出、不判——这一形用户已定第 33 批改，这里不攻。GPU 那一侧 `w_class_of_the_first_copy_verifying_at_the_length_of_its_class` 与 CPU 同一个判据（读码），没找到分叉。

## 四、打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到判别它的东西 | 满足的是判据字面哪一分句 | 跑前给的改法在这几格上还中不中 |
|---|---|---|---|---|
| 1.2 多红 I-4.9（墓碑撞 extent 等 6 对） | 分辨：today、gpuemu 中，birthrb / allrb / both / allrbmap 不中，b31 不中（b31 在这几形上漏 I-1.7） | 看得到：读回的是哪一份落点在手里（`read_a_unit_to_walk_and_the_placement_it_was_read_from` 交回），只是记诞生代号时没用它 | I-4.9「按落点认，读落点上单元头里的诞生代号」：X 落点上的头不晚于引用它的根，报红的数是另一个落点上的头；镜像是坏的，按 m3-batch22 口径不算误红，算坏镜像上的错判 | 正文跑前条款没给改法；实现员报告的候选（birthrb）在这 6 对上不中（量过），但它在 1.3、1.4 那几格上中（见下两行） |
| 1.3 I-3.5 折叠按第 0 条（谱系 4 对） | 分辨：birthrb / allrb / both / b31 中，today 被遮住，allrbmap 不中 | 看得到：同上 | I-3.5「R 引用而 R⁻ 不引用的每个落点 b」「birth(b) 取 b 的单元头里的诞生代号」：盘 1 落点上是容器（9 > 8），(a) 成立；坏镜像上的错判 | birthrb 中（量过）；只改诞生代号的改法碰不到这一格，要连折叠表一起改（allrbmap 量过不中） |
| 1.4 CPU / GPU 分叉（L 盘 0 那份坏、诞生代号晚于 A） | 分辨：CPU today 红，GPU（gpuemu 仿、读码推）不红 | GPU 看得到：`w_read_back_from_the_second_location` 记着读回的是第二条（「w_read_back_from_the_second_location = true;」（`/home/fy5090/.local/share/singlefs/work/m3-batch32-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1310`）），记诞生代号那一步没用它 | I-4.9 主句「它引用的单元…的诞生代号 ≤ R 的 `checkpoint_txg`」不成立而 GPU 不判（漏判）；I-3.5 (b)「每个读得出的单元头的诞生代号都不大于那个出生 txg」不成立而 GPU 不判（漏判） | birthrb 在直接共享那一形上 CPU 也漏 I-4.9、两形都漏 I-3.5（量过）；allrbmap 两形都红（量过）；GPU 照改是推的 |
| 2 F-零槽 32 KiB 全零检查字 | 分辨：F-零槽之后 GPU 与 CPU 分叉，F-零槽之前不分叉（读码） | 看得到：落点表第 10 字就是空段的终点，`read_at` 知道只空一槽 | I-2.1「16 KiB 与 32 KiB 两种长度都对不上才算不匹配」：两种长度都对不上，GPU 却判对得上（漏判） | F-零槽本身就是这一格的来源；Z1 / Z3 修它、Z2 连带修旧分叉（都推的） |

## 没打中的形状

- O3 族位置条目 1 撞先走过的节点（16 形 × 七条臂）：每条臂上与该臂位置条目 0 那一形之外，红的集合都相同；撞的是自己那棵树的根（`walked` = 自己）或不被更早候选根共享的节点（墓碑撞 inode、inode 叶撞 extent / inode）时，today 臂上两种盘序同判，没有多红。
- 分叉族里盘 0 那份不坏（`corrupt_loc0=false`）、或诞生代号不抬（`raise_birth=false`）的 6 形 × 七条臂：各臂 I-4.9、I-3.5 判定相同（`expected-summary.txt` 的 `SPLIT` 行）。
- P4：g+1 没写过时三种校验和 × 两种类型段，CPU 判定与 GPU（推的）相同；`zero16` 在 g+1 写过时同。
- P2：32 形，换盘全相同（第三节）。
- 合法状态：只读码（1.6），没枚举崩溃状态。
- 取样范围：三张起点镜像（新池新建文件、三个单元的文件发完意图、回退到 A 之后），两盘；先走过的节点只取树表里三棵树的根；分叉族只取回退镜像里 inode 树第一片叶；P4 只取墓碑树根第一条条目、盘 1 那条位置条目。O3 族的「直接共享」（两版各自的父节点都指同一片叶）只在分叉族里造过，O3 族没有。

## 这条腿自己的限度

- GPU 一条没跑。第 1.4、二节 GPU 那一侧都是读 wgsl 推的；gpuemu 臂只在 CPU 走读里仿「诞生代号按第 0 条找走过表那一行、找不到就记不下」这一处，不仿 GPU 的别的差别。要坐实得在卡上跑一条对拍流（提交时归主 agent）。
- 我提的臂（birthrb 以外的 allrb、allrbmap、both）与 Z1–Z3 都被攻过零轮，只在我的模型上量过（Z 那几条连量都没量）。allrbmap 的折叠表改法只在「此刻记的是这条指针第 0 条落点」时改，次序换了（读回那一份的落点先被别的指针记过）会怎样没造形。
- 副本上的数不算入库装置上的数；探针目标是 `checker_known_bad_images.rs` 原样拷贝再接上我的探针，借它的造镜像函数。这些函数在 `std::env::temp_dir()` 下建文件盘（不是只在内存里）：前几次跑在 `/tmp` 留下 12 个 `singlefs-step67-opus-b32-*` 稀疏镜像，已逐个删掉；之后 `run_batch.sh` 把每件的 `TMPDIR` 设到工作目录里、跑完删。起点镜像每个只建一次，各形从内存里那一份 clone。
- 没枚举崩溃状态，崩溃状态数 0（不调 `enumerate_layer0` 一族）。合法状态那一格（1.6）只有读码这一条路，没有第二条独立路径。
- 估时与取样：先跑一小段（P4 12 形，debug 编译，1 分 54 秒），改 release、按件并行后全量 58 件 123 秒；改成同一进程里换臂、起点镜像只建一次之后 6 件，负载低时 8 秒、负载 40 上下时约 210 秒（同机别的会话在编译）。全量估时没超过 40 分钟，没缩取样；每批 4 件、不设批限时。
- 跑之前 `ps` 看到别的会话的 `cargo build` / `cargo test` 在跑（`e171_three_oracle_co…`、`checker_known_bad_images` 等），没有性能测量；我的 cargo 加 `nice -n 19`，没等锁。
- 干到建探针那一步收到主 agent 消息「线程上限从 5 改成 4」：我第一条 cargo 起就用 `capped.sh 4`，探针每件 `capped.sh 1`、同批至多 4 件，没改。
- 内存包装没有退出码 250–254 的结局。中途两次 `rerun.sh` 被我自己的 `timeout 236` 截断（退出码 124，负载高时一批跑了 170–200 秒），之后把 `rerun.sh` 拆成 `build` / `run` 两步、`run` 可续，在新的空目录里从头跑通（见「复跑」）。

## 没做什么

- 不判 K1、K3（正推腿的格），不替主 agent 采纳。
- `read_index_node` 那几把键、F-ID-b 两份都没自证过那一形、三·4 删除流 oracle：正文列为这一轮不判，没攻。
- P2 只扫墓碑树；谱系、inode 两棵树与两份都读不出（指到池外的盘）的组合没扫。P4 只造了墓碑树根条目那一处调用点；扫描内核、恢复内核里按长度读的几处只读码。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch32-code-r1-opus/` 里只剩小文件：`tools/`（模型源码的工作副本）、`runs/`（各次探针日志，`runs/rerun4-logs/` 是「复跑」那一次的原始日志、`runs/rerun4-summary.txt` 是它的汇总）、编译与复跑的输出日志。编译目录、仓副本、`rerun4` 的树与编译目录已删。
