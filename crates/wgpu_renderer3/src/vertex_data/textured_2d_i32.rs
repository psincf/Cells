use euclid::default::{Point2D, Point3D};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex2DTexturedi32 {
    pub position: Point3D<i32>,
    pub texture_pos: Point3D<f32>,
}

pub struct Vertex2DTexturedi32Buffer {
    pub data: Vec<Vertex2DTexturedi32>,
}

impl Vertex2DTexturedi32Buffer {
    pub fn new() -> Vertex2DTexturedi32Buffer {
        Vertex2DTexturedi32Buffer {
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
        self.data.len() * std::mem::size_of::<Vertex2DTexturedi32>()
    }
}

#[inline]
pub fn triangulate_textured_2d_i32(
    pos_min: Point2D<i32>,
    pos_max: Point2D<i32>,
    pos_z: i32,
    tex_min: Point2D<f32>,
    tex_max: Point2D<f32>,
    tex_layer: f32,
) -> [Vertex2DTexturedi32;6] {
    [
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_min.x, pos_min.y, pos_z),
            texture_pos: Point3D::new(tex_min.x, tex_min.y, tex_layer),
        },
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_max.x, pos_min.y, pos_z),
            texture_pos: Point3D::new(tex_max.x, tex_min.y, tex_layer),
        },
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_min.x, pos_max.y, pos_z),
            texture_pos: Point3D::new(tex_min.x, tex_max.y, tex_layer),
        },
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_min.x, pos_max.y, pos_z),
            texture_pos: Point3D::new(tex_min.x, tex_max.y, tex_layer),
        },
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_max.x, pos_min.y, pos_z),
            texture_pos: Point3D::new(tex_max.x, tex_min.y, tex_layer),
        },
        Vertex2DTexturedi32 {
            position: Point3D::new(pos_max.x, pos_max.y, pos_z),
            texture_pos: Point3D::new(tex_max.x, tex_max.y, tex_layer),
        },
        
    ]
}