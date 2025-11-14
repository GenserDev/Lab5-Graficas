use super::types::Vertex;

pub fn create_ship() -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Cuerpo principal de la nave (prisma triangular)
    // Punta delantera
    vertices.push(Vertex { position: [0.0, 0.0, 1.5] });
    
    // Base trasera (triángulo)
    vertices.push(Vertex { position: [-0.5, 0.3, -0.5] });
    vertices.push(Vertex { position: [0.5, 0.3, -0.5] });
    vertices.push(Vertex { position: [0.0, -0.3, -0.5] });
    
    // Alas
    vertices.push(Vertex { position: [-1.2, 0.0, 0.0] }); // Ala izquierda
    vertices.push(Vertex { position: [1.2, 0.0, 0.0] });  // Ala derecha
    
    // Cabina (pequeña elevación)
    vertices.push(Vertex { position: [0.0, 0.5, 0.3] });
    
    // Cola
    vertices.push(Vertex { position: [0.0, 0.6, -0.8] });
    
    // Caras del cuerpo principal
    // Cara superior
    indices.extend_from_slice(&[0, 1, 6]);
    indices.extend_from_slice(&[0, 6, 2]);
    
    // Cara inferior
    indices.extend_from_slice(&[0, 3, 1]);
    indices.extend_from_slice(&[0, 2, 3]);
    
    // Lados
    indices.extend_from_slice(&[1, 3, 6]);
    indices.extend_from_slice(&[2, 6, 3]);
    
    // Trasera
    indices.extend_from_slice(&[1, 2, 3]);
    
    // Alas
    indices.extend_from_slice(&[0, 4, 1]);
    indices.extend_from_slice(&[0, 2, 5]);
    
    // Cola
    indices.extend_from_slice(&[6, 7, 1]);
    indices.extend_from_slice(&[6, 2, 7]);

    (vertices, indices)
}