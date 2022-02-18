use super::*;

pub fn process_subdivision(render: &mut Render, octree: &mut Octree, cpu_octree: &mut Octree) {
    let slice = render.subdivision_buffer.slice(..);
    let future = slice.map_async(wgpu::MapMode::Read);

    render.device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = pollster::block_on(future) {
        let mut data = slice.get_mapped_range_mut();
        let result: &mut [u32] = unsafe { reinterpret::reinterpret_mut_slice(&mut data) };

        // Reset atomic counter
        let len = result[0] as usize;
        result[0] = 0;

        for i in 1..=len {
            let node_index = result[i] as usize;

            if octree.get_node(node_index) < VOXEL_OFFSET {
                // println!("Doubleup!");
                continue;
            }

            // println!("Subdividing node {}", result[i]);
            
            let pos = octree.positions[node_index];
            let (voxel_index, voxel_depth, _) = octree.find_voxel(pos, None);
            let (cpu_index, _, _) = cpu_octree.find_voxel(pos, Some(voxel_depth));
            
            if voxel_depth < 20 {
                octree.subdivide(node_index, cpu_octree.get_node_mask(cpu_index), voxel_depth + 1);
            }
            

            if voxel_index != node_index {
                panic!("Incorrect voxel position!");
            }

            result[i] = 0;
        }

        drop(data);
        render.subdivision_buffer.unmap();
    } else {
        panic!("Failed to run get subdivision buffer!")
    }
}

pub fn process_unsubdivision(compute: &mut Compute, render: &mut Render) {
    let slice = compute.unsubdivision_buffer.slice(..);
    let future = slice.map_async(wgpu::MapMode::Read);

    render.device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = pollster::block_on(future) {
        let mut data = slice.get_mapped_range_mut();
        let result: &mut [u32] = unsafe { reinterpret::reinterpret_mut_slice(&mut data) };

        let len = result[0] as usize;
        result[0] = 0;

        for i in 1..=len {
            // // Compute shader returns VOXEL_OFFSET + voxel_index for a subdivision and node_index for a unsubdivision
            // if result[i] >= octree::VOXEL_OFFSET {
            //     println!("Subdivide: {}", result[i] - octree::VOXEL_OFFSET);

            //     let voxel_index = result[i] - octree::VOXEL_OFFSET;
            //     let pos = octree.voxel_positions[voxel_index as usize];
            //     Compute::subdivide_octree(pos, octree, cpu_octree);
            // } else {
            //     let tnipt = octree.nodes[result[i] as usize];
            //     if tnipt >= VOXEL_OFFSET {
            //         println!("Doubleup!");
            //     } else {
            //         println!("Unsubdivide: {}", result[i]);
            //         octree.unsubdivide(result[i] as usize);
            //     }
            // }

            result[i] = 0;
        }

        drop(data);
        compute.unsubdivision_buffer.unmap();
    } else {
        panic!("failed to run compute on gpu!")
    }
}
