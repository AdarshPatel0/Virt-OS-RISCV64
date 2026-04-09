use fdt_rs::base::*;
use fdt_rs::error::DevTreeError;
use fdt_rs::prelude::*;

fn get_system_memory_node<'a>(device_tree: &'a DevTree<'a>) -> Option<DevTreeNode<'a, 'a>> {
    let mut nodes = device_tree.nodes();
    while let Some(node) = nodes.next().unwrap() {
        if node.name().ok()?.starts_with("memory") {
            return Some(node);
        }
    }
    return None;
}

pub fn get_system_memory_info<'a>(device_tree: &'a DevTree<'a>) -> Option<(usize, usize)> {
    let memory_node = get_system_memory_node(device_tree)?;
    let mut props = memory_node.props();
    while let Some(prop) = props.next().unwrap() {
        if prop.name().unwrap().starts_with("reg") {
            return Some((prop.u64(0).unwrap() as usize, prop.u64(1).unwrap() as usize));
        }
    }
    return None;
}

pub fn read_device_tree_data(device_tree_binary_ptr: usize) -> Result<DevTree<'static>, DevTreeError> {
    unsafe {
        let device_tree_binary_header = core::slice::from_raw_parts(device_tree_binary_ptr as *const u8, 40);
        let size = DevTree::read_totalsize(device_tree_binary_header)?;
        return DevTree::new(core::slice::from_raw_parts(device_tree_binary_ptr as *const u8, size));
    }
}