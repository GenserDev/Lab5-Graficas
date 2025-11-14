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

pub fn create_ring(inner_radius: f32, outer_radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..segments {
        let theta = (i as f32) * (2.0 * std::f32::consts::PI) / segments as f32;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        // inner then outer vertex per segment
        vertices.push(Vertex { position: [inner_radius * cos_t, 0.0, inner_radius * sin_t] });
        vertices.push(Vertex { position: [outer_radius * cos_t, 0.0, outer_radius * sin_t] });
    }

    // Create two triangles per segment
    for i in 0..segments {
        let next = (i + 1) % segments;
        let i0 = i * 2;
        let i1 = i0 + 1;
        let n0 = next * 2;
        let n1 = n0 + 1;

        indices.push(i0 as u32);
        indices.push(n0 as u32);
        indices.push(i1 as u32);

        indices.push(i1 as u32);
        indices.push(n0 as u32);
        indices.push(n1 as u32);
    }

    (vertices, indices)
}

pub fn create_orbit(radius: f32, segments: u32, inclination: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..segments {
        let theta = (i as f32) * (2.0 * std::f32::consts::PI) / segments as f32;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        let y = theta.sin() * radius * inclination;
        vertices.push(Vertex { position: [x, y, z] });
    }

    // Line segments as index pairs for each edge
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(i as u32);
        indices.push(next as u32);
    }

    (vertices, indices)
}