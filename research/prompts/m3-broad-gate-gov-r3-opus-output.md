# 广谱变异门禁治理第三轮（终轮）：云端攻方（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 J1 J2 J3 J4 K1 -->

被判对象读冻结副本 `/tmp/claude-1000/m3-gov-r3-freeze/tree/`；引的五份文件（门禁、驱动、包装、admission.py、commit-stages.md）与样本 setup.sh 在主仓与冻结副本里 sha256 逐份相同（开工时核过）。副本上的数都是副本上的，不算入库装置上的数。

## 复跑

- 轻的（子 agent 跑过、可复跑）：`bash research/prompts/m3-broad-gate-gov-r3-opus-model/rerun.sh /tmp/claude-1000/m3-gov-r3-freeze/tree <草稿目录> [段…]`（段 samples j2a j2b j1c j1d j1a j2c j3 fix，不给全跑，约 7 分钟；产出覆盖 `outputs/`）。
- 重的（子 agent 被重型闸拒，要主 agent 跑）：`SINGLEFS_HEAVY_TESTS=user-request nice -n 19 bash research/prompts/m3-broad-gate-gov-r3-opus-model/main-agent-heavy.sh /tmp/claude-1000/m3-gov-r3-freeze/tree <草稿目录>`，见末尾「要主 agent 跑的命令」。

模型文件 sha256 前 16 位（`sha256sum *.sh *.py` 原样取的）：

| 文件 | sha256 | 用途 |
|---|---|---|
| run-samples.sh | d25cecbee942fc00 | 照 stage-selftest 的起法逐份起样本；`HOOK=` 在 setup.sh 之后改现场 |
| hook-baseline-rerun.sh | 5c733e52171e216d | J2-a：baseline.tsv 追加一行新绿记录 |
| hook-baseline-rerun-then-reescalate.sh | f9dd737c8768ebcb | J2-a：再照出路重判、另写一行登记 |
| hook-peak-overwrite.sh | ca7b42dc78eb2990 | J2-a：包装改写基线峰值 |
| j2a-baseline-probe.py | 6ebc3c21e76cdacc | J2-a：只换 run_step，跑 ensure_baseline 看换 --threads 追加不追加 |
| j2b-gate-mutants.py | c32917d5ac16bd1d | J2-b：门禁升档复核 12 个分支逐个删 |
| hook-j2b-case.sh | de44043e349800d0 | J2-b：10 份驱动不会写的登记行 / --summary |
| j2c-wrapper-probe.sh | ac48c604c677c662 | J2-c：真包装两句喂驱动证据判；升档那趟撞 slice |
| j2c-over-slice.sh | 858793d02ea64b67 | J2-c：升档上限超 slice 总上限 |
| j2c-pagecache.sh | 6c10e46211265609 | J2-c：基线峰值含页缓存顶到上限 → 升档上限 2 倍 |
| j2c-collateral.sh | b412ef346dcff73c | J2-c 旁支：撞 slice 总上限时内核杀了别的那一条 |
| j2d-driver-mutants.py | 861f54da148fcca6 | J2-b 驱动一侧：升档证据 6 个分支删掉（交主 agent 跑自证） |
| j1c-admission-mutants.py | 5f316de226db24d9 | J1-c：build_environment_lines 逐类写错 9 种 |
| j1c-env-gap.py | 52146a27780e2f37 | J1-c：每种写错版下哪一类变了构建环境却不变 |
| j1d-rustup-toolchain.py | 8144ca33c657ecd8 | J1-d：RUSTUP_TOOLCHAIN 与 --unit-environment |
| j3-combos.py | e67c1f9c2570d16f | J3：第二轮 H1-7 加上升档结局再抽一次 |
| main-agent-heavy.sh | d82ebec2959ffb55 | 交主 agent：拷贝上的 admission / 驱动自证 |
| j2a-fix-prototypes.py | 05dfa7b5a2d2d83c | J2-a 改法 F1、F2 在门禁拷贝上的原型 |
| rerun.sh | 3385d361808fe407 | 轻的全部复跑 |

## 各格判定一览

| 格 | 判 | 依据（产物在 `outputs/`） | 满足的判据分句 |
|---|---|---|---|
| J2-a 基线漂移 | **打中**（不该红而红，出路走不出去） | 换 `--threads` 后 baseline.tsv 追加热编译的新绿行（`j2a-baseline-probe.txt`），门禁对这一版每一行登记都复核，旧行永远「登记的基线 … 不同」，照出路重判之后照样红（`j2a-hooks.txt`）；撞顶那一边包装改写峰值同形 | 「一个输入让门禁或驱动该红不红、不该红而红」 |
| J2-b 升档复核分支没人盯 | **打中**（12 个分支删掉 36 份样本一份不翻；其中 10 个各给了一份原样红、删后绿的输入） | `j2b-sweep.txt`、`j2b-inputs.txt`；驱动一侧 6 个分支的自证待主 agent 跑 | 第三条分句的强形（没有样本盯，不只是共用样本）；最后归哪一条由主 agent 核 |
| J2-c 撞顶升档放不下 | **打中**（用户定案的出路对确定性无界分配走不通） | 基线峰值含页缓存顶到上限 → 升档上限 = 2 倍原上限（`j2c-pagecache.txt`）；升档那趟按头一趟上限排队、实际涨到 1.5–2 倍 → slice 里另有占用就 254（`j2c-wrapper-probe.txt`、`j2c-over-slice.txt`）；出路句「加大 --memory-cap」方向反 | 「五份文件里一句话与今天的做法不符」（出路句）；频率推的 |
| J2-c 旁支 | 打中（不是这一批引入的） | 无界分配的头一趟按上一条变异的小峰值排进来，撞 slice 总上限时内核杀了 slice 里另一条（`j2c-collateral.txt`），驱动文件头「撞顶只杀那一步」不成立 | 「一句话与今天的做法不符」 |
| J2-d 证据伪造 | 打中，**不分辨臂** | 门禁自己的绿样本就是 printf 加 sha256sum 写出来的登记行与日志，门禁判绿；结果表里一行普通的 caught 同样手写得出，库目录同一个用户可写是各臂共用前提 | 「门禁认下一行驱动不会写的登记行」；补法见 J2-d |
| J2-e 假抓到（K） | 没打中（一次抽样） | K 不是起作用的那一道：真库基线编译 14–24 秒、文档测试 1.5 秒，原限时 900、升档 3600 秒，K=20 恒满足；本机 nice 与 cgroup 量过，20 倍拖慢够不着 | — |
| J2-f 共用常量与字面 | 部分打中（并进 J2-b） | 倍数常量由字面值样本的 escalation 行与 multiple-red 的「20 倍」钉住；撞顶那一边门禁代码的倍数门槛没人盯（g06m）；包装那一句的字面四份各写各的，今天真包装两句驱动认（`j2c-wrapper-probe.txt` ①，量过） | 同 J2-b |
| J1-a `--marker-check` | 小打中（一句） | 路径集合与写标记、admission 指纹同一份；退 1 的「判不了」那一支（admission 退 2）commit-stages 第 19 行仍叫去找会话协商（`j1a-marker-check-no-cargo.txt`） | 「一句话与今天的做法不符」 |
| J1-b/c 字面值样本、admission 共用 | **打中** | build_environment_lines 逐类写错 7 种都让一类构建环境从全局键里消失（`j1c-env-gap.txt`），36 份样本一份不翻（`j1c-sweep.txt`）；字面值样本只钉 4 类；admission 自证已是旧红，红→红翻不了面；驱动与 admission 自证在拷贝上待主 agent 跑 | 第三条分句（共用函数、样本不翻） |
| J1-d `--unit-environment` | 小打中（条件式） | 会话设 RUSTUP_TOOLCHAIN：工具链从 1.98.0 变成 1.100.0-nightly，--unit-environment 一个都不打（`j1d-rustup-toolchain.txt`）；③ 退 7，commit-stages「带上重起」用不上 | 「不该红而红」（条件式，推的频率） |
| J1 驱动文件头 K 一句 | 小打中 | 「同一个线程数下」与 baseline.tsv 没有线程数一列不符（J2-a） | 「一句话与今天的做法不符」 |
| J3 第二条再抽 | 没打中（第二次抽样） | H1-7 加升档结局 5 400 趟 0 次误判（`j3-combos-*.txt`）；H2-b 26 个开关翻面与文件头逐项相同（`j3-switches.txt`） | — |

## J2 用户定案的升档登记表

### J2-a 基线漂移：一行驱动自己写的、证据齐的登记行，之后永远复核不过（打中）

历史（每一步指到许可它的那一句）：

1. 主 agent 照 ③ 起驱动，`--threads` 取「跑之前定下来」定的数：「nice -n 19 python3 research/scripts/broad-mutation.py --changed-since HEAD --threads <」（`.claude/skills/crash-test/commit-stages.md:26`），那个数「与线程数（驱动的 --threads，取整机核数 ÷ 同时在跑的重活数）」（`.claude/skills/crash-test/commit-stages.md:12`），每一趟随负载重定。
2. 第 N 条编译那一步两趟都超时、证据齐，驱动写一行登记，基线取「rows = [row for row in read_table(self.library.version_file("baseline.tsv"), len(BASELINE_COLUMNS)).rows if row[0] == step_name and row[1] == "ok"]」（`research/scripts/broad-mutation.py:1853`）的最后一行。
3. 同一版之后再起一趟，线程数不同（③ 后面的 `--item --rerun` 出路、或同一份源码的下一次提交尝试：只改了导出范围外的文件，版本键不变）。基线峰值按线程数分文件「return self.library.version_file(f"baseline-peaks.{self.arguments.threads}.tsv")」（`research/scripts/broad-mutation.py:1836`），新线程数下没有，于是「peak_known = self.fake_driver or self.baseline_peak(step_name) is not None」（`research/scripts/broad-mutation.py:1996`）为假，整段基线重跑，且「append_row(self.library.version_file("baseline.tsv"), BASELINE_COLUMNS, [step_name, outcome, f"{seconds:.1f}", now_text()])」（`research/scripts/broad-mutation.py:2008`）追加；而 baseline.tsv 没有线程数一列：「BASELINE_COLUMNS = ("step", "outcome", "seconds", "finished_at")」（`research/scripts/broad-mutation.py:386`）。编译目录是热的，新的最后一行绿记录是热编译的秒数。
4. ④ 门禁对这一版每一行登记都复核，不只是 ESCALATED 背后那几行：版本对上就「found = row_problems(row)」（`.claude/gate.d/checker-tier-broad-mutation.sh:606`），其中「elif baseline_text != row["baseline"]:」（`.claude/gate.d/checker-tier-broad-mutation.sh:558`）判红。
5. 出路说「--item <序号> --rerun 重判那一条」：登记表只追加，新行复核过，旧行还在、还是红。只剩换一版（改源码或全局键）。

量过（副本，`outputs/j2a-baseline-probe.txt` 原样，驱动只换了 run_step、不起子进程）：

```
--threads 8 之后：baseline.tsv 2 行；one-build 最后一行绿记录 = 312.4，这一趟 one-build 的整步限时 = 1562 秒
--threads 8 之后：baseline.tsv 2 行；one-build 最后一行绿记录 = 312.4，这一趟 one-build 的整步限时 = 1562 秒
--threads 16 之后：baseline.tsv 4 行；one-build 最后一行绿记录 = 1.2，这一趟 one-build 的整步限时 = 900 秒
```

门禁一侧（副本，gate-verdict-escalated-green 造好之后改一处，`outputs/j2a-hooks.txt` 原样）：

```
== baseline-rerun
判对 0 个、判错 1 个：gate-verdict-escalated-green(期望0→实测1，缺 want 2 条)
     登记表第 2 行（第 1 条 caught-by-timeout，版本 e5ca1a7e00000000）：登记的基线 10.0 与那一版记的 1.2 不同
     --summary 说第 1 条是升档判的 caught-by-timeout，登记表里没有这一版（e5ca1a7e00000000）复核得过的那一行
== baseline-rerun-then-reescalate
判对 0 个、判错 1 个：gate-verdict-escalated-green(期望0→实测1，缺 want 2 条)
     登记表第 2 行（第 1 条 caught-by-timeout，版本 e5ca1a7e00000000）：登记的基线 10.0 与那一版记的 1.2 不同
== peak-overwrite
判对 0 个、判错 1 个：gate-verdict-escalated-green(期望0→实测1，缺 want 2 条)
     登记表第 3 行（第 2 条 caught-by-memory，版本 e5ca1a7e00000000）：登记的基线 1073741824 与那一版记的 1074790400 不同
     --summary 说第 2 条是升档判的 caught-by-memory，登记表里没有这一版（e5ca1a7e00000000）复核得过的那一行
```

第二段是照出路重判之后（另写两份日志、登记表追加一行基线 1.2 的新行）：新行复核过、ESCALATED 那一句不再报，旧那一行照红。撞顶那一边：包装的峰值表每个键一行、每次收尾改写成这一次量的（文件头写「撞了这一条自己的上限记成上限」（`research/scripts/run-with-memory-cap.sh:67`），收尾在同一把锁里改写），同一线程数下基线那一步再跑一遍（上一行基线不绿、峰值表丢了）就同形。

附带一句与做法不符：驱动文件头「K 为什么够：超时那一边，同一台机器、同一个线程数下没改过的这一步跑 1 倍基线」（`research/scripts/broad-mutation.py:115`），超时取的基线不分线程数，是最后一行绿记录（第 3 步的 1.2 秒就是 16 线程热编的）。

四句：
- 分不分辨臂：分。打的是升档登记表（甲）；`exclude_re`（乙）与红着提交（丙）不写登记行，不中。也能在甲里修。
- 系统看不看得到：看得到。baseline.tsv 只追加，登记行记下的基线是当时最后一行绿记录，门禁能认出「是这一版较早的一行绿记录」与「哪一行都不是」。
- 满足的分句：「一个输入让门禁或驱动该红不红、不该红而红」的「不该红而红」：驱动自己写的、证据齐的行，门禁判红，出路清不掉。
- 改法中不中：判据给的改法是「改门禁或驱动」。下面三样，各修哪一格写在后面的改法表；只改其中一样，另两格照中。
  - 甲：基线按线程数分开记，登记行与门禁都按登记行的 threads 取。
  - 乙：登记行另记它取的那一行基线的 finished_at，门禁按它找那一行。
  - 丙：门禁只复核 ESCALATED 背后、定下状态的那一行登记，被后来的行盖过的不判红。

### J2-b 升档复核那一段的判红分支，样本只盯了五个（打中）

门禁 gate-verdict 这一批加的复核（`.claude/gate.d/checker-tier-broad-mutation.sh:506-622`）里，样本 escalated-* 五红一绿与六个弄坏开关只钉住：倍数（只超时那一边）、日志 sha256、版本键、日志在不在、escalated= 条数。其余分支逐个删掉（`j2b-gate-mutants.py`，每种只改门禁一处），36 份样本一份不翻（`outputs/j2b-sweep.txt` 原样）：

```
g01-label	判对 36 个、判错 0 个：无
g02-baseline-missing	判对 36 个、判错 0 个：无
g03-baseline-differs	判对 36 个、判错 0 个：无
g05-not-larger	判对 36 个、判错 0 个：无
g06m-memory-multiple	判对 36 个、判错 0 个：无
g07-trailer	判对 36 个、判错 0 个：无
g07b-trailer-prefix	判对 36 个、判错 0 个：无
g08-wrapper	判对 36 个、判错 0 个：无
g09-columns	判对 36 个、判错 0 个：无
g10-escalated-field	判对 36 个、判错 0 个：无
g11-sequence-only	判对 36 个、判错 0 个：无
g14-anywhere	判对 36 个、判错 0 个：无
```

另造 10 份「驱动不会写、门禁该判红」的现场（`hook-j2b-case.sh`，在 gate-verdict-escalated-green 上改一处，改了日志的 sha256 现算回填），原样门禁都红、对应的删分支版都绿（`outputs/j2b-inputs.txt` 前三列原样）：

```
i03-baseline-lie	原样门禁 exit=1	g03-baseline-differs exit=0
i05-not-larger	原样门禁 exit=1	g05-not-larger exit=0
i06m-memory-multiple	原样门禁 exit=1	g06m-memory-multiple exit=0
i07-no-trailer	原样门禁 exit=1	g07-trailer exit=0
i07b-trailer-wrong	原样门禁 exit=1	g07b-trailer-prefix exit=0
i08-no-wrapper	原样门禁 exit=1	g08-wrapper exit=0
i09-columns	原样门禁 exit=1	g09-columns exit=0
i10-no-escalated-field	原样门禁 exit=1	g10-escalated-field exit=0
i11-name-differs	原样门禁 exit=1	g11-sequence-only exit=0
i14-logs-elsewhere	原样门禁 exit=1	g14-anywhere exit=0
```

各现场是什么：i03 登记行的基线写 1.0、这一版记的是 300.0（K 倍门槛于是按假基线算）；i05 升档限时与原限时相同；i06m 撞顶那一行基线峰值 20G、升档 36864M 不到 2 倍；i07 两份日志没有驱动那一行；i07b 驱动那一行的 attempt / exit 写错；i08 日志里没有包装那一句；i09 登记行少一段；i10 有 ESCALATED 而判定行没有 escalated=；i11 ESCALATED 的变异名与登记行不同；i14 日志不在 escalation/ 下。g01（标签认不出）、g02（没记基线）删掉之后分别撞 KeyError 与落进 g03，碰巧照红，不是钉子。

驱动一侧同形：升档证据那一判（`escalation_evidence_problems`）的判红分支，自证 selftest_escalation_cells 只走了「两趟结局不同」一种不齐；没基线、不到 K 倍、没比原来大、日志缺包装那一句四种，按代码推自证不翻（推的：j2d 六种写错版的 `--selftest` 子 agent 被拒，写进 main-agent-heavy.sh，d1 是对照、应当红）。其中「不到 K 倍」「没比原来大」对驱动自己写的行恒不成立（升档上限就是按 max 算的），只有「没基线」「缺包装那一句」是驱动真会走到的不齐。

四句：
- 分不分辨臂：不分臂，是甲这一份实现的钉子问题。
- 系统看不看得到：看得到，每份现场门禁原样都判红了。
- 满足的分句：删分支后样本不翻，形同第一轮第 2 行（A3），比第三条分句「只有与被判代码共用…的样本盯着」更弱一档：这几支没有样本。归哪一条交主 agent 核。
- 改法中不中：判据的改法是补一份不共用的样本或开关。照第一轮第 2 行的做法，每支一份红样本配一个弄坏开关（10 份现场可直接改成样本），修 J2-b 全部格；对 J2-a 不起作用。

### J2-c 撞顶升档：确定性无界分配那一类，出路走不通（打中）

量过（副本，真包装在自己的临时 slice 里，总上限 160M–256M；`outputs/j2c-*.txt` 原样）：

1. 基线峰值含页缓存，会顶到上限。文件头说峰值「撞顶那一边，包装记的峰值含页缓存」（`research/scripts/broad-mutation.py:118`），升档上限是「wanted = max(ESCALATION_MEMORY_FACTOR * first_bytes, ESCALATION_MEMORY_BASELINE_MULTIPLE * (baseline or 0))」（`research/scripts/broad-mutation.py:1940`）。基线那一步写盘多时峰值 = 上限，升档上限 = 2 倍原上限：

```
① 基线（写 400M 的文件）exit=0
   记下的峰值 100663296 字节（上限 96M = 100663296 字节）
② 变异那一趟 exit=250
   照 escalate_step 的式子升档上限 = 192M
③ 升档那一趟 exit=254
run-with-memory-cap: 被 slice singlefs-r3attack.slice 的总上限挤掉了
```

   本机真表里就有这一形（只读，主仓 `research/scripts/memory-peaks.tsv` 未进 git，与真库旧版 19c45e02 的 baseline.tsv 各 grep 一行，原样输出只把时刻略去；两行的时刻逐字相同）：

```
25769803776	2026-09-30T（时刻略）Z	24G	broad-mutation two-test-slow-harness	31
two-test-slow-harness	ok	936.0	2026-09-30T（时刻略）Z
```

   这一步在 24G 下基线绿、峰值记成 24G（那时是加升档之前的驱动，峰值记在主仓这张表、键是 broad-mutation <步名>；今天的驱动把基线峰值记进这一版的 baseline-peaks.<线程数>.tsv，同一个包装、同一个口径，推的同样记成上限）。照今天驱动，这一步一旦撞顶：升档上限 = max(36G, 48G) = 48G，大于本机「本机 slice 总上限因此约 40 GiB」（`research/scripts/run-with-memory-cap.sh:38`），一定退 254（见下一条），照原结局记没结论。降 `--memory-cap` 也不行：这一版这个线程数下峰值已经记了，基线不重跑；换线程数让基线重跑，又触发 J2-a。

2. 升档那趟排队按的是峰值表里这个键记的量（头一趟撞顶记成头一趟的上限），scope 的上限却是升档的那个：排队「need = min(recorded[0], cap_bytes)」（`research/scripts/run-with-memory-cap.sh:574`），撞顶收尾记「admission finish "$unit" "$key" "$cap" "$cap_bytes"」（`research/scripts/run-with-memory-cap.sh:945`）。slice 里另有占用、或升档上限超过 slice 总上限，升档那趟涨过总上限，退 254：

```
== ② slice 里另有一条常驻 120M（经包装、上限 130M）时，撞顶的头一趟 100M、升档那趟 150M
slice-first exit=250
run-with-memory-cap: 撞了内存上限 100M
104857600	100M	r3-slice
slice-escalated exit=254
run-with-memory-cap: 被 slice singlefs-r3attack.slice 的总上限挤掉了
run-with-memory-cap: → 这一条的结果不算数，重跑它；常这样说明排队按峰值表放进来的几条实际占的比表里记的多，bash /tmp/claude-1000/m3-gov-r3-attack/rerun/trees/base/research/scripts/run-with-memory-cap.sh --status 看 slice 里是谁
常驻那一条 exit=0
---（j2c-over-slice.txt）
上限 200M exit=250
run-with-memory-cap: 撞了内存上限 200M
209715200	200M	r3-over
上限 300M exit=254
run-with-memory-cap: 被 slice singlefs-r3attack.slice 的总上限挤掉了
209715200	200M	r3-over
```

   驱动对升档那趟的 254 是「if exit_code in WRAPPER_INFRASTRUCTURE_EXITS:」（`research/scripts/broad-mutation.py:1956`）那一支：照原结局记没结论。按本机数推（推的）：默认上限 24G 时升档 36G，slice 总上限 40.1 GiB（`--status` 现量），slice 里别的回收不掉的用量超过约 4.1 GiB，升档那趟就是 254；上限大于约 26.7G 时 slice 空着也是 254（实现员推的是 252，按排队代码是先放行再撞总上限，254，`j2c-over-slice.txt` 量过同形）。

3. 出路句方向反：门禁「证据不齐、仍是这两种的加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun」（`.claude/gate.d/checker-tier-broad-mutation.sh:132`），驱动文件头「加大 --timeout-floor 或 --memory-cap（都不进全局键）再 --item <序号> --rerun 点名重跑（重跑里照样升档）。」（`research/scripts/broad-mutation.py:134`），commit-stages 同一句「加大 --timeout-floor 或 --memory-cap（都不进全局键）」（`.claude/skills/crash-test/commit-stages.md:28`）。对无界分配，加大上限只让升档那趟更放不下；而 1、2 两条下它不是「证据不齐」，是「证据造不出来」。第二轮判决交用户的正是这一类（「无界分配撞内存（撞顶分支在测试没过之前）」），用户定案之后它照旧走不出去，只是从「门禁红」变成「升档 254、门禁红」。

四句：
- 分不分辨臂：分。只中甲的撞顶那一半；乙（`exclude_re`）不经包装，丙不升档。
- 系统看不看得到：看得到。驱动开跑前就知道 slice 总上限（包装 `--status`）、基线峰值与要升到的上限，算得出「升档上限放不进 slice」，只是今天不算。
- 满足的分句：出路句那三处属「五份文件里一句话与今天的做法不符」；「用户定案的出路还走不走得通」是 J2 题面问的，答：撞顶这一类在本机常见负载下走不通（频率推的，真库这一版真跑结果行很少，没量）。
- 改法中不中：判据没给这一格的改法。我提的四样（只在我的模型上量过、被攻过零轮）：
  - 甲：撞顶 K 的基线峰值改成回收不掉的那一部分（anon 加 shmem），不用含页缓存的 memory.peak。改包装的峰值口径，修第 1 条。
  - 乙：升档上限截到「slice 总上限减此刻回收不掉的用量」，截了还不到 K 倍就当场记「证据造不出来」、写明出路。修第 2 条的 254，不修第 1 条。
  - 丙：升档那趟排队按它自己的上限算要的量（不按峰值表），放不下就等或 252，不撞总上限。修第 2 条与 J2-c 旁支。
  - 丁：出路句对撞顶写「降上限或等 slice 空出来」，不写「加大」。

### J2-c 旁支：撞顶只杀那一步不成立（打中，不是这一批引入的）

驱动文件头「撞顶只杀那一步、记 memory-cap」（`research/scripts/broad-mutation.py:96`）。无界分配那一条按峰值表里这个键上一条变异的小峰值排进来，scope 上限却是 `--memory-cap`；slice 里另有一条更大的常驻（别的会话的重活），slice 撞总上限时内核杀的是大的那一条（副本，`outputs/j2c-collateral.txt` 原样）：

```
记小峰值那一趟 exit=0
22020096	64M	r3-coll
无界分配那一条 exit=250
run-with-memory-cap: 撞了内存上限 200M
常驻那一条（别的会话的重活）exit=254
run-with-memory-cap: 被 slice singlefs-r3attack.slice 的总上限挤掉了
```

变异那一条照样记撞顶，被杀的是别人。升档让一条无界分配多跑一趟，按 2 的式子推，升档那趟自己才是大的那一条（推的），新增的风险在头一趟。改法同上丙。

### J2-d 照格式造的日志与登记行过得去（打中，不分辨臂）

门禁认「驱动写的」只靠这几样：日志在「if not path.startswith(os.path.join(version_directory, evidence_directory) + os.sep) or not os.path.isfile(path):」（`.claude/gate.d/checker-tier-broad-mutation.sh:576`）、sha256 对、带两句字面。全是库目录里同一个用户可写的文件。现成的例子是门禁自己的绿样本：gate-verdict-escalated-green 的登记行与日志由 setup.sh 的 printf 写、sha256 用「digest() { sha256sum "$home/versions/$1/$2" | cut -d' ' -f1; }」（`.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/gate-verdict-escalated-green/setup.sh:32`）现算，门禁判「升档判的抓到 2 条、登记行逐行复核过」（`outputs/samples-base.txt`：36 份全对）。驱动文件头「只由驱动写，手写的行门禁判红。」（`research/scripts/broad-mutation.py:121`）只对偷懒的手写（日志不在）成立，样本 escalated-handwritten-red 钉的就是这一种。

四句：
- 分不分辨臂：不分。真跑里造一条升档抓到，还要往结果表追加一行 caught；而结果表里一行普通的 caught（带随便一个测试名）今天同样手写得出、门禁同样认。库目录可写是三个候选与改前做法共用的前提，按「打中不分辨臂」另立一笔账，不拿它判甲。
- 系统看不看得到：文件这条路看不到。有一条它今天不用的旁路：包装那一句里带着 scope 名「echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2」（`research/scripts/run-with-memory-cap.sh:946`）；scope 的结局 systemd 用户实例也记进 journal，带 journald 填的可信字段。下面是本机此刻 `journalctl --user` 的原样输出：

```
systemd[4214]: singlefs-memory-cap-3225828-2587215151.scope: Failed with result 'oom-kill'.
systemd[4214]: singlefs-memory-cap-3532893-2477028366.scope: Failed with result 'timeout'.
{'_PID': '4214', '_COMM': 'systemd', '_UID': '1000', 'USER_UNIT': 'singlefs-memory-cap-3532893-2477028366.scope', 'UNIT_RESULT': 'timeout'}
```

  `systemctl show user@1000.service -p MainPID` 打「MainPID=4214」。同一个用户用 `logger` 写不出 `_PID=4214` 的条目。
- 满足的分句：「门禁认下一行驱动不会写的登记行」，字面满足；病根在共用前提。
- 最小补法（零轮、推的，只核了 journal 里有这两类条目）：登记行另记两趟的 scope 名，门禁对每一行查 journal。要看到 user@1000.service 主进程写的那个 scope 的 `UNIT_RESULT=timeout / oom-kill`，时刻落在登记行 finished_at 之前。
  - 能挡的：改文件造证据。
  - 挡不住的：故意起一个同名 scope 去真撞一次；journal 轮转掉了会误红（本机 journal 417.5M）。
  - 要不要做交用户：它只把造假的门槛从「写文件」抬到「真撞一次」，普通 caught 行照样能手写，补一处不补全局。

### J2-e 假抓到：K 站得住，但起作用的不是 K（没打中，一次抽样）

试的形状与取样：
- 超时一边（真库里两个旧版，只读）：基线是热编译的秒数，`~/.local/share/singlefs/broad-mutation/versions/*/baseline.tsv` 里 one-build 14.1 / 23.5 秒、two-test-doc 1.5 秒、two-build-checker-tier 43.5 秒。原限时都落在地板 900 秒，升档 3600 秒，是基线的 83–2400 倍。K=20 那一道对驱动写的行恒满足（升档上限按 max 算），真正挡假抓到的是 900 / 3600 两个绝对数。要两趟都超时，得一步十几秒的热编译被拖到一小时以上。
- 拖慢的来源，量了本机：slice 树里 cpu 控制器只开到 user@1000.service 这一层，singlefs.slice 往下只开 memory pids；另两层 subtree_control 是 cpu memory pids。所以同 slice 的重活按 nice 分 CPU，跨会话按 cgroup 权重分。`ps -eo ni` 看到的重活（别的会话的 rustc）都是 nice 19，与驱动的 ③ 同档。nice 0 对 nice 19 的约 68 倍差只在同一个 cgroup 里出现，今天没有这种组合。
- 换 `--threads` 后基线变热编秒数（J2-a 第 3 步）让 K×基线更小，地板照样挡住。
- 撞顶一边：头一趟上限 24G、升档 36G。同一张真表里 one-test 峰值 110 MiB、one-build 424 MiB，升档上限是它们的几十到几百倍，有界的变异要涨这么多才会被判成抓到。主 agent 若把 `--memory-cap` 设得很小，界才变窄。这是参数、不是判据；参数不进全局键，同一版里不同上限判的结论混在一起（推的）。
- 包装开了 MemorySwapMax=0，「靠换页拖成超时」那一形不存在。

推翻条件：真跑出现一条超时升档判抓到、改成不限时之后跑完且测试全过；或上限 24G 下出现撞顶升档判抓到、在 slice 外不限内存时跑完且全过。

### J2-f 门禁复核与驱动写证据共用什么（并进 J2-b）

| 共用的 | 门禁怎么拿 | 写错时谁盯 |
|---|---|---|
| 两个倍数、两个标签、ESCALATION_COLUMNS、登记表名与日志目录名 | 载入驱动取：「multiples = {timeout_label: driver.ESCALATION_TIME_BASELINE_MULTIPLE, memory_label: driver.ESCALATION_MEMORY_BASELINE_MULTIPLE}」（`.claude/gate.d/checker-tier-broad-mutation.sh:513`） | 常量的值：字面值样本的 escalation 一行（全局键全文写死）。超时倍数另有 multiple-red 的 want「20 倍」。撞顶倍数在门禁代码里的用法没人盯（g06m） |
| 包装那一句的字面 | 门禁自写一份：「wrapper = f"超过限时 {limit} 秒" if label == timeout_label else f"撞了内存上限 {limit}"」（`.claude/gate.d/checker-tier-broad-mutation.sh:588`） | 共四份：驱动 `:1674`、自证的假一步 `:3222`、真包装 `:946`/`:951`。各自的自证只核自己那一份；真包装对驱动没有自证格。今天对得上，量过（副本，`j2c-wrapper-probe.txt` ①：「超时两趟（2 → 4 秒、基线 0.1 秒）证据缺： 无（证据齐）」）。写偏了是全部升档没结论、门禁红，不出假绿 |
| 驱动那一行（BROAD_MUTATION_EVIDENCE）的格式 | 门禁自写一份（`:585`） | 没有样本：删掉或只核版本键都不翻（g07、g07b） |

## J1 第二轮站住的改法今天的样子

### J1-a `--marker-check`（小打中：一句）

- 路径集合是同一份：查标记与写标记都走 marker_inputs_clean，加的是「paths += [admission.REGISTRATION_TABLE, f".claude/gate.d/{stage}"]」（`.claude/gate.d/checker-tier-broad-mutation.sh:649`）；admission 算标记指纹的是「input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]」（`research/scripts/admission.py:2219`），前面都是 stage-inputs.tsv 这一行的路径。三处逐项相同。
- 退出码对 commit-stages 第 19 行：退 0（有标记）、退 3（两边相同、没标记）对得上。退 1 有两支：一支是「工作区与暂存区不同」，另一支是 admission 退 2「print(f"判不了 {arguments[1]} 的全绿标记：{error}")」（`research/scripts/admission.py:2311`），起因是 cargo -V、rustc -V、RUSTC 跑不出、登记行坏了。第 19 行把两支一起读：「退 1 是没过（或判不了），这一道与第 2b 步都不起（② 也不起）、不查标记，停下」（`.claude/skills/crash-test/commit-stages.md:19`），接着叫「照同一条找改那几条路径的会话协商，等它们暂存、提交或撤掉再起」（`.claude/skills/crash-test/commit-stages.md:19`）。判不了那一支没有别的会话可找，门禁自己的 → 写的是「看 .claude/gate.d/stage-inputs.tsv 里这一道那一行还在不在、cargo -V 与 rustc -V 跑不跑得出，修好再查」（`.claude/gate.d/checker-tier-broad-mutation.sh:687`）。量过（副本，PATH 去掉 cargo，`outputs/j1a-marker-check-no-cargo.txt`）：退 1，打的就是这一句。
- 四句：不分辨臂；系统看得到（两支打的 ✗ 不同）；属「一句话与今天的做法不符」；改法：第 19 行把判不了那一支单写，照门禁的 → 办。只改这一句。

### J1-b/c 字面值绿样本与 admission.py 共用的 build_environment_lines（打中）

字面值样本的独立性只到它的环境里那几类：「printf 'CARGO_HOME={root}/.broad-sample/cargo-home\nRUSTFLAGS=-C sample-literal\nRUSTDOCFLAGS=--cfg sample\n' > .broad-sample/environment」（`.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/library-version-is-staged-tree-literal-global-key-green/setup.sh:20`）。按全局键全文，它钉了四类：目录层级里的 .cargo/config.toml、CARGO_HOME 下的 config.toml、RUSTFLAGS、RUSTDOCFLAGS。别的 14 份样本的「那一版记的」由 prepare.py 调同一份 build_environment_of 现算，两边一起错。

把 admission.py 的 build_environment_lines 逐类写错（`j1c-admission-mutants.py`，每种只改 admission.py 一处），先看每种写错是不是真让一类构建环境从全局键里消失（`j1c-env-gap.py`：在一棵小树上逐类设变量或放文件，比构建环境变不变），再看 36 份样本翻不翻（`outputs/j1c-sweep.txt`）：

| 写错 | 哪一类变了而构建环境不变（`outputs/j1c-env-gap.txt`） | 36 份样本 |
|---|---|---|
| a1 CARGO_PROFILE_* 不进 | CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false | 判对 36、判错 0 |
| a2 _RUNNER 的值那一类不进 | 一类都没藏（指的程序那一行还在） | 判对 36、判错 0 |
| a3 rust-toolchain 文件不进 | 库目录上一层放 rust-toolchain.toml | 判对 36、判错 0 |
| a4 指的程序只记值 | RUSTC_WRAPPER 值不变、程序内容改了；配置里 build.rustc-wrapper 指的程序内容改了 | 判对 36、判错 0 |
| a5 不往上层走 | 库目录上一层的 rust-toolchain.toml、库目录那一层的 .cargo/config.toml（改 overflow-checks） | 判对 36、判错 0 |
| a6 RUSTC 不进 | RUSTC=rustc | 判对 36、判错 0 |
| a7 CARGO_ENCODED_RUSTFLAGS 不进 | CARGO_ENCODED_RUSTFLAGS=-Coverflow-checks=off | 判对 36、判错 0 |
| a8 配置里指的程序不进 | build.rustc-wrapper 指的程序内容改了 | 判对 36、判错 0 |
| a9（对照）RUSTFLAGS 不进 | — | 判对 35、判错 1（literal-global-key-green） |

admission 那几类的表头是「BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_PROFILE_"), re.compile(r"^CARGO_BUILD_"),」（`research/scripts/admission.py:269`）这一段。a1 正是第二轮 h1-5-env-gap 打中的那一类（CARGO_PROFILE_DEV_OVERFLOW_CHECKS），改法落地之后它只靠 admission 自己盯。

谁还盯这一份：
- 驱动 selftest_global_key_class_cells 逐类换的是六种：CARGO_TARGET_*_RUSTFLAGS、RUSTC_WRAPPER（从无到有）、CARGO_BUILD_TARGET、RUSTFLAGS 换值、RUSTDOCFLAGS、CARGO_HOME 下的配置。上表 a1、a3–a8 七种都不在里面。按代码推，驱动自证不翻（推的；拷贝上的驱动自证子 agent 被拒，写进 main-agent-heavy.sh）。
- admission.py 自己的 `--selftest` 有 skip-build-environment、runner-program-only 这几个开关，多半盯得住其中几种（推的，同样交主 agent 跑）。但它今天已是「283 格里 1 格判错」的旧红（实现员报告「逐条结果」第 5 条）。整轮门禁跑它，红→红不翻面；新写错只会把 1 格变成 2 格，要有人比格名才看得出。

四句：
- 分不分辨臂：不分臂，是第二轮第 6 行改法（共用一份）带出来的钉子问题。
- 系统看不看得到：看得到，j1c-env-gap 一条命令就分得出。
- 满足的分句：第三条分句「一个判红分支只有与被判代码共用同一段函数或常量的样本盯着、那一段写错时样本不翻面」。门禁第一格全局键不同那一支（`:372`）的样本，除字面值那一份，两边都经 build_environment_of 算。
- 改法中不中：判据的改法是「补一份不共用的样本或开关」。
  - 字面值样本的环境补上 a1、a3–a8 那几类：一个 CARGO_PROFILE_ 变量、CARGO_ENCODED_RUSTFLAGS、样本根放 rust-toolchain.toml、RUSTC 指一个打固定 -V 的假编译器、RUSTC_WRAPPER 指样本里的脚本、配置里 build.rustc-wrapper 指样本里的脚本，字面值照样在驱动之外现算。修样本这一侧。
  - 驱动自证逐类换补同样几类。修驱动这一侧。
  - 先把 admission 自证那一格旧红修绿，红→红的遮挡才去掉。
  - 只做其中一样，另两侧照中。

驱动合法加一类全局键行时字面值样本怎么跟（推的）：样本必红，修法只能重写那 16 行。最省事的是拷 prepare.py 现算出的 global-key.txt，那一次的独立性就没了。没有机制拦这一步，只能在样本 setup.sh 的注释里写明「十六进制在驱动之外现算」，交写样本的人守。

### J1-d `--unit-environment` 与 RUSTUP_TOOLCHAIN（小打中，条件式）

--unit-environment 只打「拿掉它构建环境就变」的变量：「base = build_environment_of(root, environment)」（`research/scripts/broad-mutation.py:3061`）。工具链不在构建环境里，在 toolchain.txt 与版本键里，由树里起的 rustc -Vv 定。会话 shell 设了 RUSTUP_TOOLCHAIN 时（副本，`outputs/j1d-rustup-toolchain.txt` 原样）：

```
不设 RUSTUP_TOOLCHAIN：rustc 1.98.0 (88d9e12ae 2026-08-18)
RUSTUP_TOOLCHAIN=nightly-x86_64-unknown-linux-gnu：rustc 1.100.0-nightly (330d31712 2026-09-17)
工具链变了；--unit-environment 认的变量：没有；带进单元的固定五个：SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME
```

整条流程：
- ② 在会话里，按 nightly 记这一版。
- ③ 在单元里，没有这个变量，起的是 1.98.0，退 7。
- commit-stages 第 28 行的出路是「会话 shell 里设的编译变量没照 --unit-environment 带进单元也是 7，带上重起 ③」，可 --unit-environment 一个都不打，照它做还是 7。

四句：
- 不分辨臂。
- 系统看得到：工具链是现算的，驱动退 7 时打的「记下的是…此刻是…」分得出。
- 属「不该红而红」，条件式：本机装了 5 个工具链；会话里设不设这个变量没量，推的。
- 改法：--unit-environment 另比一遍「拿掉它 toolchain_text 变不变」；或 detached-run 也带 RUSTUP_TOOLCHAIN。零轮。

## J3 第二条：前两轮「没打中」的格再抽一次

挑了两格，按「今天的改动最可能碰到的」挑：
- 第二轮 H1-7（结局组合下 --summary 两行与 gate-verdict 互核）：这一批给 --summary 加了 ESCALATED 与 escalated=、给 gate-verdict 加了登记表复核，正落在它上面。
- 第二轮 H2-b（弄坏开关翻面与文件头逐项相同）：开关从 20 个变 26 个。

H1-7 再抽：j3-combos.py 照第二轮 h1-7-combos.py 的造法，结局里加上升档判的抓到（结果行 caught、标签 caught-by-timeout / caught-by-memory），八成照 escalate_step 的写法造证据、两成不造（导入的那种）。门禁那段内嵌 python 原样抽出（`.claude/gate.d/checker-tier-broad-mutation.sh:399-624`）。判对的标准：每条有结论、没有没红、状态是升档抓到的条有这一版的登记行。原样（`outputs/j3-combos-400.txt`、`-5000.txt`）：

```
trials=400 seed=20261003
summary_exit=0 gate_exit=0 truth_green=True gate_says_inconsistent=False count=78
summary_exit=0 gate_exit=1 truth_green=False gate_says_inconsistent=False count=11
summary_exit=8 gate_exit=1 truth_green=False gate_says_inconsistent=False count=311
mismatches=0
trials=5000 seed=7
summary_exit=0 gate_exit=0 truth_green=True gate_says_inconsistent=False count=1001
summary_exit=0 gate_exit=1 truth_green=False gate_says_inconsistent=False count=162
summary_exit=8 gate_exit=1 truth_green=False gate_says_inconsistent=False count=3837
mismatches=0
```

`summary_exit=0 gate_exit=1` 那几趟是驱动 --summary 判绿、门禁因为登记行缺而判红，正是门禁该补的那一层。

H2-b 再抽：文件头 `:83-111` 列的 26 个开关逐个打开跑 36 份样本，每个开关判错的样本与文件头写的逐项相同，没有一个开关「判错 0 个」（`outputs/j3-switches.txt`，26 行）。两格这一次都没打中，连同第二轮那一次是两次抽样。

## 改法各修哪一格（全是零轮；「量过」= 在我的模型上实现并跑过，「推的」= 按代码推、没实现）

| 改法 | J2-a 超时基线漂移 | J2-a 撞顶峰值改写 | J2-a 盖过的旧行永远红 | 别的格 |
|---|---|---|---|---|
| F1 门禁超时基线认这一版这一步任何一行绿记录（登记的那一行在就对上） | 量过：不重判就绿 | 量过：照红 | — | 36 份样本照对（量过） |
| F2 复核不过的登记行只报不红，每条 ESCALATED 要有一行复核得过的 | 量过：重判之前红、照出路重判之后绿 | 推的：照出路重判之后绿 | 量过 | 36 份样本照对（量过）；放宽了门禁，登记表里的坏行不再挡提交 |
| 甲 基线按线程数分文件记 | 推的 | — | — | 驱动文件头 `:115` 那一句随之成立 |
| 乙 登记行另记基线那一行的 finished_at | 推的 | 推的：峰值表要另存一份只追加的历史 | — | — |
| J2-b 每支补一份红样本配一个开关（10 份现场量过原样红、删分支版绿；样本没做） | — | — | — | J2-b |
| J2-c 甲 撞顶 K 的基线峰值取回收不掉的那部分 | — | — | — | J2-c 第 1 条，推的 |
| J2-c 乙 升档上限截到 slice 放得下的、截了不到 K 倍当场记「证据造不出来」 | — | — | — | J2-c 第 2 条，推的 |
| J2-c 丙 升档那趟（与撞过顶的键）排队按上限算 | — | — | — | J2-c 第 2 条与旁支，推的（254 的机理量过） |
| J2-c 丁 撞顶的出路句写「降上限或等 slice 空」 | — | — | — | 门禁 `:132`、`:457`，驱动 `:134`，commit-stages `:28`，推的 |
| J2-d journal 核 scope 结局 | — | — | — | J2-d，推的（journal 里有这两类条目，量过） |
| J1-a 第 19 行把判不了那一支单写 | — | — | — | J1-a，推的 |
| J1-c 字面值样本补几类 / 驱动自证补几类 / admission 自证旧红修绿 | — | — | — | J1-c，推的 |
| J1-d --unit-environment 另比工具链 | — | — | — | J1-d，推的 |

F1、F2 是 `j2a-fix-prototypes.py` 在门禁拷贝上改的，原样（`outputs/j2a-fix-prototypes.txt`）：

```
== F1 36 份样本
判对 36 个、判错 0 个：无
   baseline-rerun：exit=0
   baseline-rerun-then-reescalate：exit=0
   peak-overwrite：exit=1
== F2 36 份样本
判对 36 个、判错 0 个：无
   baseline-rerun：exit=1
   baseline-rerun-then-reescalate：exit=0
   peak-overwrite：exit=1
== F12 36 份样本
判对 36 个、判错 0 个：无
   baseline-rerun：exit=0
   baseline-rerun-then-reescalate：exit=0
   peak-overwrite：exit=1
```

## 要主 agent 跑的命令

1. 拷贝上的 admission 与驱动自证（重型闸拒子 agent：仓副本上的 `--selftest` 不按登记表放行）。
   - 命令：`SINGLEFS_HEAVY_TESTS=user-request nice -n 19 bash research/prompts/m3-broad-gate-gov-r3-opus-model/main-agent-heavy.sh /tmp/claude-1000/m3-gov-r3-freeze/tree <草稿目录>`
   - 工作目录仓根；单线程逐份跑；每趟经 `run-with-memory-cap.sh 1G`。共 1 份原样、9 份 admission 写错版（各跑两样自证）、6 份驱动写错版（只跑驱动自证），估一趟 admission 自证一两分钟、驱动自证几分钟（推的，没量）。
   - 每行打「名字、admission 末行、驱动末行」。原样那一行应是「283 格里 1 格判错」与「74 格都对」。
   - 判读：j1c 的 a1、a3–a8 七种，哪一种两样都与原样同，就是谁都不盯它（样本那一侧已量过不翻）。j2d 的 d1 是对照、应红；d2–d6 哪一种「74 格都对」，驱动一侧就没人盯它。
2. 真仓复现 J2-a（要驱动真跑，重型，没做）：同一版先 `--threads A` 跑一条会升档超时的变异，再 `--threads B` 跑任何一条，看 `versions/<那一版>/baseline.tsv` 多没多出新绿行，④ 是否报「登记的基线 … 不同」。真库里升档要有确定性超时的变异，一般现场造不出；`outputs/j2a-baseline-probe.txt` 是只换 run_step 的同一段 ensure_baseline，可以只核这一份。

## 没打中的形状（各一次抽样，不支撑「没问题」）

- J2-e 假抓到，见上节：超时一边的热编基线与地板、nice 与 cgroup 分 CPU、MemorySwapMax=0 堵掉换页拖慢、撞顶一边 1.5 倍上限的窗口，四样。
- J1-a 路径集合：查标记、写标记、admission 指纹三处逐项相同。门禁 source 的 `.claude/scripts/preflight.sh` 不在登记行里，它的没暂存改动不挡 `--marker-check`，按代码看不影响判定，没造现场。git 的 assume-unchanged / skip-worktree 位与 clean 过滤器会让 `git diff` 看不见工作区改动，要有人故意设，推的、没造。
- J1-c --unit-environment 的集合：由「拿掉它构建环境变不变」现算，与全局键认的同一份，构造上相同。systemd 用户实例的环境（`systemctl --user show-environment` 的名字：HOME LANG LOGNAME PATH SHELL USER XDG_RUNTIME_DIR QT_ACCESSIBILITY XDG_DATA_DIRS DBUS_SESSION_BUS_ADDRESS GSM_SKIP_SSH_AGENT_WORKAROUND SSH_AUTH_SOCK）今天没有构建变量，「单元里多出会话没有的变量」那一形今天不成立。
- 测试运行时读的环境变量不进全局键：查了 harness 四个包。慢档用例的种子数是写死的常量（random_histories.rs 的 FAST_TIER_SEEDS 等）；按环境变量定规模的是 `#[ignore = "大档：…"]`，归排除表。没找到「会话变量改了杀伤力、全局键不变」的用例。
- J2 两台各跑一片：导入的结果行带升档标签、登记行与日志留在另一台，门禁判「没有这一版复核得过的那一行」，出路 `--item --rerun` 在本机重判走得通。推的，没造。
- J2 包装 252：升档那趟按头一趟上限排队，slice 忙时最多等 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（带前缀默认 3600 秒）再 252、没结论，只费时间。推的。
- 编译那一步两趟都超时记 caught-by-timeout，不记 unviable：门禁两样都算有结论，判定不受影响，只影响统计口径。
- 字面值样本依赖上层目录里有没有 .cargo/config、rust-toolchain：文件头已写，本机 /tmp、/tmp/claude-1000、~/.cargo 下都没有。
- `--marker-check` 退 0 / 3 与第 19 行的读法逐个对上。

## 这条腿自己的限度

- 驱动一次没真跑。驱动一侧的结论来自读代码，加一个只换 run_step 的 ensure_baseline 探针（j2a），加自证的格名。真包装只在总上限 ≤ 256M 的临时 slice 里起过 python 与 sleep。本机 40 GiB slice 上的数（4.1 GiB、26.7G、48G）是按代码与 `--status` 推的。
- 频率都没量：同一版里换 `--threads` 多常见、slice 平时占多少、真库撞顶升档的条有几条。真库这一版的结果行很少（第二轮判决第 4 行）。
- J2-e 引的基线与峰值是真库两个旧版与主仓峰值表（加升档之前的驱动写的），只当本机量级的参照。
- 拷贝上的自证（main-agent-heavy.sh）没跑：J1-c「谁都不盯」与 J2-b 驱动一侧，现在只到「样本不翻」，驱动、admission 自证那一侧是推的。
- 第一轮 G1–G5、第二轮 H1–H5 的题面没重攻。J3 只做了第二条，第一、三条与 J4 归辩方。

## 没做什么

- 重型一样没跑：驱动真跑、门禁整道、拷贝上的 admission / 驱动自证（写成 main-agent-heavy.sh）、gate.sh、cargo 一次没起。
- 没改仓里任何被判文件。改法都只在模型里的门禁拷贝上试（F1、F2），都是零轮。
- 没读禁读清单里的文件（别的腿报告与判决、第三轮辩方报告）。
- J2-a 真仓复现没做：真库里一般造不出确定性超时的变异。

## 草稿与清理

草稿目录 `/tmp/claude-1000/m3-gov-r3-attack/`。仓副本（`trees/` 与 `rerun/trees/`：原样、j2b 12 份、j1c 10 份、j2d 6 份、fix 3 份）与样本现场交回前删了；留下的是 `out/`、`rerun/out/`（每份样本的原样输出）与 j2c 的几个小临时目录，不含编译目录、工作树或仓副本。复跑时 rerun.sh 在给的草稿目录里重建。
