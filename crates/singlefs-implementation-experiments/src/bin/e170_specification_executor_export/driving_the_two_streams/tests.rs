//! 驱动两条流：新池新建文件流与抬 F 之后复用再卸载的流，录写表、段表与停机条款 S1 的回比的单测。

use super::*;

/// 一条流现在驱动、录下来之后，`prepare_stream` 那一批停机条款 S1（段长表、写数、根槽写数、盘数与盘字节数，
/// 流一另有与验证包库新池新建文件那条路整条流的回比，流二另有抬 F、E 落点与卸载 txg 的回比）里没过的那几条，每条写成「名字 local=… crates=…」。
fn broken_stop_checks_of_the_stream_recorded_now(name: StreamName) -> Vec<String> {
    prepare_stream(name)
        .checks
        .iter()
        .filter(|check| !check.holds())
        .map(|check| {
            format!(
                "{} local={} crates={}",
                check.name, check.local, check.crates
            )
        })
        .collect()
}

/// 流一（新池新建文件）驱动完，S1 逐条成立：本地段长表与现在录到的流分叉时这里红，用不着跑一趟 `dump`。
/// 段模型那两条只拿本地表比本地拼出的表，本地表与段模型一起错的只有这条抓得到；闭式与写数都对而段序错的，这条也红。
#[test]
fn stop_checks_of_the_new_pool_file_creation_stream_hold_against_the_stream_recorded_now() {
    let broken = broken_stop_checks_of_the_stream_recorded_now(StreamName::NewPoolFileCreation);
    assert!(
        broken.is_empty(),
        "{} 的 S1 没过：{broken:?}",
        StreamName::NewPoolFileCreation.directory_name()
    );
}

/// 流二（固定脚本走到 E 再卸载）驱动完，S1 逐条成立：本地段长表与现在录到的流分叉时这里红，用不着跑一趟 `dump`。
/// 段模型那两条只拿本地表比本地拼出的表，本地表与段模型一起错的只有这条抓得到；闭式与写数都对而段序错的，这条也红。
#[test]
fn stop_checks_of_the_fixed_script_stream_hold_against_the_stream_recorded_now() {
    let broken = broken_stop_checks_of_the_stream_recorded_now(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
    );
    assert!(
        broken.is_empty(),
        "{} 的 S1 没过：{broken:?}",
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount.directory_name()
    );
}
