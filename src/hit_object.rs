use libosu::{Color, HitObject};

pub struct HitObjectExt {
    pub inner: HitObject,
    pub stacking: usize,
    pub number: usize,
    pub color_idx: usize,
}

impl HitObjectExt {
    pub fn new(inner: HitObject) -> Self {
        HitObjectExt {
            inner,
            stacking: 0,
            number: 0,
            color_idx: 0,
        }
    }
}
