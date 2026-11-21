//! 判一个崩溃状态恢复到的版本在不在模型允许的集合里。

use super::*;

/// 崩溃之后恢复到的这一版允不允许（增补 3 第 3 件：崩溃注入）。模型这一侧给的是「提交过哪些版、每一版的内容是什么」
/// （[`IdealModel::committed_versions`] 攒出来的目录），崩溃点那一侧给的是「盘上已持久的最新根槽是哪条根」
/// （从截断了的录制流里数出来，不经模型也不经实现的择根）。四条判据：
/// ① 走读不许失败；
/// ② 择到的根必须是模型提交过的某一版——读回一版从没提交过的，是 D13（验证路线） 已定项 7 那条「记录流里没有的东西不许出现在盘上」
///    在恢复这一侧的形态；
/// ③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧（D22（单元原子性怎么合成） 已定项 7 的择新序：
///    择新 txg 为主、平局按实例代号高者赢）；
/// ④ 读回的内容要与模型记的那一版逐字节相同，那一版树表 0 条时只许报没有文件。
///
/// 与崩溃点重放的 `crash::oracle_violation_for_versions` 分开写：那一份的版本表由调用方按固定脚本手写，这一份问的是模型
/// （D13（验证路线） 已定项 5：模型不与 `singlefs-core` 共用代码）。那一份在「走到一条没发布过的更新的根上报没有文件」那一格只在
/// 有更旧的带文件版本时才判红，这一份认得每一条根有没有文件，两个方向都判得出。
#[must_use]
pub fn crash_recovery_disagreement(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> Option<ModelDisagreement> {
    let (chosen, content_read_back) = match read_back {
        ObservedReadBack::Failed { what } => {
            return Some(ModelDisagreement::new(
                ModelDisagreementAspect::ColdStartReadBack,
                "崩溃之后的冷启动要走到一条模型提交过的根上，不许失败".to_string(),
                format!("走读失败：{what}"),
            ))
        }
        ObservedReadBack::NoFile { root } => (*root, None),
        ObservedReadBack::FileRead { root, content } => (*root, Some(content)),
    };
    if let Some(persisted) = newest_persisted_root {
        if chosen < persisted {
            return Some(ModelDisagreement::new(
                ModelDisagreementAspect::ColdStartReadBack,
                format!(
                    "盘上已持久的最新根槽是实例 {} 第 {} 代，不许恢复到比它旧的根",
                    persisted.instance.0, persisted.checkpoint_txg.0
                ),
                format!(
                    "走到实例 {} 第 {} 代根",
                    chosen.instance.0, chosen.checkpoint_txg.0
                ),
            ));
        }
    }
    let Some(committed) = committed_versions.get(&chosen) else {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "择到的根要是模型提交过的某一版".to_string(),
            format!(
                "走到实例 {} 第 {} 代根，模型从没提交过这一版（{}）",
                chosen.instance.0,
                chosen.checkpoint_txg.0,
                describe_observed_read_back(read_back)
            ),
        ));
    };
    match (content_read_back, committed) {
        (None, None) => None,
        (None, Some(content)) => Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根下面有文件（{} 字节）",
                chosen.instance.0,
                chosen.checkpoint_txg.0,
                content.len()
            ),
            "报了没有文件".to_string(),
        )),
        (Some(content), None) => Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根树表 0 条，没有文件",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
            format!("读回了 {} 字节", content.len()),
        )),
        (Some(content), Some(committed_content)) => (content.as_slice() != &committed_content[..])
            .then(|| {
                ModelDisagreement::new(
                    ModelDisagreementAspect::ColdStartReadBack,
                    format!(
                        "实例 {} 第 {} 代根下面是 {} 字节（末字节 {:?}）",
                        chosen.instance.0,
                        chosen.checkpoint_txg.0,
                        committed_content.len(),
                        committed_content.last()
                    ),
                    format!("读回 {} 字节（末字节 {:?}）", content.len(), content.last()),
                )
            }),
    }
}

/// 一版里第一个文件之外每个活着的 inode 读得出的内容：号 → 内容（[`IdealModel::committed_versions_of_the_inodes_other_than_the_first`]）。
pub type ModelContentsOfTheInodesOtherThanTheFirst = BTreeMap<u64, Rc<[u8]>>;

/// 崩溃之后恢复落到 `chosen` 那一版时，第一个文件之外每个活着的 inode 读出的（`read_back`，胶水按号交）在不在允许集合里（判决
/// `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第二节 S5「崩溃读回按 inode 给允许集合」，同第一个文件那一份
/// [`crash_recovery_disagreement`]）：`chosen` 要是模型提交过的某一版，那一版里活着的号与读出的号一样，每个读出的逐字节等于
/// [`IdealModel::committed_versions_of_the_inodes_other_than_the_first`] 记的那一份。都在交 `None`。
#[must_use]
pub fn crash_recovery_disagreement_of_the_inodes_other_than_the_first(
    committed_versions: &BTreeMap<ModelRootKey, ModelContentsOfTheInodesOtherThanTheFirst>,
    chosen: ModelRootKey,
    read_back: &BTreeMap<u64, ObservedFile>,
) -> Option<ModelDisagreement> {
    let Some(committed) = committed_versions.get(&chosen) else {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "择到的根要是模型提交过的某一版".to_string(),
            format!(
                "走到实例 {} 第 {} 代根，模型从没提交过这一版",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
        ));
    };
    let committed_numbers: Vec<u64> = committed.keys().copied().collect();
    let read_numbers: Vec<u64> = read_back.keys().copied().collect();
    if committed_numbers != read_numbers {
        return Some(ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            format!(
                "实例 {} 第 {} 代根下面第一个文件之外活着的 inode {committed_numbers:?}",
                chosen.instance.0, chosen.checkpoint_txg.0
            ),
            format!("读出的号 {read_numbers:?}"),
        ));
    }
    committed.iter().find_map(|(number, content)| {
        let read = read_back.get(number).expect("上面比过两边的号一样");
        match read {
            ObservedFile::Decoded(bytes) if bytes.as_slice() == content.as_ref() => None,
            ObservedFile::Decoded(_) | ObservedFile::NoFile | ObservedFile::Undecodable { .. } => {
                Some(ModelDisagreement::new(
                    ModelDisagreementAspect::ColdStartReadBack,
                    format!(
                        "实例 {} 第 {} 代根下面 inode {number} 是 {} 字节（末字节 {:?}）",
                        chosen.instance.0,
                        chosen.checkpoint_txg.0,
                        content.len(),
                        content.last()
                    ),
                    match read {
                        ObservedFile::Decoded(bytes) => {
                            format!("读出 {} 字节（末字节 {:?}）", bytes.len(), bytes.last())
                        }
                        ObservedFile::NoFile => "读不出这个文件".to_string(),
                        ObservedFile::Undecodable { what } => format!("解不出内容：{what}"),
                    },
                ))
            }
        }
    })
}
