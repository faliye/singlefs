use super::*;

/// 回读映射的回调（`map_async` 收的那一种）装箱之后的样子。
type BoxedReadbackMappingCallback = Box<dyn FnOnce(Result<(), wgpu::BufferAsyncError>) + Send>;

/// 假的回读缓冲区：丢掉时照 wgpu 收拾还挂着映射的缓冲那样，以映射失败调起回调（等卡超时之后，展开途中丢的就是它）。
struct ReadbackBufferWithAPendingMapping {
    pending_mapping_callback: Option<BoxedReadbackMappingCallback>,
}

impl Drop for ReadbackBufferWithAPendingMapping {
    fn drop(&mut self) {
        if let Some(callback) = self.pending_mapping_callback.take() {
            callback(Err(wgpu::BufferAsyncError));
        }
    }
}

/// 等卡超时的那一次派活只让它所在的用例红，同一进程里下一次派活照常读回（m4 验证段第 4 类：超时 panic 展开时收的那一头先丢、
/// 回读缓冲区后丢，wgpu 以映射失败调起回调，回调里 `expect` 送得出去就在析构途中再 panic，整个测试二进制 SIGABRT）。
/// 不起卡：设备的情况、回读缓冲区、poll 的结局都是假的，等与回调走的是派活那几处用的同一对函数
/// （`readback_mapping_callback`、`take_the_readback_after_the_wait_or_record_the_device_lost`）；收的那一头照派活里的次序晚于缓冲区声明、先丢。
/// 超时之后设备记成丢了，卡池那一格（`DeviceStartedOnDemand`）下一次借时不再交这台、起一台新的。
/// 判别力：回调改回 `expect("回调送得出去")`，头一句（收的那一头丢了之后调回调不 panic）红——不先在展开途外核这一句的话，
/// 后面那一段展开途中再 panic、进程中止，这一条连同同一二进制里余下的用例都没有结局；超时不记设备丢了，`is_lost` 那一句与「起了一台新的」那一句红。
#[test]
fn dispatch_whose_wait_timed_out_is_red_alone_and_the_next_dispatch_in_the_same_process_reads_back()
{
    use super::{
        readback_mapping_callback, take_the_readback_after_the_wait_or_record_the_device_lost,
    };
    use crate::gpu_memory_ledger::DeviceStartedOnDemand;
    use std::sync::Arc;
    let (sender_whose_receiver_is_gone, receiver_dropped_at_once) = std::sync::mpsc::channel();
    drop(receiver_dropped_at_once);
    let callback_after_the_receiver_is_gone =
        readback_mapping_callback(sender_whose_receiver_is_gone);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            callback_after_the_receiver_is_gone(Err(wgpu::BufferAsyncError));
        }))
        .is_ok(),
        "收的那一头丢了之后 wgpu 调起回调：送不出就丢，不 panic"
    );
    let card_slot: DeviceStartedOnDemand<Arc<DeviceConditionOfACard>> =
        DeviceStartedOnDemand::new();
    let devices_started = std::sync::atomic::AtomicUsize::new(0);
    let lease_the_card = || -> Arc<DeviceConditionOfACard> {
        card_slot
            .started_or_start(
                |device: &Arc<DeviceConditionOfACard>| device.is_lost(),
                || -> Result<Arc<DeviceConditionOfACard>, std::convert::Infallible> {
                    devices_started.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(Arc::new(DeviceConditionOfACard::default()))
                },
            )
            .expect("假设备总起得来")
    };
    let timed_out_device = lease_the_card();
    let timed_out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let (sender, receiver) = std::sync::mpsc::channel();
        let _readback = ReadbackBufferWithAPendingMapping {
            pending_mapping_callback: Some(Box::new(readback_mapping_callback(sender))),
        };
        let receiver_dropped_before_the_readback_buffer = receiver;
        take_the_readback_after_the_wait_or_record_the_device_lost(
            &timed_out_device,
            Err(wgpu::PollError::Timeout),
            &receiver_dropped_before_the_readback_buffer,
            |error| format!("假的等卡超时：{error:?}"),
        );
    }));
    let panic_message = timed_out
        .expect_err("等卡超时的那一次派活要红")
        .downcast::<String>()
        .map(|message| *message)
        .unwrap_or_default();
    assert!(
        panic_message.contains("假的等卡超时：Timeout"),
        "红的那一句说等卡超时：{panic_message}"
    );
    assert!(
        timed_out_device.is_lost(),
        "等卡超时之后这张卡的设备记成丢了"
    );
    assert!(
        timed_out_device
            .lost_reason()
            .is_some_and(|reason| reason.contains("等卡没等成")),
        "丢失的缘由写等卡没等成：{:?}",
        timed_out_device.lost_reason()
    );
    let next_device = lease_the_card();
    assert!(
        !Arc::ptr_eq(&next_device, &timed_out_device),
        "下一次借这张卡不再交超时的那一台"
    );
    assert_eq!(
        devices_started.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "超时之后起了一台新的"
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let callback = readback_mapping_callback(sender);
    callback(Ok(()));
    take_the_readback_after_the_wait_or_record_the_device_lost(
        &next_device,
        Ok(wgpu::PollStatus::WaitSucceeded),
        &receiver,
        |error| format!("下一次派活不该等不成：{error:?}"),
    );
    assert!(!next_device.is_lost(), "下一次派活等成了，新的那一台没记丢");
}
