//! E152（按里程碑对比六家文件系统的文件性能）：虚机里对一个配置跑一轮负载，结果行以 `E7RESULT` 打头；
//! 宿主上用 `summarize` 模式把多轮产物汇成中位、最小、最大与离散。
//!
//!   来宾：/bench <设备...> <配置> <轮次>        （vm-bench.sh 把设备路径排在参数前面）
//!   宿主：e152-file-system-benchmark summarize <产物>
//!
//! 负载、次序、判据与作废条款照跑前登记 `research/prompts/e152-preregistration.md`（含第十节的修订与第十一节的两盘镜像）逐条写；
//! 驱动在 `research/scripts/e152-run.sh`，附加根在 `research/scripts/e152-stage-root.sh`。
//! 这个二进制自己只判两件事：块层读数与 fio 读数对不对得上（登记第六节），以及同一个判定函数在对照上会不会红。

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

const MOUNT_POINT: &str = "/mnt";
const LARGE_FILE_PATH: &str = "/mnt/large";
const METADATA_DIRECTORY: &str = "/mnt/metadata";
const SMALL_FILE_DIRECTORY: &str = "/mnt/small";
const PERSIST_DIRECTORY: &str = "/mnt/persist";
const ZFS_POOL_NAME: &str = "bench";
const MODULE_ORDER_DIRECTORY: &str = "/lib/modules/e152";
const SINGLEFS_DEVICE_BINARY: &str = "/usr/bin/first_transaction_on_device";
/// 登记第十一节：md 配置的 raid1 阵列。
const SOFTWARE_RAID_DEVICE: &str = "/dev/md0";
/// `/sys/block/<名>/stat` 的扇区恒按 512 字节计（内核 `Documentation/block/stat.rst`），与设备的逻辑块大小无关。
const SECTOR_BYTES: u64 = 512;
/// 登记第四节第 2 步：100 个新文件，每个 3000 字节。
const PERSIST_FILE_COUNT: usize = 100;
const PERSIST_FILE_BYTES: usize = 3000;
/// 登记第六节：块层字节 < 0.9 × fio 报的字节 ⇒ 那一格不算。
const DIRECT_CHECK_NUMERATOR: u64 = 9;
const DIRECT_CHECK_DENOMINATOR: u64 = 10;
/// 登记第七节：离散 ≤ 15% 记「稳定」。
const STABLE_SPREAD_PERCENT: f64 = 15.0;
const DMESG_TAIL_LINES: usize = 30;
/// 三个元数据作业作用在同一批文件上：fio 默认按作业名给文件起名，三个作业名字不同就会各找各的（登记第十节修订二）。
const METADATA_FILENAME_FORMAT: &str = "--filename_format=metadata.$filenum";

/// 单盘七个配置（登记第二节）、两盘镜像七个配置（登记第十一节）、singlefs：封闭集合，加一种就得把每个 `match` 过一遍。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Configuration {
    RawDevice,
    FourthExtendedFileSystem,
    Xfs,
    FlashFriendlyFileSystem,
    Btrfs,
    Bcachefs,
    Zfs,
    RawSoftwareRaidMirror,
    FourthExtendedFileSystemOnSoftwareRaidMirror,
    XfsOnSoftwareRaidMirror,
    FlashFriendlyFileSystemOnSoftwareRaidMirror,
    BtrfsMirror,
    BcachefsTwoReplicas,
    ZfsMirror,
    Singlefs,
}

impl Configuration {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "raw" => Some(Self::RawDevice),
            "ext4" => Some(Self::FourthExtendedFileSystem),
            "xfs" => Some(Self::Xfs),
            "f2fs" => Some(Self::FlashFriendlyFileSystem),
            "btrfs" => Some(Self::Btrfs),
            "bcachefs" => Some(Self::Bcachefs),
            "zfs" => Some(Self::Zfs),
            "raw-md" => Some(Self::RawSoftwareRaidMirror),
            "ext4-md" => Some(Self::FourthExtendedFileSystemOnSoftwareRaidMirror),
            "xfs-md" => Some(Self::XfsOnSoftwareRaidMirror),
            "f2fs-md" => Some(Self::FlashFriendlyFileSystemOnSoftwareRaidMirror),
            "btrfs-raid1" => Some(Self::BtrfsMirror),
            "bcachefs-replicas2" => Some(Self::BcachefsTwoReplicas),
            "zfs-mirror" => Some(Self::ZfsMirror),
            "singlefs" => Some(Self::Singlefs),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::RawDevice => "raw",
            Self::FourthExtendedFileSystem => "ext4",
            Self::Xfs => "xfs",
            Self::FlashFriendlyFileSystem => "f2fs",
            Self::Btrfs => "btrfs",
            Self::Bcachefs => "bcachefs",
            Self::Zfs => "zfs",
            Self::RawSoftwareRaidMirror => "raw-md",
            Self::FourthExtendedFileSystemOnSoftwareRaidMirror => "ext4-md",
            Self::XfsOnSoftwareRaidMirror => "xfs-md",
            Self::FlashFriendlyFileSystemOnSoftwareRaidMirror => "f2fs-md",
            Self::BtrfsMirror => "btrfs-raid1",
            Self::BcachefsTwoReplicas => "bcachefs-replicas2",
            Self::ZfsMirror => "zfs-mirror",
            Self::Singlefs => "singlefs",
        }
    }

    /// singlefs 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9），两盘镜像的七个配置也是 2 块（登记第十一节），单盘七个配置各 1 块。
    fn device_count(self) -> usize {
        match self {
            Self::Singlefs
            | Self::RawSoftwareRaidMirror
            | Self::FourthExtendedFileSystemOnSoftwareRaidMirror
            | Self::XfsOnSoftwareRaidMirror
            | Self::FlashFriendlyFileSystemOnSoftwareRaidMirror
            | Self::BtrfsMirror
            | Self::BcachefsTwoReplicas
            | Self::ZfsMirror => 2,
            Self::RawDevice
            | Self::FourthExtendedFileSystem
            | Self::Xfs
            | Self::FlashFriendlyFileSystem
            | Self::Btrfs
            | Self::Bcachefs
            | Self::Zfs => 1,
        }
    }

    /// `research/scripts/e152-stage-root.sh` 写的模块次序文件名；ext4 编进内核，单盘裸盘与 singlefs 不要模块，md 配置都要 raid1。
    fn module_order_name(self) -> Option<&'static str> {
        match self {
            Self::Xfs => Some("xfs"),
            Self::FlashFriendlyFileSystem => Some("f2fs"),
            Self::Btrfs | Self::BtrfsMirror => Some("btrfs"),
            Self::Bcachefs | Self::BcachefsTwoReplicas => Some("bcachefs"),
            Self::Zfs | Self::ZfsMirror => Some("zfs"),
            Self::RawSoftwareRaidMirror | Self::FourthExtendedFileSystemOnSoftwareRaidMirror => Some("md"),
            Self::XfsOnSoftwareRaidMirror => Some("xfs-md"),
            Self::FlashFriendlyFileSystemOnSoftwareRaidMirror => Some("f2fs-md"),
            Self::RawDevice | Self::FourthExtendedFileSystem | Self::Singlefs => None,
        }
    }

    /// 登记第十一节：ext4 / XFS / F2FS 与裸阵列的镜像走 md raid1，另外三家用自己的镜像。
    fn uses_software_raid(self) -> bool {
        match self {
            Self::RawSoftwareRaidMirror
            | Self::FourthExtendedFileSystemOnSoftwareRaidMirror
            | Self::XfsOnSoftwareRaidMirror
            | Self::FlashFriendlyFileSystemOnSoftwareRaidMirror => true,
            Self::RawDevice
            | Self::FourthExtendedFileSystem
            | Self::Xfs
            | Self::FlashFriendlyFileSystem
            | Self::Btrfs
            | Self::Bcachefs
            | Self::Zfs
            | Self::BtrfsMirror
            | Self::BcachefsTwoReplicas
            | Self::ZfsMirror
            | Self::Singlefs => false,
        }
    }

    /// 裸盘配置 fio 直接打的块设备：单盘是那块盘，镜像是 md 阵列；别的配置没有。
    fn raw_target(self, device_paths: &[String]) -> Option<String> {
        match self {
            Self::RawDevice => device_paths.first().cloned(),
            Self::RawSoftwareRaidMirror => Some(SOFTWARE_RAID_DEVICE.to_string()),
            Self::FourthExtendedFileSystem
            | Self::Xfs
            | Self::FlashFriendlyFileSystem
            | Self::Btrfs
            | Self::Bcachefs
            | Self::Zfs
            | Self::FourthExtendedFileSystemOnSoftwareRaidMirror
            | Self::XfsOnSoftwareRaidMirror
            | Self::FlashFriendlyFileSystemOnSoftwareRaidMirror
            | Self::BtrfsMirror
            | Self::BcachefsTwoReplicas
            | Self::ZfsMirror
            | Self::Singlefs => None,
        }
    }

    /// 登记第二节与第十一节「格式化」那一列，参数除镜像本身之外一律默认；md 配置格式化的是建好的阵列。
    fn format_command(self, device_paths: &[String]) -> Option<CommandLine> {
        let first = device_paths.first().map_or("", String::as_str);
        let all_devices: Vec<&str> = device_paths.iter().map(String::as_str).collect();
        match self {
            Self::FourthExtendedFileSystem => Some(CommandLine::new("/usr/sbin/mkfs.ext4", &["-F", "-q", first])),
            Self::Xfs => Some(CommandLine::new("/usr/sbin/mkfs.xfs", &["-f", "-q", first])),
            Self::FlashFriendlyFileSystem => Some(CommandLine::new("/usr/sbin/mkfs.f2fs", &["-f", "-q", first])),
            Self::Btrfs => Some(CommandLine::new("/usr/sbin/mkfs.btrfs", &["-f", "-q", first])),
            Self::Bcachefs => Some(CommandLine::new("/usr/sbin/bcachefs", &["format", "-f", "-q", first])),
            // 给 zpool 整盘时它自己打 GPT、再等 udev 报分区就绪，initramfs 里没有 udev；先自己分好分区再交给它（登记第十节修订五）
            Self::Zfs | Self::ZfsMirror => Some(CommandLine {
                program: "/bin/sh",
                arguments: vec!["-c".to_string(), zfs_partition_and_create_script(device_paths)],
            }),
            Self::FourthExtendedFileSystemOnSoftwareRaidMirror => {
                Some(CommandLine::new("/usr/sbin/mkfs.ext4", &["-F", "-q", SOFTWARE_RAID_DEVICE]))
            }
            Self::XfsOnSoftwareRaidMirror => Some(CommandLine::new("/usr/sbin/mkfs.xfs", &["-f", "-q", SOFTWARE_RAID_DEVICE])),
            Self::FlashFriendlyFileSystemOnSoftwareRaidMirror => {
                Some(CommandLine::new("/usr/sbin/mkfs.f2fs", &["-f", "-q", SOFTWARE_RAID_DEVICE]))
            }
            Self::BtrfsMirror => Some(CommandLine::new(
                "/usr/sbin/mkfs.btrfs",
                &[&["-f", "-q", "-d", "raid1", "-m", "raid1"][..], &all_devices[..]].concat(),
            )),
            Self::BcachefsTwoReplicas => Some(CommandLine::new(
                "/usr/sbin/bcachefs",
                &[&["format", "-f", "-q", "--replicas=2"][..], &all_devices[..]].concat(),
            )),
            Self::RawDevice | Self::RawSoftwareRaidMirror | Self::Singlefs => None,
        }
    }

    /// 登记第二节与第十一节「挂载」那一列：只有 ext4 带一个挂载选项（不让 lazyinit 线程在量的时候后台写 inode 表）；
    /// btrfs raid1 用 device= 把两块盘都告诉内核，bcachefs 双副本把两块盘用冒号连起来。
    fn mount_command(self, device_paths: &[String]) -> Option<CommandLine> {
        let first = device_paths.first().map_or("", String::as_str);
        let btrfs_devices = device_paths.iter().map(|path| format!("device={path}")).collect::<Vec<_>>().join(",");
        let bcachefs_devices = device_paths.join(":");
        match self {
            Self::FourthExtendedFileSystem => {
                Some(CommandLine::new("/bin/mount", &["-t", "ext4", "-o", "noinit_itable", first, MOUNT_POINT]))
            }
            Self::FourthExtendedFileSystemOnSoftwareRaidMirror => Some(CommandLine::new(
                "/bin/mount",
                &["-t", "ext4", "-o", "noinit_itable", SOFTWARE_RAID_DEVICE, MOUNT_POINT],
            )),
            Self::Xfs => Some(CommandLine::new("/bin/mount", &["-t", "xfs", first, MOUNT_POINT])),
            Self::XfsOnSoftwareRaidMirror => Some(CommandLine::new("/bin/mount", &["-t", "xfs", SOFTWARE_RAID_DEVICE, MOUNT_POINT])),
            Self::FlashFriendlyFileSystem => Some(CommandLine::new("/bin/mount", &["-t", "f2fs", first, MOUNT_POINT])),
            Self::FlashFriendlyFileSystemOnSoftwareRaidMirror => {
                Some(CommandLine::new("/bin/mount", &["-t", "f2fs", SOFTWARE_RAID_DEVICE, MOUNT_POINT]))
            }
            Self::Btrfs => Some(CommandLine::new("/bin/mount", &["-t", "btrfs", first, MOUNT_POINT])),
            Self::BtrfsMirror => {
                Some(CommandLine::new("/bin/mount", &["-t", "btrfs", "-o", btrfs_devices.as_str(), first, MOUNT_POINT]))
            }
            Self::Bcachefs => Some(CommandLine::new("/bin/mount", &["-t", "bcachefs", first, MOUNT_POINT])),
            Self::BcachefsTwoReplicas => {
                Some(CommandLine::new("/bin/mount", &["-t", "bcachefs", bcachefs_devices.as_str(), MOUNT_POINT]))
            }
            Self::Zfs | Self::ZfsMirror => Some(CommandLine::new("/usr/sbin/zpool", &["import", "-d", "/dev", ZFS_POOL_NAME])),
            Self::RawDevice | Self::RawSoftwareRaidMirror | Self::Singlefs => None,
        }
    }

    fn unmount_command(self) -> Option<CommandLine> {
        match self {
            Self::FourthExtendedFileSystem
            | Self::Xfs
            | Self::FlashFriendlyFileSystem
            | Self::Btrfs
            | Self::Bcachefs
            | Self::FourthExtendedFileSystemOnSoftwareRaidMirror
            | Self::XfsOnSoftwareRaidMirror
            | Self::FlashFriendlyFileSystemOnSoftwareRaidMirror
            | Self::BtrfsMirror
            | Self::BcachefsTwoReplicas => Some(CommandLine::new("/bin/umount", &[MOUNT_POINT])),
            Self::Zfs | Self::ZfsMirror => Some(CommandLine::new("/usr/sbin/zpool", &["export", ZFS_POOL_NAME])),
            Self::RawDevice | Self::RawSoftwareRaidMirror | Self::Singlefs => None,
        }
    }

    /// `zpool create -m` 格式化完就挂上了，别的几家要另挂一次。
    fn format_also_mounts(self) -> bool {
        match self {
            Self::Zfs | Self::ZfsMirror => true,
            Self::RawDevice
            | Self::FourthExtendedFileSystem
            | Self::Xfs
            | Self::FlashFriendlyFileSystem
            | Self::Btrfs
            | Self::Bcachefs
            | Self::RawSoftwareRaidMirror
            | Self::FourthExtendedFileSystemOnSoftwareRaidMirror
            | Self::XfsOnSoftwareRaidMirror
            | Self::FlashFriendlyFileSystemOnSoftwareRaidMirror
            | Self::BtrfsMirror
            | Self::BcachefsTwoReplicas
            | Self::Singlefs => false,
        }
    }
}

/// 登记第十节修订五与第十一节：每块盘一张 GPT、一个从第 2048 扇区（1 MiB）起到盘尾的分区，再在这些分区上建池；两块盘时建 mirror。
fn zfs_partition_and_create_script(device_paths: &[String]) -> String {
    let partition_steps: Vec<String> = device_paths
        .iter()
        .map(|device_path| format!("printf 'label: gpt\\nstart=2048, type=L\\n' | /usr/sbin/sfdisk --quiet {device_path}"))
        .collect();
    let partitions: Vec<String> = device_paths.iter().map(|device_path| format!("{device_path}1")).collect();
    let layout = if partitions.len() > 1 { format!("mirror {}", partitions.join(" ")) } else { partitions.join(" ") };
    format!(
        "{} && /usr/sbin/zpool create -f -m {MOUNT_POINT} {ZFS_POOL_NAME} {layout}",
        partition_steps.join(" && ")
    )
}

/// 登记第十一节：两块盘都是新 fallocate 出来的全零镜像，本来就一致，--assume-clean 不让 md 在量的时候后台同步 16 GiB。
fn software_raid_creation_command(device_paths: &[String]) -> CommandLine {
    let mut arguments = vec!["--create", SOFTWARE_RAID_DEVICE, "--level=1", "--raid-devices=2", "--metadata=1.2", "--bitmap=none", "--assume-clean", "--run"];
    arguments.extend(device_paths.iter().map(String::as_str));
    CommandLine::new("/usr/sbin/mdadm", &arguments)
}

struct CommandLine {
    program: &'static str,
    arguments: Vec<String>,
}

impl CommandLine {
    fn new(program: &'static str, arguments: &[&str]) -> Self {
        Self {
            program,
            arguments: arguments.iter().map(|argument| (*argument).to_string()).collect(),
        }
    }
}

#[derive(Debug)]
struct GuestFailure {
    stage: &'static str,
    detail: String,
}

impl GuestFailure {
    fn new(stage: &'static str, detail: impl Into<String>) -> Self {
        Self { stage, detail: detail.into() }
    }
}

struct Emitter {
    emitted: u64,
    context: String,
}

impl Emitter {
    fn emit(&mut self, name: &str, body: &str) {
        self.emitted += 1;
        println!("E7RESULT name={name} {} {body}", self.context);
    }

    /// vm-bench.sh 的完整性闸认这一行：程序声称发了几条，宿主就得抓到几条。
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 结果行按空格分字段：把自由文本（报错、dmesg）压成一个不含空白的字段。
fn single_token(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join("_");
    if joined.is_empty() {
        "none".to_string()
    } else {
        joined.chars().take(400).collect()
    }
}

fn run_checked(command: &CommandLine, stage: &'static str) -> Result<std::process::Output, GuestFailure> {
    let output = Command::new(command.program)
        .args(&command.arguments)
        .output()
        .map_err(|error| GuestFailure::new(stage, format!("{} 起不来：{error}", command.program)))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(GuestFailure::new(
            stage,
            format!(
                "{} {} 退出 {:?}：{}",
                command.program,
                command.arguments.join(" "),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

/// `/sys/block/<名>/stat` 里这一轮要的五个计数（内核 `Documentation/block/stat.rst` 的字段序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockLayerCounters {
    read_requests: u64,
    read_sectors: u64,
    write_requests: u64,
    write_sectors: u64,
    flush_requests: u64,
}

/// 两次读数之差，扇区换成字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockLayerDelta {
    read_requests: u64,
    read_bytes: u64,
    write_requests: u64,
    write_bytes: u64,
    flush_requests: u64,
}

/// FLUSH 在下标 15（5.5 起才有），下标 14 是 discard 耗时；少于 16 个字段就拒绝，不拿 0 顶上（读不到 ≠ 读到 0）。
fn parse_block_layer_counters(text: &str) -> Option<BlockLayerCounters> {
    let fields = text
        .split_whitespace()
        .map(|field| field.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    Some(BlockLayerCounters {
        read_requests: *fields.first()?,
        read_sectors: *fields.get(2)?,
        write_requests: *fields.get(4)?,
        write_sectors: *fields.get(6)?,
        flush_requests: *fields.get(15)?,
    })
}

impl BlockLayerCounters {
    fn delta_since(self, earlier: Self) -> BlockLayerDelta {
        let difference = |later_value: u64, earlier_value: u64| {
            later_value
                .checked_sub(earlier_value)
                .expect("块层计数只增不减；变小说明两次读的不是同一块盘")
        };
        BlockLayerDelta {
            read_requests: difference(self.read_requests, earlier.read_requests),
            read_bytes: difference(self.read_sectors, earlier.read_sectors) * SECTOR_BYTES,
            write_requests: difference(self.write_requests, earlier.write_requests),
            write_bytes: difference(self.write_sectors, earlier.write_sectors) * SECTOR_BYTES,
            flush_requests: difference(self.flush_requests, earlier.flush_requests),
        }
    }
}

impl BlockLayerDelta {
    fn describe(&self) -> String {
        format!(
            "block_read_bytes={} block_read_requests={} block_write_bytes={} block_write_requests={} block_flush_requests={}",
            self.read_bytes, self.read_requests, self.write_bytes, self.write_requests, self.flush_requests
        )
    }
}

fn device_name(device_path: &str) -> Result<&str, GuestFailure> {
    Path::new(device_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| GuestFailure::new("device_name", format!("{device_path} 取不出设备名")))
}

fn read_block_layer_counters(device_path: &str) -> Result<BlockLayerCounters, GuestFailure> {
    let path = format!("/sys/block/{}/stat", device_name(device_path)?);
    let text = std::fs::read_to_string(&path).map_err(|error| GuestFailure::new("block_counters", format!("{path}：{error}")))?;
    parse_block_layer_counters(&text)
        .ok_or_else(|| GuestFailure::new("block_counters", format!("{path} 解析不出 16 个字段：{text}")))
}

impl BlockLayerCounters {
    /// 登记第十一节：两块成员盘的计数逐项相加。
    fn plus(self, other: Self) -> Self {
        Self {
            read_requests: self.read_requests + other.read_requests,
            read_sectors: self.read_sectors + other.read_sectors,
            write_requests: self.write_requests + other.write_requests,
            write_sectors: self.write_sectors + other.write_sectors,
            flush_requests: self.flush_requests + other.flush_requests,
        }
    }
}

/// 所有成员盘的计数相加：镜像下读会分到两块盘上，只看一块盘，第六节的校验会把读误判成被缓存顶替（登记第十一节）；
/// 单盘配置只有一块盘，相加等于不变。md 配置传进来的是成员盘，不数 /dev/md0（数了就重复）。
fn read_summed_block_layer_counters(device_paths: &[String]) -> Result<BlockLayerCounters, GuestFailure> {
    let mut summed = BlockLayerCounters { read_requests: 0, read_sectors: 0, write_requests: 0, write_sectors: 0, flush_requests: 0 };
    for device_path in device_paths {
        summed = summed.plus(read_block_layer_counters(device_path)?);
    }
    Ok(summed)
}

/// 登记第十一节的作废条款：`/proc/mdstat` 里出现 resync 或 recovery，说明阵列在后台同步，那一轮作废。
fn software_raid_is_synchronizing(mdstat_text: &str) -> bool {
    mdstat_text.contains("resync") || mdstat_text.contains("recovery")
}

fn check_software_raid_state(moment: &str, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let text = std::fs::read_to_string("/proc/mdstat").map_err(|error| GuestFailure::new("software_raid_state", error.to_string()))?;
    let synchronizing = software_raid_is_synchronizing(&text);
    emitter.emit(
        "software_raid_state",
        &format!("moment={moment} synchronizing={synchronizing} mdstat={}", single_token(&text)),
    );
    if synchronizing {
        return Err(GuestFailure::new("software_raid_synchronizing", single_token(&text)));
    }
    Ok(())
}

fn read_device_bytes(device_path: &str) -> Result<u64, GuestFailure> {
    let path = format!("/sys/block/{}/size", device_name(device_path)?);
    let text = std::fs::read_to_string(&path).map_err(|error| GuestFailure::new("device_size", format!("{path}：{error}")))?;
    let sectors = text
        .trim()
        .parse::<u64>()
        .map_err(|error| GuestFailure::new("device_size", format!("{path}：{error}")))?;
    Ok(sectors * SECTOR_BYTES)
}

/// fio 3.36 的 terse 第 3 版一行，只取这一轮要的几个字段（字段号以同一版 fio 的 JSON 输出对过）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FioJobReport {
    error_code: u64,
    read_kibibytes: u64,
    read_bandwidth_kibibytes_per_second: u64,
    read_operations_per_second: u64,
    read_completion_latency_percentile_99_microseconds: Option<u64>,
    write_kibibytes: Option<u64>,
    write_bandwidth_kibibytes_per_second: Option<u64>,
    write_operations_per_second: Option<u64>,
    write_completion_latency_percentile_99_microseconds: Option<u64>,
}

/// 字段号从 1 数：5 error、6 读 KiB、7 读带宽、8 读每秒操作数、30 读完成延迟 p99、47 写 KiB、48 写带宽、49 写每秒操作数、71 写 p99。
/// 没有盘的引擎（filecreate 这类）只有 121 个字段，写那一侧照样在前 71 个里。
fn parse_fio_terse_version_3(line: &str) -> Option<FioJobReport> {
    let fields: Vec<&str> = line.trim_end().split(';').collect();
    if fields.first() != Some(&"3") {
        return None;
    }
    let number = |position_from_one: usize| -> Option<u64> { fields.get(position_from_one - 1)?.parse::<u64>().ok() };
    let percentile_99 = |position_from_one: usize| -> Option<u64> {
        fields
            .get(position_from_one - 1)?
            .strip_prefix("99.000000%=")?
            .parse::<u64>()
            .ok()
    };
    Some(FioJobReport {
        error_code: number(5)?,
        read_kibibytes: number(6)?,
        read_bandwidth_kibibytes_per_second: number(7)?,
        read_operations_per_second: number(8)?,
        read_completion_latency_percentile_99_microseconds: percentile_99(30),
        write_kibibytes: number(47),
        write_bandwidth_kibibytes_per_second: number(48),
        write_operations_per_second: number(49),
        write_completion_latency_percentile_99_microseconds: percentile_99(71),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirectInputOutputCheck {
    DirectRead,
    DirectWrite,
    NotChecked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellVerdict {
    Counted,
    CacheSubstituted,
    WritesAbsorbed,
    NotChecked,
    FioError,
}

impl CellVerdict {
    fn label(self) -> &'static str {
        match self {
            Self::Counted => "counted",
            Self::CacheSubstituted => "cache_substituted",
            Self::WritesAbsorbed => "writes_absorbed",
            Self::NotChecked => "not_checked",
            Self::FioError => "fio_error",
        }
    }
}

/// 登记第六节：块层字节 < 0.9 × fio 报的字节 ⇒ 那一格不算。对照的第二遍也走这个函数，它必须判「被缓存顶替」。
fn judge_direct_input_output(check: DirectInputOutputCheck, fio_bytes: u64, block_bytes: u64) -> CellVerdict {
    let below_threshold = block_bytes * DIRECT_CHECK_DENOMINATOR < fio_bytes * DIRECT_CHECK_NUMERATOR;
    match check {
        DirectInputOutputCheck::DirectRead if below_threshold => CellVerdict::CacheSubstituted,
        DirectInputOutputCheck::DirectWrite if below_threshold => CellVerdict::WritesAbsorbed,
        DirectInputOutputCheck::DirectRead | DirectInputOutputCheck::DirectWrite => CellVerdict::Counted,
        DirectInputOutputCheck::NotChecked => CellVerdict::NotChecked,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FioJob {
    WriteSequentialLarge,
    ReadSequentialLarge,
    ReadRandomFourKibibytes,
    ControlBufferedReadFirstPass,
    ControlBufferedReadSecondPass,
    WriteRandomFourKibibytes,
    MetadataCreate,
    MetadataFileStatus,
    MetadataDelete,
    SmallFileWriteWithFsync,
}

fn words(first: String, rest: &[&str]) -> Vec<String> {
    std::iter::once(first)
        .chain(rest.iter().map(|word| (*word).to_string()))
        .collect()
}

impl FioJob {
    fn label(self) -> &'static str {
        match self {
            Self::WriteSequentialLarge => "write_sequential_large",
            Self::ReadSequentialLarge => "read_sequential_large",
            Self::ReadRandomFourKibibytes => "read_random_4k",
            Self::ControlBufferedReadFirstPass => "control_buffered_read_first_pass",
            Self::ControlBufferedReadSecondPass => "control_buffered_read_second_pass",
            Self::WriteRandomFourKibibytes => "write_random_4k",
            Self::MetadataCreate => "metadata_create",
            Self::MetadataFileStatus => "metadata_stat",
            Self::MetadataDelete => "metadata_delete",
            Self::SmallFileWriteWithFsync => "small_file_write_fsync",
        }
    }

    fn check(self) -> DirectInputOutputCheck {
        match self {
            Self::WriteSequentialLarge | Self::WriteRandomFourKibibytes => DirectInputOutputCheck::DirectWrite,
            Self::ReadSequentialLarge | Self::ReadRandomFourKibibytes | Self::ControlBufferedReadSecondPass => {
                DirectInputOutputCheck::DirectRead
            }
            Self::ControlBufferedReadFirstPass
            | Self::MetadataCreate
            | Self::MetadataFileStatus
            | Self::MetadataDelete
            | Self::SmallFileWriteWithFsync => DirectInputOutputCheck::NotChecked,
        }
    }

    /// 参数逐字照登记第四节（第 8 步按第十节修订二）。
    fn arguments(self, target: &str) -> Vec<String> {
        let file = format!("--filename={target}");
        let directory = format!("--directory={target}");
        match self {
            Self::WriteSequentialLarge => words(
                file,
                &["--rw=write", "--bs=1M", "--size=4G", "--ioengine=libaio", "--iodepth=16", "--direct=1", "--end_fsync=1"],
            ),
            Self::ReadSequentialLarge => {
                words(file, &["--rw=read", "--bs=1M", "--size=4G", "--ioengine=libaio", "--iodepth=16", "--direct=1"])
            }
            Self::ReadRandomFourKibibytes => words(
                file,
                &[
                    "--rw=randread",
                    "--bs=4k",
                    "--size=4G",
                    "--ioengine=libaio",
                    "--iodepth=32",
                    "--direct=1",
                    "--time_based",
                    "--runtime=20",
                ],
            ),
            // fio 默认 invalidate=1，每个作业开始前先丢掉文件的页缓存，第二遍就读不到缓存（登记第十节修订四）
            Self::ControlBufferedReadFirstPass | Self::ControlBufferedReadSecondPass => {
                words(file, &["--rw=read", "--bs=1M", "--size=256M", "--ioengine=psync", "--invalidate=0"])
            }
            Self::WriteRandomFourKibibytes => words(
                file,
                &[
                    "--rw=randwrite",
                    "--bs=4k",
                    "--size=4G",
                    "--ioengine=libaio",
                    "--iodepth=32",
                    "--direct=1",
                    "--time_based",
                    "--runtime=20",
                    "--end_fsync=1",
                ],
            ),
            Self::MetadataCreate => words(
                directory,
                &[
                    "--ioengine=filecreate",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::MetadataFileStatus => words(
                directory,
                &[
                    "--ioengine=filestat",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::MetadataDelete => words(
                directory,
                &[
                    "--ioengine=filedelete",
                    "--nrfiles=10000",
                    "--filesize=4k",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--create_on_open=1",
                    METADATA_FILENAME_FORMAT,
                ],
            ),
            Self::SmallFileWriteWithFsync => words(
                directory,
                &[
                    "--ioengine=psync",
                    "--rw=write",
                    "--bs=4k",
                    "--filesize=4k",
                    "--nrfiles=2000",
                    "--openfiles=1",
                    "--file_service_type=sequential",
                    "--fsync=1",
                    "--create_on_open=1",
                ],
            ),
        }
    }
}

fn optional_number(value: Option<u64>) -> String {
    value.map_or_else(|| "NA".to_string(), |number| number.to_string())
}

fn run_fio_job(job: FioJob, target: &str, device_paths: &[String], emitter: &mut Emitter) -> Result<CellVerdict, GuestFailure> {
    let before = read_summed_block_layer_counters(device_paths)?;
    let started = Instant::now();
    let output = Command::new("/usr/bin/fio")
        .args(["--thread", "--output-format=terse", "--terse-version=3"])
        .arg(format!("--name={}", job.label()))
        .args(job.arguments(target))
        .output()
        .map_err(|error| GuestFailure::new("fio_start", format!("{}：{error}", job.label())))?;
    let elapsed_nanoseconds = started.elapsed().as_nanos();
    let after = read_summed_block_layer_counters(device_paths)?;
    let standard_output = String::from_utf8_lossy(&output.stdout);
    let report = standard_output.lines().find_map(parse_fio_terse_version_3).ok_or_else(|| {
        GuestFailure::new(
            "fio_output",
            format!(
                "{} 没有 terse 行；退出 {:?}；{}",
                job.label(),
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
    })?;
    let delta = after.delta_since(before);
    let verdict = if report.error_code != 0 {
        CellVerdict::FioError
    } else {
        let (fio_bytes, block_bytes) = match job.check() {
            DirectInputOutputCheck::DirectRead => (report.read_kibibytes * 1024, delta.read_bytes),
            DirectInputOutputCheck::DirectWrite => {
                let write_kibibytes = report
                    .write_kibibytes
                    .ok_or_else(|| GuestFailure::new("fio_output", format!("{} 的 terse 行没有写那一侧", job.label())))?;
                (write_kibibytes * 1024, delta.write_bytes)
            }
            DirectInputOutputCheck::NotChecked => (0, 0),
        };
        judge_direct_input_output(job.check(), fio_bytes, block_bytes)
    };
    emitter.emit(
        "fio",
        &format!(
            "job={} error={} read_kibibytes={} read_bandwidth_kibibytes_per_second={} read_operations_per_second={} read_percentile_99_microseconds={} write_kibibytes={} write_bandwidth_kibibytes_per_second={} write_operations_per_second={} write_percentile_99_microseconds={} elapsed_nanoseconds={elapsed_nanoseconds} {} verdict={}",
            job.label(),
            report.error_code,
            report.read_kibibytes,
            report.read_bandwidth_kibibytes_per_second,
            report.read_operations_per_second,
            optional_number(report.read_completion_latency_percentile_99_microseconds),
            optional_number(report.write_kibibytes),
            optional_number(report.write_bandwidth_kibibytes_per_second),
            optional_number(report.write_operations_per_second),
            optional_number(report.write_completion_latency_percentile_99_microseconds),
            delta.describe(),
            verdict.label()
        ),
    );
    Ok(verdict)
}

/// 登记第六节的自证：同一段先不带 direct 读一遍，再读一遍；第二遍块层几乎不读，判定函数必须判它「被缓存顶替」。
fn run_control_pair(target: &str, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    // 两遍之间一直开着目标：块设备最后一个打开者关闭时内核丢掉它的页缓存，两个 fio 进程各开各关，裸盘的第二遍就读不到缓存（登记第十节修订六）
    let target_held_open =
        std::fs::File::open(target).map_err(|error| GuestFailure::new("control_hold_open", format!("{target}：{error}")))?;
    run_fio_job(FioJob::ControlBufferedReadFirstPass, target, device_paths, emitter)?;
    let second_pass = run_fio_job(FioJob::ControlBufferedReadSecondPass, target, device_paths, emitter)?;
    drop(target_held_open);
    let control_has_teeth = second_pass == CellVerdict::CacheSubstituted;
    emitter.emit(
        "control",
        &format!("second_pass_verdict={} control_has_teeth={control_has_teeth}", second_pass.label()),
    );
    Ok(())
}

struct PersistMeasurement {
    latency_nanoseconds: u128,
    delta: BlockLayerDelta,
}

#[derive(Debug, PartialEq, Eq)]
struct PersistSummary {
    operations: usize,
    median_latency_nanoseconds: u128,
    percentile_99_latency_nanoseconds: u128,
    mean_write_requests_thousandths: u64,
    mean_write_bytes: u64,
    mean_flush_requests_thousandths: u64,
    first_latency_nanoseconds: u128,
    first_write_requests: u64,
    first_write_bytes: u64,
    first_flush_requests: u64,
}

/// 最近秩分位：秩 = ⌈p × n ÷ 100⌉，至少 1。
fn nearest_rank_percentile(sorted_values: &[u128], percentile: u64) -> u128 {
    assert!(!sorted_values.is_empty(), "空样本没有分位数");
    let count = u64::try_from(sorted_values.len()).expect("样本数装得进 u64");
    let rank = (percentile * count).div_ceil(100).max(1);
    sorted_values[usize::try_from(rank - 1).expect("名次装得进 usize")]
}

fn summarize_persist(measurements: &[PersistMeasurement]) -> PersistSummary {
    assert!(!measurements.is_empty(), "一次都没量就没有汇总");
    let mut latencies: Vec<u128> = measurements.iter().map(|measurement| measurement.latency_nanoseconds).collect();
    latencies.sort_unstable();
    let count = u64::try_from(measurements.len()).expect("样本数装得进 u64");
    let total = |pick: fn(&BlockLayerDelta) -> u64| -> u64 {
        measurements.iter().map(|measurement| pick(&measurement.delta)).sum()
    };
    let first = &measurements[0];
    PersistSummary {
        operations: measurements.len(),
        median_latency_nanoseconds: nearest_rank_percentile(&latencies, 50),
        percentile_99_latency_nanoseconds: nearest_rank_percentile(&latencies, 99),
        mean_write_requests_thousandths: total(|delta| delta.write_requests) * 1000 / count,
        mean_write_bytes: total(|delta| delta.write_bytes) / count,
        mean_flush_requests_thousandths: total(|delta| delta.flush_requests) * 1000 / count,
        first_latency_nanoseconds: first.latency_nanoseconds,
        first_write_requests: first.delta.write_requests,
        first_write_bytes: first.delta.write_bytes,
        first_flush_requests: first.delta.flush_requests,
    }
}

impl PersistSummary {
    fn describe(&self) -> String {
        format!(
            "operations={} file_bytes={PERSIST_FILE_BYTES} median_latency_nanoseconds={} percentile_99_latency_nanoseconds={} mean_write_requests_thousandths={} mean_write_bytes={} mean_flush_requests_thousandths={} first_latency_nanoseconds={} first_write_requests={} first_write_bytes={} first_flush_requests={}",
            self.operations,
            self.median_latency_nanoseconds,
            self.percentile_99_latency_nanoseconds,
            self.mean_write_requests_thousandths,
            self.mean_write_bytes,
            self.mean_flush_requests_thousandths,
            self.first_latency_nanoseconds,
            self.first_write_requests,
            self.first_write_bytes,
            self.first_flush_requests
        )
    }
}

fn persist_file_content() -> Vec<u8> {
    (0..PERSIST_FILE_BYTES)
        .map(|position| u8::try_from(position % 251).expect("对 251 取余一定小于 256"))
        .collect()
}

/// 登记第四节第 2 步：刚格式化的空文件系统上，每次「新建、写 3000 字节、fsync 文件、fsync 目录」单独计时、单独读块层计数。
fn run_persist_workload(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let failure = |stage: &'static str| move |error: std::io::Error| GuestFailure::new(stage, error.to_string());
    std::fs::create_dir_all(PERSIST_DIRECTORY).map_err(failure("persist_directory"))?;
    run_checked(&CommandLine::new("/bin/sync", &[]), "sync_before_persist")?;
    let directory_handle = std::fs::File::open(PERSIST_DIRECTORY).map_err(failure("persist_directory_open"))?;
    let content = persist_file_content();
    let mut measurements = Vec::with_capacity(PERSIST_FILE_COUNT);
    for index in 0..PERSIST_FILE_COUNT {
        let path = Path::new(PERSIST_DIRECTORY).join(format!("file_{index:03}"));
        let before = read_summed_block_layer_counters(device_paths)?;
        let started = Instant::now();
        {
            let mut file = std::fs::File::create(&path).map_err(failure("persist_create"))?;
            file.write_all(&content).map_err(failure("persist_write"))?;
            file.sync_all().map_err(failure("persist_fsync_file"))?;
        }
        directory_handle.sync_all().map_err(failure("persist_fsync_directory"))?;
        let latency_nanoseconds = started.elapsed().as_nanos();
        let after = read_summed_block_layer_counters(device_paths)?;
        measurements.push(PersistMeasurement {
            latency_nanoseconds,
            delta: after.delta_since(before),
        });
    }
    emitter.emit("persist", &summarize_persist(&measurements).describe());
    Ok(())
}

/// busybox `stat -f -c '%b %a %S'`：总块数、非特权可用块数、块大小（f_frsize）。
fn parse_file_system_capacity(text: &str) -> Option<(u64, u64)> {
    let numbers = text
        .split_whitespace()
        .map(|word| word.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    match numbers.as_slice() {
        [total_blocks, available_blocks, block_bytes] => Some((total_blocks * block_bytes, available_blocks * block_bytes)),
        _ => None,
    }
}

fn emit_capacity(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let output = run_checked(&CommandLine::new("/bin/stat", &["-f", "-c", "%b %a %S", MOUNT_POINT]), "statfs")?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (total_bytes, available_bytes) = parse_file_system_capacity(&text)
        .ok_or_else(|| GuestFailure::new("statfs", format!("解析不出三个数：{text}")))?;
    // 登记第十一节：分母是所有成员盘的总字节，镜像配置与 singlefs 的「两盘镜像后是总量的 47.6%」同口径
    let mut disk_bytes = 0;
    for device_path in device_paths {
        disk_bytes += read_device_bytes(device_path)?;
    }
    emitter.emit(
        "capacity",
        &format!("total_bytes={total_bytes} available_bytes={available_bytes} disk_bytes={disk_bytes}"),
    );
    Ok(())
}

struct TimedCommandOutcome {
    elapsed_nanoseconds: u128,
    delta: BlockLayerDelta,
}

/// 一串命令合起来计时、合起来读块层计数（md 配置的格式化 = 建阵列 + 格式化，登记第十一节）。
fn run_timed_checked(commands: &[CommandLine], stage: &'static str, device_paths: &[String]) -> Result<TimedCommandOutcome, GuestFailure> {
    let before = read_summed_block_layer_counters(device_paths)?;
    let started = Instant::now();
    for command in commands {
        run_checked(command, stage)?;
    }
    let elapsed_nanoseconds = started.elapsed().as_nanos();
    let after = read_summed_block_layer_counters(device_paths)?;
    Ok(TimedCommandOutcome {
        elapsed_nanoseconds,
        delta: after.delta_since(before),
    })
}

fn drop_page_cache() -> Result<(), GuestFailure> {
    run_checked(&CommandLine::new("/bin/sync", &[]), "sync_before_drop_caches")?;
    std::fs::write("/proc/sys/vm/drop_caches", "3").map_err(|error| GuestFailure::new("drop_caches", error.to_string()))
}

/// 登记第四节第 4、5 步：卸载、清缓存、再挂（zfs 是 export 再 import，ARC 跟着池一起走）。
fn remount(configuration: Configuration, device_paths: &[String]) -> Result<(), GuestFailure> {
    let unmount = configuration.unmount_command().expect("六家文件系统每一家都有卸载命令");
    run_checked(&unmount, "unmount")?;
    drop_page_cache()?;
    let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
    run_checked(&mount, "remount")?;
    Ok(())
}

fn run_file_system_round(configuration: Configuration, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let format = configuration.format_command(device_paths).expect("六家文件系统每一家都有格式化命令");
    // 登记第十一节：md 配置的格式化从建阵列开始算
    let mut preparation = Vec::new();
    if configuration.uses_software_raid() {
        preparation.push(software_raid_creation_command(device_paths));
    }
    preparation.push(format);
    let formatted = run_timed_checked(&preparation, "format", device_paths)?;
    emitter.emit(
        "format",
        &format!("elapsed_nanoseconds={} {}", formatted.elapsed_nanoseconds, formatted.delta.describe()),
    );
    if configuration.uses_software_raid() {
        check_software_raid_state("after_create", emitter)?;
    }
    if !configuration.format_also_mounts() {
        let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
        let mounted = run_timed_checked(std::slice::from_ref(&mount), "first_mount", device_paths)?;
        emitter.emit(
            "first_mount",
            &format!("elapsed_nanoseconds={} {}", mounted.elapsed_nanoseconds, mounted.delta.describe()),
        );
    }
    emit_capacity(device_paths, emitter)?;
    run_persist_workload(device_paths, emitter)?;
    run_fio_job(FioJob::WriteSequentialLarge, LARGE_FILE_PATH, device_paths, emitter)?;
    remount(configuration, device_paths)?;
    run_fio_job(FioJob::ReadSequentialLarge, LARGE_FILE_PATH, device_paths, emitter)?;
    remount(configuration, device_paths)?;
    run_fio_job(FioJob::ReadRandomFourKibibytes, LARGE_FILE_PATH, device_paths, emitter)?;
    run_control_pair(LARGE_FILE_PATH, device_paths, emitter)?;
    run_fio_job(FioJob::WriteRandomFourKibibytes, LARGE_FILE_PATH, device_paths, emitter)?;
    std::fs::create_dir_all(METADATA_DIRECTORY).map_err(|error| GuestFailure::new("metadata_directory", error.to_string()))?;
    run_fio_job(FioJob::MetadataCreate, METADATA_DIRECTORY, device_paths, emitter)?;
    run_fio_job(FioJob::MetadataFileStatus, METADATA_DIRECTORY, device_paths, emitter)?;
    run_fio_job(FioJob::MetadataDelete, METADATA_DIRECTORY, device_paths, emitter)?;
    std::fs::create_dir_all(SMALL_FILE_DIRECTORY).map_err(|error| GuestFailure::new("small_file_directory", error.to_string()))?;
    run_fio_job(FioJob::SmallFileWriteWithFsync, SMALL_FILE_DIRECTORY, device_paths, emitter)?;
    // 登记第四节第 9 步：冷挂载
    let unmount = configuration.unmount_command().expect("六家文件系统每一家都有卸载命令");
    run_checked(&unmount, "unmount_before_cold_mount")?;
    drop_page_cache()?;
    let mount = configuration.mount_command(device_paths).expect("六家文件系统每一家都有挂载命令");
    let mounted = run_timed_checked(std::slice::from_ref(&mount), "cold_mount", device_paths)?;
    emitter.emit(
        "cold_mount",
        &format!("elapsed_nanoseconds={} {}", mounted.elapsed_nanoseconds, mounted.delta.describe()),
    );
    if configuration.uses_software_raid() {
        check_software_raid_state("end_of_round", emitter)?;
    }
    Ok(())
}

/// 裸盘只跑登记第四节第 3、4、5、6、7 步，fio 直接打块设备，不重挂；镜像的裸阵列先建 md（登记第十一节）。
fn run_raw_device_round(configuration: Configuration, device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    if configuration.uses_software_raid() {
        run_checked(&software_raid_creation_command(device_paths), "software_raid_create")?;
        check_software_raid_state("after_create", emitter)?;
    }
    let target = configuration.raw_target(device_paths).expect("裸盘配置都有 fio 直接打的块设备");
    run_fio_job(FioJob::WriteSequentialLarge, &target, device_paths, emitter)?;
    run_fio_job(FioJob::ReadSequentialLarge, &target, device_paths, emitter)?;
    run_fio_job(FioJob::ReadRandomFourKibibytes, &target, device_paths, emitter)?;
    run_control_pair(&target, device_paths, emitter)?;
    run_fio_job(FioJob::WriteRandomFourKibibytes, &target, device_paths, emitter)?;
    if configuration.uses_software_raid() {
        check_software_raid_state("end_of_round", emitter)?;
    }
    Ok(())
}

/// 子进程的结果行改名转发：`name=` 换成 `inner=`，免得子进程的 `name=done` 被 vm-bench.sh 当成本进程的收尾行。
fn reemit_inner_line(at_nanoseconds: u128, inner_body: &str) -> String {
    let without_name = inner_body.strip_prefix("name=").unwrap_or(inner_body);
    format!("at_nanoseconds={at_nanoseconds} inner={without_name}")
}

/// 登记第四节 singlefs 那一段：门禁 55 号那个真设备二进制当子进程跑，按结果行到达的时刻切两段挂钟。
fn run_singlefs(device_paths: &[String], emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let before = device_paths
        .iter()
        .map(|path| read_block_layer_counters(path))
        .collect::<Result<Vec<_>, _>>()?;
    let started = Instant::now();
    let mut child = Command::new(SINGLEFS_DEVICE_BINARY)
        .args(device_paths)
        .arg("direct")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| GuestFailure::new("singlefs_start", error.to_string()))?;
    let child_output = child.stdout.take().expect("刚用 Stdio::piped() 起的子进程一定有 stdout");
    let mut geometry_at = None;
    let mut transaction_at = None;
    let mut recovery_at = None;
    let mut content_matches = false;
    for line in BufReader::new(child_output).lines() {
        let line = line.map_err(|error| GuestFailure::new("singlefs_read", error.to_string()))?;
        let at_nanoseconds = started.elapsed().as_nanos();
        let Some(inner_body) = line.strip_prefix("E7RESULT ") else {
            continue;
        };
        if inner_body.starts_with("name=geometry ") {
            geometry_at = Some(at_nanoseconds);
        }
        if inner_body.starts_with("name=transaction ") {
            transaction_at = Some(at_nanoseconds);
        }
        if inner_body.starts_with("name=recover_cold ") {
            recovery_at = Some(at_nanoseconds);
            content_matches = inner_body.contains("content_matches=true");
        }
        emitter.emit("singlefs_inner", &reemit_inner_line(at_nanoseconds, inner_body));
    }
    let status = child.wait().map_err(|error| GuestFailure::new("singlefs_wait", error.to_string()))?;
    let after = device_paths
        .iter()
        .map(|path| read_block_layer_counters(path))
        .collect::<Result<Vec<_>, _>>()?;
    let per_device: Vec<String> = after
        .iter()
        .zip(&before)
        .enumerate()
        .map(|(index, (after_counters, before_counters))| {
            let delta = after_counters.delta_since(*before_counters);
            format!(
                "device_{index}_read_bytes={} device_{index}_read_requests={} device_{index}_write_bytes={} device_{index}_write_requests={} device_{index}_flush_requests={}",
                delta.read_bytes, delta.read_requests, delta.write_bytes, delta.write_requests, delta.flush_requests
            )
        })
        .collect();
    let phase = |later: Option<u128>, earlier: Option<u128>| match (later, earlier) {
        (Some(later_value), Some(earlier_value)) => (later_value - earlier_value).to_string(),
        (_, _) => "NA".to_string(),
    };
    emitter.emit(
        "singlefs_timing",
        &format!(
            "write_path_nanoseconds={} recovery_nanoseconds={} child_exit={} content_matches={content_matches} {}",
            phase(transaction_at, geometry_at),
            phase(recovery_at, transaction_at),
            status.code().map_or_else(|| "signal".to_string(), |code| code.to_string()),
            per_device.join(" ")
        ),
    );
    if !status.success() || !content_matches {
        return Err(GuestFailure::new(
            "singlefs_outcome",
            format!("子进程退出 {:?}，读回内容相符 {content_matches}", status.code()),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct GuestArguments {
    device_paths: Vec<String>,
    configuration: Configuration,
    round: u32,
}

fn parse_guest_arguments(arguments: &[String]) -> Result<GuestArguments, String> {
    let [device_paths @ .., configuration_text, round_text] = arguments else {
        return Err("用法：/bench <设备...> <配置> <轮次>".to_string());
    };
    let configuration =
        Configuration::parse(configuration_text).ok_or_else(|| format!("不认识的配置 {configuration_text}"))?;
    let round = round_text
        .parse::<u32>()
        .map_err(|error| format!("轮次 {round_text} 不是数：{error}"))?;
    if device_paths.len() != configuration.device_count() {
        return Err(format!(
            "{} 要 {} 块盘，拿到 {} 块",
            configuration.label(),
            configuration.device_count(),
            device_paths.len()
        ));
    }
    Ok(GuestArguments {
        device_paths: device_paths.to_vec(),
        configuration,
        round,
    })
}

fn prepare_guest_environment() -> Result<(), GuestFailure> {
    // zpool 给整盘分区之后要看得到分区节点，mdev -s 只扫一次，devtmpfs 由内核自己维护（登记第三节）
    run_checked(&CommandLine::new("/bin/mount", &["-t", "devtmpfs", "devtmpfs", "/dev"]), "mount_devtmpfs")?;
    // mdadm 在 /run/mdadm 里记它建过的阵列（登记第十一节）；initramfs 里没有 /run
    std::fs::create_dir_all("/run/mdadm").map_err(|error| GuestFailure::new("run_directory", error.to_string()))?;
    std::fs::create_dir_all(MOUNT_POINT).map_err(|error| GuestFailure::new("mount_point", error.to_string()))
}

fn emit_configuration_line(parsed: &GuestArguments, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map_err(|error| GuestFailure::new("kernel_release", error.to_string()))?;
    let memory_information_text =
        std::fs::read_to_string("/proc/meminfo").map_err(|error| GuestFailure::new("memory_total", error.to_string()))?;
    let memory_total = memory_information_text
        .lines()
        .find(|line| line.starts_with("MemTotal:"))
        .map_or_else(|| "NA".to_string(), single_token);
    let mut device_sizes = Vec::new();
    for device_path in &parsed.device_paths {
        device_sizes.push(format!("{}:{}", device_name(device_path)?, read_device_bytes(device_path)?));
    }
    emitter.emit(
        "configuration",
        &format!("kernel={} memory_total={memory_total} devices={}", kernel.trim(), device_sizes.join(",")),
    );
    Ok(())
}

fn load_kernel_modules(configuration: Configuration, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    let Some(order_name) = configuration.module_order_name() else {
        return Ok(());
    };
    let order_path = format!("{MODULE_ORDER_DIRECTORY}/{order_name}.order");
    let order = std::fs::read_to_string(&order_path).map_err(|error| GuestFailure::new("module_order", format!("{order_path}：{error}")))?;
    let mut loaded = 0_u32;
    for module_path in order.lines().filter(|line| !line.trim().is_empty()) {
        run_checked(&CommandLine::new("/bin/insmod", &[module_path]), "insmod")?;
        loaded += 1;
    }
    emitter.emit("modules", &format!("order={order_name} loaded={loaded}"));
    Ok(())
}

fn run_guest_workload(parsed: &GuestArguments, emitter: &mut Emitter) -> Result<(), GuestFailure> {
    prepare_guest_environment()?;
    emit_configuration_line(parsed, emitter)?;
    load_kernel_modules(parsed.configuration, emitter)?;
    match parsed.configuration {
        Configuration::RawDevice | Configuration::RawSoftwareRaidMirror => {
            run_raw_device_round(parsed.configuration, &parsed.device_paths, emitter)
        }
        Configuration::Singlefs => run_singlefs(&parsed.device_paths, emitter),
        Configuration::FourthExtendedFileSystem
        | Configuration::Xfs
        | Configuration::FlashFriendlyFileSystem
        | Configuration::Btrfs
        | Configuration::Bcachefs
        | Configuration::Zfs
        | Configuration::FourthExtendedFileSystemOnSoftwareRaidMirror
        | Configuration::XfsOnSoftwareRaidMirror
        | Configuration::FlashFriendlyFileSystemOnSoftwareRaidMirror
        | Configuration::BtrfsMirror
        | Configuration::BcachefsTwoReplicas
        | Configuration::ZfsMirror => run_file_system_round(parsed.configuration, &parsed.device_paths, emitter),
    }
}

/// 登记第八节第 1、4 条：失败的那一轮要附来宾 dmesg 末尾。
fn report_failure(failure: &GuestFailure, emitter: &mut Emitter) {
    emitter.emit(
        "failure",
        &format!("stage={} detail={}", failure.stage, single_token(&failure.detail)),
    );
    match Command::new("/bin/dmesg").output() {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = text.lines().collect();
            let start = lines.len().saturating_sub(DMESG_TAIL_LINES);
            for (offset, line) in lines[start..].iter().enumerate() {
                emitter.emit("dmesg", &format!("line={} text={}", start + offset, single_token(line)));
            }
        }
        Err(error) => emitter.emit("dmesg_unavailable", &format!("detail={}", single_token(&error.to_string()))),
    }
}

fn run_guest(arguments: &[String]) -> ExitCode {
    let parsed = match parse_guest_arguments(arguments) {
        Ok(parsed) => parsed,
        Err(reason) => {
            eprintln!("{reason}");
            return ExitCode::from(2);
        }
    };
    std::env::set_var("PATH", "/usr/sbin:/usr/bin:/sbin:/bin");
    std::env::set_var("LD_LIBRARY_PATH", "/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu");
    // initramfs 里没有 udev，mdadm 不许等它（登记第十一节）
    std::env::set_var("MDADM_NO_UDEV", "1");
    let mut emitter = Emitter {
        emitted: 0,
        context: format!("configuration={} round={}", parsed.configuration.label(), parsed.round),
    };
    let exit_code = match run_guest_workload(&parsed, &mut emitter) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            report_failure(&failure, &mut emitter);
            ExitCode::from(1)
        }
    };
    emitter.finish();
    std::io::stdout().flush().expect("控制台写不出去，结果行也就没了");
    exit_code
}

// ── 宿主一侧：把多轮产物汇成中位、最小、最大与离散（登记第七节） ──

fn parse_key_values(body: &str) -> BTreeMap<&str, &str> {
    body.split_whitespace().filter_map(|field| field.split_once('=')).collect()
}

struct RunBlock {
    configuration: String,
    round: u32,
    virtual_machine_exit: i64,
    result_bodies: Vec<String>,
}

/// 产物由 e152-run.sh 拼成：每一次虚机跑先一行 `E152RUN`，后面跟这一次抓到的全部 `E7RESULT` 行。
fn parse_run_blocks(text: &str) -> Vec<RunBlock> {
    let mut blocks: Vec<RunBlock> = Vec::new();
    for line in text.lines() {
        if let Some(header) = line.strip_prefix("E152RUN ") {
            let fields = parse_key_values(header);
            blocks.push(RunBlock {
                configuration: fields.get("configuration").copied().unwrap_or("unknown").to_string(),
                round: fields.get("round").and_then(|value| value.parse().ok()).unwrap_or(0),
                virtual_machine_exit: fields.get("vm_exit").and_then(|value| value.parse().ok()).unwrap_or(-1),
                result_bodies: Vec::new(),
            });
        } else if let Some(body) = line.strip_prefix("E7RESULT ") {
            if let Some(current) = blocks.last_mut() {
                current.result_bodies.push(body.to_string());
            }
        }
    }
    blocks
}

struct MetricReading {
    metric: &'static str,
    value: Result<f64, String>,
    is_read_cell: bool,
}

fn metric_reading(metric: &'static str, value: Result<f64, String>, is_read_cell: bool) -> MetricReading {
    MetricReading { metric, value, is_read_cell }
}

fn number_field(fields: &BTreeMap<&str, &str>, key: &str) -> Result<f64, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("missing_{key}"))?
        .parse::<f64>()
        .map_err(|_| format!("not_a_number_{key}"))
}

fn fio_metric_readings(fields: &BTreeMap<&str, &str>) -> Vec<MetricReading> {
    let verdict = fields.get("verdict").copied().unwrap_or("missing_verdict");
    let usable = |value: Result<f64, String>| -> Result<f64, String> {
        if verdict == "counted" || verdict == "not_checked" {
            value
        } else {
            Err(verdict.to_string())
        }
    };
    let mebibytes = |key: &str| number_field(fields, key).map(|kibibytes| kibibytes / 1024.0);
    match fields.get("job").copied() {
        Some("write_sequential_large") => vec![metric_reading(
            "write_sequential_mebibytes_per_second",
            usable(mebibytes("write_bandwidth_kibibytes_per_second")),
            false,
        )],
        Some("read_sequential_large") => vec![metric_reading(
            "read_sequential_mebibytes_per_second",
            usable(mebibytes("read_bandwidth_kibibytes_per_second")),
            true,
        )],
        Some("read_random_4k") => vec![
            metric_reading(
                "read_random_4k_operations_per_second",
                usable(number_field(fields, "read_operations_per_second")),
                true,
            ),
            metric_reading(
                "read_random_4k_percentile_99_microseconds",
                usable(number_field(fields, "read_percentile_99_microseconds")),
                true,
            ),
        ],
        Some("write_random_4k") => vec![
            metric_reading(
                "write_random_4k_operations_per_second",
                usable(number_field(fields, "write_operations_per_second")),
                false,
            ),
            metric_reading(
                "write_random_4k_percentile_99_microseconds",
                usable(number_field(fields, "write_percentile_99_microseconds")),
                false,
            ),
        ],
        Some("metadata_create") => vec![metric_reading(
            "metadata_create_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("metadata_stat") => vec![metric_reading(
            "metadata_stat_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("metadata_delete") => vec![metric_reading(
            "metadata_delete_per_second",
            usable(number_field(fields, "read_operations_per_second")),
            false,
        )],
        Some("small_file_write_fsync") => vec![metric_reading(
            "small_file_write_fsync_per_second",
            usable(number_field(fields, "write_operations_per_second")),
            false,
        )],
        // 对照两遍不是指标；不认识的作业名不猜
        _ => Vec::new(),
    }
}

fn divided(numerator: Result<f64, String>, denominator: Result<f64, String>, scale: f64) -> Result<f64, String> {
    Ok(numerator? / denominator? * scale)
}

fn metric_readings(fields: &BTreeMap<&str, &str>) -> Vec<MetricReading> {
    let nanoseconds_as = |key: &str, scale: f64| number_field(fields, key).map(|nanoseconds| nanoseconds / scale);
    match fields.get("name").copied() {
        Some("fio") => fio_metric_readings(fields),
        Some("persist") => vec![
            metric_reading("persist_median_microseconds", nanoseconds_as("median_latency_nanoseconds", 1e3), false),
            metric_reading("persist_percentile_99_microseconds", nanoseconds_as("percentile_99_latency_nanoseconds", 1e3), false),
            metric_reading(
                "persist_mean_write_requests",
                number_field(fields, "mean_write_requests_thousandths").map(|thousandths| thousandths / 1e3),
                false,
            ),
            metric_reading("persist_mean_write_bytes", number_field(fields, "mean_write_bytes"), false),
            metric_reading(
                "persist_mean_flush_requests",
                number_field(fields, "mean_flush_requests_thousandths").map(|thousandths| thousandths / 1e3),
                false,
            ),
            metric_reading("persist_first_write_requests", number_field(fields, "first_write_requests"), false),
            metric_reading("persist_first_write_bytes", number_field(fields, "first_write_bytes"), false),
            metric_reading("persist_first_flush_requests", number_field(fields, "first_flush_requests"), false),
        ],
        Some("cold_mount") => vec![
            metric_reading("cold_mount_milliseconds", nanoseconds_as("elapsed_nanoseconds", 1e6), false),
            metric_reading("cold_mount_read_bytes", number_field(fields, "block_read_bytes"), false),
            metric_reading("cold_mount_read_requests", number_field(fields, "block_read_requests"), false),
        ],
        Some("format") => vec![
            metric_reading("format_milliseconds", nanoseconds_as("elapsed_nanoseconds", 1e6), false),
            metric_reading("format_write_bytes", number_field(fields, "block_write_bytes"), false),
            metric_reading("format_flush_requests", number_field(fields, "block_flush_requests"), false),
        ],
        Some("capacity") => vec![
            metric_reading(
                "capacity_total_percent",
                divided(number_field(fields, "total_bytes"), number_field(fields, "disk_bytes"), 100.0),
                false,
            ),
            metric_reading(
                "capacity_available_percent",
                divided(number_field(fields, "available_bytes"), number_field(fields, "disk_bytes"), 100.0),
                false,
            ),
        ],
        Some("singlefs_timing") => vec![
            metric_reading("singlefs_write_path_milliseconds", nanoseconds_as("write_path_nanoseconds", 1e6), false),
            metric_reading("singlefs_recovery_milliseconds", nanoseconds_as("recovery_nanoseconds", 1e6), false),
            metric_reading(
                "singlefs_read_bytes_both_devices",
                number_field(fields, "device_0_read_bytes")
                    .and_then(|first| number_field(fields, "device_1_read_bytes").map(|second| first + second)),
                false,
            ),
            metric_reading(
                "singlefs_read_requests_both_devices",
                number_field(fields, "device_0_read_requests")
                    .and_then(|first| number_field(fields, "device_1_read_requests").map(|second| first + second)),
                false,
            ),
        ],
        _ => Vec::new(),
    }
}

struct RoundStatistics {
    median: f64,
    minimum: f64,
    maximum: f64,
    spread_percent: f64,
    is_stable: bool,
}

/// 登记第七节：中位、最小、最大，离散 = (最大 − 最小) ÷ 中位。
fn round_statistics(values: &[f64]) -> RoundStatistics {
    assert!(!values.is_empty(), "空样本没有中位");
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len();
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    };
    let minimum = sorted[0];
    let maximum = sorted[count - 1];
    let spread_percent = if median == 0.0 {
        if maximum == minimum {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        (maximum - minimum) * 100.0 / median
    };
    RoundStatistics {
        median,
        minimum,
        maximum,
        spread_percent,
        is_stable: spread_percent <= STABLE_SPREAD_PERCENT,
    }
}

fn summarize_product(text: &str) -> Vec<String> {
    let blocks = parse_run_blocks(text);
    let mut attempts: BTreeMap<(String, u32), u32> = BTreeMap::new();
    let mut selected: BTreeMap<(String, u32), &RunBlock> = BTreeMap::new();
    for block in &blocks {
        let key = (block.configuration.clone(), block.round);
        *attempts.entry(key.clone()).or_insert(0) += 1;
        if block.virtual_machine_exit == 0 {
            selected.entry(key).or_insert(block);
        }
    }
    let mut values: BTreeMap<(String, &'static str), Vec<(u32, f64)>> = BTreeMap::new();
    let mut controls: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut lines = Vec::new();
    for ((configuration, round), attempt_count) in &attempts {
        let Some(block) = selected.get(&(configuration.clone(), *round)) else {
            lines.push(format!(
                "name=summary_failed_round configuration={configuration} round={round} attempts={attempt_count}"
            ));
            continue;
        };
        let parsed_bodies: Vec<BTreeMap<&str, &str>> =
            block.result_bodies.iter().map(|body| parse_key_values(body)).collect();
        let control_has_teeth = parsed_bodies
            .iter()
            .find(|fields| fields.get("name") == Some(&"control"))
            .map(|fields| fields.get("control_has_teeth") == Some(&"true"));
        if let Some(has_teeth) = control_has_teeth {
            let tally = controls.entry(configuration.clone()).or_insert((0, 0));
            tally.0 += 1;
            if has_teeth {
                tally.1 += 1;
            }
        }
        for fields in &parsed_bodies {
            for reading in metric_readings(fields) {
                // 登记第八节第 3 条：对照没判成「被缓存顶替」，这一轮的读格作废
                let value = if reading.is_read_cell && control_has_teeth == Some(false) {
                    Err("control_without_teeth".to_string())
                } else {
                    reading.value
                };
                match value {
                    Ok(number) => values
                        .entry((configuration.clone(), reading.metric))
                        .or_default()
                        .push((*round, number)),
                    Err(reason) => lines.push(format!(
                        "name=summary_excluded configuration={configuration} metric={} round={round} reason={reason}",
                        reading.metric
                    )),
                }
            }
        }
    }
    for (configuration, (rounds_checked, rounds_with_teeth)) in &controls {
        lines.push(format!(
            "name=summary_control configuration={configuration} rounds_checked={rounds_checked} rounds_with_teeth={rounds_with_teeth}"
        ));
    }
    for ((configuration, metric), readings) in &values {
        let numbers: Vec<f64> = readings.iter().map(|(_, number)| *number).collect();
        let statistics = round_statistics(&numbers);
        let listed = readings
            .iter()
            .map(|(round, number)| format!("{round}:{number:.3}"))
            .collect::<Vec<_>>()
            .join(",");
        lines.push(format!(
            "name=summary configuration={configuration} metric={metric} rounds={} median={:.3} minimum={:.3} maximum={:.3} spread_percent={:.1} stability={} values={listed}",
            numbers.len(),
            statistics.median,
            statistics.minimum,
            statistics.maximum,
            statistics.spread_percent,
            if statistics.is_stable { "stable" } else { "unstable" }
        ));
    }
    let mut output: Vec<String> = lines.into_iter().map(|line| format!("E7RESULT {line}")).collect();
    let emitted = output.len() + 1;
    output.push(format!("E7RESULT name=summary_done emitted={emitted}"));
    output
}

fn run_summarize(arguments: &[String]) -> ExitCode {
    let Some(product_path) = arguments.first() else {
        eprintln!("用法：e152-file-system-benchmark summarize <产物>");
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(product_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("读不了 {product_path}：{error}");
            return ExitCode::from(2);
        }
    };
    for line in summarize_product(&text) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("summarize") {
        return run_summarize(&arguments[1..]);
    }
    run_guest(&arguments)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 本机 fio 3.36 在一个 8 MiB 文件上跑 4 KiB 随机读写的原样 terse 行（2026-09-15，字段号拿同一次的 JSON 输出逐个对过）。
    const FIO_3_36_TERSE_LINE: &str = "3;fio-3.36;probe;0;0;3976;62125;15531;64;0;0;0.000000;0.000000;48;281;62.555513;20.863914;1.000000%=50;5.000000%=54;10.000000%=56;20.000000%=58;30.000000%=59;40.000000%=60;50.000000%=60;60.000000%=61;70.000000%=62;80.000000%=62;90.000000%=63;95.000000%=65;99.000000%=248;99.500000%=268;99.900000%=280;99.950000%=280;99.990000%=280;0%=0;0%=0;0%=0;48;281;62.575865;20.868117;0;0;0.000000%;0.000000;0.000000;4216;65875;16468;64;0;0;0.000000;0.000000;0;5;0.816231;0.361605;1.000000%=0;5.000000%=0;10.000000%=0;20.000000%=0;30.000000%=0;40.000000%=0;50.000000%=0;60.000000%=0;70.000000%=0;80.000000%=0;90.000000%=1;95.000000%=1;99.000000%=2;99.500000%=2;99.900000%=4;99.950000%=5;99.990000%=5;0%=0;0%=0;0%=0;0;5;0.840112;0.367051;0;0;0.000000%;0.000000;0.000000;0.000000%;4.761905%;995;0;0;100.0%;0.0%;0.0%;0.0%;0.0%;0.0%;0.0%;50.93%;0.39%;0.15%;0.00%;0.15%;47.90%;0.05%;0.44%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;0.00%;nvme0n1;0;0;0;0;0;0;0;0.00%";

    #[test]
    fn block_layer_counters_parse_a_seventeen_field_statistics_line() {
        let text = "    1234        5    67890      100     2222       33   444444      555        0      600      700        0        0        0        0       12       34\n";
        let counters = parse_block_layer_counters(text).expect("17 个字段的 stat 行要解得出来");
        assert_eq!(counters.read_requests, 1234);
        assert_eq!(counters.read_sectors, 67890);
        assert_eq!(counters.write_requests, 2222);
        assert_eq!(counters.write_sectors, 444444);
        assert_eq!(counters.flush_requests, 12, "FLUSH 在下标 15，下标 14 是 discard 耗时");
    }

    #[test]
    fn block_layer_counters_refuse_a_line_without_the_flush_field() {
        let eleven_fields = "1 2 3 4 5 6 7 8 9 10 11";
        assert_eq!(parse_block_layer_counters(eleven_fields), None, "读不到 FLUSH 不许拿 0 顶上");
        assert_eq!(parse_block_layer_counters("1 2 x 4 5 6 7 8 9 10 11 12 13 14 15 16"), None);
    }

    #[test]
    fn block_layer_delta_turns_sectors_into_bytes() {
        let earlier = BlockLayerCounters { read_requests: 10, read_sectors: 100, write_requests: 20, write_sectors: 200, flush_requests: 3 };
        let later = BlockLayerCounters { read_requests: 15, read_sectors: 108, write_requests: 26, write_sectors: 216, flush_requests: 5 };
        let delta = later.delta_since(earlier);
        assert_eq!(delta.read_requests, 5);
        assert_eq!(delta.read_bytes, 4096, "8 个扇区 × 512 字节");
        assert_eq!(delta.write_requests, 6);
        assert_eq!(delta.write_bytes, 8192, "16 个扇区 × 512 字节");
        assert_eq!(delta.flush_requests, 2);
    }

    #[test]
    fn fio_terse_version_3_line_from_fio_3_36_parses_both_directions() {
        let report = parse_fio_terse_version_3(FIO_3_36_TERSE_LINE).expect("真实的 terse 行要解得出来");
        assert_eq!(report.error_code, 0);
        assert_eq!(report.read_kibibytes, 3976);
        assert_eq!(report.read_bandwidth_kibibytes_per_second, 62125);
        assert_eq!(report.read_operations_per_second, 15531);
        assert_eq!(report.read_completion_latency_percentile_99_microseconds, Some(248));
        assert_eq!(report.write_kibibytes, Some(4216));
        assert_eq!(report.write_bandwidth_kibibytes_per_second, Some(65875));
        assert_eq!(report.write_operations_per_second, Some(16468));
        assert_eq!(report.write_completion_latency_percentile_99_microseconds, Some(2));
    }

    #[test]
    fn fio_terse_line_of_a_diskless_engine_with_121_fields_still_parses() {
        let fields: Vec<&str> = FIO_3_36_TERSE_LINE.split(';').take(121).collect();
        assert_eq!(fields.len(), 121);
        let report = parse_fio_terse_version_3(&fields.join(";")).expect("121 个字段的行要解得出来");
        assert_eq!(report.read_operations_per_second, 15531);
        assert_eq!(report.write_operations_per_second, Some(16468));
    }

    #[test]
    fn fio_terse_parser_refuses_a_line_that_is_not_version_3() {
        let version_2 = FIO_3_36_TERSE_LINE.replacen("3;", "2;", 1);
        assert_eq!(parse_fio_terse_version_3(&version_2), None);
        assert_eq!(parse_fio_terse_version_3("fio: pid=0, err=2/file:filesetup.c"), None);
    }

    #[test]
    fn direct_read_below_ninety_percent_of_fio_bytes_is_judged_cache_substituted() {
        let fio_bytes = 1_024_000;
        assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectRead, fio_bytes, 921_600), CellVerdict::Counted, "恰好 90% 算数");
        assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectRead, fio_bytes, 921_599), CellVerdict::CacheSubstituted);
        assert_eq!(judge_direct_input_output(DirectInputOutputCheck::NotChecked, fio_bytes, 0), CellVerdict::NotChecked);
    }

    #[test]
    fn direct_write_below_ninety_percent_of_fio_bytes_is_judged_writes_absorbed() {
        let fio_bytes = 4_294_967_296;
        assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectWrite, fio_bytes, 3_865_470_567), CellVerdict::Counted);
        assert_eq!(judge_direct_input_output(DirectInputOutputCheck::DirectWrite, fio_bytes, 3_865_470_566), CellVerdict::WritesAbsorbed);
    }

    #[test]
    fn control_second_pass_with_no_block_reads_is_judged_cache_substituted() {
        assert_eq!(FioJob::ControlBufferedReadFirstPass.check(), DirectInputOutputCheck::NotChecked);
        assert_eq!(
            judge_direct_input_output(FioJob::ControlBufferedReadSecondPass.check(), 268_435_456, 0),
            CellVerdict::CacheSubstituted,
            "对照的第二遍块层一个字节都没读，判定函数必须把它判成被缓存顶替"
        );
    }

    #[test]
    fn control_passes_keep_the_page_cache_that_fio_would_otherwise_drop() {
        for job in [FioJob::ControlBufferedReadFirstPass, FioJob::ControlBufferedReadSecondPass] {
            let arguments = job.arguments("/mnt/large");
            assert!(arguments.contains(&"--invalidate=0".to_string()), "{} 不关 invalidate，第二遍读不到缓存", job.label());
            assert_eq!(arguments.len(), 6);
        }
        assert!(!FioJob::ReadRandomFourKibibytes.arguments("/mnt/large").contains(&"--invalidate=0".to_string()));
    }

    #[test]
    fn zfs_gets_a_partition_that_starts_at_one_mebibyte_instead_of_the_whole_disk() {
        let format = Configuration::Zfs.format_command(&["/dev/vda".to_string()]).expect("zfs 有格式化命令");
        assert_eq!(format.program, "/bin/sh");
        assert_eq!(format.arguments.len(), 2);
        let script = &format.arguments[1];
        assert!(script.contains("start=2048,"), "分区要从第 2048 扇区起：{script}");
        assert!(script.ends_with("bench /dev/vda1"), "池要建在分区上，不在整盘上：{script}");
    }

    #[test]
    fn block_layer_counters_of_two_mirror_members_add_up() {
        let first = BlockLayerCounters { read_requests: 10, read_sectors: 100, write_requests: 20, write_sectors: 200, flush_requests: 3 };
        let second = BlockLayerCounters { read_requests: 1, read_sectors: 8, write_requests: 2, write_sectors: 16, flush_requests: 4 };
        let summed = first.plus(second);
        assert_eq!(summed.read_requests, 11);
        assert_eq!(summed.read_sectors, 108);
        assert_eq!(summed.write_requests, 22);
        assert_eq!(summed.write_sectors, 216);
        assert_eq!(summed.flush_requests, 7);
    }

    #[test]
    fn software_raid_state_with_resync_or_recovery_is_synchronizing() {
        let clean = "Personalities : [raid1]\nmd0 : active raid1 vdb[1] vda[0]\n      16760832 blocks super 1.2 [2/2] [UU]\n\nunused devices: <none>\n";
        let resyncing = "Personalities : [raid1]\nmd0 : active raid1 vdb[1] vda[0]\n      16760832 blocks super 1.2 [2/2] [UU]\n      [>....................]  resync =  0.3% (52224/16760832) finish=5.3min speed=52224K/sec\n\nunused devices: <none>\n";
        let recovering = "Personalities : [raid1]\nmd0 : active raid1 vdb[2] vda[0]\n      16760832 blocks super 1.2 [2/1] [U_]\n      [=>...................]  recovery =  7.1% (1190272/16760832) finish=2.1min speed=119027K/sec\n\nunused devices: <none>\n";
        assert!(!software_raid_is_synchronizing(clean), "--assume-clean 建出来的阵列不同步");
        assert!(software_raid_is_synchronizing(resyncing));
        assert!(software_raid_is_synchronizing(recovering), "recovery 也是后台同步");
    }

    #[test]
    fn mirror_configurations_name_both_devices_and_the_software_raid_array() {
        let devices = vec!["/dev/vda".to_string(), "/dev/vdb".to_string()];
        let btrfs = Configuration::BtrfsMirror.format_command(&devices).expect("btrfs raid1 有格式化命令");
        assert_eq!(btrfs.arguments, ["-f", "-q", "-d", "raid1", "-m", "raid1", "/dev/vda", "/dev/vdb"]);
        let btrfs_mount = Configuration::BtrfsMirror.mount_command(&devices).expect("btrfs raid1 有挂载命令");
        assert_eq!(btrfs_mount.arguments, ["-t", "btrfs", "-o", "device=/dev/vda,device=/dev/vdb", "/dev/vda", "/mnt"]);
        let bcachefs_mount = Configuration::BcachefsTwoReplicas.mount_command(&devices).expect("bcachefs 双副本有挂载命令");
        assert_eq!(bcachefs_mount.arguments, ["-t", "bcachefs", "/dev/vda:/dev/vdb", "/mnt"]);
        let zfs = Configuration::ZfsMirror.format_command(&devices).expect("zfs mirror 有格式化命令");
        assert!(zfs.arguments[1].ends_with("bench mirror /dev/vda1 /dev/vdb1"), "{}", zfs.arguments[1]);
        assert_eq!(zfs.arguments[1].matches("sfdisk").count(), 2, "两块盘都要分区");
        let fourth_extended_on_mirror = Configuration::FourthExtendedFileSystemOnSoftwareRaidMirror
            .format_command(&devices)
            .expect("ext4-md 有格式化命令");
        assert_eq!(fourth_extended_on_mirror.arguments, ["-F", "-q", "/dev/md0"]);
        let raid = software_raid_creation_command(&devices);
        assert!(raid.arguments.contains(&"--assume-clean".to_string()), "全零新盘本来就一致，不许后台同步");
        assert_eq!(raid.arguments.len(), 10);
        assert_eq!(Configuration::RawSoftwareRaidMirror.raw_target(&devices), Some("/dev/md0".to_string()));
        assert!(Configuration::XfsOnSoftwareRaidMirror.uses_software_raid());
        assert!(!Configuration::BtrfsMirror.uses_software_raid());
    }

    #[test]
    fn nearest_rank_percentile_of_one_hundred_and_of_five_samples() {
        let hundred: Vec<u128> = (1..=100).collect();
        assert_eq!(nearest_rank_percentile(&hundred, 50), 50);
        assert_eq!(nearest_rank_percentile(&hundred, 99), 99);
        assert_eq!(nearest_rank_percentile(&hundred, 100), 100);
        let five = [10, 20, 30, 40, 50];
        assert_eq!(nearest_rank_percentile(&five, 50), 30, "秩 ⌈2.5⌉ = 3");
        assert_eq!(nearest_rank_percentile(&five, 99), 50, "秩 ⌈4.95⌉ = 5");
    }

    #[test]
    fn persist_summary_reports_median_percentile_means_and_the_first_operation() {
        let delta = |write_requests, write_bytes, flush_requests| BlockLayerDelta { read_requests: 0, read_bytes: 0, write_requests, write_bytes, flush_requests };
        let measurements = [
            PersistMeasurement { latency_nanoseconds: 30, delta: delta(3, 12288, 2) },
            PersistMeasurement { latency_nanoseconds: 10, delta: delta(1, 4096, 1) },
            PersistMeasurement { latency_nanoseconds: 20, delta: delta(2, 8192, 1) },
        ];
        let summary = summarize_persist(&measurements);
        assert_eq!(summary.operations, 3);
        assert_eq!(summary.median_latency_nanoseconds, 20);
        assert_eq!(summary.percentile_99_latency_nanoseconds, 30);
        assert_eq!(summary.mean_write_requests_thousandths, 2000);
        assert_eq!(summary.mean_write_bytes, 8192);
        assert_eq!(summary.mean_flush_requests_thousandths, 1333);
        assert_eq!(summary.first_latency_nanoseconds, 30);
        assert_eq!(summary.first_write_requests, 3);
        assert_eq!(summary.first_write_bytes, 12288);
        assert_eq!(summary.first_flush_requests, 2);
    }

    #[test]
    fn round_statistics_report_median_spread_and_stability() {
        let unstable = round_statistics(&[100.0, 110.0, 90.0, 105.0, 95.0]);
        assert_eq!(unstable.median, 100.0);
        assert_eq!(unstable.minimum, 90.0);
        assert_eq!(unstable.maximum, 110.0);
        assert_eq!(unstable.spread_percent, 20.0);
        assert!(!unstable.is_stable, "离散 20% 超过 15%");
        let boundary = round_statistics(&[100.0, 115.0, 100.0, 100.0, 100.0]);
        assert_eq!(boundary.spread_percent, 15.0);
        assert!(boundary.is_stable, "恰好 15% 算稳定");
        let even = round_statistics(&[1.0, 2.0, 3.0, 10.0]);
        assert_eq!(even.median, 2.5, "偶数个取中间两个的平均");
    }

    #[test]
    fn capacity_comes_from_busybox_file_system_status_output() {
        assert_eq!(parse_file_system_capacity("4128768 3907213 4096\n"), Some((16_911_433_728, 16_003_944_448)));
        assert_eq!(parse_file_system_capacity("4128768 3907213"), None);
    }

    #[test]
    fn inner_singlefs_line_is_reemitted_without_a_name_equals_done_field() {
        let reemitted = reemit_inner_line(5, "name=done emitted=9");
        assert_eq!(reemitted, "at_nanoseconds=5 inner=done emitted=9");
        assert!(!reemitted.contains("name=done"), "vm-bench.sh 按 name=done emitted= 找收尾行");
    }

    #[test]
    fn guest_arguments_put_devices_before_configuration_and_round() {
        let owned = |words: &[&str]| words.iter().map(|word| (*word).to_string()).collect::<Vec<_>>();
        let single = parse_guest_arguments(&owned(&["/dev/vda", "ext4", "3"])).expect("一块盘的 ext4");
        assert_eq!(single.device_paths, vec!["/dev/vda".to_string()]);
        assert_eq!(single.configuration, Configuration::FourthExtendedFileSystem);
        assert_eq!(single.round, 3);
        let mirrored = parse_guest_arguments(&owned(&["/dev/vda", "/dev/vdb", "singlefs", "1"])).expect("两块盘的 singlefs");
        assert_eq!(mirrored.device_paths.len(), 2);
        assert!(parse_guest_arguments(&owned(&["/dev/vda", "singlefs", "1"])).is_err(), "singlefs 只给一块盘要拒绝");
        assert!(parse_guest_arguments(&owned(&["ext4"])).is_err());
        assert!(parse_guest_arguments(&owned(&["/dev/vda", "ext4-md", "1"])).is_err(), "镜像配置只给一块盘要拒绝");
        let mirror = parse_guest_arguments(&owned(&["/dev/vda", "/dev/vdb", "zfs-mirror", "2"])).expect("两块盘的 zfs mirror");
        assert_eq!(mirror.configuration, Configuration::ZfsMirror);
        assert_eq!(mirror.device_paths.len(), 2);
    }

    #[test]
    fn configuration_labels_round_trip_through_parse() {
        let all = [
            Configuration::RawDevice,
            Configuration::FourthExtendedFileSystem,
            Configuration::Xfs,
            Configuration::FlashFriendlyFileSystem,
            Configuration::Btrfs,
            Configuration::Bcachefs,
            Configuration::Zfs,
            Configuration::RawSoftwareRaidMirror,
            Configuration::FourthExtendedFileSystemOnSoftwareRaidMirror,
            Configuration::XfsOnSoftwareRaidMirror,
            Configuration::FlashFriendlyFileSystemOnSoftwareRaidMirror,
            Configuration::BtrfsMirror,
            Configuration::BcachefsTwoReplicas,
            Configuration::ZfsMirror,
            Configuration::Singlefs,
        ];
        for configuration in all {
            assert_eq!(Configuration::parse(configuration.label()), Some(configuration));
        }
        assert_eq!(all.len(), 15);
    }

    #[test]
    fn summarize_takes_the_successful_attempt_and_excludes_cache_substituted_cells() {
        let product = "E152RUN_HEADER kernel=test rounds=2\n\
E152RUN configuration=ext4 round=1 attempt=1 host_load1=0.5 vm_exit=1\n\
E7RESULT name=failure configuration=ext4 round=1 stage=format detail=x\n\
E7RESULT name=done emitted=2\n\
E152RUN configuration=ext4 round=1 attempt=2 host_load1=0.5 vm_exit=0\n\
E7RESULT name=fio configuration=ext4 round=1 job=write_sequential_large write_bandwidth_kibibytes_per_second=1048576 verdict=counted\n\
E7RESULT name=fio configuration=ext4 round=1 job=read_sequential_large read_bandwidth_kibibytes_per_second=2097152 verdict=cache_substituted\n\
E7RESULT name=control configuration=ext4 round=1 second_pass_verdict=cache_substituted control_has_teeth=true\n\
E7RESULT name=done emitted=4\n\
E152RUN configuration=ext4 round=2 attempt=1 host_load1=0.7 vm_exit=0\n\
E7RESULT name=fio configuration=ext4 round=2 job=write_sequential_large write_bandwidth_kibibytes_per_second=2097152 verdict=counted\n\
E7RESULT name=control configuration=ext4 round=2 second_pass_verdict=cache_substituted control_has_teeth=true\n\
E7RESULT name=done emitted=3\n";
        let output = summarize_product(product);
        assert!(
            output.contains(&"E7RESULT name=summary configuration=ext4 metric=write_sequential_mebibytes_per_second rounds=2 median=1536.000 minimum=1024.000 maximum=2048.000 spread_percent=66.7 stability=unstable values=1:1024.000,2:2048.000".to_string()),
            "两轮都要取成功的那一次：{output:#?}"
        );
        assert!(output.contains(
            &"E7RESULT name=summary_excluded configuration=ext4 metric=read_sequential_mebibytes_per_second round=1 reason=cache_substituted".to_string()
        ));
        assert!(output.contains(&"E7RESULT name=summary_control configuration=ext4 rounds_checked=2 rounds_with_teeth=2".to_string()));
        assert_eq!(output.iter().filter(|line| line.contains("name=summary_failed_round")).count(), 0);
        assert_eq!(output.len(), 4);
        assert_eq!(output.last(), Some(&"E7RESULT name=summary_done emitted=4".to_string()));
    }
}
