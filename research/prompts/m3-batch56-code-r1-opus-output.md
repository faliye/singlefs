# m3-batch56-code-r1 攻方腿（Opus）报告：删除线第 56 批代码轮第一轮（2026-10-10）

立场：云端攻方，攻 K1、K2、K4、K5（K3、K6 顺带），在冻结副本的拷贝上造反例。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch56-code-r1/repo/`（拷到草稿目录后 `sha256sum -c research/prompts/m3-batch56-code-r1-snapshot/crates-sha256.txt` 14 行全对），下面的代码行号按冻结副本。前一轮判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`。没读禁读清单里的三样。

## 复跑

模型目录 `research/prompts/m3-batch56-code-r1-opus-model/`。从空目录起（模型目录拷出仓用时设 `SINGLEFS_SCRIPTS=<仓根>/scripts`）：
```
SINGLEFS_SCRIPTS=$PWD/scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=4G \
  bash research/prompts/m3-batch56-code-r1-opus-model/rerun.sh <空工作目录> all
```
`all` 依次跑 base-build、k2-knob、k2-crash-counts、k2-crash-timing、k1-base、k2-random-b1/b2/b3、k4-shape、k4-family、`mutants.py --list` 列的 12 份（11 种改坏与 1 条改法臂 `fix-knob-full`，阶段名都是 `mut-<名>`）、compare；只拷冻结副本的 `Cargo.toml`、`Cargo.lock`、`crates/`，原型 crate `probe-crate/` 拷成副本工作区里的成员 `crates/opus-probe-b56`（测试目标名不带 layer0），base 与变异各用自己的 `CARGO_TARGET_DIR`，变异挨个共用一个副本目录与编译目录；同一时刻一条 cargo，每条经 `capped.sh` 与 `run-with-memory-cap.sh`。前台有超时的环境按阶段名分几次起、同一个工作目录接着跑（变异阶段被打断后重起不重拷、接着编）。挂钟（本机负载 15–30 下）：编译约 4 分钟，原型与 harness 阶段约 10 分钟，12 份变异与改法臂约 30 分钟。

每个文件的 sha256（在 `research/prompts/` 下跑 `find m3-batch56-code-r1-opus-model -type f | sort | xargs sha256sum`）：
```
caa8294fa130624eb09fbdb7a1c63d2cc8a0f2cb7a28cab71c6071010b3f27be  m3-batch56-code-r1-opus-model/compare.py
1e60a3a85dab5a44a8a02ccf9280dc98c41243d74d1833f7d4d78fb144384ef0  m3-batch56-code-r1-opus-model/expected/k1-base.txt
369257c2a61e251bef2a074668b059c5f4e0e08076e417dab87c01243e616f1b  m3-batch56-code-r1-opus-model/expected/k2-crash-counts.txt
2ef5795f7545ac1c66bab551e28533d1fa3228f6595491ec491288eb045d7747  m3-batch56-code-r1-opus-model/expected/k2-crash-timing.txt
d7fbda0728ccf28ab915ea7b50498067c85c39f6477e8619bd03038f982bbd18  m3-batch56-code-r1-opus-model/expected/k2-knob.txt
f8e342f618a5a6d05a7c55cfbf0fa3f3aeb7897dc15976fbf3e7c5bbe4bd8745  m3-batch56-code-r1-opus-model/expected/k2-random-b1.txt
2c86faaecebd8760dbbc3c703aed4016c085f3227b5e3d91c585700787619880  m3-batch56-code-r1-opus-model/expected/k2-random-b2.txt
2f8daef32c5be8ffcdaeb22ae94389a917602e6f39622d6ff861fb9c1cb33ca9  m3-batch56-code-r1-opus-model/expected/k2-random-b3.txt
8c7909f93a6ad924806ede3bafa4899207cf755ae27ed760def0353dbeca0212  m3-batch56-code-r1-opus-model/expected/k4-family.txt
0daaa6bf167af45754fd0ffb23db322ced01c3c52acf2fd426af682b709480d9  m3-batch56-code-r1-opus-model/expected/k4-shape.txt
bf6bcb2f311ef92d5c5fa0bdc980220f61b6a0196e8cdcaa620e16f54ec8104a  m3-batch56-code-r1-opus-model/expected/mut-c-knob-needs-two.txt
a36cf4f2a05f288ec71b7f6b283026fc0f0d17a2e488627c4aa1ee54394249f1  m3-batch56-code-r1-opus-model/expected/mut-c-lr-adjacent-nondecreasing.txt
aa139da87a0a33cb1fdf93156594d1e226ed2e619c9a8a2bbe94130688f28d98  m3-batch56-code-r1-opus-model/expected/mut-c-product-skips-three.txt
ee015ec834da0f922049dcb1c4f62caa41b9dc3a5271ae6944ce39ac64dc94cb  m3-batch56-code-r1-opus-model/expected/mut-fix-knob-full.txt
a633263121eb01190aaee3174f3efe4314692657f968011772a0e0fedc94ca2f  m3-batch56-code-r1-opus-model/expected/mut-g1-cold-walk-revert.txt
a1fe6e10a81f04f8b63baa7fe8537b8dfb5be0e41cc0dc5dbf19dc7e44a78ec7  m3-batch56-code-r1-opus-model/expected/mut-g1-rebuild-revert.txt
a8c220bdd313050c915c97e922ae269603fc7e60b6bcb9814aaa81455d56a620  m3-batch56-code-r1-opus-model/expected/mut-k4-absorb-from-borrower.txt
e8eb6d1004f1e2072d1862cba9f75c87e973f6471ace0d83c36f9d42722d5451  m3-batch56-code-r1-opus-model/expected/mut-k4-borrow-even-after-absorbing.txt
eaad9900cffaa35fe77dee421395bedbdd69b07be5ff29ebb8d5d0e9d7f77594  m3-batch56-code-r1-opus-model/expected/mut-k4-floor-inclusive.txt
aaa8e9ed566ec7f73aa393e78da575f7b205c7869a24e20edb556cb429d5496e  m3-batch56-code-r1-opus-model/expected/mut-k4-left-first.txt
51f721dc3c339af4972900a71e584dea1ca0e95d01139dbb0ff1c7adc9e5c6db  m3-batch56-code-r1-opus-model/expected/mut-k5-treeid-not-abandoned.txt
7edf1b7a4a617a114b6d2690a646fa8dea11f39e8232d4c504529382f30562e0  m3-batch56-code-r1-opus-model/expected/mut-w-close-one-early.txt
5519c101478f0255c825f370c78bf984ca87f0d65e55bcd6028924f8ee0b983d  m3-batch56-code-r1-opus-model/inject.py
6bec2b9e52ee3d725191f79f9f7c9f9ecbc8a8478cde3a29a6929eb65a48fee6  m3-batch56-code-r1-opus-model/mutants.py
d4c74e597b677d395c2223a51ee612de989e9eeb22b2595a6873dc3e791f3d9f  m3-batch56-code-r1-opus-model/probe-crate/Cargo.toml
9cebef9771e29cb6a1c49e0671fe2a4f930ff96b1cd3b5745bf69f5d7e23fba3  m3-batch56-code-r1-opus-model/probe-crate/tests/common_b56/mod.rs
36128192c061cdbb16edd692b2b2aa8e33688db802d108015c4e3e5e92371236  m3-batch56-code-r1-opus-model/probe-crate/tests/k2_crash_abandoned.rs
1ff119d80225237c775d8b6e5f333ed81f0eb10ad472022750d664705d0fb4ef  m3-batch56-code-r1-opus-model/probe-crate/tests/k2_knob_bad_images.rs
8ae6a169d2aec7022945c92d646b579f593d7551015d50bc5e6f18933f48c7f2  m3-batch56-code-r1-opus-model/probe-crate/tests/k2_random_histories.rs
e7493cf0f356295dda11ec215edd5acb70650f279d437d823f54281e7b574096  m3-batch56-code-r1-opus-model/probe-crate/tests/k4_leaf_shape_stress.rs
87e50674e9b15a21f63af2caf9d9c95addd84f9202e5a7b7a1bf0fcc45728efe  m3-batch56-code-r1-opus-model/probes/inode_tree_k4_family.rs.snippet
432ac8d66e7ec56f753de806df07ac5547c139b94e6669e7e383f7d5893c282f  m3-batch56-code-r1-opus-model/rerun.sh
41004ff3568faf86c1f5ed4eb33f859c9ac764a640ef8eff4762a9320aeef1a1  m3-batch56-code-r1-opus-model/sort_k2r.py
b969d4e792755a744cf3cdbdf73a15338582d9fdcdec4d0b1ad1b2ef73688773  m3-batch56-code-r1-opus-model/summarize_random.py
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节；「量过」指本腿副本上的原样输出，产物是 `expected/` 下的文件） |
|---|---|---|
| K1-1 错误成员变了而不该变 | 没打中 | 推的：G1 只换几道纯判定的先后，单一坏法的成员只在它同时触犯两道判时才随次序变；冷走读挪动的那几道（分配记录代与跨度、记账逐条判）现在与重建同序。量过：`k1-base.txt` 35 条全过；第 56 批两行 G1 变异各红自己那一条（`mut-g1-cold-walk-revert.txt`、`mut-g1-rebuild-revert.txt`） |
| K1-1 合法镜像被拒 | 没打中 | 推的：几道判对合法镜像都成立，换序不改结局。量过：30 个种子里 331 次干净的可写重挂（走重建那条路）、794 次挂着回退，一次都没拒（`k2-random-b*.txt`）；崩溃估时样本 13 态挂载 0 拒（`k2-crash-timing.txt`） |
| K1-2 三条路的次序 | 没打中（G1 射程内） | 读码：重建与冷走读在 G1 那几道上同序；冷走读不读三棵容器树的容器、重建整棵读，是旧差别，不在 G1 射程；坏盘战役钉的三个成员不落在挪动的几道上（读码，checker 档没跑） |
| K2-1 合法镜像误报 | 没打中（崩溃那一形只做了估时样本） | 量过：旋钮下 30 个种子 4000 次判（含换实例重挂、回退、根长高、三棵树旋钮各异）0 违例。崩溃恢复抛弃段：一次发布的第一段就有 20–30 个写，层 0 全量最小的窗口 1 048 588 态，估时约 24 小时（10 线程），按定义不跑全量；估时样本 13 态（不是全量）0 违例、0 挂载拒绝、0 panic；全量命令写在「要主 agent 跑的命令」 |
| K2-2 坏镜像漏报 | **打中（不分辨批次）** | 旋钮下「不是最右的那片没装满」：旋钮 1 写出每片 1 条的三棵树，交旋钮 2、3 的新入口，I-9.22、I-9.37 都 Holds（`k2-knob.txt`）；写侧早关一片的变异下 10 个种子 600 次带旋钮判 0 违例（`mut-w-close-one-early.txt`） |
| K2-3 CPU 与 GPU | 没打中（读码） | 三判在 CPU 与 GPU 逐句对得上；旋钮常量全 0 的那一份就是产品入口（放量与层 0 经 `on_card_with_packed_tables` 编 `JudgeKernelSources::of_the_judge()`）；K2-2 那一格 GPU 同样漏 |
| K3 旋钮进 GPU | 没打中（读码，顺带） | 判法摘要只罩产品那一份常量，带旋钮的那份只有对拍用例编 |
| K4-1 参照拦不拦改坏 | 没打中 | 量过四种 M3 改坏：下限含 117、借完从借方起并、先查左邻，参照用例都红；「并过也照借」参照用例绿、core 等价留档红——按可达形推等价（见 K4 节的 P） |
| K4-2 等价族漏可达形 | 没打中；另有一条观察 | 量过：放宽族 160 042 形、2 095 099 次删，两种改写与实现 0 处不同、删后前提 0 次破（`k4-family.txt`）。观察：不是最右的叶恒 ≥ 117 条（推的，量过 8 种子 × 1200 次删从未破，`k4-shape.txt`），所以「删空的那一片两边都有邻居」不可达，等价留档里那一形与「删空摘片之后从左邻起并」那一支只在不可达形上被走到 |
| K5 回退取树 ID 水位 | 没打中 | 量过：第 56 批「只取没被抛弃的根」那一行变异红（`mut-k5-treeid-not-abandoned.txt`）；读码：抽出前后同一条表达式，不读的参数一旦被读，那条构造态单测就红 |
| K6 判别力 | **打中（第 56 批新判法自己的）** | 产品那一支（没装旋钮：I-9.21、I-9.36）不判 G2 三句的变异，`checker_known_bad_images` 247 条、`container_trees_grow_a_level_when_the_root_is_full` 6 条全绿（`mut-c-product-skips-three.txt`）；另两种改坏（装了旋钮只一片不判、相邻改成不减）有用例拦 |

## K1　恢复的新次序（甲 G1）

### 两个读者今天的次序（读码，冻结副本）
- 重建（可写挂载也走它）：先整棵读墓碑、谱系、退役三棵树连同容器（「let tombstone_tree_read = read_container_tree(」（`crates/singlefs-core/src/recovery.rs:2806`）），再读分配记录树、判每盘一条（「allocation_records_are_one_placement_per_device_on_every_device(」（`crates/singlefs-core/src/recovery.rs:2888`））、判分配代与跨度（「allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;」（`crates/singlefs-core/src/recovery.rs:2894`）），之后条目宽（「every_accounting_entry_is_as_wide_as_its_field_table(」（`crates/singlefs-core/src/recovery.rs:2918`））、逐条判代与 seq（「accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(」（`crates/singlefs-core/src/recovery.rs:2924`））、数条目（「accounting_entry_count_is_three_plus_six_per_device(」（`crates/singlefs-core/src/recovery.rs:2928`））、水位那一行在、映射条目数（「mapping_entry_count_is_one_per_mapped_unit(」（`crates/singlefs-core/src/recovery.rs:2944`））。
- 冷走读：每盘一条（`crates/singlefs-core/src/recovery.rs:5387`）、分配代与跨度（「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」（`crates/singlefs-core/src/recovery.rs:5395`））、条目宽（`crates/singlefs-core/src/recovery.rs:5396`）、逐条判（「accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;」（`crates/singlefs-core/src/recovery.rs:5400`））、数条目（`crates/singlefs-core/src/recovery.rs:5402`）、映射条目数（`crates/singlefs-core/src/recovery.rs:5407`）。`grep` 这几个判定的调用点，`crates/singlefs-core/src/` 下只有这两处读者（另一处 `recovery.rs:6124` 是单测）。

### K1-1：成员变了而不该变——没打中
G1 挪动的几道判都是对同一份已经读进来的字节的纯判定，各自要么成立要么交回一个 `RecoveryFailure`。一张只坏一处的镜像，交回哪个成员只在它同时触犯两道被换了先后的判时才会因换序而变。逐个列出能同时触犯两道的单处坏法（推的）：

| 单处坏法 | 触犯哪几道 | 第 56 批前 → 后（冷走读 / 重建） | 判 |
|---|---|---|---|
| 台阶行的代写到根之后 | 逐条判、数条目 | 条目数 / 条目数 → 代晚于根 / 代晚于根 | G1 要的改变，第 56 批两条用例钉住（变异两行各红自己那条，量过） |
| 根记录的 txg 改小（整槽自证重算） | 分配代、逐条判、数条目 | 条目数 / 分配代 → 分配代 / 分配代 | 冷走读对齐到重建，不算该不变而变；这一坏法多半先在根环与 journal 那几道红（没造） |
| 这一版自己那行水位的代改成更早的值（上一轮 ⑥） | 数条目（重建另有「水位那一行在」排在数条目之后） | 条目数 / 条目数 → 不变 | — |
| 台阶行 seq 改 0 | 只逐条判（代仍早于根，仍认作台阶行） | 不变 | — |
| 一块盘上一条分配记录的代改到根之后 | 每盘一条、分配代 | 每盘一条 → 不变 | 正文第二节点名的已知线索，不算 |

映射条目数与分配代、记账几道之间没有共同的输入（不同的树），单处坏法碰不到两边（推的）。量过的部分：`expected/k1-base.txt` 里冷走读 / 可写挂载那份 harness 用例 35 条全过；第 56 批两行 G1 变异，`expected/mut-g1-cold-walk-revert.txt` 原样：
```
command args=-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking rc=101
failures:
failures:
    an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_recovery
test result: FAILED. 34 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; 
```
`expected/mut-g1-rebuild-revert.txt` 原样：
```
command args=-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking rc=101
failures:
failures:
    an_accounting_step_row_whose_generation_is_after_the_root_is_refused_for_its_generation_by_the_writable_mount
test result: FAILED. 34 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; 
```

### K1-1：合法镜像被拒——没打中
推的：被挪的几道判对合法镜像都成立，换序前后跑到的判是同一组（每道都是 `?` 串着、没有「成立但另走一支」的判），所以合法镜像的结局不随次序变。量过（换实例重挂、回退那两类合法镜像）：K2-1 的随机历史里约 8% 的步是「干净地可写重挂」（走重建），30 个种子合计 331 次重挂、794 次挂着回退，`remount-refused`、`rollback-refused` 计数一次都没出现（`expected/k2-random-b*.txt` 的 `ops=` 字段）；崩溃估时样本 13 态的可写挂载 0 拒（`expected/k2-crash-timing.txt`）。只剩台阶行、空发布、多盘那几形没有单独造（随机历史的池都是两盘、每次建 inode 都留一级台阶行；空发布只出现在挂载的暖机里）。

### K1-2：三条路的次序——没打中（G1 射程内）
可写挂载走重建，所以三条路只有两种次序；在 G1 动到的那几道上两种次序一致（上一小节两张清单）。旧差别不在 G1 射程：冷走读读三棵容器树只读码 2 节点、不读容器（「// 墓碑树与谱系树：从根往下读码 2 节点（根满长高之后不止一层）、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）。」（`crates/singlefs-core/src/recovery.rs:5306`）），重建整棵连容器读——一张只坏了一片容器的镜像，只读恢复收下、可写挂载拒（推的，没造；不是这一批引入的）。

checker 档坏盘战役：钉到这几道判的三处是「分配记录不是每个落点每盘各一条」（`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs:686`，每盘一条仍排在分配代之前）、映射根缩成一条时的判全那一道（判全在读树时，早于这几道）、记账根窄成 key 宽（「第 52 批起重建数条目之前先核条目宽，可写挂载红在「记账条目宽不到字段表」上，不再是「记账条目数不是 3 + 6 × 盘数」；」（`crates/singlefs-checker-tier/src/bad_disk_input.rs:182`），条目宽仍排第一）。三处钉的成员按读码都不变；checker 档没跑。

## K2　容器号三判（乙 G2）

写侧只有一条开新片的路：「.is_some_and(|open| open.records.len() < records_per_container);」（`crates/singlefs-core/src/container_tree.rs:537`）不成立时才开一片，新号取最右那片的号加一。下面的攻法都是放开用户动作、换旋钮、换实例与崩溃点去找另一条路。

### K2-1 合法镜像误报：没打中

**随机历史**（`probe-crate/tests/k2_random_histories.rs`，内存稀疏盘，mkfs 与第一个文件只做一次、之后每个种子从它 `clone`）：每个种子随机取一组旋钮（墓碑 1–3、谱系 1–2、退役 1–2、码 2 节点 产品 / 2 / 3），写入口、分配器开关与池级 checker 新入口给同一组值；先建 234–241 个空 inode，再走 100 或 150 步：22% 建 1–3 个，20% 从记录最少的叶里删一个，20% 删最大的活号，10% 随便删一个，20% 挂着回退到最近 12 版之一（版本含之前各实例的），8% 干净地可写重挂（换实例）。用户动作全放开，只固定第一个文件那一前缀。每一步之后带同一组旋钮交 `check_pool_image_given_the_container_capacity_knobs`。三批（每批 10 个种子、10 个线程，先跑 2 个种子 × 20 步估时 0.13 秒一步，再定步数），挂钟 40 秒、99 秒、96 秒。`python3 summarize_random.py expected` 的原样输出：
```
合计：30 个种子，判 4000 次，有违例的 0 次，重挂 331 次，回退 794 次
| 种子 | 旋钮 墓碑/谱系/退役/节点 | 重挂 | 回退 | 判次 | 有违例 | I-9.22 成立 | I-9.37 成立 | 末版片数 墓碑/谱系/退役 |
|---|---|---|---|---|---|---|---|---|
| 1 | 3/2/1/3 | 9 | 22 | 100 | 0 | 100 | 98 | 18/11/4 |
| 2 | 2/1/2/0 | 8 | 23 | 100 | 0 | 99 | 94 | 24/23/0 |
| 3 | 1/2/2/3 | 8 | 21 | 100 | 0 | 100 | 95 | 45/11/1 |
| 4 | 2/1/2/0 | 11 | 18 | 100 | 0 | 100 | 98 | 22/18/1 |
| 5 | 3/1/2/3 | 9 | 17 | 100 | 0 | 100 | 94 | 16/17/0 |
| 6 | 3/2/1/0 | 10 | 18 | 100 | 0 | 100 | 88 | 17/9/0 |
| 7 | 1/1/1/0 | 7 | 21 | 100 | 0 | 98 | 99 | 48/21/2 |
| 8 | 2/2/2/2 | 5 | 17 | 100 | 0 | 100 | 98 | 25/9/0 |
| 9 | 2/1/1/0 | 9 | 23 | 100 | 0 | 100 | 98 | 23/23/4 |
| 10 | 2/1/2/2 | 9 | 19 | 100 | 0 | 100 | 96 | 24/19/1 |
| 11 | 1/2/2/3 | 6 | 34 | 150 | 0 | 149 | 148 | 70/17/3 |
| 12 | 1/2/1/0 | 15 | 30 | 150 | 0 | 147 | 148 | 62/15/0 |
| 13 | 2/2/1/2 | 13 | 27 | 150 | 0 | 150 | 144 | 38/14/7 |
| 14 | 3/1/1/2 | 10 | 23 | 150 | 0 | 148 | 145 | 24/23/0 |
| 15 | 3/1/2/3 | 11 | 25 | 150 | 0 | 149 | 148 | 23/25/1 |
| 16 | 3/2/1/2 | 9 | 25 | 150 | 0 | 150 | 146 | 27/13/3 |
| 17 | 1/2/1/0 | 9 | 34 | 150 | 0 | 148 | 149 | 75/17/4 |
| 18 | 2/1/1/3 | 12 | 37 | 150 | 0 | 150 | 147 | 38/37/4 |
| 19 | 1/1/2/2 | 14 | 27 | 150 | 0 | 149 | 148 | 78/27/0 |
| 20 | 1/2/2/2 | 8 | 27 | 150 | 0 | 148 | 149 | 76/14/0 |
| 21 | 2/2/2/2 | 10 | 40 | 150 | 0 | 148 | 149 | 35/20/4 |
| 22 | 3/1/2/3 | 17 | 27 | 150 | 0 | 150 | 140 | 26/27/1 |
| 23 | 2/2/1/0 | 16 | 23 | 150 | 0 | 150 | 148 | 41/12/3 |
| 24 | 1/2/1/3 | 12 | 29 | 150 | 0 | 150 | 149 | 79/15/3 |
| 25 | 1/1/2/2 | 9 | 32 | 150 | 0 | 150 | 138 | 75/32/4 |
| 26 | 2/2/1/0 | 19 | 27 | 150 | 0 | 146 | 147 | 35/14/1 |
| 27 | 2/1/2/0 | 10 | 34 | 150 | 0 | 150 | 148 | 36/34/0 |
| 28 | 2/1/2/3 | 13 | 34 | 150 | 0 | 150 | 149 | 38/34/4 |
| 29 | 3/1/2/3 | 17 | 30 | 150 | 0 | 149 | 140 | 25/30/4 |
| 30 | 3/1/1/3 | 16 | 30 | 150 | 0 | 150 | 149 | 28/30/1 |
```
覆盖到：第 56 批三判（第一片是 1、相邻加一、每片至少一条）在三棵树各几十片的版本上判过（I-9.22 / I-9.37 成立的判次见表）、根长高（码 2 节点 2 / 3 的种子）、回退照抄现行那一版的三棵树、换实例之后接着追加。

**崩溃恢复抛弃段里写的容器**（`probe-crate/tests/k2_crash_abandoned.rs`，上一轮点名没造的那一形）：前缀在内存盘上走（三棵树旋钮一片 1 条：建 234 个、删两个、再建一个删掉、回退两次，墓碑 3 片、谱系 2 片、退役 2 片），之后录一个窗口，窗口按层 0 的枚举域展开（`enumerate_layer0_selecting_versions_observing_each_state`，段内整写子集逐个、原地覆写三态）；每个崩溃状态 ① 崩溃镜像交带旋钮的新入口，② 按同一组旋钮可写挂载，③ 挂载之后按状态序号播种接 3 步用户动作（建、删最大的活号、随便删、挂着回退到前缀里某一版），每一步之后再交新入口。

全量的状态数（`expected/k2-crash-counts.txt` 原样，三个窗口）：
```
OPUS_K2C window=rollback txgs=14..=14 tombstone=[(1, 1), (2, 1), (3, 1)] lineage=[(1, 1), (2, 1), (3, 1)] retirement=[(1, 1), (2, 1)] leaves=[233, 2]
OPUS_K2C segments=4 lengths=[20, 2, 1, 2] writes=25 closed_form_state_count=1048583 layer0_states=1048588
OPUS_K2C window=create txgs=14..=14 tombstone=[(1, 1), (2, 1), (3, 1)] lineage=[(1, 1), (2, 1)] retirement=[(1, 1), (2, 1)] leaves=[233, 3]
OPUS_K2C segments=4 lengths=[24, 2, 1, 2] writes=29 closed_form_state_count=16777223 layer0_states=16777228
OPUS_K2C window=unlink-largest txgs=14..=15 tombstone=[(1, 1), (2, 1), (3, 1), (4, 1)] lineage=[(1, 1), (2, 1)] retirement=[(1, 1), (2, 1)] leaves=[233, 1]
OPUS_K2C segments=8 lengths=[30, 2, 1, 2, 20, 2, 1, 2] writes=60 closed_form_state_count=1074790413 layer0_states=1074790423
```
一次发布的单元写落在同一段（20–30 个写），最小的窗口（一次回退，追加一片谱系容器）全量就是 1 048 588 态，略过约 10⁶；追加墓碑或退役容器的窗口在 1.7×10⁷ 以上。估时：只展开不长于 2 个写的段（估时用，不是全量）跑 13 态，挂载与续 3 步约 0.85 秒一态一线程（第一次跑估时那一次挂钟 2.6 秒、10 线程分 13 态；`expected/` 里的输出把挂钟字段滤掉了，这个数只在那一次的终端输出里），全量 1 048 588 态 × 0.85 秒 ÷ 10 线程约 25 小时，另加枚举器自己每态两遍恢复与产品入口 checker。按定义「一段历史自己就超过约 10⁶ 的，那段不跑」，全量没跑，交主 agent（「要主 agent 跑的命令」）。估时样本的原样输出（`expected/k2-crash-timing.txt`）：
```
OPUS_K2C timing_only expand_up_to=2（估时用，不是全量）
OPUS_K2C layer0 states=13 violations=0 failed_states=0 checker_violated={"I-9.21": 13, "I-9.36": 13} 
OPUS_K2C states_with_an_abandoned_segment=4 of 13
OPUS_K2C_SUMMARY window=rollback knob=1 states=13 checks=65 mounts_refused=0 panics=0 steps={"create": 12, "rollback-ok": 9, "unlink-any": 5, "unlink-largest": 13} violated={} 
```
（`checker_violated` 那一项是枚举器自己按产品入口判的：旋钮 1 写出的多片容器按产品容量判 I-9.21、I-9.36 红，是预期的，不是误报。）13 态里 4 态是「那次发布有写持久了而它的根没持久」（`some_publish_persisted_without_its_root`；探针的 `states_with_an_abandoned_segment` 这个名字比它说的宽：其中由记录重建那次发布的、与真抛弃的没分开数），挂载之后接着在恢复所取那一版的树上开片，带旋钮的新入口 65 次判 0 违例。估时样本不展开 20 个写的那一段，单元写只落一部分的那一大类一态都没有，所以这 13 态不能当「抛弃段那一形没打中」的证据。读码：I-9.21 / I-9.22 / I-9.36 / I-9.37 每一版只读这一版自己的树（容器经这一版的根读，被抛弃那一版的单元不在任何一版的树里），推不出被抛弃段里的同号容器能让哪一版自相矛盾——这一句是推的，全量要主 agent 跑。

### K2-2 坏镜像漏报：打中（旋钮下「不是最右的那片没装满」）

条款：「3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」（`.claude/kb/decisions/18-块里携带什么信息.md:578`）。写侧在旋钮 K 下照样满 K 条才关（`crates/singlefs-core/src/container_tree.rs:537` 那一判，K 就是 `records_per_container`），所以旋钮 K ≥ 2 的写侧写不出「不是最右的那片少于 K 条」。判法装了旋钮那一支只判不超：「judgements.judge(invariant, record_count <= records_the_knob_allows, || {」（`crates/singlefs-checker/src/walk.rs:12046`，谱系与退役）、「judgements.judge(invariant, record_count <= knob_records, || {」（`crates/singlefs-checker/src/walk.rs:12127`，墓碑）；GPU 那一支同样只判超：「if (d_container_record_count(container_index) > knob_records) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3177`）。第 56 批加的三句（第一片是 1、相邻加一、每片至少一条）管容器号与空片，不管「没装满就关」。

造法（`probe-crate/tests/k2_knob_bad_images.rs`）：旋钮 1 写出三棵树（墓碑 3 片、谱系 2 片、退役 2 片，每片 1 条），同一张镜像分别带旋钮 1、2、3 交新入口。`expected/k2-knob.txt` 原样：
```
OPUS_K2K trees tombstone=[(1, 1), (2, 1), (3, 1)] lineage=[(1, 1), (2, 1)] retirement=[(1, 1), (2, 1)] leaves=[233, 2]
OPUS_K2K judged_with=knob1 I-9.22=Holds I-9.37=Holds violations=[]
OPUS_K2K judged_with=knob2 I-9.22=Holds I-9.37=Holds violations=[]
OPUS_K2K judged_with=knob3 I-9.22=Holds I-9.37=Holds violations=[]
```
带旋钮 2、3 时这张镜像是写侧写不出的（每片都在第 1 条就关了），新入口照样 Holds。

写侧真这么坏时（`mutants.py` 的 `w-close-one-early`：旋钮 K ≥ 2 时装 K − 1 条就开新片，旋钮 1 不变，产品容量少装一条），`expected/mut-w-close-one-early.txt` 里：core 的容器树单测 4 条红、`container_trees_grow_a_level_when_the_root_is_full` 1 条红、`checker_known_bad_images` 2 条红——红的都是靠写侧装满来造坏镜像的那几条（「多于旋钮值」用例按旋钮 2 写、按旋钮 1 判，写侧早关之后每片只剩 1 条，造不出坏镜像），不是 checker 认出了早关；同一份变异上带旋钮的随机历史 10 个种子 600 次判 0 违例，例如种子 1（墓碑旋钮 3、谱系旋钮 2）末版墓碑树 17 片、前 16 片各 2 条，谱系树 13 片各 1 条，每片都比旋钮少一条，那一行照样 `violating=0`（`expected/mut-w-close-one-early.txt` 种子 1 那一行）。产品容量下这一变异要墓碑 816 条、退役 960 片、回退 1360 次才显出来（I-9.21 / I-9.36 的「== 产品容量」），两条产品容量用例是慢档、默认 ignored（推的：它们会红，没跑）。

四句：
1. 分不分辨批次：不分辨。I-9.22 从立起就只判 ≤（第 24 批），I-9.37 照抄（第 53 批），第 56 批没动这一支；它不是 G1–G6 那几形。
2. 判别它的东西 checker 当时看得到吗：看得到——每片记录数、片在叶序里的位置与旋钮值都已在手里。前提是一个池从头到尾用同一组旋钮（旋钮不落盘，checker 只能信调用方给的值）；中途换旋钮的测试池上，这一判会误报（推的：没扫全仓测试有没有中途换旋钮的写法）。
3. 满足的分句：K2 第 2 问「三判之外还有哪种容器号或记录数的坏法漏」——记录数那一种（不是最右的那片少于旋钮值）。
4. 改法在这格上中不中：第 56 批 G2 三判在这一格上不中（上面输出就是在第 56 批的判法上量的）；正文与上一轮判决没给别的改法。

改法（只在我的模型上量过、被攻过零轮）：装了旋钮那一支也判「不是最右的那片记录数 == 旋钮值」（`mutants.py` 的 `fix-knob-full`，墓碑与谱系 / 退役两处各加一判；GPU 那一份没改、没跑）。

| 格 | 改法前 | 改法后 |
|---|---|---|
| 旋钮 1 写出、带旋钮 2 / 3 判 | Holds（量过，`k2-knob.txt`） | I-9.22、I-9.37 都红（量过，`mut-fix-knob-full.txt`） |
| 合法镜像：旋钮下随机历史 10 个种子 × 100 步 | 0 违例（量过） | 0 违例（量过，同一份文件） |
| `checker_known_bad_images` 247 条、`container_trees_grow…` 6 条 | 全绿 | 全绿（量过，同一份文件） |
| 写侧早关一片（`w-close-one-early`）的随机历史 | 0 违例（量过） | 红（推的，没在同一副本上叠两份改动跑） |
| GPU 扫描内核 | 只判 > 旋钮值 | 要同改 `d_judge_containers_under_the_capacity_knob`（推的） |

`expected/mut-fix-knob-full.txt` 里新入口那两行：
```
OPUS_K2K judged_with=knob2 I-9.22=Violated I-9.37=Violated violations=["I-9.22: 根（实例 1、txg 9）：墓碑树（树 65545）里不是最右的容器 1 装了 1 条，不是旋钮值 2（opus 改法臂）", "I-9.37: 根（实例 1、txg 12）：退役树（树 65548）里不是最右的容器 1 装了 1 条，不是旋钮值 2（opus 改法臂）"]
OPUS_K2K judged_with=knob3 I-9.22=Violated I-9.37=Violated violations=["I-9.22: 根（实例 1、txg 9）：墓碑树（树 65545）里不是最右的容器 1 装了 1 条，不是旋钮值 3（opus 改法臂）", "I-9.37: 根（实例 1、txg 12）：退役树（树 65548）里不是最右的容器 1 装了 1 条，不是旋钮值 3（opus 改法臂）"]
```

另一形，记在这里不当打中：一版的容器树比它父版少片（谱系或退役树在某次发布被换成更早一版的），每一版自己仍是「1 起、逐片加一、每片至少一条」，单版三判判不出；墓碑树那一半是 `.claude/kb/checks-owed.md` C614 ① 的题面（「管理员回退那次发布把墓碑树错取成 R_old 那一版」），谱系、退役两棵同形、C614 没点名。没造。

### K2-3 CPU 与 GPU：没打中（读码，GPU 没跑）
- 三句逐句对：第一片是 1（「if (!eq64(d_container_number(0u), u64_of(D_FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE))) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3152`）对 `crates/singlefs-checker/src/walk.rs:12003`、`:12078`）；相邻加一（「if (!(lt64(left_container, right_container) && eq64(add64(left_container, u64_of(1u)), right_container))) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3158`），左片号是 2⁶⁴ − 1 时 `lt64` 那一半挡住回绕，与 CPU 的 `checked_add` 同判）；每片至少一条（「if (d_container_record_count(container_index) == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3163`）对 `crates/singlefs-checker/src/walk.rs:12025`、`:12105`）。有没有对象：CPU 没装旋钮至少两片、装了至少一片；GPU 墓碑 `D_TOMBSTONE_CONTAINER_CAPACITY_KNOB != 0u`（`:3187`）、谱系与退役 `knob_records != 0u`（`:3208`）走旋钮那一支、零片返回，否则少于两片返回——同。
- 旋钮常量为 0 的那一份就是产品入口：放量与层 0 建判器走 `on_card_with_packed_tables`，它编的是「&JudgeKernelSources::of_the_judge(),」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:517`），即三个常量都是 0 的那一份；判法摘要也只罩这一份常量（「container_capacity_knob_wgsl_constants(ContainerCapacityKnobs::PRODUCT).as_bytes(),」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:203`））。旋钮值从 `Option` 映成常量时没装是 0（「knob_records_per_container.map_or(0, |records_per_container| {」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:233`）），三棵树的旋钮构造函数都拒 0，所以「0 = 没装」不撞合法旋钮值。带旋钮的源码只有对拍用例编（`grep` `on_card_compiling_these_kernel_sources`、`of_the_judge_given_the_container_capacity_knobs` 在 `crates/` 的非测试代码里只有定义与这一处产品调用）。
- 判不了的边：一棵树多于「const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:845`）片时 GPU 记判不了（`crash_judge_checker_scan.wgsl:2787`），CPU 照判——不是判得不同，是与正文第二节「GPU 谱系记录表上限 256 条」同族的上限；产品容量下要 256 × 815 条墓碑才到，旋钮 1 的对拍流 257 次删就到（推的，没造）。
- K2-2 那一格 GPU 与 CPU 一起漏（`crash_judge_checker_scan.wgsl:3177` 只判超），不是判得不同。

## K3　旋钮进 GPU（乙 G6，顺带）：没打中（读码）
常量为 0 时编出来的扫描内核走的是「没装旋钮」那两条分支（上一节），与第 56 批之前的产品内核在 G2 三句之外同判；展开量钉值 387 470 → 388 815 是实现员用 `kernel_inlined_size` 量的（`crash_judge_dispatch.rs` 那条钉值单测的注释），不是从原理算的，我没有在 checker 档之外复量（那条单测在 checker 档包里，不跑）。带旋钮的对拍走 `on_card_compiling_these_kernel_sources` 编带旋钮的内核、CPU 那一侧换成新入口（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 56 批那几处，读码），比的是 GPU 与 CPU 新入口。

## K4　M3 参照与等价留档（丙 G4）

### 走得到的叶形状：不是最右的叶恒 ≥ 117 条（推的，量过）
记 P：不是最右的每片叶至少 117 条；Q：相邻两片之和至少 234。按 `crates/singlefs-core/src/inode_tree.rs` 今天的写法逐个转移推：
- 插入只落最右那片，它满 233 条才开新片（`rightmost_container_with_room`），被挤成「不是最右」的那片恰好 233 条：P、Q 保持。
- 删最右那片的一条：删空就摘掉（左邻成了最右）；没删空时只可能与左邻并（并完那片 ≤ 233 且成了最右）；它没有右邻、不借。P、Q 保持。
- 删不是最右的那片（删前 ≥ 117）的一条：剩 ≥ 116，不会删空。能与右邻或左邻并时，因为删前那一对之和 ≥ 234、删后 ≤ 233，并完那片恰好 233；都并不了而剩 116 时从右邻借 ⌊(r − 116) ÷ 2⌋ 条，并不了说明 116 + r ≥ 234、r ≥ 118，借完两片都 ≥ 117，借出的那片再并右邻得 ≤ 233 且 ≥ 117。P、Q 保持。
- 回退取 R_old 的叶，崩溃恢复取更早的一版：都是走过的版本。
所以「删空的那一片两边都有邻居」走不到：删空的只能是最右那片，`has_new_neighbours`（「let has_new_neighbours = holding_position > 0 && holding_position < tree.containers.len();」（`crates/singlefs-core/src/inode_tree.rs:383`））在走得到的状态上恒假，「tree.absorb_until_no_pair_fits(holding_position - 1);」（`crates/singlefs-core/src/inode_tree.rs:385`）是死代码。

量（`probe-crate/tests/k4_leaf_shape_stress.rs`，产品容量，建 933–992 个空 inode 成五片，再 1200 次 unlink：55% 挑不是最右、记录最少的那片里的号，20% 删最大的活号，20% 随便删，5% 建一个；每次发布之后核 P、Q）。`expected/k4-shape.txt` 原样（`final=` 之后的树摘要截掉，原文件里有）：
```
OPUS_K4_SHAPE seed=1 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=2 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=3 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=4 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=5 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=6 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=7 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
OPUS_K4_SHAPE seed=8 unlinks=1200 most_leaves=5 smallest_non_rightmost=117 non_rightmost_below_117=0 adjacent_below_234=0
```
八个种子都把不是最右的叶压到 117 条（下限，正好是借的下限）、从没压到 116 以下，相邻之和从没 < 234。

### K4-2 等价族漏不漏可达形：没打中
放宽那一族（`probes/inode_tree_k4_family.rs.snippet` 插进副本 `inode_tree.rs` 的 `mod tests`，用第 56 批留档里的两种改写函数）：1–2 片时每片记录数取遍 1..=233，3–5 片时取 {1, 2, 3, 58, 59, 115, 116, 117, 118, 119, 175, 176, 231, 232, 233}，相邻之和都 ≥ 234，每片删第一条、中间一条、最后一条，实现与两种改写整份比、删后核 Q。`expected/k4-family.txt` 原样：
```
OPUS_K4_FAMILY shapes=160042 removals=2095099 differ_stop_after_one_pair=0 differ_no_absorb_after_drop=0 precondition_broken=0 first=["", ""]
```
加上上一小节的 P：走得到的形状是这一族（与第 56 批那一族）的子集，Q 下「并一对就停」与原式同值的推法（并完那片恰好 233，与谁相加都 > 233；借后同理）对每一个满足 Q 的形状都成立，所以没有「满足相邻和 ≥ 234 而两种写法结局不同」的可达形。

观察（不当打中）：第 56 批两条等价留档的族是可达形的超集，「删空摘片之后两边还有邻居」那一形（族里中间一片只有 1 条，`not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container` 断言它至少走到一次）按 P 不可达；「删空摘片后不再并」那一种改写因此在走得到的状态上不只是等价，是改的那一支根本跑不到。参照用例（G4 第二段）的删空摘片走的是最右那片、没有右邻，也碰不到这一支。

### K4-1 参照拦不拦 M3 的改坏：没打中
参照不调 `inode_tree.rs`：用例文件 `use` 的只有 `common`、`common_deletion_in_progress`、`TransactionOutput`、`INODE_LEAF_RECORDS`、`SeededRandomSource`（冻结副本 `crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs` 第 27–31 行）。量了四种第 56 批变异行之外的改坏（变异副本挨个跑 core 的 `inode_tree::` 与参照用例）：

| 改坏（`mutants.py`） | core `inode_tree::` | 参照用例 | 走得到吗 |
|---|---|---|---|
| 借的下限 < 117 改 ≤ 117（`k4-floor-inclusive`） | 3 红（含两条等价留档） | 红 | — |
| 借完从借方起并（`k4-absorb-from-borrower`） | 1 红 | 红 | — |
| 先查左邻那一对（`k4-left-first`） | 2 红 | 红 | — |
| 删一条并过之后也照样借（`k4-borrow-even-after-absorbing`） | 2 红（两条等价留档） | 绿 | 推的等价：右并的幸存片恰好 233；左并时被挪到原位置的那片要 < 117 才借，按 P 不是最右的叶 ≥ 117，走不到 |

`expected/mut-k4-borrow-even-after-absorbing.txt` 原样：
```
command args=-p singlefs-core --lib inode_tree:: rc=101
failures:
failures:
    inode_tree::tests::not_absorbing_after_dropping_an_emptied_container_changes_nothing_when_every_adjacent_pair_overflows_one_container
    inode_tree::tests::stopping_every_absorb_pass_after_one_pair_changes_nothing_when_every_adjacent_pair_overflows_one_container
test result: FAILED. 21 passed; 2 failed; 0 ignored; 0 measured; 225 filtered out; 
command args=-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference rc=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; 
```
等价留档在这一条上红，是因为它的族里有不可达形（例如 [1, 233, 2, 233] 删第二片一条：左并之后 2 条那片按改坏去借），参照的历史只走可达形。第 56 批 G4 自己那六行变异（m2、借后吸收、m5、m9、两条等价留档的放宽）我没逐行跑，交主 agent 的变异表。

## K5　回退取树 ID 水位（甲 G5）：没打中
- 抽出前后：函数体是原来内联的那一条表达式（`ring_roots.iter().map(…).fold(cur 那一项, u64::max)`，`crates/singlefs-core/src/mount.rs:7865` 起），调用点把原来的 `current.root.tree_identifier_watermark` 作第三个参数、`&current_table` 作第二个参数传进去（`crates/singlefs-core/src/mount.rs:8147` 起那一句，`&current_table` 在 `:8149`）；多出的那个参数只是一个引用，不求值任何东西，行为不变（读码）。
- 单测钉的是 D23 已定项 14「水位」那一格里树 ID 水位那半句（那一格在 `.claude/kb/decisions/23-journal的角色与格式.md` 第 385 行；同一节「依据」里第 454 行把它概括成「树 ID 水位照旧取环里全部根」）：第 56 批那一行「只取没被抛弃的根」的变异在我的副本上红，`expected/mut-k5-treeid-not-abandoned.txt` 原样：
```
command args=-p singlefs-core --lib tree_identifier_watermark_of_the_rollback_tests rc=101
failures:
failures:
    mount::tree_identifier_watermark_of_the_rollback_tests::an_abandoned_ring_root_whose_tree_identifier_watermark_is_above_the_current_version_sets_the_watermark
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 246 filtered out; 
```
- 不读的参数开不开门：给它一个用处的最近的错写法就是上面那一行变异（拿它去滤抛弃的根），它红；别的用法（例如拿实例表行的 txg 去比水位）类型上就对不上（实例表行里没有树 ID）。没有别的读法能经这个参数改结局而照绿（推的）。四句不适用（没打中）。

## K6　判别力：打中（第 56 批 G2 产品那一支）
第 56 批三判在「没装旋钮」与「装了旋钮」两支共用同一段代码（`crates/singlefs-checker/src/walk.rs:12003`、`:12025`、`:12078`、`:12105` 与相邻加一那两句），会红的用例（G3 的坏镜像与对拍）全在旋钮那一支：产品那一支要容器树有两片以上才判，产品容量下那是 816 条墓碑、960 片退役、1360 次回退，第 56 批两条产品容量用例是合法镜像（判它成立），没有一张产品那一支的坏镜像。量过：只让产品那一支不判三句（`mutants.py` 的 `c-product-skips-three`：六处判式前加「没装旋钮就成立」），`expected/mut-c-product-skips-three.txt` 原样：
```
command args=-p singlefs-harness --test checker_known_bad_images rc=0
test result: ok. 247 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; 
command args=-p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full rc=0
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; 
```
同时量的另两种改坏都被拦：装了旋钮时只一片不判（`c-knob-needs-two`）红两条「多于旋钮值」用例，相邻加一改成不减（`c-lr-adjacent-nondecreasing`）红四条（`expected/mut-c-knob-needs-two.txt`、`expected/mut-c-lr-adjacent-nondecreasing.txt`）。

四句：
1. 分不分辨批次：分辨——三判与它们的用例都是第 56 批加的，缺口在这一批的用例只造旋钮镜像。
2. 判别它的东西 checker 当时看得到吗：看得到（同一张 (容器号, 记录数) 表，与旋钮那一支同一段代码）。
3. 满足的分句：K6「还有哪几种改坏照绿」。
4. 改法在这格上中不中：上一轮判决 G3 的验收清单（「两片同号」、产品容量合法镜像、旋钮下「多于旋钮值」、读不全、旧墓碑入口）没有一条是产品那一支的坏镜像，照 G3 做完这一格仍不中。

改法（推的、被攻过零轮，没实现）：在 `walk.rs` 已有的 `#[cfg(test)]` 模块（`crates/singlefs-checker/src/walk.rs:13197`）里直接调 `judge_tombstone_container_capacity_of_a_version` 与 `judge_lineage_or_retirement_container_capacity_of_a_version`，交「没装旋钮、两片、(2, 3) / (1, 3) / 最右 0 条」三张构造态的表，断言 I-9.21、I-9.36 红——不用写 816 条墓碑；或者在产品容量那两条慢档用例的镜像上改号再判一次。GPU 产品内核那一半同样没有坏镜像对拍（推的，GPU 没跑）。

## 要主 agent 跑的命令

K2-1「崩溃恢复抛弃段里写的容器」的层 0 全量（一次回退那个窗口，1 048 588 态）。前台跑不完，不起；估时约 25 小时（10 线程，挂载与续 3 步按 0.85 秒一态一线程算，另加枚举器自己每态两遍恢复与产品入口 checker），线程多给按比例缩。
- 工作目录：先照「复跑」那一段建好 `<工作目录>/base`（`rerun.sh <工作目录> base-build`），再在 `<工作目录>/base` 下起。
- 命令（仓根 `/home/fy5090/code/singlefs`）：
```
OPUS_WINDOW=rollback OPUS_THREADS=<N> OPUS_STATE_CAP=1100000 CARGO_TARGET_DIR=<工作目录>/target-base \
  bash scripts/hosts/capped.sh <N> bash scripts/hosts/run-with-memory-cap.sh 10G \
  cargo test --offline --release -p opus-probe-b56 --test k2_crash_abandoned -- --nocapture
```
（`OPUS_STATE_CAP` 默认 10⁶，这个窗口 1 048 588 态，要放到 110 万才跑；枚举器自己的工作线程数读 `SINGLEFS_LAYER0_THREADS`（`crates/singlefs-checker-tier/src/crash.rs` 里 `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`），`OPUS_THREADS` 只管挂载与续写那一段。）
- 看哪几行：`OPUS_K2C layer0 states=` 那一行的 `states` 要等于 1048588；`OPUS_K2C_SUMMARY` 那一行的 `mounts_refused`、`panics`、`violated` 都该是 0 / 空；有 `OPUS_K2C_VIOLATION`、`OPUS_K2C_MOUNT_REFUSED`、`OPUS_K2C_PANIC` 行的，每行带状态序号，是打中的线索。
- 内存：录下的持久集合 1 048 588 × 25 个布尔约 30 MiB，每个工作线程同时只造一份镜像，估 2 GiB 以内（推的；13 态那一次峰值 787 MiB）。
- 做完了的：计数、估时样本；没做的：全量。

## 没打中的形状

| 格 | 试过的形状 | 取样 |
|---|---|---|
| K1-1 成员 | 能同时触犯两道被换序的判的单处坏法，逐个列（K1 节表） | 读码；第 56 批两条用例与两行变异量过 |
| K1-1 合法镜像 | 干净重挂、挂着回退（含换实例之后回退到旧实例的版本） | 30 个种子 331 次重挂、794 次回退；崩溃估时样本 13 态挂载 |
| K2-1 误报 | 旋钮下随机历史（四类删、回退、换实例重挂、根长高、三棵树旋钮各异） | 30 个种子 × 100 / 150 步，判 4000 次 |
| K2-1 崩溃 | 回退窗口的估时样本（只展开 ≤ 2 个写的段） | 13 态 × 挂载后 3 步，判 65 次；全量没跑 |
| K2-3 | 三句、旋钮分支、常量 0 的那一份、表上限 | 读码，GPU 没跑 |
| K4-2 | 放宽族、可达形的 P | 160 042 形 × 2 095 099 次删；8 个种子 × 1200 次删 |
| K4-1 | 四种 M3 改坏 | 每种 core `inode_tree::` 与参照用例各一遍 |
| K5 | 「只取没被抛弃的根」那一行变异；读码比抽出前后 | 构造态单测 2 条 |
| K6 | 装了旋钮只一片不判、相邻改成不减 | harness 两份 253 条 |

## 这条腿自己的限度
- 崩溃那一形只做了估时样本：层 0 全量最小的窗口也略过 10⁶ 态、估时约 25 小时，按定义不跑；这一轮原型跑过的层 0 状态合计 13 个（K2-1 估时样本，正式全量 0 个），离 10⁶ / 10⁷ 的上限都远。13 态不展开单元写那一段，不能当「抛弃段那一形没打中」的证据。
- 随机历史与 K4 叶形状压测都是内存稀疏盘上自己建的池（`probe-crate/tests/common_b56/mod.rs`），不是 harness `build_pool` 那份文件盘；harness 用例照它们自己的设计建文件盘（`TMPDIR` 指到工作目录自己的 `tmp/`）。
- 随机历史每批 10 个种子、10 个线程，先跑 2 个种子 × 20 步估时（约 0.13 秒一步），再定每批 100 / 150 步；不设限时。挂钟 40 / 99 / 96 秒。
- 改坏与改法臂都只在副本上量；副本上的数不算入库装置上的数。K2-2 的改法臂 `fix-knob-full` 与 K6 的改法都被攻过零轮；`fix-knob-full` 没改 GPU 那一份。
- 读码推的几句：K1 单处坏法清单是否列全、K1-2 坏盘战役三处成员不变、K2-3 全部、K4 的 P（量过不破，但量的是我取的历史）、K5「不读的参数没有别的错写法」。
- 编译与测试经 `run-with-memory-cap.sh`，上限 4G（派发给的上限 10G；10G 时被别的会话占满的 slice 排队 3 分钟以上，降到 4G 才排得进，峰值 2.9 GiB 以内）；线程 10。看负载时有别的会话的 checker 档 cargo 与另一份 harness 用例在跑（`ps` 看到的是 `cargo test --offline --release -p singlefs-checker-tier --lib --tests` 与 `inode_leaf_merge_and_borrow_against_an_independent_reference`），没有性能测量在跑。
- 写 `probe-crate/tests/common_b56/mod.rs` 第一段时一次写进了 162 行，超过每次 150 行的分段要求一次（文件已在，没有重写）。

## 没做什么
- 不判正推那几格，不替主 agent 采纳；没读禁读清单里的三样（另一条腿的报告与模型、主 agent 的核对稿）。
- 没跑 checker 档（含坏盘战役、GPU 对拍、dispatch 钉值单测）、门禁阶段、变异整表；第 56 批变异表里除 G1 两行、G5 一行之外的行没逐行跑。
- 没造的：只剩台阶行、空发布、多盘以外的盘数这几形合法镜像单独交读者；容器树跨版本回缩那一形（C614 同族）；GPU 上旋钮表多于 256 片的流。
- 没改冻结副本与主工作区的任何文件。草稿目录 `~/.local/share/singlefs/work/m3-batch56-code-r1-opus/` 里的仓副本与编译目录（base、run1–run3 与它们的 target）交回前都删了；留下的只有 `out/`（探索时的终端日志）与 `kept/run3-out/`（交回前那次复跑的 21 份输出与 compare）。

## 交回前的复跑
模型目录定稿之后（上面那张 sha256 表），在空目录 `~/.local/share/singlefs/work/m3-batch56-code-r1-opus/run3/` 里从头跑全部阶段（`SINGLEFS_SCRIPTS=/home/fy5090/code/singlefs/scripts OPUS_THREADS=10 OPUS_MEMORY_CAP=4G`，前台每次至多 228 秒、按阶段名分 19 次起；`mut-k4-borrow-even-after-absorbing`、`mut-k4-floor-inclusive` 各被前台时限打断一次、同名重起接着编），compare 的原样输出（副本与编译目录随后删了，这一次的输出拷在 `~/.local/share/singlefs/work/m3-batch56-code-r1-opus/kept/run3-out/`）：
```
一致 k1-base.txt
一致 k2-crash-counts.txt
一致 k2-crash-timing.txt
一致 k2-knob.txt
一致 k2-random-b1.txt
一致 k2-random-b2.txt
一致 k2-random-b3.txt
一致 k4-family.txt
一致 k4-shape.txt
一致 mut-c-knob-needs-two.txt
一致 mut-c-lr-adjacent-nondecreasing.txt
一致 mut-c-product-skips-three.txt
一致 mut-fix-knob-full.txt
一致 mut-g1-cold-walk-revert.txt
一致 mut-g1-rebuild-revert.txt
一致 mut-k4-absorb-from-borrower.txt
一致 mut-k4-borrow-even-after-absorbing.txt
一致 mut-k4-floor-inclusive.txt
一致 mut-k4-left-first.txt
一致 mut-k5-treeid-not-abandoned.txt
一致 mut-w-close-one-early.txt
compare：21 份，不一致 0 份
```
