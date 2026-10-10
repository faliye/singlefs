//! 写死步序的历史：逗号分开的记号，一个记号一步。
//! mount｜cold｜pwu｜unlink｜inode｜abandon（崩溃恢复抛弃根）｜zeros（每块盘最新那一槽系统配置读回全 0 的可写挂载）｜
//! ff:<长>｜ow:<长>｜lose:<长>（掉电在第一条记录之前再挂）｜rb:<第几新>｜rbb:<n>（根环之外）｜rbf（候选集下沿）｜rf:<k>（抬 F）。
//! <长>：e 空｜s<n> 一个单元以内按选择子｜below｜exact｜above｜span<单元数>。
use singlefs_harness::history::{
    ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice,
};

fn length(token: &str) -> ContentLength {
    match token {
        "e" => ContentLength::Empty,
        "below" => ContentLength::OneByteBelowDataUnitPayloadCapacity,
        "exact" => ContentLength::ExactlyDataUnitPayloadCapacity,
        "above" => ContentLength::OneByteAboveDataUnitPayloadCapacity,
        other if other.starts_with("span") => ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units: other[4..].parse().expect("单元数"),
            selector: 7,
        },
        other if other.starts_with('s') => ContentLength::InsideOneDataUnit {
            selector: other[1..].parse().expect("选择子"),
        },
        other => panic!("不认得的长度 {other}"),
    }
}

fn content(token: &str, fill_seed: u64) -> ContentChoice {
    ContentChoice { length: length(token), fill_seed }
}

pub fn history(start: &str, ops: &str, seed: u64) -> GeneratedHistory {
    let starting_point = match start {
        "mkfs" => HistoryStartingPoint::AfterMakeFilesystem,
        "file" => HistoryStartingPoint::AfterFirstFile,
        other => panic!("不认得的起点 {other}"),
    };
    let operations = ops
        .split(',')
        .filter(|token| !token.is_empty())
        .enumerate()
        .map(|(index, token)| {
            let fill = seed.wrapping_mul(1_000_003).wrapping_add(index as u64);
            let (head, tail) = token.split_once(':').unwrap_or((token, ""));
            match head {
                "mount" => HistoryOperation::CloseAndMountWritable,
                "cold" => HistoryOperation::ColdStartRecover,
                "pwu" => HistoryOperation::PublishWithoutUnits,
                "unlink" => HistoryOperation::Unlink,
                "inode" => HistoryOperation::CreateInode,
                "abandon" => HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
                "zeros" => HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
                "ff" => HistoryOperation::PublishFirstFile(content(tail, fill)),
                "ow" => HistoryOperation::PublishOverwrite(content(tail, fill)),
                "lose" => HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content(tail, fill)),
                "rb" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                    index_from_newest: tail.parse().expect("第几新"),
                }),
                "rbb" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::BeyondNewestRoot {
                    txg_beyond_newest: tail.parse().expect("n"),
                }),
                "rbf" => HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRootAtTheNewestFloor),
                "rf" => HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                    steps_above_current_floor: tail.parse().expect("k"),
                }),
                other => panic!("不认得的步 {other}"),
            }
        })
        .collect();
    GeneratedHistory { seed: HistorySeed(seed), starting_point, operations }
}
