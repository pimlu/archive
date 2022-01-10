use cgmath::*;

use crate::*;


pub struct BoundBox {
    pub a: P2,
    pub b: P2
}

impl BoundBox {
    fn scale(&self, factor: Num) -> Self {
        let mid = (self.a + self.b)/2;
        return BoundBox {
            a: mid + (self.a - mid) * factor,
            b: mid + (self.b - mid) * factor
        };
    }
}

pub struct Line {
    pub a: P2,
    pub b: P2
}

pub enum ShapeKind {
    Circle,
    Square
}

pub struct Trans {
    pub scale: V2,
    pub rot: f64,
    pub pos: P2,
}

pub struct Shape {
    pub kind: ShapeKind,
    pub trans: D2
}



// pub trait Aabb {
//     fn aabb(&self) -> BoundBox;
// }

// impl Aabb for Shape {
//     fn aabb(&self) -> BoundBox {
//         let rot_factor = match self.kind {
//             Circle => V2(1, 1),
//             Rect(dim) => dim * 
//         };
//     }
// }