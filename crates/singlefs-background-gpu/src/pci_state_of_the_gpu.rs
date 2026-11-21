//! 判一张卡用不了、要报警的那一刻照它的 PCI 地址读本机的 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」）：
//! PCI 设备目录（本机是 `/sys/bus/pci/devices`，调用方可以换成别的目录）下以卡的地址为名的那个目录在不在、使能位（`enable`）、
//! 链路此刻与最大的速度和宽度（`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`）、
//! 高级错误报告的三类出错计数（`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`）。每一样各自是读到、文件不在、读不通之一，读不到的不当成 0、不当成正常。
//! 读数的类型住核心层（`singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable`），核心层只带着它进停下记录与报警；读只在这里做。
//! 不读内核日志、不起任何外部程序，只读普通用户读得到的那几份文件。
//!
//! 读的时刻只有三种（第 4 条）：挂着时一批在同一张卡上等超时用完、判它用不了（`GpuFailureCause::UnusableAfterResubmissions`）；开卡重开用完都开不成
//! （挂着切换的与挂载那一刻的都算）；卡当场报出错（设备丢了、wgpu 报了没接住的错，`GpuFailureCause::UncapturedErrorOrDeviceLost`）使后台任务停下。
//! 平常的批、每次超时重交之前、开卡每一次没开成而还要重开时都不读：读的那几处在 `crate::background_gpu` 与 `crate::gpus_of_this_host`，
//! 判一个出错是不是读的时刻在 [`whether_the_cause_judges_the_gpu_unusable_in_a_batch`]。

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use singlefs_core::background_compute::{
    GpuDeviceIdentifier, PciDeviceDirectoryReading, PciDeviceEnable, PciFileReading,
    PciStateOfTheGpu, PciStateOfTheGpuReadWhenJudgedUnusable,
};

use crate::background_gpu::GpuFailureCause;

/// 本机的 PCI 设备目录：每个 PCI 设备在它下面有一个以地址为名的目录。
pub const PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST: &str = "/sys/bus/pci/devices";

/// 照卡的 PCI 地址读 PCI 状态的那一份：在哪个目录下读（产品是 [`PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST`]，测试换成自己造的目录），
/// 与读了几次（每查一张卡算一次，地址当不了目录名、没真读的那一次也算；克隆出来的几份共用这一个数）。
#[derive(Clone, Debug)]
pub struct PciStateReaderOfTheGpus {
    pci_device_directories_root: PathBuf,
    readings_taken: Arc<AtomicU64>,
}

impl PciStateReaderOfTheGpus {
    /// 产品那一份：读本机的 PCI 设备目录。
    #[must_use]
    pub fn of_this_host() -> Self {
        Self::under_the_pci_device_directories_root(PathBuf::from(
            PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST,
        ))
    }

    /// 在 `pci_device_directories_root` 下读（每张卡读它下面以地址为名的那个目录）：测试拿它读自己造的目录，不读本机。
    #[must_use]
    pub fn under_the_pci_device_directories_root(pci_device_directories_root: PathBuf) -> Self {
        Self {
            pci_device_directories_root,
            readings_taken: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 到此刻为止查了几次卡（这一份与它的克隆合起来）。
    #[must_use]
    pub fn readings_taken(&self) -> u64 {
        self.readings_taken.load(Ordering::SeqCst)
    }

    /// 查一次 `device` 这张卡：它的标识当得了一个目录名就读那个目录与九份文件（[`pci_state_under_the_device_directory`]），
    /// 当不了就不读、报 [`PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName`]。查的次数加一。
    #[must_use]
    pub fn pci_state_of_the_gpu(
        &self,
        device: &GpuDeviceIdentifier,
    ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        self.readings_taken.fetch_add(1, Ordering::SeqCst);
        match pci_bus_address_usable_as_a_directory_name(device.opaque_bytes()) {
            Some(pci_bus_address) => PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(
                pci_state_under_the_device_directory(
                    pci_bus_address,
                    self.pci_device_directories_root.join(pci_bus_address),
                ),
            )),
            None => PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
                device: device.clone(),
            },
        }
    }

    /// 挂着时一批在 `device` 这张卡上以 `cause` 出错：是判它用不了的时刻（[`whether_the_cause_judges_the_gpu_unusable_in_a_batch`]）就查一次，
    /// 不是就不查、交回没读。
    #[must_use]
    pub fn pci_state_if_the_cause_judges_the_gpu_unusable(
        &self,
        device: &GpuDeviceIdentifier,
        cause: &GpuFailureCause,
    ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        if whether_the_cause_judges_the_gpu_unusable_in_a_batch(cause) {
            self.pci_state_of_the_gpu(device)
        } else {
            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
        }
    }
}

/// 挂着时一批在一张卡上的出错是不是判那张卡用不了、要读 PCI 状态的时刻：等超时用完（[`GpuFailureCause::UnusableAfterResubmissions`]）
/// 与卡当场报出错（[`GpuFailureCause::UncapturedErrorOrDeviceLost`]）是；一次超时（还要重交）、等提交没等成、回读坏了、核留下认不出的判定字、
/// 建缓冲显存不足、调用方在停而放弃的那一等（[`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]：这张卡没被判坏）与开卡那几种不是。
/// 重开到上限都没开成（[`GpuFailureCause::NotOpenedAfterTheReopenings`]）也是判用不了的时刻，可它的读数在判它开不成
/// 那一刻就读了、随开不成的那一份带着，这里不再读。
#[must_use]
pub fn whether_the_cause_judges_the_gpu_unusable_in_a_batch(cause: &GpuFailureCause) -> bool {
    match cause {
        GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. } => true,
        GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        | GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
        | GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => false,
    }
}

/// 卡的标识当得了 PCI 设备目录下的一个目录名时交回那个名字：是 UTF-8、不空、不是 `.` 或 `..`、不带 `/` 与 NUL 字节；
/// 当不了交回 `None`（拼进路径会读到那个目录之外去，或拼不成路径）。
#[must_use]
pub fn pci_bus_address_usable_as_a_directory_name(opaque_bytes: &[u8]) -> Option<&str> {
    let pci_bus_address = std::str::from_utf8(opaque_bytes).ok()?;
    let is_one_directory_name = !pci_bus_address.is_empty()
        && pci_bus_address != "."
        && pci_bus_address != ".."
        && !pci_bus_address.contains('/')
        && !pci_bus_address.contains('\0');
    is_one_directory_name.then_some(pci_bus_address)
}

/// 读 `device_directory`（地址 `pci_bus_address` 的那个目录）：目录在不在，与九份文件各自的读数。目录不在时那九份照样各读一次（都报文件不在）。
#[must_use]
pub fn pci_state_under_the_device_directory(
    pci_bus_address: &str,
    device_directory: PathBuf,
) -> PciStateOfTheGpu {
    let text_of = |file_name: &str| {
        pci_file_reading_with_the_text_mapped(
            text_of_the_pci_file(&device_directory.join(file_name)),
            text_without_the_line_end,
        )
    };
    PciStateOfTheGpu {
        pci_bus_address: pci_bus_address.to_string(),
        device_directory_reading: reading_of_the_device_directory(&device_directory),
        device_enable: pci_file_reading_with_the_text_mapped(text_of("enable"), |text| {
            pci_device_enable_of_the_text(&text)
        }),
        current_link_speed: text_of("current_link_speed"),
        current_link_width: text_of("current_link_width"),
        maximum_link_speed: text_of("max_link_speed"),
        maximum_link_width: text_of("max_link_width"),
        advanced_error_reporting_correctable: text_of("aer_dev_correctable"),
        advanced_error_reporting_fatal: text_of("aer_dev_fatal"),
        advanced_error_reporting_nonfatal: text_of("aer_dev_nonfatal"),
        device_directory,
    }
}

/// 一个 PCI 设备目录在不在：打得开是在，系统报找不到是不在，别的错是读不通（带系统报的原话）。
fn reading_of_the_device_directory(device_directory: &Path) -> PciDeviceDirectoryReading {
    match std::fs::read_dir(device_directory) {
        Ok(_entries) => PciDeviceDirectoryReading::Present,
        Err(error) if error.kind() == ErrorKind::NotFound => PciDeviceDirectoryReading::NotPresent,
        Err(error) => PciDeviceDirectoryReading::NotReadable {
            system_error: error.to_string(),
        },
    }
}

/// 读一份 PCI 属性文件的全文：系统报找不到是文件不在，别的错是读不通（带系统报的原话）。内容按 UTF-8 解，解不了的字节换成替换字符（这几份文件都是 ASCII）。
fn text_of_the_pci_file(path_of_the_file: &Path) -> PciFileReading<String> {
    match std::fs::read(path_of_the_file) {
        Ok(bytes) => PciFileReading::Read(String::from_utf8_lossy(&bytes).into_owned()),
        Err(error) if error.kind() == ErrorKind::NotFound => PciFileReading::FileNotPresent,
        Err(error) => PciFileReading::NotReadable {
            system_error: error.to_string(),
        },
    }
}

/// 读到的那一份原文换成 `content_of_the_text` 交回的；文件不在、读不通照原样。
fn pci_file_reading_with_the_text_mapped<Content>(
    reading: PciFileReading<String>,
    content_of_the_text: impl FnOnce(String) -> Content,
) -> PciFileReading<Content> {
    match reading {
        PciFileReading::Read(text) => PciFileReading::Read(content_of_the_text(text)),
        PciFileReading::FileNotPresent => PciFileReading::FileNotPresent,
        PciFileReading::NotReadable { system_error } => {
            PciFileReading::NotReadable { system_error }
        }
    }
}

/// 去掉文件原文末尾的一个换行（内核写这几份文件时末尾带一个），别的照原样：多行的出错计数中间的换行留着。
fn text_without_the_line_end(mut text: String) -> String {
    if text.ends_with('\n') {
        text.pop();
    }
    text
}

/// `enable` 的原文换成使能位：去掉首尾空白是一个十进制数时，0 是关的、别的是开着（带那个数）；不是十进制数的照原文报认不得。
#[must_use]
pub fn pci_device_enable_of_the_text(text: &str) -> PciDeviceEnable {
    let trimmed_text = text.trim();
    match trimmed_text.parse::<u64>() {
        Ok(0) => PciDeviceEnable::Disabled,
        Ok(enable_count) => PciDeviceEnable::Enabled { enable_count },
        Err(_not_a_decimal_number) => PciDeviceEnable::UnrecognizedText {
            text: trimmed_text.to_string(),
        },
    }
}

/// 测试用：一个不在的 PCI 设备目录（根目录下、名字带本包：建它要超级用户，测试与产品都不建），照它读的每张卡都是目录不在、九份文件都不在，
/// 不随哪台机器、哪个临时目录变。用例照它造读数的期望。
#[cfg(test)]
pub(crate) fn pci_device_directories_root_not_present_for_tests() -> PathBuf {
    PathBuf::from("/singlefs-background-gpu-tests-pci-device-directories-never-created")
}

/// 测试用：在 [`pci_device_directories_root_not_present_for_tests`] 下读的那一份。
#[cfg(test)]
pub(crate) fn pci_state_reader_under_a_root_not_present_for_tests() -> PciStateReaderOfTheGpus {
    PciStateReaderOfTheGpus::under_the_pci_device_directories_root(
        pci_device_directories_root_not_present_for_tests(),
    )
}

/// 测试用：照 [`pci_state_reader_under_a_root_not_present_for_tests`] 读 `pci_bus_address` 那张卡该得的读数：目录不在、九份文件都不在。
#[cfg(test)]
pub(crate) fn pci_state_read_under_the_root_not_present_for_tests(
    pci_bus_address: &str,
) -> PciStateOfTheGpuReadWhenJudgedUnusable {
    PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(PciStateOfTheGpu {
        pci_bus_address: pci_bus_address.to_string(),
        device_directory: pci_device_directories_root_not_present_for_tests().join(pci_bus_address),
        device_directory_reading: PciDeviceDirectoryReading::NotPresent,
        device_enable: PciFileReading::FileNotPresent,
        current_link_speed: PciFileReading::FileNotPresent,
        current_link_width: PciFileReading::FileNotPresent,
        maximum_link_speed: PciFileReading::FileNotPresent,
        maximum_link_width: PciFileReading::FileNotPresent,
        advanced_error_reporting_correctable: PciFileReading::FileNotPresent,
        advanced_error_reporting_fatal: PciFileReading::FileNotPresent,
        advanced_error_reporting_nonfatal: PciFileReading::FileNotPresent,
    }))
}

/// 测试用：用例自己造的 PCI 设备目录：建在临时目录下（名字带进程号、用例名与本进程里的序号，几条用例并行互不相撞），丢掉时整个删掉（成功、失败、panic 都删）。
#[cfg(test)]
pub(crate) struct PciDeviceDirectoriesMadeByTheTest {
    root: PathBuf,
}

#[cfg(test)]
impl PciDeviceDirectoriesMadeByTheTest {
    pub(crate) fn named_after_the_test(test_name: &str) -> Self {
        static DIRECTORIES_MADE_IN_THIS_PROCESS: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);
        let directory_number = DIRECTORIES_MADE_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "singlefs-pci-device-directories-{}-{test_name}-{directory_number}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("临时目录下建得了用例自己的目录");
        Self { root }
    }

    /// 在地址 `pci_bus_address` 的目录下写一份 `file_name`，内容 `text`（目录没有就建）。
    pub(crate) fn with_the_file(self, pci_bus_address: &str, file_name: &str, text: &str) -> Self {
        let device_directory = self.root.join(pci_bus_address);
        std::fs::create_dir_all(&device_directory).expect("临时目录下建得了设备目录");
        std::fs::write(device_directory.join(file_name), text).expect("临时目录下写得了文件");
        self
    }

    /// 在地址 `pci_bus_address` 的目录下把 `file_name` 建成一个目录：读它报读不通（是目录、不是文件）。
    pub(crate) fn with_a_directory_in_place_of_the_file(
        self,
        pci_bus_address: &str,
        file_name: &str,
    ) -> Self {
        std::fs::create_dir_all(self.root.join(pci_bus_address).join(file_name))
            .expect("临时目录下建得了目录");
        self
    }

    /// 地址 `pci_bus_address` 那张卡的设备目录。
    pub(crate) fn device_directory_of(&self, pci_bus_address: &str) -> PathBuf {
        self.root.join(pci_bus_address)
    }

    pub(crate) fn reader(&self) -> PciStateReaderOfTheGpus {
        PciStateReaderOfTheGpus::under_the_pci_device_directories_root(self.root.clone())
    }
}

#[cfg(test)]
impl Drop for PciDeviceDirectoriesMadeByTheTest {
    fn drop(&mut self) {
        // 删不掉不让用例因此 panic（丢的时候可能正在 panic）；留下的只在临时目录里。
        let _left_in_the_temporary_directory = std::fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use singlefs_core::background_compute::{
        hardware_judgement_of_the_pci_state, HardwareJudgementOfTheGpu, ReasonTheGpuIsOffTheBus,
    };

    fn card(pci_bus_address: &str) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
    }

    const CORRECTABLE_ERRORS_OF_THE_TEST: &str = "RxErr 0\nBadTLP 2\nTOTAL_ERR_COR 2\n";

    /// 一张卡的目录与九份文件全在：读到的每一样是原文去掉末尾换行，使能位解成开着，链路与出错计数照原文带出（多行的中间换行留着），
    /// 读数里带着照哪个地址、哪个目录读的；判不出硬件坏。读的次数是 1。
    #[test]
    fn every_file_present_is_read_as_its_text_with_the_address_and_the_directory_read() {
        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("every-file")
            .with_the_file("0000:01:00.0", "enable", "1\n")
            .with_the_file("0000:01:00.0", "current_link_speed", "5.0 GT/s PCIe\n")
            .with_the_file("0000:01:00.0", "current_link_width", "4\n")
            .with_the_file("0000:01:00.0", "max_link_speed", "32.0 GT/s PCIe\n")
            .with_the_file("0000:01:00.0", "max_link_width", "16\n")
            .with_the_file(
                "0000:01:00.0",
                "aer_dev_correctable",
                CORRECTABLE_ERRORS_OF_THE_TEST,
            )
            .with_the_file("0000:01:00.0", "aer_dev_fatal", "TOTAL_ERR_FATAL 0\n")
            .with_the_file("0000:01:00.0", "aer_dev_nonfatal", "TOTAL_ERR_NONFATAL 1\n");
        let reader = directories.reader();
        let reading = reader.pci_state_of_the_gpu(&card("0000:01:00.0"));
        assert_eq!(
            reading,
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(PciStateOfTheGpu {
                pci_bus_address: "0000:01:00.0".to_string(),
                device_directory: directories.root.join("0000:01:00.0"),
                device_directory_reading: PciDeviceDirectoryReading::Present,
                device_enable: PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
                current_link_speed: PciFileReading::Read("5.0 GT/s PCIe".to_string()),
                current_link_width: PciFileReading::Read("4".to_string()),
                maximum_link_speed: PciFileReading::Read("32.0 GT/s PCIe".to_string()),
                maximum_link_width: PciFileReading::Read("16".to_string()),
                advanced_error_reporting_correctable: PciFileReading::Read(
                    "RxErr 0\nBadTLP 2\nTOTAL_ERR_COR 2".to_string()
                ),
                advanced_error_reporting_fatal: PciFileReading::Read(
                    "TOTAL_ERR_FATAL 0".to_string()
                ),
                advanced_error_reporting_nonfatal: PciFileReading::Read(
                    "TOTAL_ERR_NONFATAL 1".to_string()
                ),
            }))
        );
        assert_eq!(
            reading.hardware_judgement(),
            Some(HardwareJudgementOfTheGpu::NotJudgedOffTheBus)
        );
        assert_eq!(reader.readings_taken(), 1);
    }

    /// 地址的目录不在：目录报不在、九份文件都报文件不在（不当成 0），判卡不在总线上。
    #[test]
    fn the_device_directory_not_present_is_read_as_off_the_bus_with_every_file_not_present() {
        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("no-directory")
            .with_the_file("0000:10:00.0", "enable", "1\n");
        let reading = directories
            .reader()
            .pci_state_of_the_gpu(&card("0000:01:00.0"));
        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
        };
        assert_eq!(
            pci_state.device_directory_reading,
            PciDeviceDirectoryReading::NotPresent
        );
        let file_not_present = PciFileReading::<String>::FileNotPresent;
        assert_eq!(
            [
                &pci_state.current_link_speed,
                &pci_state.current_link_width,
                &pci_state.maximum_link_speed,
                &pci_state.maximum_link_width,
                &pci_state.advanced_error_reporting_correctable,
                &pci_state.advanced_error_reporting_fatal,
                &pci_state.advanced_error_reporting_nonfatal,
            ],
            [&file_not_present; 7]
        );
        assert_eq!(pci_state.device_enable, PciFileReading::FileNotPresent);
        assert_eq!(
            hardware_judgement_of_the_pci_state(pci_state),
            HardwareJudgementOfTheGpu::OffTheBus(
                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent
            )
        );
    }

    /// 目录在、使能位读到 0：解成关的，判卡不在总线上。没有的三份出错计数文件报文件不在。
    #[test]
    fn the_device_enable_reading_zero_is_read_as_disabled_and_off_the_bus() {
        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("disabled")
            .with_the_file("0000:01:00.0", "enable", "0\n");
        let reading = directories
            .reader()
            .pci_state_of_the_gpu(&card("0000:01:00.0"));
        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
        };
        assert_eq!(
            pci_state.device_enable,
            PciFileReading::Read(PciDeviceEnable::Disabled)
        );
        assert_eq!(
            pci_state.advanced_error_reporting_fatal,
            PciFileReading::FileNotPresent
        );
        assert_eq!(
            hardware_judgement_of_the_pci_state(pci_state),
            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
        );
    }

    /// 使能位那份读不通（那里是个目录）：报读不通、带系统报的原话，不当成关的、不判坏。
    #[test]
    fn the_device_enable_not_readable_is_reported_with_the_system_error_and_not_judged_off_the_bus()
    {
        let directories =
            PciDeviceDirectoriesMadeByTheTest::named_after_the_test("enable-unreadable")
                .with_the_file("0000:01:00.0", "current_link_width", "16\n")
                .with_a_directory_in_place_of_the_file("0000:01:00.0", "enable");
        let reading = directories
            .reader()
            .pci_state_of_the_gpu(&card("0000:01:00.0"));
        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
        };
        assert!(
            matches!(
                &pci_state.device_enable,
                PciFileReading::NotReadable { system_error } if !system_error.is_empty()
            ),
            "使能位那份是目录：读不通，读数是 {:?}",
            pci_state.device_enable
        );
        assert_eq!(
            pci_state.current_link_width,
            PciFileReading::Read("16".to_string())
        );
        assert_eq!(
            hardware_judgement_of_the_pci_state(pci_state),
            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
        );
    }

    /// 卡的标识当不了一个目录名（带 `/`、是 `..`、不是 UTF-8）：不读那个目录之外的东西，报地址当不了目录名、带着那张卡；读的次数照样每张加一。
    #[test]
    fn identifiers_not_usable_as_a_directory_name_are_not_read_outside_the_root() {
        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("bad-address");
        let reader = directories.reader();
        let identifiers = [
            card("../0000:01:00.0"),
            card(".."),
            GpuDeviceIdentifier::of_opaque_bytes(vec![0xff, 0x30]),
        ];
        let readings: Vec<PciStateOfTheGpuReadWhenJudgedUnusable> = identifiers
            .iter()
            .map(|device| reader.pci_state_of_the_gpu(device))
            .collect();
        assert_eq!(
            readings,
            identifiers
                .iter()
                .map(|device| {
                    PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
                        device: device.clone(),
                    }
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(reader.readings_taken(), 3);
    }

    /// 卡的标识当不了一个目录名的另三种（是空的、是 `.`、带 NUL 字节）各判当不了；地址的样子（`0000:01:00.0`）判当得了、原样交回。
    /// 空的与 `.` 拼进路径读到的是 PCI 设备目录本身，带 NUL 的拼不成路径。
    #[test]
    fn empty_dot_and_nul_identifiers_are_not_usable_as_a_directory_name() {
        assert_eq!(
            [
                pci_bus_address_usable_as_a_directory_name(b""),
                pci_bus_address_usable_as_a_directory_name(b"."),
                pci_bus_address_usable_as_a_directory_name(b"0000:01:00.0\0"),
                pci_bus_address_usable_as_a_directory_name(b"0000:01:00.0"),
            ],
            [None, None, None, Some("0000:01:00.0")]
        );
    }

    /// `enable` 的原文：去掉首尾空白之后的十进制数，0 是关的、别的带计数开着；不是十进制数的照去掉空白的原文报认不得。
    #[test]
    fn device_enable_texts_are_read_as_disabled_enabled_with_the_count_or_unrecognized() {
        assert_eq!(
            [
                pci_device_enable_of_the_text("0\n"),
                pci_device_enable_of_the_text("2\n"),
                pci_device_enable_of_the_text(" on\n"),
            ],
            [
                PciDeviceEnable::Disabled,
                PciDeviceEnable::Enabled { enable_count: 2 },
                PciDeviceEnable::UnrecognizedText {
                    text: "on".to_string()
                },
            ]
        );
    }

    /// 一批里哪些出错是读的时刻：等超时用完与卡当场报出错是；一次超时（还要重交）、等提交没等成、回读坏了不是。
    #[test]
    fn only_the_resubmissions_used_up_and_the_device_lost_judge_the_gpu_unusable_in_a_batch() {
        let wait_limit = std::time::Duration::from_secs(10);
        assert_eq!(
            [
                GpuFailureCause::UnusableAfterResubmissions {
                    resubmissions: 3,
                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit
                    }),
                },
                GpuFailureCause::UncapturedErrorOrDeviceLost {
                    reason: "替身：设备丢了".to_string(),
                },
                GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
                GpuFailureCause::WaitForTheSubmissionFailed {
                    reason: "替身：没等成".to_string(),
                },
                GpuFailureCause::ReadbackMappingFailed {
                    reason: "替身：映射不成".to_string(),
                },
            ]
            .iter()
            .map(whether_the_cause_judges_the_gpu_unusable_in_a_batch)
            .collect::<Vec<bool>>(),
            vec![true, true, false, false, false]
        );
    }

    /// 调用方在停而放弃的那一等不是判卡用不了的时刻：不读 PCI 状态（第 4 条「卸载时不等还在跑的提交」只放弃那一等，不报这张卡坏了）。
    #[test]
    fn giving_up_the_wait_for_the_caller_stopping_does_not_judge_the_gpu_unusable() {
        assert!(!whether_the_cause_judges_the_gpu_unusable_in_a_batch(
            &GpuFailureCause::GaveUpWaitingForTheCallerIsStopping
        ));
        let pci_state_reader = pci_state_reader_under_a_root_not_present_for_tests();
        assert_eq!(
            pci_state_reader.pci_state_if_the_cause_judges_the_gpu_unusable(
                &card("0000:01:00.0"),
                &GpuFailureCause::GaveUpWaitingForTheCallerIsStopping,
            ),
            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
        );
        assert_eq!(pci_state_reader.readings_taken(), 0, "一次都没读");
    }
}
