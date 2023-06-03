use euclid::default::{Point2D, Point3D};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex2DColoredi32 {
    pub position: Point3D<i32>,
    pub color: (f32, f32, f32, f32),
}

pub struct Vertex2DColoredi32Buffer {
    pub data: Vec<Vertex2DColoredi32>,
}

impl Vertex2DColoredi32Buffer {
    pub fn new() -> Vertex2DColoredi32Buffer {
        Vertex2DColoredi32Buffer {
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
        self.data.len() * std::mem::size_of::<Vertex2DColoredi32>()
    }
}

#[inline]
pub fn triangulate_colored_2d_i32(
    pos_min: Point2D<i32>,
    pos_max: Point2D<i32>,
    pos_z: i32,
    color: (f32, f32, f32, f32),
) -> [Vertex2DColoredi32;6] {
    [
        Vertex2DColoredi32 {
            position: Point3D::new(pos_min.x, pos_min.y, pos_z),
            color: color,
        },
        Vertex2DColoredi32 {
            position: Point3D::new(pos_max.x, pos_min.y, pos_z),
            color: color,
        },
        Vertex2DColoredi32 {
            position: Point3D::new(pos_min.x, pos_max.y, pos_z),
            color: color,
        },
        Vertex2DColoredi32 {
            position: Point3D::new(pos_min.x, pos_max.y, pos_z),
            color: color,
        },
        Vertex2DColoredi32 {
            position: Point3D::new(pos_max.x, pos_min.y, pos_z),
            color: color,
        },
        Vertex2DColoredi32 {
            position: Point3D::new(pos_max.x, pos_max.y, pos_z),
            color: color,
        },
        
    ]
}