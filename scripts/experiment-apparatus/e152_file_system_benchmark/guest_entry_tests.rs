use super::*;

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
