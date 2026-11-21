//! 六家文件系统的配置：标签与往返解析、每个配置的设备布局、建盘与挂载命令。

use super::*;

/// 单盘七个配置（登记第二节）、两盘镜像七个配置（登记第十一节）、singlefs：封闭集合，加一种就得把每个 `match` 过一遍。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Configuration {
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
    pub(super) fn parse(text: &str) -> Option<Self> {
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

    pub(super) fn label(self) -> &'static str {
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
    pub(super) fn device_count(self) -> usize {
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

    /// `scripts/devices/e152-stage-root.sh` 写的模块次序文件名；ext4 编进内核，单盘裸盘与 singlefs 不要模块，md 配置都要 raid1。
    pub(super) fn module_order_name(self) -> Option<&'static str> {
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
    pub(super) fn uses_software_raid(self) -> bool {
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
    pub(super) fn raw_target(self, device_paths: &[String]) -> Option<String> {
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
    pub(super) fn format_command(self, device_paths: &[String]) -> Option<CommandLine> {
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
    pub(super) fn mount_command(self, device_paths: &[String]) -> Option<CommandLine> {
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

    pub(super) fn unmount_command(self) -> Option<CommandLine> {
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
    pub(super) fn format_also_mounts(self) -> bool {
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
pub(super) fn software_raid_creation_command(device_paths: &[String]) -> CommandLine {
    let mut arguments = vec!["--create", SOFTWARE_RAID_DEVICE, "--level=1", "--raid-devices=2", "--metadata=1.2", "--bitmap=none", "--assume-clean", "--run"];
    arguments.extend(device_paths.iter().map(String::as_str));
    CommandLine::new("/usr/sbin/mdadm", &arguments)
}

#[cfg(test)]
#[path = "configurations_and_setup_scripts_tests.rs"]
mod tests;
