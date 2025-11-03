use super::types::Vertex;

pub fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    let stack_step = std::f32::consts::PI / stacks as f32;

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            vertices.push(Vertex {
                position: [x, y, z],
            });
        }
    }

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for _ in 0..sectors {
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            if i != (stacks - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    (vertices, indices)
}