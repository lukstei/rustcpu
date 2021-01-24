use crate::game::{Collide, PosF};

impl Collide for [f64; 4] {
    type CollideDesc = ();

    fn collide(&self, point: [f64; 2]) -> Option<()> {
        let [x1, y1] = point;

        if x1 >= self[0]
            && x1 <= self[0] + self[2]
            && y1 >= self[1]
            && y1 <= self[1] + self[3] {
            Some(())
        } else {
            None
        }
    }
}

pub fn rect_center(rect: [f64; 4]) -> PosF {
    [rect[0] + rect[2]/2., rect[1] + rect[3]/2.]
}