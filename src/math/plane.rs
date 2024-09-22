use super::vector3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    direction: Vec3,
    distance: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PlaneSide {
    Frontside,
    Onside,
    Backside,
}

impl Plane {
    pub fn new(direction: Vec3, distance: f32) -> Self {
        Self {
            direction: direction.normalized(),
            distance,
        }
    }

    pub fn signed_distance(&self, position: &Vec3) -> f32 {
        Vec3::dot(&self.direction, position) - self.distance
    }

    pub fn side(&self, position: &Vec3) -> PlaneSide {
        const EPS: f32 = 0.000060915946;
        let distance = self.signed_distance(position);
        if distance < -EPS {
            PlaneSide::Backside
        } else if distance > EPS {
            PlaneSide::Frontside
        } else {
            PlaneSide::Onside
        }
    }
}
