use std::marker::PhantomData;

use libosu::Point;
use num::Float;

#[derive(Default)]
pub struct Math<T>(PhantomData<T>);

impl<T: Float> Math<T> {
    pub fn circumcircle(p1: Point<T>, p2: Point<T>, p3: Point<T>) -> (Point<T>, T) {
        let (x1, y1) = (p1.0, p1.1);
        let (x2, y2) = (p2.0, p2.1);
        let (x3, y3) = (p3.0, p3.1);

        let two = num::cast::<_, T>(2.0).unwrap();
        let d = two.mul_add(x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2), T::zero());
        let ux = ((x1 * x1 + y1 * y1) * (y2 - y3)
            + (x2 * x2 + y2 * y2) * (y3 - y1)
            + (x3 * x3 + y3 * y3) * (y1 - y2))
            / d;
        let uy = ((x1 * x1 + y1 * y1) * (x3 - x2)
            + (x2 * x2 + y2 * y2) * (x1 - x3)
            + (x3 * x3 + y3 * y3) * (x2 - x1))
            / d;

        let center = Point(ux, uy);
        (center, center.distance(p1))
    }
}
