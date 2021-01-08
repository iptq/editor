use libosu::HitObject;

pub struct HitObjectExt {
    pub inner: HitObject,
    pub stacking: usize,
}

impl HitObjectExt {
    pub fn new(inner: HitObject) -> Self {
        HitObjectExt { inner, stacking: 0 }
    }
}
