use libosu::{Beatmap, HitObjectKind, Point};

use crate::hit_object::HitObjectExt;

const STACK_DISTANCE: f64 = 3.0;

pub struct BeatmapExt {
    pub inner: Beatmap,
    pub hit_objects: Vec<HitObjectExt>,
}

impl BeatmapExt {
    pub fn new(inner: Beatmap) -> Self {
        let hit_objects = inner
            .hit_objects
            .iter()
            .cloned()
            .map(HitObjectExt::new)
            .collect();

        BeatmapExt { inner, hit_objects }
    }

    pub fn compute_stacking(&mut self, start_idx: usize, end_idx: usize) {
        let mut extended_end_idx = end_idx;

        if end_idx < self.hit_objects.len() - 1 {
            // Extend the end index to include objects they are stacked on
            for i in (start_idx..=end_idx).rev() {
                let mut stack_base_idx = i;

                for n in stack_base_idx + 1..self.hit_objects.len() {
                    let stack_base_obj = &self.hit_objects[stack_base_idx];
                    if let HitObjectKind::Spinner(_) = &stack_base_obj.inner.kind {
                        break;
                    }

                    let object_n = &self.hit_objects[n];
                    if let HitObjectKind::Spinner(_) = &object_n.inner.kind {
                        break;
                    }

                    let end_time = self.inner.get_hitobject_end_time(&stack_base_obj.inner);
                    let stack_threshold =
                        self.inner.difficulty.approach_preempt() as f64 * self.inner.stack_leniency;

                    // We are no longer within stacking range of the next object.
                    if (object_n.inner.start_time.0 - end_time.0) as f64 > stack_threshold {
                        break;
                    }

                    let stack_base_pos: Point<f64> = stack_base_obj.inner.pos.to_float().unwrap();
                    let object_n_pos: Point<f64> = object_n.inner.pos.to_float().unwrap();
                    // if stack_base_pos.distance(object_n_pos) < STACK_DISTANCE
                    //     || (stack_base_obj.inner.kind.is_slider()
                    //         && self
                    //             .inner
                    //             .get_hitobject_end_pos(stack_base_obj)
                    //             .distance(object_n_pos)
                    //             < STACK_DISTANCE)
                    // {}
                }
            }
        }

        let mut extended_start_idx = start_idx;
    }
}
