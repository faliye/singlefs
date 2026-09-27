from edit_lib import edit

MC = "crates/singlefs-harness/src/model_comparison.rs"
edit(MC, """        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致：随机历史每次都交整池两块盘、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。""", """        // 盘表里同一个身份交了几次、调用方参数与盘上系统配置不一致、有盘落后于现行那一版又缺它的单元：随机历史每次都交整池两块盘
        // （没换过盘）、用建池的那一份参数，走不到。
        | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        // 盘上的实例代号或 txg 已到顶：坏镜像才有。""")
edit(MC, """        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,""", """        | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
        | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
        | MountError::SequenceNumberPastTheTopOfItsRange(_)
        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,""", count=2)
HI = "crates/singlefs-harness/src/history.rs"
edit(HI, """        MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. } => {
            "CallerParametersDisagreeWithTheSelectedSystemConfiguration"
        }""", """        MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. } => {
            "CallerParametersDisagreeWithTheSelectedSystemConfiguration"
        }
        MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. } => {
            "DevicesBehindTheCurrentVersionAndMissingItsUnits"
        }""")
edit(HI, """            | UserChangeRefused::NoFileVersionToChange
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
        ) => &[],""", """            | UserChangeRefused::NoFileVersionToChange
            | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
            | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
            | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. },
        ) => &[],""")
edit(HI, """        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
            "UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration".to_string(),
            ObservedRefusalReason::Unexplained,
        ),""", """        UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. } => (
            "UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        // 执行器交的盘体各在自己的身份上、没换过旧快照（代码三方 m2-closeout-code-r2 那两判）：走不到，走到了就按说不出理由的拒绝判。
        UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. } => (
            "UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable".to_string(),
            ObservedRefusalReason::Unexplained,
        ),
        UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. } => (
            "UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits".to_string(),
            ObservedRefusalReason::Unexplained,
        ),""")
FT = "crates/singlefs-checker-tier/src/bin/first_transaction_on_device.rs"
edit(FT, """                | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                    ..
                }
""", """                | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                    ..
                }
                // 有盘落后于现行那一版又缺它的单元：挂着之后的入口在任何写之前拒，这个二进制不换盘，走不到。
                | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. }
""", count=2)
E1 = "crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs"
edit(E1, """                    | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
                ) => &[],""", """                    | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                    | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                    | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. },
                ) => &[],""")
CA = "crates/singlefs-harness/tests/common_admission/mod.rs"
edit(CA, """                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. },
            ) => {}""", """                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits { .. },
            ) => {}""")
print("04_matches done")
