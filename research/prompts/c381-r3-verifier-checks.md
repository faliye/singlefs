# c381-r3 核查员核对表（2026-09-21）

**这是观测，不是判决**：核对表里的 ✗ 不免除主 agent 对推论本身的逐条现查。

行号一律按工作区现状核（另一会话正在做 `SystemConfiguration` → `SystemConfiguration` 改名，`crates/` 今天编不过；本轮未编译 `crates/` 本体，仅在草稿目录 `/tmp/claude-1000/c381-r3-verifier/repo/`——HEAD `0ec0be2` 的独立副本——里按攻方给的命令复跑）。

## 开头：判别力自证

取 sonnet 报告的引用「`crates/singlefs-core/src/recovery.rs:67` = `impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {`」，在草稿副本 `/tmp/claude-1000/c381-r3-verifier/selftest/recovery.rs` 里把行号加 1（改核 `:68`），用下面第 2 步同一套方法（`awk 'NR==行号'` 取该行、逐字比对引文）核：

```
$ awk 'NR==68{print NR": "$0}' /tmp/claude-1000/c381-r3-verifier/selftest/recovery.rs
68:     fn device_identities(&self) -> Vec<DeviceIdentity> {
```

行 68 是 `fn device_identities(&self) -> Vec<DeviceIdentity> {`，与引文「`impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {`」逐字不同 ⇒ **判 ✗**。核查方法有判别力，往下按同一方法核正式引用。

## 一、Opus 攻方腿（`c381-r3-opus-output.md`，554 行）

### 1.1 模型目录 10 个文件的 sha256（第〇节）

命令：`cd research/prompts/c381-r3-opus-model && sha256sum *`，逐个与报告第〇节列的哈希比对，10 个全部逐字相同：

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

**判 ✓**（10/10）。

### 1.2 复跑：按第〇节命令在草稿副本上重走一遍

草稿：`/tmp/claude-1000/c381-r3-verifier/repo/`（`rsync` 拷主树 → `crates/` 换成 `git archive HEAD`，HEAD 现查确为 `0ec0be2` → `patch_core.py`（改过路径）→ 复制三份 `c381r3_*.rs` → 三处 `once()` 补丁，均按报告断言的「命中且仅命中一次」成立）。

| 复跑目标 | 命令 | 结果 |
|---|---|---|
| `c381-r3-arms-core.diff` 能否从 HEAD 重新推出 | 对 `transaction.rs`/`history.rs`/`model_comparison.rs` 各生成 `diff -u HEAD vs 副本`，拼成一份 | 176 行，`sha256sum` = `da592bf2...`，与模型目录里的文件**逐字节相同** ✓ |
| `landing_point_of_the_failed_write` | `nice -n 19 cargo test --release -p singlefs-harness --test c381r3_landing -- --ignored --nocapture landing_point_of_the_failed_write` | `ok`；提取 `^(base\|wrapped\|remounted)\t` 396 行，与模型 `landing.log` 同段落**逐字节相同**（两边 `sha256sum` = `e68700ef...`）✓ |
| `bad_sector_under_three_probes` | 同上，换测试名 | `ok`，产物见下方计数核对 ✓ |
| `lying_range_on_fixed_structures` | 同上 | `ok`，产物见下方计数核对 ✓ |
| 决定性单段（K3-2 展品） | `C381_PREFIX=wrapped C381_ARM=7 C381_FAULTS=18:transient C381_PROBE_MODE=failed C381_ROLLBACK=1:3 cargo test --release -p singlefs-harness --test c381r3_single -- --nocapture` | `ok`；输出的「戊-乙a」整段（`steps=`、`探针落点=`、`探针写抹掉=`、`回退试挂=`）与模型 `exhibit-wrapped18.log` 的乙a 段**逐字节相同** ✓ |

**判 ✓**（5/5）：报告「我在另一份干净副本上照本节命令重走过一遍，输出与 `exhibit-wrapped18.log` 的乙a 那一段逐字相同」这句坐实。


### 1.3 报告里的段数与计数（K3-1、K3-2、K3-4、K3-5）

全部按报告给的命令、在自己复跑出的日志上重新数一遍（不是抄模型目录里现成的数）：

| 引用 | 结果 | 判 |
|---|---|---|
| 「18 段 = 3 段前缀 × 2 道屏障 × 3 条臂」 | `grep -cP '^\w+\t屏障#' 我的run-landing.log` = 18 | ✓ |
| 「乙a、乙b 各 6 段，6/6 段的落点都是 None」 | `grep -c '落点=\["落点没有定义'` 命中 12 行（乙a 6 + 乙b 6，逐行核对确为各 6）| ✓ |
| 「戊-A 段数=132 抹掉段数=0 没定义段数=0；乙a 132/16/6；乙b 132/7/6」 | 按报告给的 `for a in 戊-A 戊-乙a 戊-乙b; do …` 脚本原样在我的日志上重跑，三行数字逐个相同 | ✓ |
| 「5 探针写抹掉了根记录、18 探针写抹掉了系统配置」 | `grep -oP '探针写抹掉了 [^"]*' … \| sort \| uniq -c` → `5 …根记录` `18 …系统配置` | ✓ |
| K3-2「txg 27」与「24 代之前」 | 复跑单段输出「[前缀 wrapped] … C 是 txg 27」，抹掉的是「根记录(实例 1, txg 3)」；27 − 3 = 24，算术核 | ✓ |
| K3-5「24 个场景；戊-A 切换合计 46，乙a/乙b 合计 0；戊-A 转只读 17 段，乙a 21 段」 | 用报告给的口径（`badsector3.log`+`lyingrange.log` 拼一起、按臂分组求和/计数）在我的两份复跑日志上重算 | ✓（数字与报告逐一相同） |
| K3-5 代价「4 个场景戊-A 池仍可写而乙a 转只读；乙a 抹掉权威结构 6 段」 | 自己写脚本按 3 行一组（A/乙a/乙b）分组判断（`只读=true`/`抹掉=["` 两个字段），24 组里数出 4 与 6 | ✓ |

**判 ✓**（7/7）。

### 1.4 代码 / kb 引用逐条核（工作区现查，`awk 'NR==行号'` 或 `grep -n` 现取）

| 引用 | 核 | 判 |
|---|---|---|
| `block_device.rs:107` `fn barrier(&mut self) -> Result<(), BlockDeviceError>;` | 逐字相同 | ✓ |
| `block_device.rs:42` `InputOutput(io::Error),` | 逐字相同 | ✓ |
| `block_device.rs:239-249`（`write_all_at`→`sync_data` 同错误类型）、`:251`（trait `barrier` 声明处，报告写作另一份实现在 `:107` 附近，本体两份实现在 `:251`/`:455`） | 245/449 两处 `ForceUnitAccess =>`、251/455 两处 `fn barrier`，四处均调 `self.file.sync_data()` | ✓ |
| `transaction.rs:983` `BlockDevice(BlockDeviceError),` | 逐字相同 | ✓ |
| `transaction.rs:1274`（`allocator.clone()`）、`:1276-1277`（换回） | 逐字相同；报告自己也说明与背景材料正文第四节的 `:1265`/`:1267-1269` 不同，是工作区改名挪的行——**这处「对不上」报告自己已如实交代，不算它的差错** | ✓ |
| `allocator.rs:212` `let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;` | 逐字相同 | ✓ |
| `mount.rs:190` `pub struct Mounted {` | 逐字相同 | ✓ |
| `.claude/kb/decisions/23-journal的角色与格式.md:363`（回退候选集定义）、`:365`（失败表两支判别子）、`:370`（重发推进一格） | 三处整句逐字相同 | ✓ |
| `.claude/kb/decisions/16-发布语义.md:173`（「返回之后这一代只由那一个根槽罩着（根槽不镜像）」） | 逐字相同 | ✓ |
| `.claude/kb/layout/01-first-txn.md:38`（表头）、`:40-46`（表体 7 行） | 表头列名、7 行表体逐字相同 | ✓ |
| `.claude/kb/decisions/02-RAID条带策略.md:225` | 主 agent 已核（不重复） | — |

**判 ✓**（10/10，另 1 项主 agent 已核不重复）。


### Opus 腿小计

sha256 10 条 + 复跑 5 条 + 段数计数 7 条 + 代码/kb 引用 10 条（其中 1 条主 agent 已核不重复计）= 31 条。
**核了 31 条，✓ 31 条，✗ 0 条。**

## 二、Sonnet 正推腿（`c381-r3-sonnet-output.md`，111 行）

### 2.1 代码引用逐条核

| 引用 | 核 | 判 |
|---|---|---|
| `transaction.rs:126-191`（`perform`，六步对应 `CommitStep` 五个变体） | 126 行为 `pub fn perform(...)`，191 行为该函数闭合 `}`，中间五个 `match` 分支与描述一致 | ✓ |
| `:137`、`:145`、`:184` 三处 `write_at`/`barrier` 调用点 | 逐字相同 | ✓ |
| `:162-169`（根槽 FUA 的 `write_at` 调用，报告合并写成一处） | 162-169 确为该调用的完整参数列表 | ✓ |
| `:231`（`RotateSystemConfigurationSlots` 内部 `write_at`） | 逐字相同 | ✓ |
| `:1275`（`publish_admitted` 调用）、`:1276-1277`（`if outcome.is_err() { *allocator = ... }`）、`:1240`（注释「准入之后任何一步失败…这次发布没有成立」） | 三处逐字相同，注释整句逐字相同 | ✓ |
| `:98`（`pub devices: &'pool mut [(DeviceIdentity, Device)],`） | 逐字相同 | ✓ |
| `:1246`（`pub fn publish_version`）、`:1581`（`fn publish_admitted`） | 逐字相同 | ✓ |
| `recovery.rs:289-305`（`choose_root`）、`:262-285`（`visit_valid_roots`） | 函数起止行与内容逐字相同 | ✓ |
| `recovery.rs:67`（`impl … PoolReader for [(DeviceIdentity, Device)]`） | 逐字相同（即判别力自证用的那一条） | ✓ |
| `recovery.rs:220-259`（`choose_system_configuration`）、`:224-225`（读第一槽）、`:232-233`（读第二槽） | 函数起止与两处读槽逐字相同 | ✓ |
| `recovery.rs:1355`、`mount.rs:407`、`:1113`、`:1192`（`choose_root(` 全部调用点） | `grep -n "choose_root("` 命中恰好 5 处，与报告列的 5 处一一对应 | ✓ |
| 「`grep -rn -i 'probe_write\|探针写\|read_only\|ReadOnly\|转只读\|instance_switch\|实例切换' crates/singlefs-core/src/*.rs` 零命中」 | 工作区重跑同一条命令，`wc -l` = 0 | ✓ |

**判 ✓**（12/12）。


### 2.2 kb 引用逐条核（行号现查 kb 文件本身）

| 引用 | 报告怎么写 | 现查结果 | 判 |
|---|---|---|---|
| D16（发布语义） 已定项 7 定案句（「一次发布的持久顺序恒为…」整段） | `.claude/kb/decisions/16-发布语义.md:99-100` | `grep -n "一次发布的持久顺序恒为"` 命中 **169**；`99` 是「#### 已定项 4：一次发布整体施加或整体不施加…」的标题行，`167` 才是「#### 已定项 7：发布的持久顺序」标题、`169` 是定案句 | **✗ 行号错**：应为 `:167`（标题）/`:169`（定案句），不是 `:99-100`——那两行是已定项 4 的标题与空行，不是已定项 7 |
| D16 已定项 4「一次发布整体施加或整体不施加」（K1.3 节判据） | `.claude/kb/decisions/16-发布语义.md:99-100` | 行 99 标题「#### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层」字面即含被引短语，行 100 为空行 | ✓（被引短语确实逐字出现在标题行本身，可判可用） |
| D23（journal 的角色与格式） 已定项 14「所选根取被重发的那个在飞 checkpoint 所基于的根…」 | `.claude/kb/decisions/23-journal的角色与格式.md:371` | 逐字相同 | ✓ |
| D23 已定项 14 全文范围 `:340-391` | 报告称在此范围内找过「重新核对磁盘根」一步、没找到 | `#### 已定项 14` 标题在 340，下一个 `####` 标题（已定项 15）在 392，范围核实为 340–391 | ✓ |
| checks-owed.md C381 一行 `:341` | 整行摘引 | 与主 agent 已核过的那句逐字相同（不重复计） | — |

**判**：**核了 4 条（另 1 条不重复计），✓ 3 条，✗ 1 条。**


### 2.3 `man 2 fsync`、`man 2 close` 原文核对（K2 节，草稿落盘副本 `/tmp/claude-1000/c381-r3-sonnet/*-rendered.txt`）

| 引用 | 报告标注 | 现查结果 | 判 |
|---|---|---|---|
| fsync EIO 段整段抄 | `fsync-rendered.txt:66-72`，标「整段抄」 | 逐行比对，**第 2 行漏一个空格**：文件原文「on the same **  **file.」（两个空格），报告写成「on the same file.」（一个空格）；且该 EIO 段落在文件里实际到 `:74`（「…on the file when the error was recorded.」）才结束，`:66-72` 只到段中一个连字符断词「filesys‐」，不是完整段落，与「整段抄」的自我标注不符 | **✗**：一处空格丢字，且段落被截断而未在报告里注明是节选 |
| close(2) NOTES「Retrying…wrong thing to do…closed.」句 | `close-rendered.txt:94-107`，标「整段抄」 | 逐字比对，**文字本身逐字相同**；但引文最后一词「closed.」实际位于文件第 **108** 行行首（`:94-107` 只到「…from another thread to be」），行区间应为 `:94-108`；该段在文件里还继续到 `:111`（解释为什么会重用 fd）才结束 | **✗（行区间）**：末行应为 108 不是 107；核心句「Retrying the close() after a failure return is the wrong thing to do」本身逐字准确、且报告要考的那句话完整可信 |

**判**：**核了 2 条，✓ 0 条，✗ 2 条**（均为文本核对层面的瑕疵：一处丢空格、两处行区间少算一行且段落被截断未注明；报告用来支撑论证的关键句「报错之后重试是错的」本身逐字为真，未被这两处瑕疵动摇）。

### Sonnet 腿小计

**核了 18 条（12 代码引用 + 4 kb 引用不重复计 1 + 2 man 页引用），✓ 15 条，✗ 3 条。**


## 三、本地攻方腿转述核对表（`c381-r3-local-attack-translation-audit.md`）

### 3.1 两处「首稿已改正」，核改后的说法与 kb / 代码原文对不对得上

| 项 | 改动后的说法 | 核 | 判 |
|---|---|---|---|
| K3 Fact 3：删掉「任何已 fsync 返回的事务都会丢」的过度推广，只留「根槽不镜像，一次发布的那一代只由那一个物理槽罩着」 | 现查 `.claude/kb/layout/01-first-txn.md:43-44,58,62,64`（根环三区域、区域归属 0→盘0/1→盘1/2→盘0、S=8、mkfs 三区域各写一份）与 `.claude/kb/decisions/16-发布语义.md:169,173`（「返回之后这一代只由那一个根槽罩着（根槽不镜像）」，且 `:173` 明写这句只管**每个新实例第一个根**，「同一实例里退一代，靠 journal 重放追得上」） | 改后的说法与两处原文逐字对得上；首稿被删掉的那句（把「新实例第一个根是单点」的射程扩成「任何已确认事务」）确实是过度推广，`:173` 原文的限定词（「每个新实例…的第一个根」）首稿丢了、改稿补上了 | ✓ 改得对 |
| K1 Fact 5：`Mounted` 字段描述改回中性说法「carries exactly three fields, named output, allocator, and current」，不再替字段发明「handle」「version pointer」类型 | 现查 `crates/singlefs-core/src/mount.rs:190` | `pub struct Mounted { pub output: MountOutput, pub allocator: PoolAllocator, pub current: PoolVersion }`——三个字段名逐字为 `output`/`allocator`/`current`，改稿没有替它们发明类型描述；首稿的「an output handle…a current version pointer」确实是代码里没有的措辞 | ✓ 改得对 |

**判**：**核了 2 条，✓ 2 条，✗ 0 条。**


### 3.2 转述核对表其余「原文文件:行」逐条核（K3、K1 两份 Fact 表）

| 项 | 引用 | 核 | 判 |
|---|---|---|---|
| K3/K1 Fact 1 | `.claude/kb/decisions/16-发布语义.md:169`（持久顺序定案句） | 逐字相同（且与 sonnet 报告 §2.2 的错误引用对照——这里的 `:169` 才是对的） | ✓ |
| K3/K1 Fact 2 | `research/prompts/d13-item4-fua-r1-user-verdicts.md:11,13,19`（`ForceUnitAccess`/`barrier()` 同调 `sync_data`；`write_all_at` 走 `pwrite`、不设 `REQ_FUA`；「写完再整盘刷一次…比真 FUA 强」） | 三处逐字相同 | ✓ |
| K3 Fact 4 | `_c381-r3-body.md:25-26`（候选乙定义与理由，整句） | 逐字相同 | ✓ |
| K3 Fact 5 / K1 Fact 6（前半） | `.claude/kb/decisions/23-journal的角色与格式.md:365`（探针写机制、取号全或无） | 逐字相同 | ✓ |
| K3 Fact 6 / K1 Fact 3 | `_c381-r3-body.md:46`（`transaction.rs` 那一行摘要） | 逐字相同 | ✓ |
| K3 Fact 7 / K1 Fact 4 | `_c381-r3-body.md:49`（零命中那一行） | 逐字相同 | ✓ |
| K3 Fact 8 | `.claude/kb/layout/01-first-txn.md:81`（「自己那条的落点在写出前已定」） | 逐字相同 | ✓ |
| K3 Fact 9 | `.claude/kb/layout/01-first-txn.md:45`（journal 环几何） | 逐字相同 | ✓ |
| K3 Fact 10 | `.claude/kb/layout/01-first-txn.md:40-41`（系统配置槽几何） | 逐字相同 | ✓ |
| K3 Fact 11 / K1 Fact 7 | `.claude/kb/checks-owed.md:341`（C381 一行） | 与主 agent 已核过的那句逐字相同（不重复计） | — |
| K1 Fact 6（后半） | `.claude/kb/layout/01-first-txn.md:60`（取号先写系统配置、全或无） | 逐字相同 | ✓ |
| K1 Fact 8 | `_c381-r3-body.md:22-23`（候选甲定义与理由，整句） | 逐字相同 | ✓ |
| **K1「T1–T4 四段定义」** | `_c381-r3-body.md:84`（称引「T1 单元写到一半、T2 记录发出之后根槽 FUA 返回之前、T3 根槽 FUA 成功之后系统配置槽轮换之中、T4 取号那几次写」） | 现查 `_c381-r3-body.md:84` 原文是「本地攻方（`three-way-local-attack`…）」那一行任务分工描述，只提了一句「四段（T1–T4）」，**不含**被引的那四句定义；对 `_c381-r3-body.md`、`_c381-r3-checklist.md`、`_c381-r3-appendix.md` 全文 `grep` 这四句定义的关键词（单元写到一半、记录发出之后、系统配置槽轮换之中、取号那几次），**全部零命中**——这四句定义在本轮（r3）任何背景材料里都不存在，只在上一轮材料里（`_c381-r2-body.md:45`、`_c381-r1-background.md:42`）以相近但不完全相同的说法出现 | **✗**：引用的文件:行不含被引内容，且该内容在本轮任何背景材料里都找不到，只是沿用了上一轮材料的说法却挂了本轮一个不相关的行号 |

**判**：**核了 12 条（另 1 条不重复计），✓ 11 条，✗ 1 条。**

### 本地攻方腿小计

**核了 14 条（2 条改正核实 + 12 条转述引用，其中 1 条不重复计），✓ 13 条，✗ 1 条。**


## 四、总计

| 腿 | 核了 | ✓ | ✗ | 核不动 / 分不清 |
|---|---|---|---|---|
| Opus 攻方 | 31 | 31 | 0 | 0 |
| Sonnet 正推 | 18 | 15 | 3 | 0 |
| 本地攻方（转述核对表） | 14 | 13 | 1 | 0 |
| **合计** | **63** | **59** | **4** | **0** |

**✗ 的四处一览**：
1. sonnet §2.2：D16 已定项 7 定案句引用行号 `:99-100`，实际在 `:167`（标题）/`:169`（定案句）；`:99-100` 是已定项 4 的标题与空行。
2. sonnet §2.3：fsync EIO 段引文第 2 行丢一个空格（原文「same␣␣file.」被写成「same␣file.」），且引用范围 `:66-72` 未覆盖到该段落实际结尾 `:74`，报告仍标「整段抄」。
3. sonnet §2.3：close(2) NOTES 段引文末词「closed.」实际在文件第 108 行行首，行区间应为 `:94-108` 而非 `:94-107`；引文文字本身逐字准确，问题只在行区间少算一行、且段落在 `:111` 才真正结束却未注明是节选。
4. 本地攻方转述核对表：K1「T1–T4 四段定义」引 `_c381-r3-body.md:84`，该行不含被引的四句定义，且这四句定义在本轮全部背景材料（body/checklist/appendix）里都检索不到，只是上一轮材料里相近说法的延续。

## 没做什么

- 不判任何一条打中成不成立、该不该采纳；只核引用、产物与复跑，推理本身留给主 agent 逐条现查。
- 未重复核主 agent 已交代「自己核过」的四处（`recovery.rs:220/289/67` 与 `transaction.rs` 零命中；正推腿 H1 与 `checks-owed.md:341` 的对应；本地攻方两份样本在「屏障没落点」「根槽与系统配置槽权威数据」两处一致；攻方 K4-b 引 `02-RAID条带策略.md:225`）——这四处在上表里出现时按「不重复计」标注，仍顺手复核过并确认与主 agent 的记录一致，未发现新问题。
- 未编译工作区 `crates/` 本体（按派发提示禁止）；复跑全部在草稿目录 `/tmp/claude-1000/c381-r3-verifier/repo/` 的独立副本（HEAD `0ec0be2` + 攻方给的补丁）里进行，不在攻方自己的 `/tmp/claude-1000/c381-r3-opus/` 目录里跑。
- 未核本地攻方腿 K3/K1 两份 output（s1/s2）模型答复本身的判定内容（那是主 agent 已核过的第 3 项，「屏障没落点」「权威数据落点」两处一致），只核了转述核对表里「原文文件:行」这一半。
- 未对 sonnet 报告 1.2 节「一次尝试不是穷举」「三段可达历史」这类推理性陈述做任何评判，只核了它引的代码行号与 kb 行号。
- 未跑门禁（`.claude/gate.d/stage-owners.tsv` 里没有登记给核查员的阶段）。
- Postgres 二进制 `strings` 那一条（sonnet §2 末尾，只当反证信号）未复核——报告自己已声明「不写成正证」，不含可核的文件:行引用，跳过。
- 报告文件本身：本轮由核查员直接产出，不存在「云端腿交回的 sha256」需要核对的情形（派发提示给的报告 sha256 一栏为空）。

