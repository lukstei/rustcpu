use crate::game::Collide;

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