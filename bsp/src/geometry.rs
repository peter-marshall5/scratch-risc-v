use crate::vector;
use crate::vector::Vec3;

#[derive(Debug)]
pub struct Plane {
    normal: Vec3,
    dist: f32
}

#[derive(Debug)]
pub struct Line {
    normal: Vec3,
    origin: Vec3,
    length: f32
}

impl Line {
    pub fn from_points(p1: &Vec3, p2: &Vec3) -> Line {
        let direction = [
            p2[0] - p1[0],
            p2[1] - p1[1],
            p2[2] - p1[2],
        ];
        let length = vector::magnitude(&direction);
        let normal = vector::normalize(&direction);
        Line {
            normal: normal,
            origin: *p1,
            length: length
        }
    }
    pub fn to_point(&self, step: &f32) -> Vec3 {
        [
            self.normal[0] * step + self.origin[0],
            self.normal[1] * step + self.origin[1],
            self.normal[2] * step + self.origin[2]
        ]
    }
}

impl Plane {
    pub fn new(normal: &Vec3, point: &Vec3) -> Plane {
        Plane {
            normal: *normal,
            dist: -(normal[0] * point[0]) - (normal[1] * point[1]) - (normal[2] * point[2])
        }
    }
    pub fn from_tri(tri: &[Vec3; 3]) -> Plane {
        let dba = vector::subtract(&tri[1], &tri[0]);
        let dca = vector::subtract(&tri[2], &tri[0]);
        let cross_product = vector::cross_product3(&dba, &dca);
        let normal = vector::normalize(&cross_product);
        Plane::new(&normal, &tri[0])
    }
    pub fn point_dist(&self, point: &Vec3) -> f32 {
        self.normal[0] * point[0] + self.normal[1] * point[1] + self.normal[2] * point[2] + self.dist
    }
    fn intersection_step(&self, dist: &f32, divergence: &f32) -> f32 {
        // Compute where the line step function intersects with the plane
        -dist / divergence
    }
    pub fn intersect_line(&self, line: &Line) -> Option<(f32, Vec3)> {
        let divergence = line.normal[0] * self.normal[0] + line.normal[1] * self.normal[1] + line.normal[2] * self.normal[2];
        if divergence == 0.0  {
            // Perpendicular to the plane
            return None
        }
        let dist = self.point_dist(&line.origin);
        let step = self.intersection_step(&dist, &divergence);
        let amt = step / line.length;
        Some((amt, line.to_point(&step)))
    }
}
