//! 报本次未跑那一行与显存不够的原因：本次未跑的记号与那一行、借不到与卡上 Out of Memory 的原因，
//! 按状态数借草稿区被拒之后的去向与借卡被拒的种类。

use super::*;

/// GPU 用例与产品路径报「本次未跑」那一行的记号（`singlefs_verification::gpu_unit_checks::gpu_test_start::not_run_line` 照它打；
/// 门禁 `crash-point-replay` 的 feature-gated-tests 格按它逐条报本次未跑）。
pub const GPU_NOT_RUN_MARKER: &str = "NOT_RUN_NO_GPU_ADAPTER";

/// 「本次未跑」那一行：`<记号> test=<用例或产品路径的名字> reason=<为什么>`。
#[must_use]
pub fn gpu_not_run_line(test_name: &str, reason: &str) -> String {
    format!("{GPU_NOT_RUN_MARKER} test={test_name} reason={reason}")
}

/// 显存不够而报「本次未跑」时原因的开头（借不到、卡上报 Out of Memory 都是它）：分项驱动与门禁按 [`GPU_NOT_RUN_MARKER`] 认这一行，
/// 人照这几个字认出是显存不够（记号照 `NOT_RUN_NO_GPU_ADAPTER` 一族，原因写显存不够）。
pub const GPU_MEMORY_SHORT_REASON_PREFIX: &str = "显存不够：";

/// 此刻借不到的那一句（GPU 用例借卡与产品路径借缓冲同一句）。
#[must_use]
pub fn reason_for_a_booking_without_room_now(
    need_mebibytes: u64,
    card_rooms_mebibytes: &[u64],
) -> String {
    format!(
        "{GPU_MEMORY_SHORT_REASON_PREFIX}此刻池里没有一张卡借得出 {need_mebibytes} MiB（这个进程看每张卡此刻能借的 {card_rooms_mebibytes:?}：\
         现读的空闲减这个进程借了还没占的；卡上别的 GPU 用户或这个进程里别的用例占着）"
    )
}

/// wgpu 在卡上报 Out of Memory 时「本次未跑」那一句（`what_ran_out` 说是哪一段、哪张卡）：账只记这个进程借的，
/// 两个进程同一刻读到同一份空闲、一起借超了的那一形由这一句兜底（文件头）。
#[must_use]
pub fn reason_for_running_out_of_gpu_memory(what_ran_out: &str) -> String {
    format!(
        "{GPU_MEMORY_SHORT_REASON_PREFIX}{what_ran_out}上 wgpu 报了 Out of Memory（账只记这个进程借的，卡上别的进程同一刻借走了；判过的块在库里）"
    )
}

/// 产品路径借缓冲借不到的结局（[`scratch_booking_refusal_outcome`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScratchBookingRefusalOutcome {
    /// 空着时放得下而此刻借不到：打这一行，这一趟照产品路径的「本次未跑」交回（判过的块在库里，没判的留给下一趟）。
    NotRun { not_run_line: String },
    /// 要的比那张卡空着时能借的都大、卡上的显存读不出：判红。
    Red { reason: String },
}

/// 产品路径借缓冲被拒时怎么报：此刻借不到照 GPU 对拍用例那一路报「本次未跑」（同一个记号、同一句），别的判红。
#[must_use]
pub fn scratch_booking_refusal_outcome(
    run_name: &str,
    card_position: usize,
    refusal: &BookingRefusal,
) -> ScratchBookingRefusalOutcome {
    match refusal {
        BookingRefusal::NoCardHasRoomNow {
            need_mebibytes,
            card_rooms_mebibytes,
        } => ScratchBookingRefusalOutcome::NotRun {
            not_run_line: gpu_not_run_line(
                run_name,
                &reason_for_a_booking_without_room_now(*need_mebibytes, card_rooms_mebibytes),
            ),
        },
        BookingRefusal::NeedLargerThanEveryCard(refusal) => ScratchBookingRefusalOutcome::Red {
            reason: format!(
                "{run_name}：第 {card_position} 张卡要借 {} MiB，那张卡空着时也只借得出 {} MiB",
                refusal.need_mebibytes, refusal.largest_ceiling_mebibytes
            ),
        },
        BookingRefusal::CardMemoryUnreadable { reason } => ScratchBookingRefusalOutcome::Red {
            reason: format!("{run_name}：卡上的显存读不出：{reason}"),
        },
    }
}

/// 借不到的几种结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookingRefusal {
    /// 要的比每张卡空着时能借的都大：调用方判红。
    NeedLargerThanEveryCard(NeedLargerThanEveryCard),
    /// 空着时放得下而此刻每张卡都借不出：调用方报「本次未跑」（不等）。带着这一刻每张卡能借的。
    NoCardHasRoomNow {
        need_mebibytes: u64,
        card_rooms_mebibytes: Vec<u64>,
    },
    /// 卡上的显存读不出、读到的卡数与池对不上：调用方判红。
    CardMemoryUnreadable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::{
        gpu_not_run_line, reason_for_running_out_of_gpu_memory, scratch_booking_refusal_outcome,
        BookingRefusal, ScratchBookingRefusalOutcome, GPU_MEMORY_SHORT_REASON_PREFIX,
    };
    use crate::gpu_memory_ledger::NeedLargerThanEveryCard;

    /// 产品路径此刻借不到照 GPU 对拍用例那一路报「本次未跑」，同一个记号、原因写显存不够；比卡大、读不出卡判红。
    #[test]
    fn booking_without_room_now_is_reported_not_run_with_the_gpu_not_run_marker() {
        let no_room = BookingRefusal::NoCardHasRoomNow {
            need_mebibytes: 4_000,
            card_rooms_mebibytes: vec![1_000],
        };
        let ScratchBookingRefusalOutcome::NotRun { not_run_line } =
            scratch_booking_refusal_outcome("crash_amplification::run", 0, &no_room)
        else {
            panic!("此刻借不到要报本次未跑");
        };
        assert!(
            not_run_line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够："
            ),
            "本次未跑那一行用 GPU 用例的记号（门禁 feature-gated-tests 格认的就是这一串）、原因写显存不够：{not_run_line}"
        );
        for red in [
            BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
                need_mebibytes: 40_000,
                largest_ceiling_mebibytes: 30_047,
            }),
            BookingRefusal::CardMemoryUnreadable {
                reason: "nvidia-smi 起不来".to_string(),
            },
        ] {
            assert!(
                matches!(
                    scratch_booking_refusal_outcome("crash_amplification::run", 0, &red),
                    ScratchBookingRefusalOutcome::Red { .. }
                ),
                "{red:?} 判红"
            );
        }
    }

    /// K3：卡上 wgpu 报 Out of Memory 那一句以「显存不够：」起头，打成本次未跑那一行时跟着 GPU 用例的记号
    /// （门禁 feature-gated-tests 格认记号，人认这几个字）。
    #[test]
    fn the_out_of_memory_reason_says_the_gpu_memory_was_short_in_the_not_run_line() {
        let reason = reason_for_running_out_of_gpu_memory("序号 0 的卡建判器时");
        assert!(
            reason.starts_with(GPU_MEMORY_SHORT_REASON_PREFIX),
            "原因写显存不够：{reason}"
        );
        assert!(
            gpu_not_run_line("crash_amplification::run", &reason).starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run reason=显存不够：序号 0 的卡建判器时"
            ),
            "本次未跑那一行：GPU 用例的记号、原因写显存不够"
        );
    }
}
