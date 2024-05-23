use crate::geometry::*;
use crate::vector::{Vec3, Vec2};
use std::rc::Rc;

type Vertex = Rc<(Vec3, Vec2)>;
type Vertices = Vec<Vertex>;
type Ngon = Rc<Vertices>;

#[derive(Debug)]
pub enum Node {
    Split(Box<SplitNode>),
    Leaf(Box<LeafNode>)
}

#[derive(Debug)]
pub struct SplitNode {
    children: [Option<Node>; 2]
}

#[derive(Debug)]
pub struct LeafNode {
    contents: Vec<Ngon>,
    portals: Vec<Ngon>
}

#[derive(Debug)]
enum Intersection {
    OnPlane,
    InFront,
    Behind,
    Split(Vertices, Vertices)
}

#[derive(PartialEq)]
enum PointInt {
    OnPlane,
    InFront,
    Behind
}

#[derive(PartialEq)]
enum Side {
    InFront,
    Behind
}

impl Node {
    pub fn from_obj(obj: &obj::raw::object::RawObj) -> Option<Node> {
        let vertices: Vertices = obj.points.iter()
            .map(|&p| {
                let vertices: [f32; 3] = Into::<[f32; 4]>::into(obj.positions[p])[0..3].try_into().unwrap();
                let texture: [f32; 2] = Into::<[f32; 3]>::into(obj.tex_coords[p])[0..2].try_into().unwrap();
                Rc::new((vertices, texture))
            })
            .collect();
        let faces = obj.polygons.iter()
            .map(|f| match f {
                obj::raw::object::Polygon::PTN(p) => {
                    p.iter()
                        .map(|&(vi, ti, _)| Rc::clone(&vertices[vi]))
                        .collect::<Vertices>()
                }
                _ => {
                    panic!();
                }
            })
            .map(|v| Rc::new(v))
            .collect();
        let root_node = Self::create_node(&faces);
        root_node
    }
    fn create_node(polygons: &Vec<Ngon>) -> Option<Node> {
        if polygons.len() == 0 {
            return None
        }
        let mut in_front = vec![];
        let mut behind = vec![];
        let mut on_plane = vec![Rc::clone(&polygons[0])];
        let mut portal_vertices = vec![];
        let tri_points = polygons[0][0..3].iter()
            .map(|v| v.0)
            .collect::<Vec<Vec3>>()
            .try_into().unwrap();
        let plane = Plane::from_tri(&tri_points);
        println!("{:?}", plane);
        for polygon in polygons[1..].iter() {
            let (intersection, curr_portal_vertices) = Self::intersect(polygon, &plane);
            portal_vertices.push(Rc::new(curr_portal_vertices));
            match intersection {
                Intersection::OnPlane => {
                    on_plane.push(Rc::clone(polygon));
                }
                Intersection::InFront => {
                    in_front.push(Rc::clone(polygon));
                }
                Intersection::Behind => {
                    behind.push(Rc::clone(polygon));
                }
                Intersection::Split(a, b) => {
                    in_front.push(Rc::new(b));
                    behind.push(Rc::new(a));
                }
            }
        };
        // There will always be at least 1 polygon on the splitting plane
        let leaf_node = Some(Node::Leaf(Box::new(LeafNode {
            contents: on_plane,
            portals: portal_vertices
        })));
        if in_front.len() == 0 && behind.len() == 0 {
            return leaf_node
        }
        let in_front_node = Some(Node::Split(Box::new(SplitNode {
            children: [
                leaf_node,
                Node::create_node(&in_front)
            ]
        })));
        if behind.len() == 0 {
            return in_front_node
        }
        Some(Node::Split(Box::new(SplitNode {
            children: [
                Node::create_node(&behind),
                in_front_node
            ]
        })))
    }
    fn interpolate_texture(amt: f32, tex_a: &Vec2, tex_b: &Vec2) -> Vec2 {
        [
            tex_a[0] + (tex_b[0] - tex_a[0]) * amt,
            tex_a[1] + (tex_b[1] - tex_a[1]) * amt,
        ]
    }
    fn intersect(polygon: &Ngon, plane: &Plane) -> (Intersection, Vertices) {
        let dists: Vec<f32> = polygon.iter()
            .map(|v| plane.point_dist(&v.0))
            .collect();
        let point_sides: Vec<PointInt> = dists.iter()
            .map(|&d| {
                if d.abs() < 0.02 {
                    PointInt::OnPlane
                } else if d < 0.0 {
                    PointInt::Behind
                } else {
                    PointInt::InFront
                }
            }).collect();
        if point_sides.iter().all(|s| *s == PointInt::OnPlane) {
            return (Intersection::OnPlane, (*polygon).to_vec())
        }
        let mut intersecting_vertices = vec![];
        let mut in_front: Vertices = vec![];
        let mut behind: Vertices = vec![];
        let mut prev_i = polygon.len() - 1;
        let mut state = point_sides.iter().rev().find(|&s| *s != PointInt::OnPlane).unwrap();
        for i in 0..polygon.len() {
            if point_sides[i] != PointInt::OnPlane {
                let new_state = &point_sides[i];
                if new_state != state {
                    // Side changed, the triangle intersects the plane
                    let side_line = Line::from_points(&polygon[prev_i].0, &polygon[i].0);
                    let (amt, intersection_point) = plane.intersect_line(&side_line).unwrap();
                    let texture_point = Self::interpolate_texture(amt, &polygon[prev_i].1, &polygon[i].1);
                    let intersection_v = Rc::new((intersection_point, texture_point));
                    behind.push(intersection_v.clone());
                    in_front.push(intersection_v.clone());
                    intersecting_vertices.push(intersection_v.clone());
                    state = new_state;
                }
            } else {
                intersecting_vertices.push(polygon[i].clone())
            }
            if *state == PointInt::Behind {
                behind.push(polygon[i].clone());
            } else {
                in_front.push(polygon[i].clone());
            }
            prev_i = i;
        }
        if in_front.len() == 0 {
            return (Intersection::Behind, intersecting_vertices)
        }
        if behind.len() == 0 {
            return (Intersection::InFront, intersecting_vertices)
        }
        (Intersection::Split(in_front, behind), intersecting_vertices)
    }
    fn triangulate_polygon (polygon: &Ngon) -> Vec<Ngon> {
        // Split the polygon into chains
        let half_length = polygon.len() / 2;
        let c1 = &polygon[0..half_length];
        let c2: Vertices = polygon[half_length..].iter()
            .map(|v| Rc::clone(&v))
            .rev().collect();
        let mut tris: Vec<Ngon> = vec![];
        let mut p1 = &c1[0];
        let mut p2 = &c2[0];
        for i in 1..half_length {
            let new_p1 = &c1[i];
            tris.push(Rc::new(vec![p1.clone(), p2.clone(), new_p1.clone()]));
            p1 = new_p1;
            let new_p2 = &c2[i];
            tris.push(Rc::new(vec![p1.clone(), p2.clone(), new_p2.clone()]));
            p2 = new_p2;
        }
        if 2 * half_length < polygon.len() {
            tris.push(Rc::new(vec![p1.clone(), p2.clone(), c2[half_length].clone()]));
        }
        tris
    }
    fn flatten(node: &Option<Node>) -> Vec<stl_io::Triangle> {
        match node {
            Some(Node::Split(node)) => {
                let mut tris = vec![];
                tris.append(&mut Self::flatten(&node.children[0]));
                tris.append(&mut Self::flatten(&node.children[1]));
                tris
            }
            Some(Node::Leaf(node)) => {
                node.contents.iter()
                    .map(|v| Node::triangulate_polygon(v))
                    .flatten()
                    .map(|t| stl_io::Triangle {
                        normal: stl_io::Normal::new([0.0, 0.0, 0.0]),
                        vertices: t.iter()
                            .map(|v| stl_io::Vertex::new(v.0))
                            .collect::<Vec<stl_io::Vertex>>()[..3]
                            .try_into().unwrap()
                    })
                    .collect()
            }
            None => {
                vec![]
            }
        }
    }
    pub fn to_stl(root_node: Option<Node>) -> Vec<stl_io::Triangle> {
        Self::flatten(&root_node)
    }
}
