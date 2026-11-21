//! 实验包的库：实验装置（`src/bin/`）共用的那几样。
//!
//! - 装置造的池（[`experiment_pool`]）：两块录进同一条流的文件镜像上 mkfs，或接着取号、暖机、新池新建文件；池的样子取验证包库的公开项，
//!   实现是这个包自己写的（装置不经 `#[path]` 引验证包 `tests/` 下的共用模块，`.claude/rules/directory-layout.md`「crates/：各包放什么」）。
//!   连同验证包库的新池新建文件那条路录下的流（[`experiment_pool::new_pool_file_creation_of_the_verification_library`]）：
//!   E170（O3规约执行器第一版）、E171（三oracle合谋测试） 的停机 S1 拿装置自己驱动出的流与它比。
//! - 装置自己的判法单元入口登记表（[`judging_units_entries`]）与构建脚本按它算出、编进来的键表（[`JUDGING_UNITS_KEYS_TABLE_TEXT`]）：
//!   装置先录后判的参照那几组按组名取键，装置挂在崩溃枚举上的观察者经 [`OBSERVER_KEYS_OF_THE_EXPERIMENTS`] 交给崩溃枚举。

#![deny(unsafe_code)]

use singlefs_verification::crash_identity::ObserverJudgingUnitsKeyTable;

pub mod experiment_pool;
pub mod judging_units_entries;

/// 构建脚本写进 `OUT_DIR` 的那份键表（`singlefs_unit_graph::entry_groups` 的写法，文件名与 `build.rs` 的 `JUDGING_UNITS_KEYS_FILE_NAME` 相同）。
pub const JUDGING_UNITS_KEYS_TABLE_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"));

/// 装置挂在崩溃枚举上的观察者那几组的键从这一张取（[`judging_units_entries::OBSERVER_ENTRIES`] 登记，构建脚本算）：交给
/// `singlefs_verification::crash_state_enumeration::enumerate_crash_states_with_expansion_keying_the_observers_in`。
pub const OBSERVER_KEYS_OF_THE_EXPERIMENTS: ObserverJudgingUnitsKeyTable =
    ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage {
        table_text: JUDGING_UNITS_KEYS_TABLE_TEXT,
    };

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_verification::crash_identity::{
        judging_units_key_of_the_group_in_the_table_text,
        judging_units_key_with_the_observers_keyed_in, JudgingUnits,
    };
    use singlefs_verification::crash_library::toolchain_of_this_process;

    /// 场景：观察者登记表里的每一个观察者，这个包拼的组名与崩溃枚举拼来去表里找的组名（验证包的拼法与 `JudgingUnits::group_name`）。
    /// 预期：三处逐字相同。
    /// 判别力（`crates/mutations.tsv`）：这个包的拼法改一个字，这条红。
    #[test]
    fn the_observer_group_names_are_the_ones_the_crash_enumeration_looks_up() {
        assert!(!judging_units_entries::OBSERVER_ENTRIES.is_empty());
        for observer in judging_units_entries::OBSERVER_ENTRIES {
            let named_here = judging_units_entries::observer_group_name(
                observer.observer_code_file,
                observer.observer_name,
            );
            assert_eq!(
                named_here,
                singlefs_verification::judging_units_entries::observer_group_name(
                    observer.observer_code_file,
                    observer.observer_name,
                )
            );
            assert_eq!(
                named_here,
                JudgingUnits::ObserverCode {
                    observer_code_file: observer.observer_code_file.to_string(),
                    observer_name: observer.observer_name.to_string(),
                }
                .group_name()
            );
        }
    }

    /// 场景：编进来的键表里，登记表的每一组参照按组名取键，每一个观察者照装置交给崩溃枚举的那一张（[`OBSERVER_KEYS_OF_THE_EXPERIMENTS`]）
    /// 经崩溃枚举取观察者键的那一个入口取键（不读盘上的树，表是构建时编进来的）。
    /// 预期：都取得出（不拒算）。
    /// 判别力（`crates/mutations.tsv`）：装置交的那一张改成验证包那一张，观察者取键拒算，这条红。
    #[test]
    fn every_registered_group_takes_its_key_from_the_compiled_table() {
        for group in judging_units_entries::JUDGING_UNITS_ENTRY_GROUPS {
            judging_units_key_of_the_group_in_the_table_text(
                JUDGING_UNITS_KEYS_TABLE_TEXT,
                group.group_name,
            )
            .unwrap_or_else(|refusal| {
                panic!("参照那一组 {} 取不出键：{refusal}", group.group_name)
            });
        }
        for observer in judging_units_entries::OBSERVER_ENTRIES {
            judging_units_key_with_the_observers_keyed_in(
                &JudgingUnits::ObserverCode {
                    observer_code_file: observer.observer_code_file.to_string(),
                    observer_name: observer.observer_name.to_string(),
                },
                OBSERVER_KEYS_OF_THE_EXPERIMENTS,
                toolchain_of_this_process(),
            )
            .unwrap_or_else(|refusal| {
                panic!(
                    "观察者 {} 取不出键：{}",
                    observer.observer_name,
                    refusal.sentence()
                )
            });
        }
    }
}
