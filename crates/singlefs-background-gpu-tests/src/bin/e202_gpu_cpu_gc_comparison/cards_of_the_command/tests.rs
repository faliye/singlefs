use super::*;

/// 一份五个变量都给了的领卡通道（值是什么不看，只当有通道）。
fn lease_channel_given() -> CardLeaseChannelOfTheCaller {
    CardLeaseChannelOfTheCaller::of_the_variable_values(&|_name| {
        Some(std::ffi::OsString::from("given"))
    })
    .expect("五个变量都给了")
}

/// 环境变量里服务交的卡串与领卡通道领到的卡串（两串不同，看得出用的是哪一串；地址是文档用的保留地址）。
const CARDS_IN_THE_ENVIRONMENT_VARIABLE: &str = "192.0.2.1:0:32G";
const CARDS_LEASED_THROUGH_THE_CHANNEL: &str = "192.0.2.1:1:32G";

/// 场景：环境变量给了卡串，也有领卡通道。
/// 预期：照环境变量那一串，一次都不领；开头的来路核判有来路。
#[test]
fn the_cards_in_the_environment_variable_are_used_without_leasing() {
    let leases = Cell::new(0_u32);
    let cards = cards_handed_to_the_apparatus(
        Some(CARDS_IN_THE_ENVIRONMENT_VARIABLE.to_string()),
        Some(&lease_channel_given()),
        &|_channel| {
            leases.set(leases.get() + 1);
            Ok(CARDS_LEASED_THROUGH_THE_CHANNEL.to_string())
        },
    );
    assert_eq!(
        (cards, leases.get()),
        (Ok(CARDS_IN_THE_ENVIRONMENT_VARIABLE.to_string()), 0)
    );
    assert!(the_task_server_hands_over_cards(
        Some(CARDS_IN_THE_ENVIRONMENT_VARIABLE),
        None
    ));
}

/// 场景：环境变量没给卡，有领卡通道。
/// 预期：经通道领一回，用领到的那一串；开头的来路核判有来路（只看、不领）。
#[test]
fn without_the_environment_variable_the_cards_are_leased_through_the_channel() {
    let channel = lease_channel_given();
    let leases = Cell::new(0_u32);
    let cards = cards_handed_to_the_apparatus(None, Some(&channel), &|_channel| {
        leases.set(leases.get() + 1);
        Ok(CARDS_LEASED_THROUGH_THE_CHANNEL.to_string())
    });
    assert_eq!(
        (cards, leases.get()),
        (Ok(CARDS_LEASED_THROUGH_THE_CHANNEL.to_string()), 1)
    );
    assert!(the_task_server_hands_over_cards(None, Some(&channel)));
}

/// 场景：环境变量没给卡、也没有领卡通道（没经服务交）。
/// 预期：取卡报错（那一句点名经 task-server/task submit 交），一次都不领、不退回自己枚举；开头的来路核判没有来路。
#[test]
fn without_the_variable_and_the_channel_the_apparatus_refuses_instead_of_enumerating_the_cards() {
    let leases = Cell::new(0_u32);
    let cards = cards_handed_to_the_apparatus(None, None, &|_channel| {
        leases.set(leases.get() + 1);
        Ok(CARDS_LEASED_THROUGH_THE_CHANNEL.to_string())
    });
    assert_eq!(
        (cards, leases.get()),
        (
            Err(REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER.to_string()),
            0
        )
    );
    assert!(REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER.contains("task-server/task submit"));
    assert!(!the_task_server_hands_over_cards(None, None));
}

/// 场景：本机看得到三张独显，服务只交了总线地址居中的那一张。
/// 预期：只留交来的那一张；点名一张卡臂点的就是交来的那一张。
#[test]
fn only_the_handed_over_card_among_those_seen_on_this_host_is_kept() {
    let kept = discrete_gpus_among_the_handed_over(
        three_discrete_gpus_seen_on_this_host(),
        &["0000:02:00.0".to_string()],
    )
    .expect("交来的那一张在本机看得到的独显里");
    assert_eq!(
        kept.iter()
            .map(|gpu| gpu.pci_bus_address.as_str())
            .collect::<Vec<&str>>(),
        vec!["0000:02:00.0"]
    );
    assert_eq!(
        background_compute_settings_of_the_arm(
            ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
            &kept
        )
        .background_gpu_devices,
        Some(vec![gpu_device_identifier_of_the_pci_bus_address(
            "0000:02:00.0"
        )])
    );
}

/// 场景：服务交了一张本机看不到的卡。
/// 预期：取卡报错，不退回用本机看得到的那几张。
#[test]
fn a_handed_over_card_not_seen_on_this_host_is_refused() {
    assert!(discrete_gpus_among_the_handed_over(
        three_discrete_gpus_seen_on_this_host(),
        &["0000:04:00.0".to_string()],
    )
    .is_err());
}

/// 场景：本机看得到三张独显，服务交了其中两张（总线地址在两头的那两张）；另一趟三张都交了。
/// 预期：全部卡臂照跑，点名的正是交来的那两张、上限两份，没交的那一张不点；三张都交了就点三张、上限三份。
#[test]
fn the_every_card_arm_opens_as_many_cards_as_are_handed_over() {
    let handed_over_pci_bus_addresses = ["0000:01:00.0".to_string(), "0000:03:00.0".to_string()];
    let two_of_three = CardsSeenByThisProcess {
        discrete_gpus: discrete_gpus_among_the_handed_over(
            three_discrete_gpus_seen_on_this_host(),
            &handed_over_pci_bus_addresses,
        )
        .expect("交来的两张都在本机看得到的独显里"),
        handed_over_pci_bus_addresses_in_the_handed_over_order: handed_over_pci_bus_addresses
            .to_vec(),
        vulkan_instance: None,
    };
    assert_eq!(
        reason_the_arm_is_not_run_on_the_cards(ComputeOfTheArm::GpuOnEveryCard, &two_of_three),
        None
    );
    let settings = background_compute_settings_of_the_arm(
        ComputeOfTheArm::GpuOnEveryCard,
        &two_of_three.discrete_gpus,
    );
    assert_eq!(
        (
            settings.background_gpu_devices,
            settings.background_gpu_memory_limit
        ),
        (
            Some(vec![
                gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
                gpu_device_identifier_of_the_pci_bus_address("0000:03:00.0"),
            ]),
            Some(vec![GpuMemoryInBytes(1 << 30); 2])
        )
    );
    let every_one_of_three = background_compute_settings_of_the_arm(
        ComputeOfTheArm::GpuOnEveryCard,
        &three_discrete_gpus_seen_on_this_host(),
    );
    assert_eq!(
        every_one_of_three
            .background_gpu_devices
            .map(|devices| devices.len()),
        Some(3)
    );
}

/// 场景：服务交来三张（交来的先后 03、01、02），`--cards 2`。
/// 预期：全部卡臂只点交来的卡串里排在前头的那两张（03 与 01，按 PCI 地址次序交回），上限两份；交来的另一张不点。
#[test]
fn the_cards_option_names_the_first_handed_over_cards_in_the_handed_over_order() {
    let cards = three_cards_handed_over_out_of_the_pci_order();
    let named = discrete_gpus_named_for_the_every_card_arm(
        &cards,
        CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(2).expect("2 不是 0")),
    )
    .expect("要两张、交了三张");
    assert_eq!(
        pci_bus_addresses_of(&named),
        ["0000:01:00.0", "0000:03:00.0"]
    );
    let settings = background_compute_settings_of_the_arm(ComputeOfTheArm::GpuOnEveryCard, &named);
    assert_eq!(
        (
            settings.background_gpu_devices,
            settings.background_gpu_memory_limit
        ),
        (
            Some(vec![
                gpu_device_identifier_of_the_pci_bus_address("0000:01:00.0"),
                gpu_device_identifier_of_the_pci_bus_address("0000:03:00.0"),
            ]),
            Some(vec![GpuMemoryInBytes(1 << 30); 2])
        )
    );
}

/// 场景：服务交来三张，没给 `--cards`。
/// 预期：全部卡臂点交来的三张全部（按 PCI 地址次序）。
#[test]
fn without_the_cards_option_the_every_card_arm_names_every_handed_over_card() {
    let cards = three_cards_handed_over_out_of_the_pci_order();
    let named = discrete_gpus_named_for_the_every_card_arm(
        &cards,
        CardsOfTheEveryCardArm::EveryHandedOverCard,
    )
    .expect("没给 --cards 不会多于交来的");
    assert_eq!(
        pci_bus_addresses_of(&named),
        ["0000:01:00.0", "0000:02:00.0", "0000:03:00.0"]
    );
}

/// 场景：服务交来三张，`--cards 4`。
/// 预期：报错，不截成交来的三张。
#[test]
fn the_cards_option_asking_for_more_than_are_handed_over_is_refused() {
    let cards = three_cards_handed_over_out_of_the_pci_order();
    assert_eq!(
        discrete_gpus_named_for_the_every_card_arm(
            &cards,
            CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(4).expect("4 不是 0")),
        )
        .map(|named| pci_bus_addresses_of(&named).len()),
        Err("--cards 4 多于服务交给这一台的 3 张卡（[\"0000:03:00.0\", \"0000:01:00.0\", \"0000:02:00.0\"]）".to_string())
    );
}

/// 场景：这一台服务一张卡都没交，`--cards 2`。
/// 预期：交回空（照「服务交的卡里没有这一台的」退 77 那一路办），不当命令行不对。
#[test]
fn the_cards_option_on_a_host_handed_no_card_names_none_without_a_usage_error() {
    let no_card = CardsSeenByThisProcess {
        discrete_gpus: Vec::new(),
        handed_over_pci_bus_addresses_in_the_handed_over_order: Vec::new(),
        vulkan_instance: None,
    };
    assert_eq!(
        discrete_gpus_named_for_the_every_card_arm(
            &no_card,
            CardsOfTheEveryCardArm::FirstHandedOverCards(NonZeroUsize::new(2).expect("2 不是 0")),
        ),
        Ok(Vec::new())
    );
}

/// 场景：核这个进程在哪几张卡上时 `nvidia-smi` 读不成。
/// 预期：交回本次未跑（调用方看得见，对拍用例照它交回没跑成、不入库），不判红。
#[test]
fn a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller() {
    assert_eq!(
        common_discrete_gpus_of_this_host::gpus_this_process_is_on_judged_against_the_handed_over(
            "a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller",
            Err("nvidia-smi 起不来".to_string()),
            &[],
        ),
        common_discrete_gpus_of_this_host::GpusThisProcessIsOn::NotReadSoThisRunIsNotRun
    );
}

/// 场景：核这个进程在哪几张卡上时读成了，可这个进程一张卡都不在（读法看不见它开着的设备）。
/// 预期：交回本次未跑，不当成只在交来的卡上。
#[test]
fn a_process_seen_on_no_card_is_reported_not_run_to_the_caller() {
    assert_eq!(
        common_discrete_gpus_of_this_host::gpus_this_process_is_on_judged_against_the_handed_over(
            "a_process_seen_on_no_card_is_reported_not_run_to_the_caller",
            Ok(BTreeSet::new()),
            &[],
        ),
        common_discrete_gpus_of_this_host::GpusThisProcessIsOn::NotReadSoThisRunIsNotRun
    );
}

/// 场景：环境变量没给卡串，有领卡通道，领卡那一端拒了。
/// 预期：取卡报错，那一句点名是按需领卡没成、带上那一端的话，不当成「服务没给卡的来路」。
#[test]
fn a_failed_lease_through_the_channel_is_reported_as_the_lease_failing() {
    let channel = lease_channel_given();
    assert_eq!(
        cards_handed_to_the_apparatus(None, Some(&channel), &|_channel| {
            Err("领卡那一端拒了".to_string())
        }),
        Err("按需领卡没成：领卡那一端拒了".to_string())
    );
}
