//! O1 判官（`singlefs_harness::model::crash_recovery_disagreement`）判据 ③：盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧
//! （D22（单元原子性怎么合成） 已定项 7 的择新序：txg 为主、平局按实例代号高者赢）。两版都是模型提交过的、内容也逐字节对得上，
//! 判据 ②④ 都放行，只有 ③ 判得出。

use std::collections::BTreeMap;
use std::rc::Rc;

use singlefs_harness::model::{
    crash_recovery_disagreement, ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey,
    ObservedReadBack,
};

fn root(instance: u64, checkpoint_txg: u64) -> ModelRootKey {
    ModelRootKey {
        checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
        instance: ModelInstanceGeneration(instance),
    }
}

fn committed(versions: &[(ModelRootKey, &[u8])]) -> BTreeMap<ModelRootKey, Option<Rc<[u8]>>> {
    versions
        .iter()
        .map(|(key, content)| (*key, Some(Rc::from(*content))))
        .collect()
}

#[test]
fn landing_on_an_older_txg_than_the_newest_persisted_root_is_red() {
    let older = root(1, 3);
    let newest = root(1, 4);
    let versions = committed(&[(older, b"older"), (newest, b"newest")]);
    let read_back = ObservedReadBack::FileRead {
        root: older,
        content: b"older".to_vec(),
    };
    assert!(
        crash_recovery_disagreement(&versions, None, &read_back).is_none(),
        "前提：不给盘上最新根槽时，落到提交过的旧版、内容对得上，判绿"
    );
    assert!(
        crash_recovery_disagreement(&versions, Some(newest), &read_back).is_some(),
        "盘上已持久的最新根槽是实例 1 第 4 代，落到第 3 代要判红"
    );
}

#[test]
fn landing_on_the_lower_instance_of_the_same_txg_as_the_newest_persisted_root_is_red() {
    let lower_instance = root(1, 5);
    let newest = root(2, 5);
    let versions = committed(&[(lower_instance, b"instance one"), (newest, b"instance two")]);
    let read_back = ObservedReadBack::FileRead {
        root: lower_instance,
        content: b"instance one".to_vec(),
    };
    assert!(
        crash_recovery_disagreement(&versions, None, &read_back).is_none(),
        "前提：不给盘上最新根槽时，落到提交过的那一版、内容对得上，判绿"
    );
    assert!(
        crash_recovery_disagreement(&versions, Some(newest), &read_back).is_some(),
        "同一个 txg 上实例 2 的根已持久，落到实例 1 的那一条要判红（平局按实例代号高者赢）"
    );
}
