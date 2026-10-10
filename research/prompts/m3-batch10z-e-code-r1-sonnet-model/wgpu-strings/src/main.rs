fn main() {
    println!("hal OutOfMemory = {:?}", wgpu::hal::DeviceError::OutOfMemory.to_string());
    println!("hal Lost        = {:?}", wgpu::hal::DeviceError::Lost.to_string());
    println!("hal Unexpected  = {:?}", wgpu::hal::DeviceError::Unexpected.to_string());
    println!("wgc DeviceError::OutOfMemory = {:?}", wgpu::wgc::device::DeviceError::OutOfMemory.to_string());
    println!("wgc DeviceError::Lost        = {:?}", wgpu::wgc::device::DeviceError::Lost.to_string());
    println!("wgpu::Error::OutOfMemory     = {:?}", wgpu::Error::OutOfMemory { source: Box::new(std::fmt::Error) }.to_string());
}
