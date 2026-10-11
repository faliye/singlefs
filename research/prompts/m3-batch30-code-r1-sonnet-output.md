# m3-batch30-code-r1 正推腿（Sonnet）报告（2026-10-06）

腿：三方正推（`three-way-forward`），格 K1（M1–M4）与 K3；立场：从条款、判决与 `crates/` 今天的实现推出「应该是什么」，再与被判代码逐格比。
冻结副本根 `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/`（下文引代码一律写这个根下的绝对路径加行号）。

## 0 前提与复核

### 0.1 被判文件与冻结副本一致

命令（在冻结副本根下）：`sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch30-code-r1-snapshot/crates-sha256.txt`，原样输出：

```
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-checker-tier/src/crash_judge_checker.wgsl: OK
crates/singlefs-harness/tests/checker_known_bad_images.rs: OK
crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: OK
crates/mutations.tsv: OK
```

### 0.2 我自己复跑的东西（模型目录 `research/prompts/m3-batch30-code-r1-sonnet-model/`）

`rerun.sh`（环境变量 `SINGLEFS_REPO` = 冻结副本根，`SINGLEFS_WRAPPER` 缺省取本脚本所在仓）在空工作目录从头跑通一次（一次调用里开新的一步的时间预算缺省 150 秒（我设了 100），到了退 3，同一个工作目录再调一次接着做，共 4 次调用；工作目录里一份仓副本、一个编译目录，同一时刻一条 cargo，编译经 `run-with-memory-cap.sh 6G`、跑经 `4G`，线程上限 7）。它做两件事：

1. 基线：第 30 批新增的 5 条 F-W 坏镜像、X2、X13、X14，连同第 28、29 批的 H-V 坏镜像（共 14 条）全绿，并在副本里加两处只打印的探针，打出每张镜像红了哪几条不变量。
2. 变异：把变异表里第 30 批的 CPU 行（8 条）与第 2919 行（1 条）逐条改坏、跑点名的参数、看点名的测试红不红；另加我自己的 6 条追加变异（`extra-rows.tsv`，不在变异表里，只用来量测试尖度；这 6 条的参数不带筛子，跑 `checker_known_bad_images.rs` 整份 169 条）。GPU 两行要卡，不跑。

产物对账：`expected-baseline.txt` sha256 `da6d935b68c86e48290871ce94e5a33b88c4e3889470fdd986f6125cd64052a4`，`expected-mutations.txt` sha256 `a2382addef0ec0346687fc4cdd55235ab24902a82d7c842a46d8062499bbb391`；`rerun.sh compare`（`SONNET_WORK=<工作目录>`）排序后比，原样输出 `same: baseline.txt`、`same: mutations.txt`。基线 `rc=0`，14 条 `ok`、0 条 `FAILED`。

本机负载：开工时 `ps` 看到别的会话的 `cargo`、`gate.sh`、一台 ssh 多机预检在跑，没有 `qemu-system`、`vm-bench`、`e152-file-system-benchmark`、`fio`；我的 cargo 命令经内存包装排队，编译一步最长等约 3 分钟才开跑。

### 0.3 变异表静态判（在冻结副本上）

命令：`python3 scripts/mutation/crates-mutation-rows.py static-check /home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree`，原样输出：

```
COPIED	crates Cargo.toml Cargo.lock litmus scripts/hosts/run-with-memory-cap.sh scripts/mutation/crates-mutation-rows.py
ANCHOR	1352	C577 结清 ②：litmus「轮换之后屏障」的 P0 去掉轮换之后那道屏障，与录制流里发布最后三步对不上	文件 litmus/publish-returns-after-the-rotation-barrier.litmus 不存在
CHECKED	3039	301	9
exit=1
```

唯一的 ANCHOR 行是第 1352 行（变异表里第 30 批之前就有的一行）：冻结副本只拷了 `crates/` 与 `Cargo.*`，没有 `litmus/` 目录，所以这一行在副本上指不到文件；与本批的 10 行无关（本批 10 行没有一行进 ANCHOR / MALFORMED / NAME 报告）。主 agent 记在 `records` 里的是主工作区上 `static-check` 退 0；这一条我在副本上复核不了，原因就是副本缺 `litmus/`。

### 0.4 对着判决的两个出处

本轮要核的改法出自 m3-batch28-code-r1 判决第三节：
- 「见本类自证过的就照类型段往下走、槽数按类型段那一类」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:38`）
- 「实例表链 I-1.2：照 F-IT，GPU 链循环判出生身份那一半；对拍补攻方三形。」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:39`）
- 「测试尖度：inode 读不出孩子补」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:40`）、「两形（钉住 X13、X14）」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:40`）

判决第二节 K2 两行把 F-W 与 F-IT 定成「进第 30 批」：
- 「进第 30 批：照攻方 F-W（零轮）——有一份本类自证过就判条目那一条红、再照类型段那一支往下走（槽数按类型段那一类登记）」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:27`）
- 「F-W 同时定了 C739 后一半的取法（一份本类一份别类时槽数按类型段那一类）」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:27`）
- 「进第 30 批：照攻方 F-IT（零轮）链循环里只判 I-1.2 那一半，对拍补三形」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:31`）

kb 里 I-9.2 / I-9.17 / I-9.23 三条今天的原文没有两份副本的读法（C739 的写回随第 30 批由主 agent 做，不归这一轮改）：「类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:290`）、「类型段 2 ⇒ 子单元偏移 6 = 3」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:290`）、「类型段 5 ⇒ 子单元偏移 6 = 3」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:305`）、「1 ⇒ 子单元是码 3」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:311`）。所以 M1 的「两份副本时怎么办」对 kb 条款是「规则没说」，对判决第三节第 1 条才有「一致 / 冲突」可判。

## 1 K1 · M1：F-W（三棵树「孩子是另一类」那一支，另有一份按类型段那一类自证过）

### 1.1 应该是什么（从判决、core 与今天的实现推）

core 读孩子的口径是逐份按条目那一类的长度读、整单元 CRC 对得上就用，不看头里的类标签：「if crc32_castagnoli(&bytes) == location.unit_checksum {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-core/src/recovery.rs:384`）。判决 F-W 说的就是这个读者：「core 照条目那一类长度逐份读、对得上就用」（`/home/fy5090/code/singlefs/research/prompts/m3-batch28-code-r1-main-verification.md:27`）。由此推出 M1 该有的样子：

1. 触发：只在「认出别类」之后才再问「另有一份按类型段那一类自证过」，没认出别类的条目一份都不多读。
2. 「按类型段那一类自证过」= 头里类标签就是类型段那一类、按那一类的长度读回的整单元校验和等于那一份的位置条目校验和；两份次序无关（有一份就行）。
3. 触发之后：条目那一条（I-9.2 / I-9.17 / I-9.23）判红；不走「别类」那一支（不按别类登记槽、不单判每份副本 I-2.1、不 `continue`）；落回类型段那一支的同一段代码（槽数、读孩子、判身份、走子树）。
4. 没触发（没认出别类，或认出别类而没有一份按类型段那一类自证过）：与第 29 批原样。
5. CPU 三棵树的处、GPU 的三处判法相同。

### 1.2 CPU 实际（`crates/singlefs-checker/src/walk.rs`）

- 问「另有一份」的函数：「fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4447`）。长度取类型段那一类：「let unit_bytes = unit_bytes_of_the_class(class_the_entry_type_names)」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4452`）；两条位置条目逐份：「child.locations.iter().any(|location| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4454`）；头里类标签：「MappingKeyUnitClass::of(content[UNIT_HEADER_CLASS_TAG_OFFSET])」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4458`）；整单元校验和：「&& crc32_castagnoli_table(&content) == location.checksum」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4460`）。与 2 一致。
- 墓碑树、谱系树：先认别类 「if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2337`），认出才问 「if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2346`）；问到就判 `identity_invariant` 红、不 `continue`，落到 「match ContainerTreeEntryType::of(entry_type, record_type) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2369`） 那一段与类型段那一支同一份代码（类型段 2 的码 3 一支登记两槽：「self.note_references_of_a_pointer_followed_by_the_walk(child, 2, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2548`））；问不到才走原来的 「self.judge_a_container_tree_entry_whose_child_is_of_another_class(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2357`） 并 `continue`。与 1、3、4 一致。
- inode 树：先认别类 「match child_unit_class_other_than_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2709`），认出才在 「class_the_entry_type_names.is_some_and(|class| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2717`） 里问；问到就判 I-9.2 红并把别类取消（「if a_copy_of_the_class_the_entry_type_names_verifies {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2724`） 那一支交回 `None`）；之后的 「if let Some(child_class) = child_class_the_walk_does_not_go_below {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2736`） 不进，落到 「let (length, span_in_slots) = match registered_entry_type {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2788`） 那一段（叶容器 32 KiB 两槽、内部节点 16 KiB 一槽）。类型段不认得的条目 `class_the_entry_type_names` 是 `None`，`is_some_and` 恒假，仍走原来的别类那一支。与 1、3、4 一致。
- 「别类」那一支自己没改，只在注释里加了「只在没有一份按类型段那一类自证过时走到这里」。

### 1.3 GPU 实际（`crash_judge_checker.wgsl`）与 CPU 同一判法的逐项对

| 项 | CPU | GPU | 同不同 |
|---|---|---|---|
| 问法 | 「MappingKeyUnitClass::of(content[UNIT_HEADER_CLASS_TAG_OFFSET])」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4458`）、「&& crc32_castagnoli_table(&content) == location.checksum」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4460`） | 「if (first_copy.version != NONE && rd8(first_copy.base, 6u) == class_the_entry_type_names」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3063`）；「return second_copy.version != NONE && rd8(second_copy.base, 6u) == class_the_entry_type_names」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3068`） | 同：两份各问一次，标签等于类型段那一类且按那一类的长度校验和对得上 |
| 长度 | `unit_bytes_of_the_class` 的注释：「码 1 数据单元与码 3 打包记录单元 32 KiB，码 2 节点 16 KiB」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:4379`） | `w_unit_verifies_at_the_length_of_its_class` 的注释：「码 2 16 KiB，码 1 / 码 3 32 KiB」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3017`） | 同 |
| 墓碑树 / 谱系树，类型段 0 | 「if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2346`） | 「if (class_of_the_child_of_an_internal_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 2u)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3156`） | 同：认出别类才问；问到判 `identity_invariant` 红后取消别类，落到类型段 0 那一支 |
| 墓碑树 / 谱系树，类型段 = 记录类型 | 同上 | 「if (class_of_the_child_of_a_container_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 3u)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3209`），随后 「w_judge(identity_invariant, class_of_the_child_of_a_container_entry == NONE);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3210`） | 同；之后登记两槽（`w_note_pointer_references(child, 2u)`）与 CPU 的 `note_references…(child, 2, …)` 同 |
| inode 树根，类型段 2 | 「if let Some(child_class) = child_class_the_walk_does_not_go_below {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2736`） | 「if (class_of_the_inode_child != NONE && class_the_inode_entry_type_names != NONE」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2678`）、「&& w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, class_the_inode_entry_type_names)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2679`），随后 「w_judge(INVARIANT_I_9_2, class_of_the_inode_child == NONE);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2681`） | 同：认出别类才问，类型段不认得（`NONE`）不问 |
| inode 树根，类型段 0 | 落到类型段 0 那一支，CPU 照走 | 「类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2827`）（同一段 `undecidable = UNDECIDABLE_TOO_DEEP`） | 不同但已知：GPU 记「判不了」，由 CPU 兜（主 agent 已按字面认下，进第 31 批不动） |

GPU 的 `&&` 在 WGSL 里是短路的，`w_some_copy…` 只在认出别类之后才被求值，没认出别类的条目不多读。

### 1.4 我跑出来的数：F-W 与 H-V 两族坏镜像各红了哪几条不变量

命令：`SINGLEFS_REPO=<冻结副本根> bash research/prompts/m3-batch30-code-r1-sonnet-model/rerun.sh`；下面是 `expected-baseline.txt` 里的探针行（原样；`kind=2` inode、`10` 墓碑、`11` 谱系；`loc` 是码 2 那一份在位置条目 0 还是 1；`only_violation` 是只放违例不做 H-V 那一步时红的，`fw` 是 F-W 那一形红的）：

```
PROBE-FW kind=10 loc=0 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.23"}
PROBE-FW kind=10 loc=1 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.23"}
PROBE-FW kind=11 loc=0 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.17"}
PROBE-FW kind=11 loc=1 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.17"}
PROBE-FW kind=2 loc=0 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.2"}
PROBE-FW kind=2 loc=0 only_violation={"I-9.15"} fw={"I-2.1", "I-4.8", "I-7.4", "I-9.15", "I-9.2"}
PROBE-FW kind=2 loc=0 only_violation={"I-9.7"} fw={"I-2.1", "I-4.8", "I-7.4", "I-9.2", "I-9.7"}
PROBE-FW kind=2 loc=1 only_violation={"I-1.7"} fw={"I-1.7", "I-2.1", "I-4.8", "I-7.4", "I-9.2"}
PROBE-FW kind=2 loc=1 only_violation={"I-9.15"} fw={"I-2.1", "I-4.8", "I-7.4", "I-9.15", "I-9.2"}
PROBE-FW kind=2 loc=1 only_violation={"I-9.7"} fw={"I-2.1", "I-4.8", "I-7.4", "I-9.2", "I-9.7"}
PROBE-HV kind=10 loc=0 I-9.23 violated={"I-2.1", "I-4.8", "I-7.2", "I-7.4", "I-9.23"}
PROBE-HV kind=10 loc=0 I-9.23 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.23"}
PROBE-HV kind=10 loc=1 I-9.23 violated={"I-2.1", "I-4.8", "I-7.2", "I-7.4", "I-9.23"}
PROBE-HV kind=10 loc=1 I-9.23 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.23"}
PROBE-HV kind=11 loc=0 I-9.17 violated={"I-2.1", "I-4.8", "I-7.2", "I-7.4", "I-9.17"}
PROBE-HV kind=11 loc=0 I-9.17 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.17"}
PROBE-HV kind=11 loc=1 I-9.17 violated={"I-2.1", "I-4.8", "I-7.2", "I-7.4", "I-9.17"}
PROBE-HV kind=11 loc=1 I-9.17 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.17"}
PROBE-HV kind=2 loc=0 I-9.2 violated={"I-1.1", "I-1.2", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4", "I-9.2"}
PROBE-HV kind=2 loc=0 I-9.2 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.2"}
PROBE-HV kind=2 loc=1 I-9.2 violated={"I-1.1", "I-1.2", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4", "I-9.2"}
PROBE-HV kind=2 loc=1 I-9.2 violated={"I-2.1", "I-4.8", "I-7.4", "I-9.2"}
```

读法（我的推，不是产物自带的标签）：`PROBE-HV` 每个（`kind`，`loc`）出两行，红得多的那一行带 I-7.2，是「类型段 0」那一形（它走到 「if node.level <= 1 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2496`） 记走读失败，I-7.2 由走读失败驱动：「judgements.judge("I-7.2", newest_failures.is_empty(), || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:11301`））；红得少的是「条目不改」那一形。

- F-W 五形（`PROBE-FW`）：两种盘序红的集合逐条相同；条目那一条（`I-9.2` / `I-9.23` / `I-9.17`）与子树里的违例（`I-9.15`、`I-9.7`、`I-1.7`）都红；`I-3.1`、`I-3.11`、`I-5.1` 都不红，槽数确实按类型段那一类（码 3、两槽）登记。与 1.1 的 3 一致。
- 还红着的 `I-2.1`、`I-4.8`、`I-7.4`：另一份是 16 KiB 的码 2，按条目那一类的 32 KiB 读回校验和对不上，`read_referenced_unit` 对每份副本判 I-2.1（「judgements.judge("I-2.1", matches, || {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/image.rs:1530`）），I-4.8、I-7.4 由 I-2.1 违例数驱动（「!newest_failures.is_empty() || walk.judgements.violation_count("I-2.1") > 0;」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:11061`））。这是主 agent 已认下的「I-2.1 一族照红」，不是新冲突。
- 类型段 0 的 H-V 形（`PROBE-HV kind=2` 红 10 条的那一行）：F-W 之后孩子按类型段 0 的一支当 16 KiB 节点往下走，所以多红了 `I-1.1`、`I-1.2`（我的推：孩子是根的字节，层级与出生身份对不上；没逐条查违例说明）与 `I-3.1`、`I-3.11`、`I-5.1`（我的推：节点登记 1 槽，真实分配的容器占 2 槽）。这正是 C739 后一半「一份本类一份别类时槽数按类型段那一类」的后果，判决已定；这条是观察，不记冲突（见 1.5）。

### 1.5 观察（不判冲突）

- O1 类型段 0 的 F-W 形在 CPU 三棵树上今天有行为（多红上面那几条），但没有任何一条测试钉它，见 3.4 的 G1。
- O2 GPU 在 inode 树根上对类型段 0 的 F-W 形记「判不了」，CPU 给结论；已知，主 agent 报告里第 2 条已认。
- O3 F-W 之后墓碑树、谱系树、inode 树把「走过」的键取 `child.locations[0]`，而读回的那一份可能是位置条目 1 那一份；实例表链那一处为同类问题改过（`placement_it_was_read_from`），这三处没改。我没造出会翻面的反例（两种盘序红的集合逐条相同，见上），写在这里交攻方腿的格，不当判据。

### 1.6 M1 判定

一致。判法在 CPU 三处与 GPU 三处是同一个（上表）；往下走那一支与类型段那一支是同一段代码；没认出别类的条目一份不多读；kb 条款对两份副本「规则没说」。

什么现象会推翻：造出一张镜像，CPU 与 GPU 对「是否另有一份自证过」给不同答案（例如一份类标签是类型段那一类而校验和只在另一类的长度上对得上）；或「没认出别类」的条目被多读一份；或类型段 0 / 2 的 F-W 形里 CPU 与 GPU 对槽数登记不一样。

## 2 K1 · M2：F-IT（GPU 实例表链每一片在 I-1.12 之后判 I-1.2）

### 2.1 应该是什么

kb 的 I-1.2 把实例表单元单拎出来：「打包记录类型 4 的实例表单元按本行的例外只免谓词那一半」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:27`）；「指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向」（`/home/fy5090/code/singlefs/.claude/kb/invariants.md:27`）。CPU 今天是这么做的：链上每一片过了「走过没有」那一关（「if !self.visited_units.insert(placement_it_was_read_from) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:3167`））、过了容器格式那一关（`judge_packed_container` 返 `None` 就 `break`）、判完 I-1.12（「self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:3177`））之后，调 「self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:3181`）；那个函数比三样（「let matches_the_pointer = identity.write_order.birth_txg() == pointer.birth_txg」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:1056`），另两样在其后两行）；头里的三个值取码 3 的偏移：「birth_txg: read_u64(unit, 73),」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:983`）、「tail: TailOfTheBirthIdentity::BirthSequence(read_u32(unit, 103)),」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:986`）（写序实例在 93，同一函数）；指针一侧的三个值在指针偏移 42、78、82（「tail: u64::from(read_u32(bytes, 82)),」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/image.rs:1363`））。I-4.2 那一半（已发布谓词）不判。

所以 GPU 该有的样子：同一个位置（「走过」之后、容器格式之后、I-1.12 之后）、同样三样比较、偏移相同、不判 I-4.2、链上每一片（含第 0 片与链指针指着的后继片）都判。

### 2.2 GPU 实际

- 第 0 片的指针用 `parse_node_pointer` 取：「w_walk_instance_table_chain(parse_node_pointer(record_base, instance_table_pointer_offset), is_newest, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3318`）；后继片同：「pointer = parse_node_pointer(p.base, packed_record_offset(p, p.record_count - 1u) + 2u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1721`）；节点指针的尾段在偏移 82 取 4 字节：「p.tail = vec2<u32>(rd32(base, o + 82u), 0u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:557`）。
- 顺序：「if (!w_visit(device_read_from, slot_read_from)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1620`） → 「let packed = w_judge_packed_container(s, 4u, fsid_low);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1627`） → 「w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1636`） → 新加的 「w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1641`） 「&& eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail));」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1642`）。与 CPU 的顺序一致（走过 → 容器格式 → I-1.12 → I-1.2）。
- 偏移：新加的三处 73、93、103 与 `w_judge_birth_identity` 码 3 那一支同偏移（「birth_txg = rd64(s.base, 73u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1394`）、「tail = vec2<u32>(rd32(s.base, 103u), 0u);」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1396`）），也与 CPU 的 73、93、103 相同。
- 不判 I-4.2：全文件 `INVARIANT_I_4_2` 只有一处使用，在 `w_judge_birth_identity` 里（「w_judge(INVARIANT_I_4_2, w_published(birth_txg, instance, unit_class == 1u, transaction));」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1411`）），链循环不调它。
- 对拍用例三形取的指针偏移：「BirthIdentityFieldOfANodePointer::BirthTxg => 42,」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8597`）、「BirthIdentityFieldOfANodePointer::Instance => 78,」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8598`）、「BirthIdentityFieldOfANodePointer::Tail => 82,」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8599`），与 CPU 的指针解法一致（42、78、82）；断言 CPU 只红 I-1.2：「std::collections::BTreeSet::from(["I-1.2"]),」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8646`）。

### 2.3 我复核了什么、没复核什么

- CPU 一侧只红 I-1.2：我跑了已有的最终镜像用例 `an_instance_table_pointer_whose_birth_txg_differs_from_the_instance_table_header_reddens_only_the_birth_identity_invariant`，原样输出 `test … ok`、`test result: ok. 1 passed; 0 failed`。实例（78）与尾段（82）两形的 CPU 期望、以及三条流上「每个崩溃状态的并集只有 I-1.2」我没复算：要整流枚举，在 `singlefs-checker-tier` 的对拍用例里，按定义是重型测试、要卡，复核不了。
- GPU 与 CPU 的逐位一致本身要卡，我只读了 wgsl 与 CPU 代码，没有跑。

### 2.4 共享片 / 走过的片

CPU 与 GPU 都在「走过」之后才判；实例表单元被根环里不同根共享、链途经已走过的片时两边都不再判那一片（`break` 前就出去了），所以 I-1.2 不会重复判，也不会判错片（CPU 的键是读回的那一份的落点，GPU 的键也是 `device_read_from`、`slot_read_from`，见 「if (!w_visit(device_read_from, slot_read_from)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1620`））。

### 2.5 M2 判定

一致。推翻它的现象：对拍里任一条 F-IT 流上 GPU 与 CPU 在某个状态不同；或实例表链上某片被 GPU 判了 I-1.2 而 CPU 因「走过」没判（或反过来）。

## 3 K1 · M3：测试尖度（X2、X13、X14、F-W 五形）

### 3.1 应该是什么

判决第三节第 3 条要三类：inode 读不出孩子补「子指针出生树加 1」一形；补「第一份头类标签不登记 / 头读不出、第二份自证成别类」两形；第 1 条要 F-W 的几形做成坏镜像（两种盘序同判、子树里的违例红）。每份期望该写：红哪一条（条目那条、子树里的违例）、不该红哪几条（槽数连带的三条）、两种盘序同一批、有对照（只放违例不做 H-V，红的是容器里的违例；不放违例，子树那一条不红）。

### 3.2 实际写了什么

- F-W 五形共用一个断言函数 「fn assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12058`）：先判对照 「对照：只放违例、不做 H-V，{subtree_invariant} 红」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12069`），再两种盘序各判一次，每次先判「不放违例、子树那一条不红」（「不放违例，{subtree_invariant} 不红」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12084`）），再判子树违例红、条目那一条红、「for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12102`） 三条不红，最后 「violated_on_each_device[0], violated_on_each_device[1],」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12111`） 两种盘序同一批。五形：inode 预留段非 0（I-9.7）、blocks 加 1（I-9.15）、inode 记录宽 139、墓碑记录宽 39、谱系记录宽 15（I-1.7）。
- X2：「fn an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12242`），走已有的 `assert_a_birth_tree_field_one_off…` 那个函数，字段取子指针头部出生树；该函数先判对照（不改条目、读不出孩子，I-9.2 不红），再加 1 判红且违例说明里带两段出生树。
- X13、X14：共用 「fn assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12258`）（盘 1 那一份自证成码 2，再把盘 0 那一份弄坏：类标签写成登记表外的 9，或位置条目的槽指到盘尾之后读不出），判 I-9.2 红且说明点名孩子头里的类：「assert_the_violation_names_the_child_class(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12270`）、「"IndexNode",」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:12274`）。
- GPU 对拍：F-W 一条流（「fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8548`），断言 「for invariant in ["I-9.2", "I-9.15"] {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8558`） 两条红）与 F-IT 三条流。GPU 那一侧不跑，见 2.3。

### 3.3 我量到的尖度：变异表第 30 批的 CPU 行 + 第 2919 行 + 我追加的 6 条

`rerun.sh` 逐条改坏、跑点名的参数（原样结果在 `expected-mutations.txt`，下面是逐列摘要；追加的 6 条跑 `checker_known_bad_images.rs` 整份 169 条）：

| 变异表行 | 改坏什么 | 点名测试红不红 | 参数筛到几条 / 红几条 |
|---|---|---|---|
| 2919（第 26 批，只换了原文锚点） | inode 读不出孩子时 I-9.2 不判 | 红 | 9 / 1 |
| 3035 | `some_copy…` 里 `.take(0)`，三棵树 F-W 全关 | 红 | 5 / 5 |
| 3036 | inode 那一处 F-W 关（预留段形） | 红 | 5 / 3 |
| 3037 | 同 3036（原文、替换文、参数逐字相同，只点名的测试不同） | 红 | 5 / 3 |
| 3038 | 墓碑树、谱系树那一处 F-W 关（墓碑记录宽形） | 红 | 5 / 2 |
| 3039 | 同 3038（逐字相同） | 红 | 5 / 2 |
| 3040（X2） | 子指针出生树 `==` 改 `>=` | 红 | 3 / 1 |
| 3041（X13） | 第一份类标签不登记 `continue` 改 `break` | 红 | 3 / 1 |
| 3042（X14） | 第一份头读不出 `continue` 改 `break` | 红 | 3 / 1 |
| 追加 S1 | 墓碑树、谱系树那一处只在类型段说码 3 时走 F-W（类型段 0 那一支关） | 无测试点名，**169 条里没有一条红** | 169 / 0 |
| 追加 S2 | inode 那一处只在类型段 2 时走 F-W（类型段 0 那一支关） | **169 条里没有一条红** | 169 / 0 |
| 追加 S3 | `some_copy…` 不比整单元校验和（只看头里类标签） | 红：被 4 条已有的陈旧副本用例（`…with_a_stale_copy_of_the_child…`）抓到，不是第 30 批新测试 | 169 / 4 |
| 追加 S4 | `some_copy…` 不比头里类标签（只看校验和） | **169 条里没有一条红** | 169 / 0 |
| 追加 S5 | `some_copy…` 只看第一条位置条目那一份 | 红 | 169 / 11 |
| 追加 S6 | 墓碑树、谱系树那一处 F-W 之后不判条目那一条红 | 红 | 169 / 6 |

### 3.4 写全了没有：判决点名的三形都钉住了，字面范围里另有四处没钉

判决第三节第 3 条点名的三形（X2、X13、X14）与第 1 条的 F-W 坏镜像：各有一条测试，各有一行变异替它报，我复跑都红（3.3 前九行）。X2 抓的是 `==` 改 `>=`（子指针出生树加 1 时 `>=` 判成立、测试要红却不红）；已有的「身份出生树加 1」那条抓 `==` 改 `<=`；两条合起来这两种改坏都抓得到。F-W 五形的期望写得全：条目那一条、子树违例、槽数连带三条不红、两种盘序同一批、对照两条都在。

字面范围里没钉住的（都是我追加变异量出来的，不是判决点名的，记「缺口」不记冲突）：

- G1 类型段 0 那一支的 F-W 没有测试钉（S1、S2 在整份 169 条里没有一条红）。判决第三节第 1 条写的是「CPU 三棵树……见本类自证过的就照类型段往下走」，类型段 0 在字面范围里；第 28、29 批的类型段 0 的 H-V 形与陈旧副本形在 F-W 关掉之后照样过，因为它们只断言条目那一条红、两种盘序同判，不断言类型段 0 一支独有的后果（走读失败 I-7.2、多红的 I-1.1 / I-1.2 / I-3.1 / I-3.11 / I-5.1，见 1.4）。
- G2 `some_copy…` 里「头里类标签就是类型段那一类」这个条件没有测试钉（S4）：对应的形是一份在类型段那一类的长度上校验和对得上、头里类标签却是同长度的别的类（码 1 数据单元对码 3，都是 32 KiB）。同一函数里「校验和」那个条件（S3）由 4 条已有的陈旧副本用例钉着，不是第 30 批的新测试。
- G3 F-W 五形与 X2、X13、X14 全部只用层级 1 的根；类型段 0 的孩子是下一层节点（层级 2 的树）、F-W 要「往下走那一份」的子树这一形没有测试（层级 1 的根下类型段 0 的一支在 「if node.level <= 1 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2496`） 记走读失败就停了）。
- G4 F-W 形不钉 I-2.1 一族（I-2.1、I-4.8、I-7.4）红不红：F-W 五形只断言「槽数那三条不红」，I-2.1 一族是否红由 `read_referenced_unit` 的每份副本都判决定，已被主 agent 认下，但没有断言；我没为它做变异验证，只是读断言得出。

### 3.5 M3 判定

一致（判决点名的形都补了、钉住了）；另有 G1–G4 四处字面范围里没钉住的形，写成缺口交主 agent 判要不要补。什么现象会推翻缺口：整张变异表或别的测试二进制（我只跑了 `checker_known_bad_images.rs` 这一份的 169 条）里有测试在 S1、S2、S4 下红。什么现象会推翻「一致」：某条判决点名的形在提交时的整表复跑里没红。

## 4 K1 · M4：变异表追加 10 行、第 2919 行替换、补修

### 4.1 应该是什么

每条变异行六段、原文在被改文件里恰好命中一次、替换文真改行为、点名的测试在参数筛得到的集合里且改坏之后红；第 2919 行的替换只换锚点（行为不变）；补修只把两条对拍用例的取卡换成卡池。

### 4.2 实际

- 追加的 10 行是变异表第 3035–3044 行（`awk -F'\t' 'NR>=3035{print NR": "$1" | "NF}'` 每行 6 段，第 3035–3044 行全为 6）；`static-check` 没有为这 10 行报 ANCHOR / MALFORMED / NAME（0.3）。
- 参数筛得到：筛子是测试名子串。我对冻结副本里 `checker_known_bad_images.rs` 的 169 条 `#[test]` 名逐个做子串匹配：`beside_a_copy_verifying_as_a_node` 命中 5 条（正是 F-W 五形），`child_pointer_birth_tree_is_one_off_over_a_leaf` 命中 1 条（X2），`whose_second_copy_verifies_as_a_node` 命中 2 条（X13、X14）；每行点名的测试都在自己的筛子里，我又逐行改坏复跑（3.3）：9 条 CPU 行全红，点名的测试红。
- 第 2919 行替换只换锚点：变异表第 2919 行改前（主 agent 的改前副本 `/home/fy5090/.local/share/singlefs/work/m3-batch30/before/crates/mutations.tsv`）与改后逐段比，只有第 3 段（原文）不同，第 1、2、4、5、6 段逐字相同；原文里变的只是那句违例说明的末尾，从「不读」改成 「（孩子没有一份按它头里那一类的长度自证过，不往下走）」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs:2782`） 那一句的今天写法。改前这一行的原文在 `walk.rs` 命中 0 次（主 agent 的 `/home/fy5090/.local/share/singlefs/work/m3-batch30/m/static-check-before.txt` 里有 `ANCHOR\t2919\t… 原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次`），改后命中 1 次，我复跑这一行改坏之后点名的测试红（3.3 第一行）。
- GPU 两行（3043、3044）：点名的两条用例在对拍文件里存在（「fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8548`）；F-IT 点名的 `…names_another_birth_txg_than_its_page` 在同一文件）；原文锚点命中 1 次（`static-check` 不报）；改坏之后红不红要卡，没跑。
- 补修：两条对拍用例改用卡池取卡，`the_card_pool_or_report_not_run` 在 「fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(」（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8548`） 同一文件里原本就有定义（行 224），`compare_on_every_state` 收 `&'static GpuCardPool`（行 1986）；改法是 4 增 4 删，我读到的两处调用与别的已有用例同形。编译与 clippy 我没跑（`singlefs-checker-tier` 是 checker 档），主 agent 记的 clippy 退 0 我复核不了。

### 4.3 观察

- 3036 与 3037、3038 与 3039 逐字同一处改坏，只是点名的测试不同；改坏效果上各自等于一条，不是缺陷。
- F-IT 的三条对拍流只有一条被变异行点名（`…birth_txg…`）；实例（偏移 93）与尾段（偏移 103）那两个偏移若被改错，变异表没有行替它们报，只有对拍流自己会红（要卡）。

### 4.4 M4 判定

一致（CPU 9 行我复跑全红；GPU 2 行未跑；第 2919 行只换锚点）。推翻它的现象：某条 CPU 行在提交时的整表复跑里没红；或第 2919 行的替换文与改前相比除原文外还有别的段不同。

## 5 K3 · 五份被判文件，每份一格

| 文件（冻结副本） | 本批改了什么 | 判到的格 | 判定 |
|---|---|---|---|
| `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker/src/walk.rs` | 新函数 `some_copy_…`（4447）；墓碑树、谱系树一处调用（2346）；inode 树一处调用（2717）；注释 | M1（CPU 三处）、M4 的锚点所在文件 | 一致 |
| `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` | 新函数 `w_some_copy_…`（3061）；三处调用（2678、3156、3209）；链循环加 I-1.2（1641） | M1（GPU 三处）、M2 | 一致（读码，GPU 未跑） |
| `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs` | F-W 五形、X2、X13、X14（12047 起） | M3、M4 的筛子 | 一致，另有四处没钉（3.4 的 G1–G4） |
| `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | F-W 一条流、F-IT 三条流、补修取卡（8512 起） | M3、M4 | 一致（读码，要卡，未跑） |
| `/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1/tree/crates/mutations.tsv` | 追加 10 行（3035–3044），第 2919 行换锚点 | M4 | 一致（CPU 行复跑全红，GPU 两行未跑） |

## 6 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 · M1 | 一致 | CPU 三处与 GPU 三处判法是同一个；往下走那一支与类型段那一支是同一段代码；没认出别类的条目一份不多读；kb 条款对两份副本规则没说。观察 O1–O3 不记冲突 |
| K1 · M2 | 一致（GPU 未跑） | GPU 链循环在「走过」「容器格式」「I-1.12」之后判 I-1.2，偏移 73 / 93 / 103 与指针 42 / 78 / 82 同 CPU，不判 I-4.2 |
| K1 · M3 | 一致，另有缺口 G1–G4 | 判决点名的 X2、X13、X14 与 F-W 五形都钉住（变异行复跑全红）；类型段 0 的 F-W、`some_copy…` 的类标签条件、层级 2、I-2.1 一族没钉 |
| K1 · M4 | 一致（GPU 两行未跑） | 追加 10 行成形、CPU 9 行复跑全红、筛子筛得到点名的测试；第 2919 行只换了锚点；两对变异逐字同一处改坏 |
| K3 · walk.rs | 一致 | M1 CPU 三处 |
| K3 · crash_judge_checker.wgsl | 一致（读码） | M1 GPU 三处、M2 |
| K3 · checker_known_bad_images.rs | 一致，另有缺口 | M3 |
| K3 · crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs | 一致（读码，要卡，未跑） | M3、M4 的 GPU 对拍与补修取卡 |
| K3 · mutations.tsv | 一致 | M4 |

## 7 没做什么

- K2 的格（哪一格会错、造反例）不归我，没攻；1.5 的 O3 只是读码时顺带看到的一处，没造反例、没验。
- GPU 不跑：F-W 对拍流、F-IT 三条流、变异表 3043、3044 两行，都只读代码、用 CPU 一侧算期望；F-IT 实例与尾段两形的 CPU 期望、F-W 与 F-IT 各流上 CPU「每个崩溃状态的并集」我没复算（要整流枚举与卡）。
- 没跑重型测试：没有 `singlefs-checker-tier` 的测试与编译，没有 crates 变异整表，没有全量 `cargo test`；只跑了 `singlefs-harness` 的 `checker_known_bad_images`（含追加变异下的整份 169 条）。
- 补修之后的 clippy 与 fmt 退 0 我没跑，复核不了（`singlefs-checker-tier` 是 checker 档）；`static-check` 只在冻结副本上跑（副本缺 `litmus/`，第 1352 行 ANCHOR 与本批无关），主工作区上退 0 我没复核。
- 不替主 agent 采纳或出判决；G1–G4 与 O1–O3 只是交主 agent 判。
- 没读攻方腿的报告、模型与工作目录，没读本轮判决。
- 自己的追加变异 S1–S6 只在副本里用，不进变异表；模型目录 `research/prompts/m3-batch30-code-r1-sonnet-model/` 的 `rerun.sh` 在空工作目录从头跑通过一次（结果对账见 0.2）。
- 自己建的编译目录与仓副本（`/home/fy5090/.local/share/singlefs/work/m3-batch30-code-r1-sonnet/` 下）交回前删掉。
