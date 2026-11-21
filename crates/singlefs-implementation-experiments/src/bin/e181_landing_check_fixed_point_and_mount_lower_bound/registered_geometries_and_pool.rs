//! 登记的几何、建池参数与 `build_pool_after_the_first_file` 建出的池。

use super::*;

/// 池的几何：两块盘的字节数。都用默认 768 MiB 环、S = 8（跑前登记第五节「几何」）。
#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub(super) name: &'static str,
    pub(super) device_bytes: [u64; 2],
    journal_ring_bytes: u64,
}

/// G默认：2 × 4 GiB。
pub(super) const GEOMETRY_DEFAULT: Geometry = Geometry {
    name: "default",
    device_bytes: [4 * GIBIBYTE, 4 * GIBIBYTE],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G窄：2 × (3 GiB + 33 槽)（单元区 146465 槽 = 2288 个整段加 33 槽）。
pub(super) const GEOMETRY_NARROW: Geometry = Geometry {
    name: "narrow",
    device_bytes: [
        3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL,
        3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL,
    ],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G不等大：4 GiB + (3 GiB + 33 槽)。
pub(super) const GEOMETRY_UNEQUAL: Geometry = Geometry {
    name: "unequal",
    device_bytes: [4 * GIBIBYTE, 3 * GIBIBYTE + 33 * SLOT_BYTES_LOCAL],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};
/// G宽：2 × 16 GiB（稀疏设备，只给 F11 量扫描随单元区的增长）。
pub(super) const GEOMETRY_WIDE: Geometry = Geometry {
    name: "wide",
    device_bytes: [16 * GIBIBYTE, 16 * GIBIBYTE],
    journal_ring_bytes: DEFAULT_JOURNAL_RING_BYTES_LOCAL,
};

/// 补一个小单元区的几何（登记修订 1）：2 × 432 MiB，96 MiB 环（在飞记录上限 8192 条，装得下一次写 7000 个单元的 7000 多条记录），
/// 单元区起点 U = 1024 + 6144 = 7168（段边界上），单元区 20480 槽 = 320 个整段。K1 在这里只有几十个节点，
/// 中央映射树的删插才是 ckpt_cost 里的大头——大几何里 K1 的几百个节点把这一项盖住了。
pub(super) const GEOMETRY_SMALL: Geometry = Geometry {
    name: "small_unit_area",
    device_bytes: [SMALL_GEOMETRY_DEVICE_BYTES, SMALL_GEOMETRY_DEVICE_BYTES],
    journal_ring_bytes: SMALL_GEOMETRY_JOURNAL_RING_BYTES,
};
pub(super) const SMALL_GEOMETRY_JOURNAL_RING_BYTES: u64 = 96 << 20;
pub(super) const SMALL_GEOMETRY_UNIT_AREA_START_SLOT: u64 = 1024 + (96 << 20) / SLOT_BYTES_LOCAL;
const SMALL_GEOMETRY_UNIT_AREA_SLOTS: u64 = 20480;
const SMALL_GEOMETRY_DEVICE_BYTES: u64 =
    (SMALL_GEOMETRY_UNIT_AREA_START_SLOT + SMALL_GEOMETRY_UNIT_AREA_SLOTS) * SLOT_BYTES_LOCAL;

fn make_filesystem_parameters(geometry: Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E181_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn first_file_content() -> Vec<u8> {
    (0..FIRST_FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

/// 一个池：盘、分配器、现行那一版、实例代号。拷贝它（[`Pool::fork`]）不拷盘上的扇区。
pub(super) struct Pool {
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Vec<(DeviceIdentity, ForkedDevice)>,
    pub(super) allocator: PoolAllocator,
    pub(super) current: TransactionOutput,
    pub(super) instance: InstanceGeneration,
}

impl Pool {
    /// 封存各盘的顶层（之后用 [`Pool::fork_of_a_sealed_base`] 在几个线程里各拷一份）。
    pub(super) fn seal(&mut self) {
        for (_identity, device) in &mut self.devices {
            drop(device.fork());
        }
    }

    /// 底已经封存过（[`Pool::seal`]）时不需要 `&mut` 的拷贝。
    pub(super) fn fork_of_a_sealed_base(&self) -> Pool {
        Pool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.fork_of_a_sealed_base()))
                .collect(),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }

    pub(super) fn fork(&mut self) -> Pool {
        Pool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter_mut()
                .map(|(identity, device)| (*identity, device.fork()))
                .collect(),
            allocator: self.allocator.clone(),
            current: self.current.clone(),
            instance: self.instance,
        }
    }
}

/// mkfs、取号、暖机、新池新建文件（3000 字节）都在同一个进程里做完的池（没经过可写挂载）：A3、A4 钉的 H1 就是它。
pub(super) fn build_pool_after_the_first_file(
    geometry: Geometry,
    capacities: CodeTwoTreeNodeCapacities,
) -> Pool {
    let parameters = make_filesystem_parameters(geometry);
    let mut devices: Vec<(DeviceIdentity, ForkedDevice)> = geometry
        .device_bytes
        .iter()
        .enumerate()
        .map(|(device_number, device_bytes)| {
            (
                DeviceIdentity(u32::try_from(device_number).expect("盘号")),
                ForkedDevice::empty(*device_bytes),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs 收下这个几何");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = first_file_content();
    let (instance, first_file) = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        if let CodeTwoTreeNodeCapacities::CappedForTests { .. } = capacities {
            writer.set_code_two_tree_node_capacities(capacities);
        }
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件");
        (instance, first_file)
    };
    Pool {
        parameters,
        devices,
        allocator,
        current: first_file,
        instance,
    }
}
