use super::*;

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
