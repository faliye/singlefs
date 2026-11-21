//! 额度（`server` 模块文档「额度」）：整个服务与每条连接的待处理字节与待处理帧数、收着的帧体、编着与没写出去的回应各自记的名额，到了上限回忙。

use super::*;

/// 准入：这条连接与整个服务的待处理帧数、记着的字节（文件头「额度」）。进了的帧拿一个 `PendingSlot`，
/// 回应写出去或丢掉时放手、计数减回去。
pub struct Admission {
    pub(super) limits: AdmissionLimits,
    is_unbounded: bool,
    pending_total: AtomicUsize,
    pub(super) pending_bytes: AtomicUsize,
    pub(super) pending_peak: AtomicUsize,
    pub(super) pending_bytes_peak: AtomicUsize,
    receiving_body_bytes: AtomicUsize,
    pub(super) receiving_body_bytes_peak: AtomicUsize,
}

/// 一条连接在准入里的账：待处理帧数、记着的字节。
#[derive(Default)]
pub struct ConnectionBudget {
    pending_frames: AtomicUsize,
    held_bytes: AtomicUsize,
}

/// 记进额度的一笔字节（整个服务与这条连接各记一份），放手时销掉。
pub struct ByteReservation {
    pub(super) admission: Arc<Admission>,
    pub(super) budget: Arc<ConnectionBudget>,
    pub(super) bytes: usize,
}

impl Drop for ByteReservation {
    fn drop(&mut self) {
        self.admission
            .pending_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
        self.budget
            .held_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

/// 在 `counter` 上再记 `amount`：记完不超 `limit`，或记之前 `counter` 里只有这一笔自己的 `already_held_here`
/// （没有别的在用额度；不放过的话比上限大的一帧永远进不来），就记下回真；否则不记、回假。
fn reserve_against(
    counter: &AtomicUsize,
    amount: usize,
    limit: usize,
    already_held_here: usize,
) -> bool {
    let before = counter.fetch_add(amount, Ordering::SeqCst);
    if before + amount <= limit || before == already_held_here {
        return true;
    }
    counter.fetch_sub(amount, Ordering::SeqCst);
    false
}

impl ByteReservation {
    /// 这一笔再多记 `extra`（回应编好之后记它的字节）；`is_forced` 的不看上限（已经做了、撤不回的写确认）。记不进回假、这一笔不变。
    fn grow(&mut self, extra: usize, is_forced: bool) -> bool {
        let admission = &self.admission;
        let limits = admission.limits;
        if is_forced || admission.is_unbounded {
            self.budget.held_bytes.fetch_add(extra, Ordering::SeqCst);
            admission.pending_bytes.fetch_add(extra, Ordering::SeqCst);
        } else {
            if !reserve_against(
                &self.budget.held_bytes,
                extra,
                limits.maximum_pending_bytes_per_connection,
                self.bytes,
            ) {
                return false;
            }
            if !reserve_against(
                &admission.pending_bytes,
                extra,
                limits.maximum_pending_bytes,
                self.bytes,
            ) {
                self.budget.held_bytes.fetch_sub(extra, Ordering::SeqCst);
                return false;
            }
        }
        self.bytes += extra;
        admission.note_pending_bytes_peak();
        true
    }

    /// 这一笔放回 `released` 字节（工作线程边编边记、编到一半记不进时把已经记下的放回）。
    fn shrink(&mut self, released: usize) {
        let released = released.min(self.bytes);
        self.budget.held_bytes.fetch_sub(released, Ordering::SeqCst);
        self.admission
            .pending_bytes
            .fetch_sub(released, Ordering::SeqCst);
        self.bytes -= released;
    }
}

/// 一帧占着的待处理名额：帧数一个，字节记在 `reservation` 里（请求，回应编好之后再加上回应）。
pub struct PendingSlot {
    pub(super) admission: Arc<Admission>,
    pub(super) budget: Arc<ConnectionBudget>,
    reservation: ByteReservation,
}

impl PendingSlot {
    /// 回应编好之后记它的字节；记不进回假（调用方把回应换成忙）。
    pub fn reserve_response(&mut self, response_bytes: usize) -> bool {
        self.reservation.grow(response_bytes, false)
    }

    /// 已经做了、撤不回的回应（写确认）照记，不看上限。
    pub fn force_response(&mut self, response_bytes: usize) {
        self.reservation.grow(response_bytes, true);
    }

    /// 放回先前 `reserve_response` 记下的 `response_bytes`（回应编到一半记不进、换成忙时）。
    pub fn release_response(&mut self, response_bytes: usize) {
        self.reservation.shrink(response_bytes);
    }
}

/// 工作线程编一帧回应时的账（文件头「额度」）：边编边记进这一帧的名额（`take`），记不进就由调用方整帧回忙、已经记下的放回（`release`），
/// 不先分配整份回应；手里编着的字节另记进 `ServerCounters::worker_response_bytes`（统计报峰值），放手时减回去。
/// 弄坏开关 response-built-before-budget 下 `take` 不记额度、只记手里的字节，编完整份由工作线程再记（改前的写法）。
pub(super) struct ResponseBudget<'work> {
    slot: &'work mut PendingSlot,
    pub(super) counters: &'work ServerCounters,
    reserves_while_building: bool,
    pub(super) reserved_bytes: usize,
    held_bytes: usize,
}

impl<'work> ResponseBudget<'work> {
    pub(super) fn new(
        slot: &'work mut PendingSlot,
        counters: &'work ServerCounters,
        reserves_while_building: bool,
    ) -> ResponseBudget<'work> {
        ResponseBudget {
            slot,
            counters,
            reserves_while_building,
            reserved_bytes: 0,
            held_bytes: 0,
        }
    }

    /// 回应的缓冲要再分配 `bytes` 字节：先记进额度（记不进回假，调用方回忙），再算进手里编着的字节；0 字节不用记。
    pub(super) fn take(&mut self, bytes: usize) -> bool {
        if bytes == 0 {
            return true;
        }
        if self.reserves_while_building {
            if !self.slot.reserve_response(bytes) {
                return false;
            }
            self.reserved_bytes += bytes;
        }
        self.held_bytes += bytes;
        let now_held = self
            .counters
            .worker_response_bytes
            .fetch_add(bytes, Ordering::SeqCst)
            + bytes;
        self.counters
            .worker_response_bytes_peak
            .fetch_max(now_held, Ordering::SeqCst);
        true
    }

    /// 编到一半记不进：已经记下的放回。
    pub(super) fn release(&mut self) {
        self.slot.release_response(self.reserved_bytes);
        self.reserved_bytes = 0;
    }

    /// 编到一半记不进时回的忙（这一帧做过、回应没留，读与扫不改库，重发即可）。
    pub(super) fn busy_response(
        &mut self,
        shared: &SharedState,
        sequence: u64,
        operation_code: u8,
        wanted_bytes: usize,
    ) -> Vec<u8> {
        self.release();
        let reason = shared
            .admission
            .describe_bytes_refusal(&self.slot.budget, wanted_bytes);
        shared
            .counters
            .busy_rejections
            .fetch_add(1, Ordering::Relaxed);
        let retry_after = shared.retry_after_milliseconds();
        encode_busy(
            sequence,
            operation_code,
            retry_after,
            &format!("忙：回应编到一半{reason}；这一帧做过、回应没留，{retry_after} 毫秒后再来"),
        )
    }
}

impl Drop for ResponseBudget<'_> {
    fn drop(&mut self) {
        self.counters
            .worker_response_bytes
            .fetch_sub(self.held_bytes, Ordering::SeqCst);
    }
}

impl Drop for PendingSlot {
    fn drop(&mut self) {
        self.budget.pending_frames.fetch_sub(1, Ordering::SeqCst);
        self.admission.pending_total.fetch_sub(1, Ordering::SeqCst);
    }
}

/// 给一帧帧体分配着的字节，收齐或放弃时放手（统计里报峰值）。
pub(super) struct ReceivingBody<'admission> {
    pub(super) admission: &'admission Admission,
    pub(super) bytes: usize,
}

impl<'admission> ReceivingBody<'admission> {
    pub(super) fn start(
        admission: &'admission Admission,
        bytes: usize,
    ) -> ReceivingBody<'admission> {
        let now_receiving = admission
            .receiving_body_bytes
            .fetch_add(bytes, Ordering::SeqCst)
            + bytes;
        admission
            .receiving_body_bytes_peak
            .fetch_max(now_receiving, Ordering::SeqCst);
        ReceivingBody { admission, bytes }
    }
}

impl Drop for ReceivingBody<'_> {
    fn drop(&mut self) {
        self.admission
            .receiving_body_bytes
            .fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

/// 先加再核：加完超了上限就减回去拒掉，所以进了的帧数任何时刻都不超上限。
fn reserve(
    counter: &AtomicUsize,
    amount: usize,
    limit: usize,
    is_unbounded: bool,
) -> Result<usize, usize> {
    let reached = counter.fetch_add(amount, Ordering::SeqCst) + amount;
    if reached > limit && !is_unbounded {
        counter.fetch_sub(amount, Ordering::SeqCst);
        return Err(reached - amount);
    }
    Ok(reached)
}

impl Admission {
    pub(super) fn new(limits: AdmissionLimits, is_unbounded: bool) -> Admission {
        Admission {
            limits,
            is_unbounded,
            pending_total: AtomicUsize::new(0),
            pending_bytes: AtomicUsize::new(0),
            pending_peak: AtomicUsize::new(0),
            pending_bytes_peak: AtomicUsize::new(0),
            receiving_body_bytes: AtomicUsize::new(0),
            receiving_body_bytes_peak: AtomicUsize::new(0),
        }
    }

    fn note_pending_bytes_peak(&self) {
        self.pending_bytes_peak
            .fetch_max(self.pending_bytes.load(Ordering::SeqCst), Ordering::SeqCst);
    }

    /// 收一帧帧体之前先记它的字节（文件头「额度」）；记不进回 `None`（调用方把帧体读完丢掉、回忙）。
    pub(super) fn reserve_bytes(
        self: &Arc<Self>,
        budget: &Arc<ConnectionBudget>,
        bytes: usize,
    ) -> Option<ByteReservation> {
        let mut reservation = ByteReservation {
            admission: Arc::clone(self),
            budget: Arc::clone(budget),
            bytes: 0,
        };
        reservation.grow(bytes, false).then_some(reservation)
    }

    /// 帧数过准入；`reservation` 记着这一帧的字节，进了就交给名额，没进随错误放手。
    pub(super) fn admit(
        self: &Arc<Self>,
        budget: &Arc<ConnectionBudget>,
        reservation: ByteReservation,
    ) -> Result<PendingSlot, String> {
        let limits = self.limits;
        reserve(
            &budget.pending_frames,
            1,
            limits.maximum_pending_per_connection,
            self.is_unbounded,
        )
        .map_err(|held| {
            format!(
                "这条连接已有 {held} 帧待处理，上限 {}",
                limits.maximum_pending_per_connection
            )
        })?;
        let total = match reserve(
            &self.pending_total,
            1,
            limits.maximum_pending_total,
            self.is_unbounded,
        ) {
            Ok(total) => total,
            Err(held) => {
                budget.pending_frames.fetch_sub(1, Ordering::SeqCst);
                return Err(format!(
                    "服务已有 {held} 帧待处理，上限 {}",
                    limits.maximum_pending_total
                ));
            }
        };
        self.pending_peak.fetch_max(total, Ordering::SeqCst);
        Ok(PendingSlot {
            admission: Arc::clone(self),
            budget: Arc::clone(budget),
            reservation,
        })
    }

    /// 记不进额度时说明里报的现状。
    pub(super) fn describe_bytes_refusal(&self, budget: &ConnectionBudget, bytes: usize) -> String {
        format!(
            "{bytes} 字节记不进额度：服务记着 {} 字节（上限 {}），这条连接记着 {} 字节（上限 {}）",
            self.pending_bytes.load(Ordering::SeqCst),
            self.limits.maximum_pending_bytes,
            budget.held_bytes.load(Ordering::SeqCst),
            self.limits.maximum_pending_bytes_per_connection
        )
    }

    pub(super) fn retry_after_milliseconds(&self) -> u32 {
        u32::try_from(self.limits.busy_retry.as_millis()).unwrap_or(u32::MAX)
    }
}
