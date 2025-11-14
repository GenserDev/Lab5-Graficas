use super::types::Vertex;

pub fn create_skybox() -> (Vec<Vertex>, Vec<u32>) {
    let size = 150.0;
    
    let vertices = vec![
        // Cara frontal
        Vertex { position: [-size, -size, size] },
        Vertex { position: [size, -size, size] },
        Vertex { position: [size, size, size] },
        Vertex { position: [-size, size, size] },
        
        // Cara trasera
        Vertex { position: [-size, -size, -size] },
        Vertex { position: [size, -size, -size] },
        Vertex { position: [size, size, -size] },
        Vertex { position: [-size, size, -size] },
    ];

    let indices = vec![
        // Frontal
        0, 1, 2, 0, 2, 3,
        // Trasera
        5, 4, 7, 5, 7, 6,
        // Izquierda
        4, 0, 3, 4, 3, 7,
        // Derecha
        1, 5, 6, 1, 6, 2,
        // Superior
        3, 2, 6, 3, 6, 7,
        // Inferior
        4, 5, 1, 4, 1, 0,
    ];

    (vertices, indices)
}