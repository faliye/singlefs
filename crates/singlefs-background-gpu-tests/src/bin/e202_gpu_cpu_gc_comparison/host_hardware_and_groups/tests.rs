use super::*;

/// 场景：只点名一张卡臂真跑、交来的那一张；另一组两条用卡的臂都没跑（只点 CPU 臂）。
/// 预期：组标识是组键（四段、每段一行、行尾换行）SHA-256 的前 16 个十六进制字，两个值由仓外算出钉在这里。
#[test]
fn the_hardware_group_identifier_is_the_first_sixteen_hexadecimal_digits_of_the_sha256_of_the_group_key(
) {
    assert_eq!(
        group_of_the_one_card_with_the_driver_information("driver information at 0000:01:00.0")
            .identifier,
        "59c4e56d00ade705"
    );
    let without_a_card_arm = hardware_group_of(&host_hardware_of_the_tests(), None, None);
    assert_eq!(
        (
            without_a_card_arm.identifier.as_str(),
            without_a_card_arm.one_card.as_str(),
            without_a_card_arm.every_card.as_str()
        ),
        ("0eb989158a4403c5", "none", "none")
    );
}

/// 场景：两趟只差点名一张卡臂那一张的驱动信息；另两趟只点 CPU 臂（两段卡都是 none），只差 CPU 型号。
/// 预期：都是两组：驱动与 CPU 型号都进组键。
#[test]
fn groups_differing_only_in_the_driver_information_or_only_in_the_cpu_model_are_two_groups() {
    assert_ne!(
        group_of_the_one_card_with_the_driver_information("alpha").identifier,
        group_of_the_one_card_with_the_driver_information("beta").identifier
    );
    let another_cpu_model = HostHardware {
        cpu_model: "another_cpu_model".to_string(),
        logical_cpus: host_hardware_of_the_tests().logical_cpus,
    };
    assert_ne!(
        hardware_group_of(&host_hardware_of_the_tests(), None, None).identifier,
        hardware_group_of(&another_cpu_model, None, None).identifier
    );
}

/// 场景：`/proc/cpuinfo` 的文字两个逻辑 CPU、型号带空白；另一份没有 `model name` 行。
/// 预期：头一份读成，型号去掉空白、逻辑 CPU 数 2；后一份读不成：打 `readable=false` 与 `name=stopped clause=host_hardware_not_readable`
/// （带出路），这条命令退 78，不当成一组「不知道」接着跑。
#[test]
fn host_hardware_without_a_model_name_line_stops_instead_of_going_on_as_an_unknown_group() {
    assert_eq!(
        host_hardware_of_the_session(
            4,
            Ok("processor\t: 0\nmodel name\t: Test CPU 9000\n\nprocessor\t: 1\nmodel name\t: Test CPU 9000\n".to_string())
        ),
        HostHardwareOfTheSession::Read {
            host_hardware: HostHardware {
                cpu_model: "Test_CPU_9000".to_string(),
                logical_cpus: 2,
            },
            fields: "session=4 cpu_model=Test_CPU_9000 logical_cpus=2".to_string(),
        }
    );
    let HostHardwareOfTheSession::NotReadable { lines } = host_hardware_of_the_session(
        4,
        Ok("processor\t: 0\nvendor_id\t: TestVendor\n".to_string()),
    ) else {
        panic!("没有 model name 行：该读不成");
    };
    assert_eq!(
        lines
            .iter()
            .map(|(kind, fields)| format!(
                "{kind} {}",
                fields.split(" reason=").next().unwrap_or_default()
            ))
            .collect::<Vec<String>>(),
        [
            "host_hardware session=4 readable=false".to_string(),
            "stopped clause=host_hardware_not_readable".to_string()
        ]
    );
    assert!(
        lines[1]
            .1
            .ends_with(" way_out=再交同一条命令，两台都读不到交主agent"),
        "{:?}",
        lines[1]
    );
}
