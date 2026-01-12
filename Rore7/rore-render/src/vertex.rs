use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2], // Faqat 2D koordinata (0..1)
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }

    // Standart Kvadrat (Quad)
    // (0,0) -> (1,1) oralig'idagi to'rtburchak
    pub const QUAD: &[Vertex] = &[
        Vertex { position: [0.0, 0.0] }, // Chap-Yuqori
        Vertex { position: [0.0, 1.0] }, // Chap-Past
        Vertex { position: [1.0, 0.0] }, // O'ng-Yuqori
        Vertex { position: [1.0, 1.0] }, // O'ng-Past
    ];
}