# 虚机 harness：怎么在真块设备上跑一个实验

**为什么单独一篇**：块设备行为是好几个实验的被测对象，而「怎么把实验送进虚机」
每次都被重新推导一遍，两次卡在同一处（内核镜像不可读、二进制不是静态的）。
本文只写现状，不写历史。

## 一句话

```bash
cd research
cargo build --release --target x86_64-unknown-linux-musl --bin <实验二进制名>
bash scripts/vm-bench.sh target/x86_64-unknown-linux-musl/release/<名> /dev/vda <参数...>
```

## 挂几块盘

`VM_DISKS`（默认 1）。>1 时来宾看到 `/dev/vda` `/dev/vdb` …，
**全部设备路径按顺序作为前缀参数传给二进制**，二进制自己的参数跟在后面。

```bash
VM_DISKS=4 VM_DISK_MB=64 bash scripts/vm-bench.sh <静态二进制>
```

多盘是 E53（丢一整块盘之后根环还挂不挂得上） 要的：「丢一整块盘之后还挂不挂得上」
只有在真的有多块盘时才问得出来。改完 harness **先跑 `--selftest`**，通过了才算数。

⚠️ **最小 initramfs 里没有 `dm-error` / `mdadm`**：
做不出「读返回 EIO」和「条带阵列」，只做得出「把某块盘的内容真的抹掉」。

## 三个前置，每个都有确定的处置

| 前置 | 怎么查 | 不满足怎么办 |
|---|---|---|
| **KVM 可读写** | `ls -l /dev/kvm` 加 `getfacl -p /dev/kvm` | 本机靠 **ACL**（`user:fy5090:rw-`）拿到权限，**不在 `kvm` 组也能用**。查组会得出错误结论 |
| **内核镜像可读** | `bash scripts/vm-kernel.sh --check` | 跑 `bash scripts/vm-kernel.sh`，它打印一个可用路径；必要时用 `.env` 里的口令从 `/boot` 复制一份到 `TMPDIR` 并 chown |
| **二进制是静态的** | `file <二进制>` | 用 `--target x86_64-unknown-linux-musl` 编。musl 目标已装（`rustup target list --installed`） |

⚠️ **本机 `/boot` 下只有一个可读内核：`vmlinuz-6.17.0-lockdep`**（其余是 `-rw-------`）。
**它带 lockdep**，锁校验开销很大 ⇒ **虚机里量到的时间不可与宿主比**，
但**计数类指标（I/O 次数、块数）不受影响**。引用虚机跑出来的时间数必须带这一句。

## 跑之前先做卫生检查

`vm-bench.sh` 现在带 `EXIT` 清理工作目录，但**开跑前仍然值得看一眼**
（这条清理是补上去的，补之前一次清出过 154 个目录、4.5 GB，详见文末历史版本）：

```bash
ps -eo pid,etimes,args --no-headers | grep '[q]emu-system'   # 残留虚机
ls -d /tmp/singlefs-vmbench.* 2>/dev/null | wc -l            # 残留工作目录
losetup -a                                                    # 残留 loop
```

⚠️ **不许用 `pkill -f`**（`.claude/singlefs-ai-sop/rules/command-safety.md`）：
模式串会命中 wrapper 自己的命令行。要停虚机就先 `ps` 看清楚，再用字面量 pid 杀。
`VM_KEEP=1` 可以保留工作目录供失败后翻现场。

## harness 自己要先自检

```bash
bash scripts/vm-bench.sh --selftest
```

它跑三个用例：必然成功的、必然失败的、以及查得到 `/dev/vda` 的。
**分辨不出失败的那个，说明这个 harness 会把失败当成成功**——
这与 `.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」同一条。

## 虚机里多了什么、宿主上没有的

| 量 | 宿主上跑（普通文件） | 虚机里跑（`/dev/vda`） |
|---|---|---|
| I/O 次数、块数 | 有 | 有 |
| **块层独立读数**（`/sys/block/vda/stat`） | **没有**（`blkstat=false`） | **有** |
| 时间 | 可比 | **不可比**（lockdep 内核） |

⚠️ **块层独立读数是好几个实验的校验路径**——
E7（离线索引 harness） 把「块层与程序计数器逐格相符」列为它四层校验之一，
E12（攒批的顺序追加 vs 不攒批的随机页读改写） 更是靠它做过一次故障注入自证
（摘掉 `O_DIRECT` 后块层读数归零而程序计数不变）。
⇒ **在普通文件上复跑这些实验，复现的是数，不是那条校验。** 引用时必须分开说。

## 结果抓取有一道完整性闸

`vm-bench.sh` 比对「程序声称发了几条」与「宿主抓到几条」，对不上整轮作废。
理由见 `.claude/singlefs-ai-sop/rules/command-safety.md`：控制台会被 BIOS 转义序列
顶掉行首，按 `^` 锚定会静默漏掉那一行。所以抓取用的是 `grep -ao`，不锚行首。

## 屏蔽 CPU 特性那一档另有一道闸：已知答案测试

`VM_CPU` 能让来宾看不见某个 CPU 特性（`VM_CPU="host,-aes,-vaes"`）。
**一旦 `VM_CPU` 里出现 `-<特性>`，`vm-bench.sh` 就要求被测程序发过至少一条
`E7RESULT name=kat ... ok=true`**，否则整轮作废。

**为什么要这道闸**（2026-08-31 实测踩过，E6（加密算法选型））：
`-cpu host,-aes` 只摘掉 CPUID 的 `aes` 位，而 RustCrypto 的 `aes` 0.9.3
**分别**探 `aes` 与 `vaes` 两个位（`src/lib.rs` 第 152–154 行）⇒ 来宾仍走 AES 指令：

| 观测 | `-aes` | `-aes,-vaes` |
|---|---|---|
| AES 4 KiB 吞吐 | 3747.63 MiB/s（与没屏蔽差 0.5%）| 244.74 MiB/s（掉 14.94×）|
| AES-256-GCM 的标签 | **`2112edc9…`，错的** | `62d27233…`，对的 |

⇒ **那一档跑得飞快、条数对得上、退出码 0，而算的不是那个算法。**
只看吞吐分不出「屏蔽没生效」和「屏蔽生效了但算错了」——两者都表现为「数字没变」。

**怎么写这条 kat**：期望值必须由**另一个实现**给出（例：`python3` 调 `cryptography`），
不许拿被测程序自己在别的档上的输出当期望值——那是自己和自己比。

**这道闸自己证过会红**（2026-08-31，三个用例）：屏蔽档不发 kat ⇒ 判红；
屏蔽档发 `ok=false` ⇒ 判红；不屏蔽且不发 kat ⇒ 放行。

## 历史版本

### 2026-08-29
- **曾经**：`vm-bench.sh` 在子 shell 里 `mktemp -d` 出工作目录，路径传不回父进程，
  于是从不清理。**现在**：用它已有的日志指针文件反推工作目录，在 `EXIT` 里删掉
  （只删名字符合 `singlefs-vmbench.*` 的那一个；`VM_KEEP=1` 保留）。
  **依据**：当日清出 **154 个残留目录、4.5 GB**；病根是
  `.claude/singlefs-ai-sop/rules/command-safety.md`「子 shell 里的赋值传不回父进程」。
- **新增** `scripts/vm-kernel.sh`：每次跑虚机都卡在「`/boot` 下的内核不可读」，
  做成脚本一次解决。**依据**：本机 `/boot/vmlinuz-*` 除 lockdep 那个外都是 `-rw-------`。
