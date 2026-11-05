
    /// 攻方探针（m4-gpugc-code-r1，T1）：只点名一张好卡，另一张没点名的独显驱动没报 PCI 总线地址。
    /// 条款第 2 条「点名了就只用点名的卡」；开包那一步先对全部独显判清单（`background_gpu.rs` 的 `open_background_gpu`），这里就拒。
    #[test]
    fn attack_probe_an_unnamed_discrete_gpu_without_a_bus_address_refuses_a_mount_naming_another_card() {
        let named_card = adapter("named card", AdapterDeviceKind::DiscreteGpu, "0000:01:00.0", 8);
        let unnamed_card_without_an_address =
            adapter("unnamed card", AdapterDeviceKind::DiscreteGpu, "", 8);
        let enumeration = vec![named_card.clone(), unnamed_card_without_an_address];
        let listed = discrete_gpus_of_the_enumeration(&enumeration);
        println!("ATTACK name=unnamed_card_refuses_the_list listed={listed:?}");
        let listed_without_it = discrete_gpus_of_the_enumeration(&[named_card])
            .expect("只剩点名的那张：清单交得成");
        let selection = BackgroundGpuSelection {
            background_gpu_devices: Some(vec![gpu_device_identifier_of_the_pci_bus_address(
                "0000:01:00.0",
            )]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
        };
        let admitted = participating_gpus_admitted_by_the_mount_checks(&selection, &listed_without_it);
        println!("ATTACK name=same_selection_without_the_unnamed_card admitted_ok={}", admitted.is_ok());
        assert!(listed.is_err(), "没点名的那张卡让清单交不成，开包在判点名之前就拒");
        assert!(admitted.is_ok(), "同一组设置在没有那张卡时照收");
        let virtual_card = adapter("virtual card", AdapterDeviceKind::VirtualGpu, "0000:02:00.0", 8);
        let listed_with_a_virtual_card = discrete_gpus_of_the_enumeration(&[virtual_card])
            .expect("虚拟显卡不算显卡：清单是空的");
        let naming_the_virtual_card = BackgroundGpuSelection {
            background_gpu_devices: Some(vec![gpu_device_identifier_of_the_pci_bus_address(
                "0000:02:00.0",
            )]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: WAIT_LIMIT_UNDER_TEST,
        };
        let refused = participating_gpus_admitted_by_the_mount_checks(
            &naming_the_virtual_card,
            &listed_with_a_virtual_card,
        );
        println!("ATTACK name=naming_a_virtual_gpu refused={refused:?}");
        assert!(refused.is_err(), "点名一张虚拟显卡：照「点名的卡不存在」拒");
    }
}
