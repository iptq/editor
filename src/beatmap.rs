use ggez::graphics::Color;
use libosu::{beatmap::Beatmap, hitobject::HitObjectKind, math::Point};

use crate::hitobject::HitObjectExt;

pub const STACK_DISTANCE: f64 = 3.0;

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

    pub fn compute_colors(&mut self, colors: &[Color]) {
        let mut color_idx = 0;
        let mut number = 1;
        for ho in self.hit_objects.iter_mut() {
            if ho.inner.new_combo {
                number = 1;
                color_idx = (color_idx + 1) % colors.len();
            }

            ho.number = number;
            ho.color_idx = color_idx;
            number += 1;
        }
    }

    pub fn compute_stacking(&mut self) {
        if self.inner.stack_leniency > 0.0 {
            self.compute_stacking_inner(0, self.hit_objects.len() - 1)
        }
    }

    fn compute_stacking_inner(&mut self, start_idx: usize, end_idx: usize) {
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
                    if stack_base_pos.distance(object_n_pos) < STACK_DISTANCE
                        || (stack_base_obj.inner.kind.is_slider()
                            && stack_base_obj
                                .inner
                                .end_pos()
                                .unwrap()
                                .distance(object_n_pos)
                                < STACK_DISTANCE)
                    {
                        stack_base_idx = n;
                        self.hit_objects[n].stacking = 0;
                    }
                }

                if stack_base_idx > extended_end_idx {
                    extended_end_idx = stack_base_idx;
                    if extended_end_idx == self.hit_objects.len() - 1 {
                        break;
                    }
                }
            }
        }

        // Reverse pass for stack calculation.
        let mut extended_start_idx = start_idx;

        for i in (start_idx..=extended_end_idx).rev() {
            let mut n = i;

            // We should check every note which has not yet got a stack.
            // Consider the case we have two interwound stacks and this will make sense.
            // o <-1      o <-2
            //  o <-3      o <-4
            // We first process starting from 4 and handle 2,
            // then we come backwards on the i loop iteration until we reach 3 and handle 1.
            // 2 and 1 will be ignored in the i loop because they already have a stack value.

            let object_i = &self.hit_objects[i];
            let mut iidx = i;
            let start_time = object_i.inner.start_time.0;
            if object_i.stacking != 0 || object_i.inner.kind.is_spinner() {
                continue;
            }

            let stack_threshold =
                self.inner.difficulty.approach_preempt() as f64 * self.inner.stack_leniency;

            match object_i.inner.kind {
                HitObjectKind::Circle => {
                    for n in (0..n).rev() {
                        if self.hit_objects[n].inner.kind.is_spinner() {
                            continue;
                        }

                        let end_time = self
                            .inner
                            .get_hitobject_end_time(&self.hit_objects[n].inner);

                        if (self.hit_objects[iidx].inner.start_time.0 - end_time.0) as f64
                            > stack_threshold
                        {
                            break;
                        }

                        if n < extended_start_idx {
                            self.hit_objects[n].stacking = 0;
                            extended_start_idx = n;
                        }

                        if self.hit_objects[n].inner.kind.is_slider()
                            && self.hit_objects[n]
                                .inner
                                .end_pos()
                                .unwrap()
                                .distance(self.hit_objects[iidx].inner.pos.to_float().unwrap())
                                < STACK_DISTANCE
                        {
                            let offset =
                                self.hit_objects[iidx].stacking - self.hit_objects[n].stacking + 1;

                            for j in n + 1..=i {
                                if self.hit_objects[n]
                                    .inner
                                    .end_pos()
                                    .unwrap()
                                    .distance(self.hit_objects[j].inner.pos.to_float().unwrap())
                                    < STACK_DISTANCE
                                {
                                    self.hit_objects[j].stacking -= offset;
                                }
                            }

                            break;
                        }

                        if self.hit_objects[n]
                            .inner
                            .pos
                            .to_float::<f64>()
                            .unwrap()
                            .distance(self.hit_objects[iidx].inner.pos.to_float().unwrap())
                            < STACK_DISTANCE
                        {
                            self.hit_objects[n].stacking = self.hit_objects[iidx].stacking + 1;
                            iidx = n;
                        }
                    }
                }
                HitObjectKind::Slider(_) => {
                    for n in (start_idx..n).rev() {
                        if self.hit_objects[n].inner.kind.is_spinner() {
                            continue;
                        }

                        if (self.hit_objects[iidx].inner.start_time.0
                            - self.hit_objects[n].inner.start_time.0)
                            as f64
                            > stack_threshold
                        {
                            break;
                        }

                        if self.hit_objects[n]
                            .inner
                            .end_pos()
                            .unwrap()
                            .distance(self.hit_objects[iidx].inner.pos.to_float().unwrap())
                            < STACK_DISTANCE
                        {
                            self.hit_objects[n].stacking = self.hit_objects[iidx].stacking + 1;
                            iidx = n;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
