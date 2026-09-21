# c381-r3 云端攻方腿（Opus）报告：K3（候选乙的安全性与定义域）、K4（两条定案落地）

立场按派发提示：假设候选乙（「探针写改成写这次失败的那个落点，不是固定落点」）是错的。
不碰 K1、K2。下面每个数都出自**仓副本**上的装置，按 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」，
副本上的数不进 kb，要主 agent 在入库装置上重做才能引。

## 〇、复跑命令、模型目录与 sha256

模型目录 `research/prompts/c381-r3-opus-model/`：

```
53a79230b16895594cf32681833b35bebebc05d350563f7eac7ec2150495115e  badsector3.log
da592bf2288ae06ec3dcf03d3859cdd30faee5edc5029cce8c4d8ef656d2e45a  c381-r3-arms-core.diff
b19cd28c9dfde85f5e9096cdaa48cdd247d1c26f452df7ba60ebedf913284892  c381r3_landing.rs
e2f2a897888ccb0f3d203e5de783174d3ddabfe7672d8de0a318f05e86cd79fc  c381r3_probe_common.rs
f48c4b0a4bdc0022df8df785146ed053c6e856c8bbb12d7e8ba8d23cf79f0775  c381r3_single.rs
78ec43bcbfbd058567d36780b9c73a319f99b69295d57dd493668a2a6e8d06ed  exhibit-wrapped18.log
589e062aef8c8ed2aa4d9f69f4e00e24edf1ae09356d62b4368f182d39391611  landing.log
074d99df55fb27cc711b184d61898ca055c763fc80525d154013e4f54cbf5947  lyingrange.log
77e8f6f7f819e093916c2d4d1f07bce4e84bf29e9ee5719ff1df55963528f85f  patch_core.py
4599fee16d7fb6c451dd9c8fe0a8333d23c20bcab1673d09553fb9ed238a2b0a  patch_probe.py
```

复跑（副本目录 `/tmp/claude-1000/c381-r3-opus/`，`repo/` 是仓副本）：

```
# 1) 建副本：工作区拷一份 → 用 HEAD 的 crates 盖回去（见第二节「工作区今天编不过」）→ 打两个补丁
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/claude-1000/c381-r3-opus/repo/
cd /home/fy5090/code/singlefs && rm -rf /tmp/claude-1000/c381-r3-opus/repo/crates \
  && git archive HEAD crates Cargo.toml Cargo.lock | tar -x -C /tmp/claude-1000/c381-r3-opus/repo/
find /tmp/claude-1000/c381-r3-opus/repo/crates -name '*.rs' -o -name 'Cargo.toml' | xargs touch
sed -i "s|/tmp/claude-1000/c381-r3-opus/repo|<你的副本>|" patch_core.py   # 它的 root 路径写死，复跑先改这一行
python3 research/prompts/c381-r3-opus-model/patch_core.py   # c381-r2 的臂开关（core）
cp research/prompts/c381-r3-opus-model/c381r3_{probe_common,single,landing}.rs \
   /tmp/claude-1000/c381-r3-opus/repo/crates/singlefs-harness/tests/
# core 的臂号放宽 + harness 两处穷尽匹配（编译器报出来的三处，逐字）：
python3 - <<'PY'
import pathlib
root = pathlib.Path("/tmp/claude-1000/c381-r3-opus/repo")
def once(p, old, new):
    s = p.read_text(); assert s.count(old) == 1, (p, s.count(old)); p.write_text(s.replace(old, new))
t = root/"crates/singlefs-core/src/transaction.rs"
once(t, "if (state.arm == 2 || state.arm == 5) &&", "if (state.arm == 2 || state.arm >= 5) &&")
once(t, "                5 => {\n", "                5 | 6 | 7 | 8 => {\n")
once(root/"crates/singlefs-harness/src/history.rs",
     "        PublishError::AllocationRecordsExceedOneNode { .. } => \"AllocationRecordsExceedOneNode\",",
     "        PublishError::C381WriterStopped => \"C381WriterStopped\",\n        PublishError::AllocationRecordsExceedOneNode { .. } => \"AllocationRecordsExceedOneNode\",")
once(root/"crates/singlefs-harness/src/model_comparison.rs",
     "        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,",
     "        | PublishError::C381WriterStopped\n        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,")
PY
# ⚠️ `patch_probe.py` 只留档（它是我从 r2 的 common 生成 r3 common 的脚本，`lying_ranges` 那一步是后来单独打的）：
#    复跑请直接用模型目录里的 `c381r3_probe_common.rs`，别再跑它。

# 2) 三张表
cd /tmp/claude-1000/c381-r3-opus/repo
nice -n 19 cargo test --release -p singlefs-harness --test c381r3_landing -- --ignored --nocapture \
  landing_point_of_the_failed_write        # → landing.log（396 段）
nice -n 19 cargo test --release -p singlefs-harness --test c381r3_landing -- --ignored --nocapture \
  bad_sector_under_three_probes            # → badsector3.log
nice -n 19 cargo test --release -p singlefs-harness --test c381r3_landing -- --ignored --nocapture \
  lying_range_on_fixed_structures          # → lyingrange.log

# 3) 决定性的那一段（第四节的展品）。这一串我在另一份干净副本上照本节的命令重走过一遍，
#    输出与 exhibit-wrapped18.log 的乙a 那一段逐字相同（抹掉根记录(实例 1, txg 3)、回退 REFUSED）。
C381_PREFIX=wrapped C381_ARM=7 C381_FAULTS=18:transient C381_PROBE_MODE=failed C381_ROLLBACK=1:3 \
  nice -n 19 cargo test --release -p singlefs-harness --test c381r3_single -- --nocapture
```

三条臂**只差探针写的落点**，切换、N_switch、转只读、重发逐字相同（同一个 `failure_table_driver`）：

| 臂号 | 名字 | 探针写往哪写 |
|---|---|---|
| 5 | 戊-A | 目标设备的**固定落点**（条款今天的字面；用户第 1 题定的「地址空间表登记一行」就是给它的） |
| 7 | 戊-乙a | **这次失败的那个落点**，写探针图样 0xC3 |
| 8 | 戊-乙b | **这次失败的那个落点**，把那次写的**原样字节**再写一遍 |

## 一、各格判定一览

| 格 | 判 | 一句话 | 凭什么 |
|---|---|---|---|
| K3-1 定义域 | **打中** | 用户第 3 题点名的 8 个失败点里 **2 格（两道屏障）没有落点**，加上已定项 14 自己拆的分配失败三支，**一共五格填不出来**；根槽 FUA 那一格**有落点也观测不到** | `block_device.rs:107` 的 `barrier()` 不带偏移；`:42` `InputOutput(io::Error)` 不带落点；`transaction.rs:983` 往上传的也只有它。实测：6/6 段屏障失败上落点为 None（`landing.log`） |
| K3-2 毁数据 | **打中** | 一段可达历史：探针写抹掉**24 代之前那条仍在回退候选集里的根记录**，管理员回退由 `ok` 变 `RollbackTargetNotACandidate`；对照臂（固定落点）在同一段历史上一个字节都没多写 | `exhibit-wrapped18.log`；`landing.log` 里 126 段注入故障的历史：戊-A 抹掉 0 段、乙a 16 段、乙b 7 段 |
| K3-3 两种读法 | **打中** | 「原样重写」那种读法（乙b）把**失败的那次根槽写补做成功了一次**：根落了盘、调用方收到的是错误、失败表照样走切换，切换的所选根从 txg 26 变成 txg 27 | `exhibit-wrapped18.log` 三段对照 |
| K3-4 分不出的故障类 | **打中（而且是最常见那一类）** | 「写落了盘、设备照样报错」——今天 `ForceUnitAccess` = `write_all_at` 之后 `sync_data`，**刷失败时前半已经落了**，就是这一形。乙与固定落点在这一类上**逐字同判**（都判瞬时、都切换一次），乙a 还多抹掉刚落盘的那条根 | `landing.log` 的 `写#18(Lying,根环区域)` 三行 |
| K3-5 乙修掉了什么 | **乙这一格赢**（如实报） | r2 K1 打中三那一类（落点持续坏、设备整体可写）：24 个场景里戊-A 合计做了 **46 次**白切换，乙 **0 次**、一次探针就判持续 | `badsector3.log` + `lyingrange.log` 按脚本数 |
| K3-5 的代价 | **打中** | 同一批里 **4 个场景**：戊-A 切换之后重发成功、池仍可写，乙 一次探针就把整次挂载转只读 | 同上 |
| K4-a 定案 1 × 候选乙 | **打中（互相矛盾）** | 定案 1 花一个格式常量登记一个不可分配的落点，为的就是**不让探针写打进权威结构**；候选乙又把探针写导回权威结构——定案 3 列出的 8 个失败点里有 4 格（根槽 FUA、系统配置槽两盘、取号那两次写、切换自己的取号）的落点**就是权威结构本身** | 第七节 |
| K4-b 候选乙 × D2 | **打中（一处定案要改两份决策）** | `02-RAID条带策略.md:225` 逐字写着可写设备数的判别子「与 D23 已定项 14 的两支判别子**同一个动作**」。候选乙一改，这句话当场为假；而 D2 那一支要判的是**没失败过的那些盘**，「这次失败的那个落点」对它们没有定义 | 第七节 |
| K4-c 「是改条款不是改实现」 | **打中** | 候选乙的理由里那句站不住：今天**落点传不到失败表那一层**（`BlockDeviceError::InputOutput` 与 `PublishError::BlockDevice` 都不带偏移），要落地得改 core 的错误类型、再给它找个挂载期的地方住（`Mounted` 只有三个字段） | 第七节 |
| K4-d 候选甲要不要改 | **要改射程，不是要改结论** | 定案 3 落地之后，失败表判「切换」的那几格里调用方**根本不会收到错误**（切换重发成功，实测 `重发在飞 checkpoint ok(txg 30)`）；候选甲那句语义只在「转只读」与「重做失败」那几格上生效 | 第七节 |

## 二、装置：拼在哪、与入库装置差在哪

**工作区今天编不过。** 开工快照 `research/prompts/c381-r3-start-snapshot.sha256` 我核过，17 个文件全部 OK：

```
$ sha256sum -c research/prompts/c381-r3-start-snapshot.sha256
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/block_device.rs: OK
.claude/kb/decisions/23-journal的角色与格式.md: OK
.claude/kb/decisions/16-发布语义.md: OK
.claude/kb/decisions/28-挂载期承诺量.md: OK
.claude/kb/layout/01-first-txn.md: OK
research/prompts/c381-r1-main-verification.md: OK
research/prompts/c381-r2-main-verification.md: OK
research/prompts/c381-r2-opus-output.md: OK
research/prompts/c381-r2-sonnet-output.md: OK
research/prompts/c381-r2-verifier-checks.md: OK
research/prompts/_c381-r3-body.md: OK
research/prompts/_c381-r3-checklist.md: OK
research/prompts/_c381-r3-appendix.md: OK
research/prompts/_c381-r3-background.md: OK
```

但**快照没覆盖到的文件**里，另一个会话正把 `SystemConfiguration` 拆成子结构体，拆到一半：把工作区整个拷成副本之后

```
$ cargo check --release -p singlefs-core
error[E0609]: no field `journal_instance` on type `SystemConfiguration`
   --> crates/singlefs-core/src/transaction.rs:323:58
…
error: could not compile `singlefs-core` (lib) due to 60 previous errors
```

⇒ 按派发提示「如实报、别修别回退」，我**没有动工作区**，把副本的 `crates/` 换成 `git archive HEAD`（`0ec0be2`）那一版，副本从此编得过。
后果写清楚：

- 副本里的标识符是改名**之前**的（`SystemConfiguration`、`RotateSystemConfigurationSlots`、`choose_system_configuration`）。正文第四节点名的那几件事实不随改名变：
  我在**工作区**（不是副本）上逐条现查过，行号见第三、七节。
- HEAD 与工作区在我用到的四个文件上的差额：`transaction.rs` 22/22 行、`mount.rs` 43/43 行、`recovery.rs` 31/31 行（都是改名的同增同删），
  `block_device.rs` 131/11 行（是另一件事：镜像文件排他新建，`create_image_file_exclusively`；与探针写、屏障、FUA 无关，
  `ForceUnitAccess` 与 `barrier()` 两边都还是 `file.sync_data()`）。

**装置从哪来**：c381-r2 攻方腿的 `c381r2_probe_common.rs`（`research/prompts/c381-r2-opus-model/`，1083 行）整份拿来，加三样：

1. `Control.last_failed_landing`：每一次**报错的写**记下（盘, 偏移, 长度）；**屏障报错时置 `None`**——`barrier()` 不带偏移（这是 K3-1 的装置形态）。
2. 三种探针读法（`ProbeMode::Fixed` / `FailedLanding` / `FailedLandingRetry`），别的一个字没改。
3. `disk_census()`：探针写**前后各数一次**盘上还读得出的权威结构——每条自证过的根记录（`recovery::readable_roots`）、
   每份自证过的系统配置槽（`recovery::verified_system_configuration_slots`）。差集就是这一次探针写抹掉的东西。
   另加一个收尾动作：按 (实例, txg) 试一次 `mount_rollback`（在镜像副本上做，不污染这段历史）。

**新前缀 `wrapped`**：`base` + 22 次空发布 ⇒ C 是 txg 27。根环 R × S = 3 × 8 = 24 个槽，
所以 txg 27 的根槽（区域 0 槽 1，盘 0 偏移 1052672）里此刻装着 **txg 3 那条根**——第一个事务那一版，仍在回退候选集里。
r2 的五段前缀一段都没绕过根环一圈，这一格因此在 r2 里看不见。

**装置的近似**（与 r2 同，照抄它的限度）：实例切换用「进程内可写重开」近似；用户动作在这几张表里**一个都没排**（`actions: []`），
只固定前缀与故障——按 `.claude/rules/three-way-inference.md`「攻方腿的装置把用户动作写死时，它报的「分辨臂」可能是装置造出来的」，
我这一轮的分辨不靠用户动作：三条臂在**同一段历史、同一个故障、同一串后续步骤**上跑，差的只有探针写往哪写。

## 三、K3-1：用户第 3 题定的那张逐点表，候选乙有五格填不出来，还有一格填得出、观测不到

用户 2026-09-21 第 3 题定的是「逐个失败点列一张表」，主 agent 在 `c381-r2-main-verification.md:74` 写明了要逐个写明哪些点
（整行抄）：

> | 3 | 失败表的射程 | **逐个失败点列一张表** | 失败表要逐个写明：单元写、第一道屏障、记录写两盘、第二道屏障、根槽 FUA、系统配置槽两盘、取号那两次写、切换自己的取号与写行，各落不落在表里 |

候选乙要给这张表**多加一列**：「这次失败的那个落点是哪一个」。逐格填出来是这样（落点那一列按 `crates/` 今天的代码，
「毁什么」那一列是第四节实测出来的）：

| # | 失败点 | 候选乙的落点 | 填得出吗 | 写进去毁什么 |
|---|---|---|---|---|
| 1 | 单元写（每盘一次） | 那个单元的槽 | 填得出 | 回滚之后那个槽是空闲的，实测 0 处权威结构被抹 |
| 2 | **第一道屏障** | **没有** | **填不出** | — |
| 3 | 记录写两盘 | 那条记录的环内槽 | 填得出 | 这几段历史里那一槽全零；绕过一圈之后装的是水位之下的旧记录 |
| 4 | **第二道屏障** | **没有** | **填不出** | — |
| 5 | 根槽 FUA | 那个根槽 | **填得出、但观测不到**（见下） | **一条自证过的根记录**（实测） |
| 6 | 系统配置槽两盘 | 那一份系统配置槽 | 填得出 | **一份自证过的系统配置**（实测） |
| 7 | 取号那两次写 | 系统配置槽（全或无写每一份） | 填得出 | 同上，而且取号是全或无 |
| 8 | 切换自己的取号与写行 | 取号 = 系统配置槽；写行 = 实例表链的新片 | 填得出 | 取号那一半同上 |
| — | 分配失败三支（已定项 14 自己拆的） | **没有**（一个写都没发出去） | **填不出** | — |

⇒ **填不出的是 2 格（两道屏障）加分配失败三支，一共五格。**

**为什么屏障那两格填不出**——不是没想清楚，是接口上就没有那个量（行号 2026-09-21 在工作区现查）：

```
$ grep -nF 'fn barrier(&mut self) -> Result<(), BlockDeviceError>;' crates/singlefs-core/src/block_device.rs
107:    fn barrier(&mut self) -> Result<(), BlockDeviceError>;
```

`barrier()` 不带偏移、不带长度，刷的是整块盘；`transaction.rs` 的 `CommitStep::Barrier` 也只是对每块盘调一次它。
装置里我把这一格实现成「落点为 `None`，按最宽松的读法退回固定落点」，并记一笔。实测（`landing.log`）：

```
$ grep -cP '^\w+\t屏障#' landing.log
18
```

18 段 = 3 段前缀 × 2 道屏障 × 3 条臂；乙a、乙b 各 6 段，**6/6 段的落点都是 `None`**：

```
base	屏障#0	戊-乙a（探针写这次失败的那个落点）	干净	切换=1 只读=false 探针=1 没定义=1	抹掉=[]	落点=["落点没有定义（失败发生在屏障上，barrier() 不带偏移）→ 装置按最宽松的读法退回固定落点", …]
```

⇒ **候选乙在这五格上必须退回固定落点**，也就是说它不能替掉固定落点，只能在固定落点之上再加一支。
而候选乙今天的措辞（正文第二节，整句抄）是：

> **候选乙（第 4 题）：探针写改成写「这次失败的那个落点」，不是固定落点。**

「不是固定落点」这半句在这五格上取不到真。

**根槽 FUA 那一格更麻烦：落点存在，但失败表这一层看不到它。** 今天的「根槽 FUA」是写完再刷（2026-09-20 用户定案维持），
两步共用一个错误类型（工作区现查）：

```
$ awk 'NR>=239 && NR<=249' crates/singlefs-core/src/block_device.rs
        self.check_request(offset, length_of(bytes))?;
        self.file
            .write_all_at(bytes, offset.0)
            .map_err(BlockDeviceError::InputOutput)?;
        match durability {
            WriteDurability::Plain => Ok(()),
            WriteDurability::ForceUnitAccess => {
                self.file.sync_data().map_err(BlockDeviceError::InputOutput)
            }
        }
    }
```

- `write_all_at` 失败与 `sync_data` 失败**返回同一种错误**（`InputOutput`），调用方分不出是哪一半失败的；
- `sync_data` 失败时**前半已经落了盘**——这正是第六节那一类故障；
- 而 `sync_data` 报的是整块盘的回写错，**它可能根本不是这个偏移上的事**（前面某一次写的回写错在这里才浮上来）。

⇒ 「这次失败的那个落点」在根槽 FUA 这一格上，**字面存在、语义不对**：候选乙会拿一个刷失败的错误去认定一个写成功了的落点，
然后往那个落点上写探针图样。第四节量到的就是这一格的后果。

## 四、K3-2：一段可达历史——探针写抹掉一条仍在回退候选集里的根记录，管理员回退由 ok 变 REFUSED

**条款一侧**（`.claude/kb/decisions/23-journal的角色与格式.md:363`，摘管回退候选集那一句；整段在背景材料附录）：

> 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（D16（发布语义） 已定项 1）的根

根槽还不镜像（`.claude/kb/decisions/16-发布语义.md:173`，整句）：

> - **「fsync 等根槽持久之后才返回」的射程**：返回之后这一代只由那一个根槽罩着（根槽不镜像）。

⇒ 一个根槽被写坏，就是**这一代永远没了**，没有第二份。

**那段历史**（前缀 `wrapped` = `base` + 22 次空发布，C 是 txg 27；C 的第 18 次写 = 根槽 FUA、盘 0 偏移 1052672 = 区域 0 槽 1；
故障 = 瞬时错，写不落盘；之后**没有任何用户动作**、不断电）。txg 27 与 txg 3 落在同一个槽（`(txg div 3) mod 8`，R × S = 24）。
三条臂逐字对照（`exhibit-wrapped18.log` 原样，各取 steps 与收尾两行）：

```
[戊-A] 分类=红
  steps=["C err(BlockDevice)", "探针写(盘 0, Fixed) 过", "切换#1（近似：进程内可写重开）ok(所选根 txg 26，新实例 2，现行 txg 29)", "重发在飞 checkpoint ok(txg 30)"]
  探针写抹掉=[]
  回退试挂=ok(回退到 txg 30，新实例 3)

[戊-乙a（探针写这次失败的那个落点）] 分类=红+探针写抹掉权威结构
  steps=["C err(BlockDevice)", "探针写(盘 0, FailedLanding) 过，抹掉 [\"根记录(实例 1, txg 3)\"]", "切换#1（近似：进程内可写重开）ok(所选根 txg 26，新实例 2，现行 txg 29)", "重发在飞 checkpoint ok(txg 30)"]
  探针落点=["探针落点 盘0 偏移1052672 长512（根环区域，盖掉之前那一段非零）"]
  探针写抹掉=["探针写抹掉了 根记录(实例 1, txg 3)"]
  回退试挂=REFUSED RollbackTargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_t

[戊-乙b（把那次写的原样字节再写一遍）] 分类=红+探针写抹掉权威结构
  steps=["C err(BlockDevice)", "探针写(盘 0, FailedLandingRetry) 过，抹掉 [\"根记录(实例 1, txg 3)\"]", "切换#1（近似：进程内可写重开）ok(所选根 txg 27，新实例 2，现行 txg 29)", "重发在飞 checkpoint ok(txg 30)"]
  探针写抹掉=["探针写抹掉了 根记录(实例 1, txg 3)"]
  回退试挂=REFUSED RollbackTargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_t
```

**四句话**（按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：

1. **分不分辨臂**：分辨。三条臂同一段历史、同一个故障、同一串后续步骤，只有探针写的落点不同；
   固定落点那条臂 `抹掉=[]`、回退 `ok`，两条乙臂各抹掉一条根、回退 `REFUSED`。
2. **被判的系统当时看不看得到**：看得到——`readable_roots` 是恢复路径自己的函数，回退挂载当场就报 `RollbackTargetNotACandidate`。
3. **满足判据字面的哪一个分句**：正文第七节 K3 那一格的第一句「写『失败的那个落点』毁掉别的数据的一段历史」。
   被毁的是 **txg 3 那条根**——它是**24 代之前**的一版，与这次失败的发布毫无关系，而且按 `:363` 仍在回退候选集里。
4. **前条款给的改法在这一格还中不中**：D23 已定项 14 自己的兜底改不掉它——`:370` 那句
   「根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`…）」意思是**重发去的是另一个槽**，
   被探针抹掉的那个槽在这次挂载里再也不会被重写。要等 txg 再绕一圈（24 次发布）才会有新根盖回去。

**为什么固定落点那条臂一点事都没有**：失败的那次写**没落盘**（瞬时错），按定义盘上那一槽还是 txg 3 那条根；
条款里的探针写打的是另一个地方。候选乙把探针打回了那个槽。

**这不是孤例**（`landing.log`，21 个写下标 × 2 种故障 × 3 段前缀 = 126 段注入故障的历史，每条臂各跑一遍）：

```
$ for a in 戊-A 戊-乙a 戊-乙b; do printf '%s 段数=%s 抹掉段数=%s 没定义段数=%s\n' "$a" \
    "$(grep -P "^(base|wrapped|remounted)\t" landing.log | grep -c -P "\t$a")" \
    "$(grep -P "^(base|wrapped|remounted)\t" landing.log | grep -P "\t$a" | grep -c '抹掉=\["')" \
    "$(grep -P "^(base|wrapped|remounted)\t" landing.log | grep -P "\t$a" | grep -c '没定义=1')"; done
戊-A 段数=132 抹掉段数=0 没定义段数=0
戊-乙a 段数=132 抹掉段数=16 没定义段数=6
戊-乙b 段数=132 抹掉段数=7 没定义段数=6
```

（132 = 126 段注入写故障 + 6 段屏障故障。）抹掉的东西按类型数：

```
$ grep -oP '探针写抹掉了 [^"]*' landing.log | sed 's/(.*//' | sort | uniq -c
      5 探针写抹掉了 根记录
     18 探针写抹掉了 系统配置
```

落在哪几个失败点上，一条不落（`landing.log`，只有这几段抹掉了东西）：

| 前缀 | 失败点 | 乙a 抹掉 | 乙b 抹掉 |
|---|---|---|---|
| base | 写#18(Lying, 根槽) | 根记录(实例 1, txg 5) | —（原样重写，写回去的是同一条根） |
| base | 写#19/#20(Transient, 系统配置槽) | 系统配置(盘 0/1, 世代 5) | 系统配置(盘 0/1, 世代 5) |
| base | 写#19/#20(Lying, 系统配置槽) | 系统配置(盘 0/1, 世代 7) | — |
| wrapped | 写#18(**Transient**, 根槽) | **根记录(实例 1, txg 3)** | **根记录(实例 1, txg 3)** |
| wrapped | 写#18(Lying, 根槽) | 根记录(实例 1, txg 27) | — |
| wrapped | 写#19/#20(Transient / Lying, 系统配置槽) | 系统配置(盘 0/1, 世代 27 / 29) | 系统配置(盘 0/1, 世代 27) |
| remounted | 写#18(Lying, 根槽) | 根记录(实例 2, txg 8) | — |
| remounted | 写#19/#20(Transient / Lying, 系统配置槽) | 系统配置(盘 0/1, 世代 9 / 11) | 系统配置(盘 0/1, 世代 9) |

⇒ 规律很干净：**落点落在单元区或 journal 环时乙不毁东西；落在固定结构（根环、系统配置槽）上时必毁一件。**
而固定结构正是失败表最关心的那几格——C381 这一整条线就是从「系统配置槽那一步失败时根已落盘」起头的。

**什么现象会推翻它**：主 agent 在入库装置上重做这一段，回退挂载在三条臂上都 `ok`（那样我的 `disk_census` 或
`mount_rollback` 的近似写错了）；或者用户定案「探针写只许打进第 1 题登记的那一行」——那就等于收回候选乙。

## 五、K3-3：候选乙有两种读法，「原样重写」那种把失败的那次写补做了一次

候选乙只写了「写这次失败的那个落点」，**没写写什么字节**。两种读法都成立：

| 读法 | 是什么 | 毛病 |
|---|---|---|
| 乙a 写探针图样 | 真的是一次探针写 | 第四节：落在固定结构上就毁一件 |
| 乙b 把那次写的原样字节再写一遍 | **它不是探针写，是重试** | 见下；而且今天做不到：失败之后没有人保存这次发布要写什么 |

**乙b 把失败的那次发布补做了一半**（第四节同一段历史，`exhibit-wrapped18.log` 的第三段）：

- 乙b 的探针写成功 ⇒ **C 的根记录（实例 1, txg 27）落了盘**：收尾普查里多出一条
  `根记录(实例 1, txg 27)`（戊-A 与乙a 的普查里都没有这一条）；
- 失败表同时判「瞬时 ⇒ 走实例切换」，切换的所选根从 **txg 26 变成 txg 27**
  （`切换#1（近似：进程内可写重开）ok(所选根 txg 27，…)`，另两条臂是 `所选根 txg 26`）；
- 而调用方收到的是 `C err(BlockDevice)`。

⇒ **乙b 自己造出了 C381 的题面**（「根已落盘之后发布失败」）：判别子把一次失败的根槽写补成功了，
盘上这次发布成立，调用方手里是错误。C381 这一整条线要修的就是这个形态。

**乙b 今天还做不到**：要「把那次写的原样字节再写一遍」，得有人留着那些字节。
c381-r2 第七节已经坐实「`publish_version` 不保存 plan，失败之后调用方手里只有错误」，
我这一轮在工作区复核过承载它的那个对象没变：

```
$ grep -nF 'pub struct Mounted {' crates/singlefs-core/src/mount.rs
190:pub struct Mounted {
```

`Mounted` 还是 `output` / `allocator` / `current` 三个字段。我的装置是在**设备层**（`Device::write_at`）把字节抄下来的，
实现里没有这条路。

## 六、K3-4：它分不出的那一类，是今天最可能发生的那一类

**那一类**：写落了盘、设备照样报错。今天的「根槽 FUA」= `write_all_at` 之后 `sync_data`（第三节的原样代码），
**刷失败时前半已经落盘**，于是这一类不是假想：它是这段代码的正常行为。c381-r1 坐实甲的那一格（「根槽 FUA 报错而根已落盘」）也是它。

`landing.log` 里 `base` 前缀、C 的第 18 次写（根槽）注入 `Lying`，三条臂：

```
base	写#18(Lying,根环区域)	戊-A	干净	切换=1 只读=false 探针=1 没定义=0	抹掉=[]	落点=[]	remount=ok(chosen 8)	after红=[]	内容=同一份	回退=ok(回退到 txg 8，新实例 3)
base	写#18(Lying,根环区域)	戊-乙a（…）	红+探针写抹掉权威结构	切换=1 只读=false 探针=1 没定义=0	抹掉=["探针写抹掉了 根记录(实例 1, txg 5)"]	落点=["探针落点 盘0 偏移7344128 长512（根环区域，盖掉之前那一段非零）"]	remount=ok(chosen 8)	after红=["I-3.1: Violated(…)"]	内容=同一份	回退=ok(回退到 txg 8，新实例 3)
base	写#18(Lying,根环区域)	戊-乙b（…）	干净	切换=1 只读=false 探针=1 没定义=0	抹掉=[]	落点=["探针落点 盘0 偏移7344128 长512（根环区域，盖掉之前那一段非零）"]	remount=ok(chosen 8)	after红=[]	内容=同一份	回退=ok(回退到 txg 8，新实例 3)
```

**判定一格不差**：三条臂都判「瞬时」（`切换=1 只读=false`）。候选乙在这一类上**一点判别力都没多出来**——
落点是好的（字节明明落进去了），探针写当然过。而乙a 在这一类上：

- 多抹掉一条刚落盘的根记录（`根记录(实例 1, txg 5)`）；
- 把一段 checker 全绿的历史变成 I-3.1 红（对照臂戊-A 在同一段历史上 `after红=[]`）。

⇒ **这一类是候选乙的净亏**：判别力 0 增，破坏 +1。而按第三节，它还正是「根槽 FUA 失败」最可能的物理成因。

## 六之二、如实报：候选乙确实修掉了 r2 那一格，代价在这里

r2 K1 打中三那一类（某个落点持续写不进去、设备整体还活着）候选乙**确实修掉了**。
`badsector3.log` + `lyingrange.log` 一共 24 个持续故障场景，三条臂逐场景对照（脚本数出来的）：

```
场景数 24
戊-A 切换次数合计 46
乙a 切换次数合计 0
乙b 切换次数合计 0
戊-A 转只读段 17
乙a 转只读段 21
戊-A 池仍可写而乙a 转只读 4
乙a 抹掉权威结构段 6
```

- **赢的那一格**：戊-A 在这 24 个场景里合计做了 **46 次**切换（每次都白做，r2 第五节那条账），乙 **0 次**——
  第一次探针就落在坏的那一段上，当场判持续、转只读。
- **输的那一格**：同一批里有 **4 个场景**，戊-A 切换之后重发**成功**、池仍可写，而乙一次探针就把整次挂载转只读：
  `wrapped 坏扇区@写#0(单元区)`、`base / wrapped / remounted 坏扇区@写#18(根环区域)`。
  机理：切换是「挂载内一次恢复，重建态里失败那次的分配不存在」，重发时分配器**可以挑到别的落点**；
  一个坏扇区不等于这块盘不能写。候选乙把「这个落点坏」直接读成「持续失败 ⇒ 转只读到下次挂载」。
- 另外 6 个场景里乙a 照样抹掉了权威结构（落点落在根槽或系统配置槽上的那几个）。

⇒ 这一格**不拿来判臂**：它同时是候选乙的战果和它的代价，两边都要写进交用户的表里。

## 七、K4：两条定案落地之后，候选甲、乙要改什么；哪两条对撞

### 7.1 定案 1（探针落点进地址空间表）与候选乙：**互相顶**

用户第 1 题定的（`c381-r2-main-verification.md:72`，整行抄）：

> | 1 | 探针写的落点要不要进盘上格式 | **进：地址空间表登记一行** | D23（journal 的角色与格式） 已定项 14 的失败表要写明探针落点住哪；`.claude/kb/layout/01-first-txn.md` 的地址空间表加一行；一个新格式常量（门禁 27 号绑 `format-const`）；走 D15（格式冻结政策） 的判定表；字节表跟着改 |

这一行要付的是：一个格式常量、mkfs 写它、挂载比对它、D15 判一档、字节表跟着改。**它买的是什么？**
买的是一个**不可分配、不被任何结构引用**的落点——因为今天没有这样的地方：地址空间表现查是 7 行表体
（`.claude/kb/layout/01-first-txn.md` 第 40–46 行：系统配置槽 0、系统配置槽 1、根环区域、区域内根槽、区域归属、journal 环、单元区），
而单元区一直铺到盘尾：

```
$ grep -nF 'let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;' crates/singlefs-core/src/allocator.rs
212:        let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;
```

⇒ **定案 1 的全部意义，就是让探针写有一个「写坏了也不心疼」的地方。**
候选乙把探针写导回**权威结构本身**：按第三节那张逐点表，定案 3 点名的 8 个失败点里有 4 格
（根槽 FUA、系统配置槽两盘、取号那两次写、切换自己的取号）的落点就是根槽或系统配置槽，
第四节实测这 4 格每一格都真的抹掉了一件权威结构。

**两条定案与候选乙合起来读，是自相矛盾的**：一边花一个格式常量买一个专用落点，另一边规定探针写不往那儿写。
⇒ 两条只能留一条：要么留定案 1、撤候选乙；要么采候选乙、而定案 1 那一行只剩「屏障与分配失败那五格的退路」这一个用途
（那它还是得登记，但理由要改写，D15 判定表那一档也按新理由重走）。

### 7.2 候选乙与 D2：**一处定案要改两份决策，而且改不掉的那一半在 D2**

`.claude/kb/decisions/02-RAID条带策略.md:225`（已定项 13「降级期间只读」的运行期那一格，整行抄）：

> | 运行期 | 可写设备数掉到 w 的下限以下、**或独占打开成功的设备数掉到过半以下**即转只读。**「可写设备数」的口径**：一台设备算不可写，当且仅当对它的固定落点做一次探针写失败（与 D23（journal 的角色与格式） 已定项 14 的两支判别子同一个动作）——不按失败次数、不按失败时长算，那两样在故障发生那一刻都判不出来 |

三件事：

1. 括注「**与 D23 已定项 14 的两支判别子同一个动作**」在候选乙之下**当场为假**。改 D23 就得同时改 D2，
   而正文第二节说候选乙「是改条款不是改实现」时只算了 D23 那四个字。
2. D2 那一支要判的是**可写设备数**——它要对**每一块盘**判，包括这次一个字节都没失败过的那几块。
   「这次失败的那个落点」对它们**没有定义**。⇒ 固定落点在 D2 这一支上删不掉。
3. 反过来，若两处保持「同一个动作」而动作改成候选乙，那么**一个坏扇区就会把整块盘判成不可写**，
   可写设备数当场掉到 w 的下限以下 ⇒ 整池只读。第六之二节量到的 4 个场景（戊-A 池仍可写而乙转只读）是同一件事的小号版本。

### 7.3 候选乙的理由里那句「是改条款不是改实现」站不住

正文第二节（整句抄）：

> ⇒ 代价是把条款里「目标设备的固定落点」这四个字改掉，是改条款不是改实现。

**今天落点传不到失败表那一层**（工作区现查）：

```
$ grep -nF 'InputOutput(io::Error),' crates/singlefs-core/src/block_device.rs
42:    InputOutput(io::Error),
$ grep -nF 'BlockDevice(BlockDeviceError),' crates/singlefs-core/src/transaction.rs
983:    BlockDevice(BlockDeviceError),
```

设备层的 I/O 错只装一个 `io::Error`，往上传的 `PublishError::BlockDevice` 也只装它——**没有偏移、没有长度、没有盘号**。
（`OutOfRange` 与 `Unaligned` 两个成员带偏移，但那两个是调用方算错落点，不是设备故障。）
⇒ 候选乙要落地，至少要：① 给 `BlockDeviceError::InputOutput` 加落点（或另加一个带落点的成员），
两处 harness 的穷尽匹配跟着改；② 找个地方存「上一次失败的落点」活到失败表跑完——而 `Mounted`（`mount.rs:190`）只有三个字段，
c381-r2 第七节那条「今天没有活到一次挂载那么久的对象」在这里又要用一次；③ 在根槽 FUA 那一格给「写成功而刷失败」定一个读法（第三节）。
我的副本是在装置的 `Device::write_at` 里记的落点——**那不是实现里有的路**。

### 7.4 定案 1 与定案 3 之间：我没找到直接矛盾

逐条对过：定案 3 是「把失败表的射程逐个失败点列出来」，定案 1 是「给探针写登记一个落点」，两者的量不同、不共用字段。
定案 3 的表会**多出五格要填**（第三节），但那是候选乙带来的，不是定案 1 带来的。
⇒ **问三的第二问我的回答是：两条定案之间不矛盾，矛盾在「定案 1 + 候选乙」与「定案 3 + 候选乙」。**

### 7.5 候选甲要不要改：要改射程，不改结论

定案 3 落地之后，失败表判「走实例切换」的那几格里，**调用方根本不会收到错误**——切换重发成功时发布返回 Ok
（实测 `重发在飞 checkpoint ok(txg 30)`，第四节三条臂都是）。
⇒ 候选甲那句「fsync 报错之后数据可以出现也可以不出现」的射程，落地后只剩失败表判**转只读**的那几格、
以及切换之后重做**又失败**的那几格。按 `.claude/rules/three-way-inference.md`「引一条已定项之前，把它那一节从头读到下一个标题为止」的同一条理由，
候选甲写进条款时要连这个射程一起写，否则它字面上覆盖了一批实际上不会报错的历史。
（K1、K2 归正推腿，这一节只答「定案落地之后要不要改」这一问。）

## 八、我自己提的改法：**只在我的副本上量过、被攻过零轮**

按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算『没被攻过』」，
下面三条一条都没被攻过，交用户时要标「零轮」。

**改法 A（最小）**：候选乙收成「**只读探针**」——失败之后**读**那个落点（或读一段），读得出且与要写的内容不符 ⇒ 这次写没落；
读不出 ⇒ 落点坏。判别力从「写得进去吗」换成「读得出吗」，**不写一个字节**。
代价：读成功不等于写得进去（判别力弱于写探针），要与固定落点的写探针并用。

**改法 B**：候选乙收成「**固定落点 + 失败落点的只读复核**」两步：第一步仍打定案 1 登记的那一行（决定瞬时/持续），
第二步只读失败的那个落点，把结论细化成「这个落点坏」。两步都不碰权威结构。

**改法 C**：候选乙只在**落点落在单元区**时启用（第四节实测：单元区落点抹掉 0 件权威结构），
落在固定结构上时退回固定落点。条款要写成一张按区段分的表，而区段的边界正好是定案 1 要登记的那张地址空间表。

**三个改法各修哪一格**（「量过」= 副本上跑出来的原样输出；「推的」= 按代码推、没实现没跑）：

| 格 | 候选乙（乙a） | 改法 A | 改法 B | 改法 C |
|---|---|---|---|---|
| K3-1 屏障那两格没有落点 | 中（**量过**：6/6 段落点为 None） | 还中（**推的**：读也要偏移） | 不中（**推的**：第一步不要落点） | 不中（**推的**：退回固定落点） |
| K3-2 抹掉权威结构 | 中（**量过**：126 段里 16 段） | 不中（**推的**：不写） | 不中（**推的**：不写） | 不中（**推的**：单元区那几格量过是 0 件） |
| K3-4 写落盘而报错那一类分不出 | 中（**量过**：三条臂逐字同判） | 部分修（**推的**：读回来的内容与要写的一致 ⇒ 判得出「其实落了」） | 部分修（**推的**：同左） | 还中（**推的**：这一格的落点是根槽，退回固定落点 = 今天的行为） |
| K3-5 落点持续坏那一类（r2 K1 打中三） | 修掉（**量过**：46 → 0 次白切换） | 不修（**推的**：读得出不等于写得进） | 修掉（**推的**：第二步读得出/读不出分得开落点级） | 单元区那几格修掉、固定结构那几格不修（**推的**） |
| K4-b 与 D2「同一个动作」 | 中（现查条款） | 不中（**推的**：D2 那一支仍用固定落点的写探针） | 不中（**推的**：同左） | 中（**推的**：动作按区段分叉，D2 括注仍要改） |

⇒ 我自己的倾向是**改法 B**，但它没被任何人攻过；而且它把第 4 题的答案从「② 探针写改成写失败的那个落点」
挪到了一个**新候选**上——按「第三轮之后停，不开第四轮」，这算这一轮里新冒出来的零轮形态，只能进交用户表标「零轮」。

## 九、没打中的形状（试过、没成）

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 探针写落在**单元区**时毁掉别的数据 | 21 个写下标里 16 个是单元区写 × 2 种故障 × 3 段前缀 = 96 段 | **没打中**：0 段抹掉权威结构。机理：失败之后分配器整个换回（`transaction.rs` 的 `if outcome.is_err() { *allocator = allocator_before_this_publish; }`），那个槽回到空闲；切换重发又挑回同一批落点、原样盖掉探针图样 |
| 探针写落在 **journal 环**时毁掉还要用的记录 | 2 段（记录写两盘）× 2 种故障 × 3 段前缀 = 12 段 | **没打中**：这几段历史里那一槽本来全零（环一圈 196608 槽，这几段历史连一圈的零头都没走完）。绕过一圈之后那一槽装的是水位之下的旧记录，按已定项 14 第二条本来就不施加 ⇒ 抹掉它大概率无害（**推的**，没量） |
| 把一块盘的**两份系统配置槽**都抹掉、让下次挂载挂不上 | `lying_ranges` 盖住盘 0 的 0–8192 共 3 段前缀 × 3 条臂 | **没打中**：三条臂都 `REFUSED Acquisition(AcquisitionFailed …)`，**不分辨臂**——取号本来就写不进去，不是探针写造成的。按正文第七节「两条以上的臂在同一格一起中，那一格不拿来判臂」，这一格作废 |
| 让候选乙在**撕裂写**上与固定落点分开 | 没跑 | 没试：r2 已记「根槽宽 512 = 判定宽度 ⇒ 根槽撕不开」，撕裂只在单元与记录上有意义，而那两段按上面两行是安全区 |
| 让 `N_switch` 在候选乙下被烧光 | 24 个持续故障场景 | **没打中，反向**：候选乙一次探针就转只读，**切换 0 次**（第六之二节）。要烧 N_switch 得让探针写恰好时好时坏，我没造出这段历史 |

## 十、这条腿自己的限度

1. **副本的数不进 kb**：全部数字出自 `/tmp/claude-1000/c381-r3-opus/repo`（HEAD `0ec0be2` + 两个补丁），
   主 agent 要在入库装置上重做才能引（`.claude/rules/three-way-inference.md`）。
2. **装置拼在 HEAD 上，不是工作区**：工作区今天编不过（第二节），改名后的标识符与副本不同；
   我引的实现事实（屏障不带偏移、`InputOutput` 不带落点、`Mounted` 三个字段、单元区铺到盘尾）**都是在工作区现查的**，行号见正文。
3. **实例切换是近似**（进程内可写重开），与条款写的切换不是一回事——这是 r2 K1 打中二的结论，我没改它。
   三条臂共用同一个近似，所以臂间的差是探针写造成的；但「切换之后池还能不能写」这一类绝对值带着这个近似的误差。
4. **`disk_census` 只数两种权威结构**（根记录、系统配置槽）。实例表链页、树节点、数据单元被探针写抹掉它看不见——
   也就是说第四节那张表是**下界**，不是全部。
5. **每段历史只跑一次**（确定性装置，无随机种子）：`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」
   管的是模型答复；这里的「一次」是确定性程序的一次执行，重跑逐字相同（命令在第〇节，主 agent 可复核）。
6. **没跑门禁**：`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段（我查过），而且改动全在副本里。
7. **没碰 K1、K2**：候选甲的历史与外部 fsync 语义不在我的格里，第 7.5 节只答「定案落地之后要不要改射程」这一问。
8. **没写 kb、没改 `crates/`**：这一轮只写报告、模型目录与草稿目录。

## 附：这一轮现查过的行号一览（都在工作区，不是副本；命令与原样输出在正文各节）

| 引到的东西 | 文件:行 | 用在哪一节 |
|---|---|---|
| 失败表的两支与探针写 | `.claude/kb/decisions/23-journal的角色与格式.md:365` | 三、七 |
| 回退候选集 | 同上 `:363` | 四 |
| 根槽写失败重发推进一格 | 同上 `:370` | 四 |
| 可写设备数的口径「同一个动作」（已定项 13 的运行期那一格） | `.claude/kb/decisions/02-RAID条带策略.md:225`（`#### 已定项 13` 在 `:218`） | 七 |
| 「返回之后这一代只由那一个根槽罩着（根槽不镜像）」 | `.claude/kb/decisions/16-发布语义.md:173` | 四 |
| 地址空间表表头与 7 行表体 | `.claude/kb/layout/01-first-txn.md:38`（表头）、`:40`–`:46`（表体） | 七 |
| 用户第 1 题定案 / 第 3 题定案 | `research/prompts/c381-r2-main-verification.md:72` / `:74` | 三、七 |
| `barrier()` 不带偏移 | `crates/singlefs-core/src/block_device.rs:107` | 三 |
| `InputOutput(io::Error)` 不带落点 | 同上 `:42` | 三、七 |
| `ForceUnitAccess` = `write_all_at` 之后 `sync_data` | 同上 `:239`–`:249`（另一份实现同形，在 `:421` 起） | 三、六 |
| `PublishError::BlockDevice(BlockDeviceError)` | `crates/singlefs-core/src/transaction.rs:983` | 七 |
| 失败之后整个换回分配器 | 同上 `:1276`–`:1277`（`allocator.clone()` 在 `:1274`） | 九 |
| `Mounted` 三个字段 | `crates/singlefs-core/src/mount.rs:190` | 五、七 |
| 单元区铺到盘尾 | `crates/singlefs-core/src/allocator.rs:212` | 七 |

⚠️ **两处与背景材料正文第四节的行号对不上，如实记**（材料写的是 r2 那一轮的行号，工作区这一轮改名之后行号挪了，内容没变）：

- 正文第四节写「克隆分配器（`:1265`）、`publish_admitted` 返回 `Err` 就整个换回（`:1267-1269`）」；
  **工作区现查是 `:1274` 与 `:1276-1277`**（HEAD `0ec0be2` 里那一行确实还在 `:1265`，所以差额来自工作区没提交的改名）。
- 正文第四节写 `block_device.rs` 的 `ForceUnitAccess`（`:245-246`、`:449-450`）与 `barrier()`（`:251-252`、`:455-456`）：
  **这四处工作区现查逐个对上**（`grep -n 'ForceUnitAccess =>'` 报 245、449；`grep -n 'fn barrier'` 报 107 的 trait 声明与 251、455 两份实现），
  两份实现调的都还是 `file.sync_data()`。⇒ 对不上的只有 `transaction.rs` 那两处（上一条）。
