use fdt::Fdt;

pub fn get_device_tree(device_tree_binary_ptr: usize) -> Fdt<'static> {

    unsafe {
        let device_tree_binary_header = core::slice::from_raw_parts(device_tree_binary_ptr as *const u32, 40);
        let total_size = device_tree_binary_header.get(1).unwrap();
        let device_tree_binary_data = core::slice::from_raw_parts(device_tree_binary_ptr as *const u8, *total_size as usize);
        return fdt::Fdt::new(device_tree_binary_data).expect("Failed to parse full FDT")
    }
}
