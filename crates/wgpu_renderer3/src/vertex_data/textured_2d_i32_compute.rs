use euclid::default::Point2D;

#[derive(Clone, Copy)]
pub struct Vertex2DTexturedi32Compute {
    pub position_min: Point2D<i32>,
    pub position_max: Point2D<i32>,
    pub position_z: i32,
    pub texture_min: Point2D<f32>,
    pub texture_max: Point2D<f32>,
    pub texture_layer: f32,
}

pub struct Vertex2DTexturedi32ComputeBuffer {
    pub data: Vec<Vertex2DTexturedi32Compute>,
}

impl Vertex2DTexturedi32ComputeBuffer {
    pub fn new() -> Vertex2DTexturedi32ComputeBuffer {
        Vertex2DTexturedi32ComputeBuffer {
            data: Vec::new(),
        }
    }

    pub fn as_u8(&self) -> &[u8] {
        unsafe { self.data.align_to::<u8>().1 }
    }

    pub fn vertex_count(&self) -> usize {
        self.data.len()
    }

    pub fn size(&self) -> usize {
        self.data.len() * std::mem::size_of::<Vertex2DTexturedi32Compute>()
    }
}