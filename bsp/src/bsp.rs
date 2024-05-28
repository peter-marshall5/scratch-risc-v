use crate::geometry::*;
use crate::vector;
use crate::vector::{Vec3, Vec2};
use std::rc::Rc;

#[derive(Debug)]
#[derive(Clone)]
struct Vertex {
    pos: Vec3,
    uv: Vec2
}

type Vertices = Vec<Rc<Vertex>>;

#[derive(Debug)]
struct Ngon {
    v: Vertices,
    norm: Rc<Vec3>
}

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
    contents: Vec<Rc<Ngon>>,
    portals: Vec<Vec<Rc<Vertex>>>
}

#[derive(Debug)]
enum Intersection {
    OnPlane,
    InFront,
    Behind,
    Split(Rc<Ngon>, Rc<Ngon>)
}

#[derive(PartialEq)]
enum PointInt {
    OnPlane,
    InFront,
    Behind
}

impl Node {
    pub fn from_obj(obj: &obj::raw::object::RawObj) -> Option<Node> {
        let vertices: Vec<Rc<Vec3>> = obj.positions.iter()
            .map(|&p| Rc::new(Into::<[f32; 4]>::into(p)[0..3].try_into().unwrap()))
            .collect();
        let normals: Vec<Rc<Vec3>> = obj.normals.iter()
            .map(|&p| Rc::new(p.try_into().unwrap()))
            .collect();
        let textures: Vec<Rc<Vec2>> = obj.tex_coords.iter()
            .map(|&t| Rc::new(Into::<[f32; 3]>::into(t)[0..2].try_into().unwrap()))
            .collect();
        let faces = obj.polygons.iter()
            .map(|f| match f {
                obj::raw::object::Polygon::PTN(p) => {
                    (p.iter()
                        .map(|&(vi, ti, _)| (vertices[vi].clone(), textures[ti].clone()))
                        .collect::<Vec<(Rc<Vec3>, Rc<Vec2>)>>(),
                    normals[p[0].2].clone())
                }
                obj::raw::object::Polygon::PN(p) => {
                    (p.iter()
                        .map(|&(vi, _)| (vertices[vi].clone(), Rc::new([0.0, 0.0])))
                        .collect::<Vec<(Rc<Vec3>, Rc<Vec2>)>>(),
                    normals[p[0].1].clone())
                }
                _ => {
                    panic!();
                }
            })
            .map(|(v, n)| Rc::new(Ngon {
                v: v.iter()
                    .map(|(p, u)| Rc::new(Vertex {
                        pos: *p.clone(),
                        uv: *u.clone()
                    }))
                    .collect(),
                norm: n
            }))
            .collect();
        let root_node = Self::create_node(&faces);
        root_node
    }
    fn create_node(polygons: &Vec<Rc<Ngon>>) -> Option<Node> {
        if polygons.len() == 0 {
            return None
        }
        let mut in_front = vec![];
        let mut behind = vec![];
        let mut on_plane = vec![Rc::clone(&polygons[0])];
        let mut portal_vertices = vec![];
        let tri_points = polygons[0].v[0..3].iter()
            .map(|v| v.pos)
            .collect::<Vec<Vec3>>()
            .try_into().unwrap();
        let plane = Plane::from_tri(&tri_points);
        for polygon in polygons[1..].iter() {
            let (intersection, curr_portal_vertices) = Self::intersect(polygon, &plane);
            portal_vertices.push(curr_portal_vertices);
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
                    in_front.push(b);
                    behind.push(a);
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
        let dists: Vec<f32> = polygon.v.iter()
            .map(|v| plane.point_dist(&v.pos))
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
            return (Intersection::OnPlane, (*polygon.v).to_vec())
        }
        let mut intersecting_vertices = vec![];
        let mut in_front: Vertices = vec![];
        let mut behind: Vertices = vec![];
        let mut prev_i = polygon.v.len() - 1;
        let mut state = point_sides.iter().rev().find(|&s| *s != PointInt::OnPlane).unwrap();
        for i in 0..polygon.v.len() {
            if point_sides[i] != PointInt::OnPlane {
                let new_state = &point_sides[i];
                if new_state != state {
                    // Side changed, the triangle intersects the plane
                    let side_line = Line::from_points(&polygon.v[prev_i].pos, &polygon.v[i].pos);
                    let (amt, intersection_point) = plane.intersect_line(&side_line).unwrap();
                    let texture_point = Self::interpolate_texture(amt, &polygon.v[prev_i].uv, &polygon.v[i].uv);
                    let intersection_v = Rc::new(Vertex {
                        pos: intersection_point,
                        uv: texture_point
                    });
                    behind.push(intersection_v.clone());
                    in_front.push(intersection_v.clone());
                    intersecting_vertices.push(intersection_v.clone());
                    state = new_state;
                }
            } else {
                intersecting_vertices.push(polygon.v[i].clone())
            }
            if *state == PointInt::Behind {
                behind.push(polygon.v[i].clone());
            } else {
                in_front.push(polygon.v[i].clone());
            }
            prev_i = i;
        }
        if in_front.len() == 0 {
            return (Intersection::Behind, intersecting_vertices)
        }
        if behind.len() == 0 {
            return (Intersection::InFront, intersecting_vertices)
        }
        (Intersection::Split(Rc::new(Ngon {
            v: in_front,
            norm: polygon.norm.clone()
        }), Rc::new(Ngon {
            v: behind,
            norm: polygon.norm.clone()
        })), intersecting_vertices)
    }
    fn check_ear(tri: &[Rc<Vertex>; 3], normal: &Vec3, points: &Vec<Rc<Vertex>>) -> bool {
        let (axis, sign) = if normal[0].abs() > normal[1].abs() {
            if normal[0].abs() > normal[2].abs() {
                ((1, 2), 0)
            } else {
                ((0, 1), 2)
            }
        } else {
            if normal[2].abs() > normal[1].abs() {
                ((0, 1), 2)
            } else {
                ((2, 0), 1)
            }
        };
        let sign = normal[sign] > 0.0;
        let tri: [Vec2; 3] = tri.clone().map(|p| [p.pos[axis.0], p.pos[axis.1]]);
        let points: Vec<Vec2> = points.iter().map(|p| [p.pos[axis.0], p.pos[axis.1]]).collect();
        let ab = vector::subtract(&tri[1], &tri[0]);
        let bc = vector::subtract(&tri[2], &tri[1]);
        let ab_x_bc_sign = vector::cross_product2(&ab, &bc) > 0.0;
        if ab_x_bc_sign != sign {
            return false
        }
        let ca = vector::subtract(&tri[0], &tri[2]);
        for p in points.iter() {
            let ap = vector::subtract(p, &tri[0]);
            let bp = vector::subtract(p, &tri[1]);
            let cp = vector::subtract(p, &tri[2]);
            let ab_x_ap_sign = vector::cross_product2(&ab, &ap) > 0.0;
            let bc_x_bp_sign = vector::cross_product2(&bc, &bp) > 0.0;
            let ca_x_cp_sign = vector::cross_product2(&ca, &cp) > 0.0;
            if ab_x_ap_sign == bc_x_bp_sign && bc_x_bp_sign == ca_x_cp_sign {
                return false
            }
        }
        return true
    }
    fn triangulate_polygon(polygon: &Ngon) -> Vec<Ngon> {
        // Ear clipping triangulation
        assert_eq!(polygon.v.len() >= 3, true);
        let mut vertices: Vertices = polygon.v.to_vec();
        let mut result: Vec<Vertices> = vec![];
        'remove_ears: while vertices.len() > 3 {
            'find_ear: for i in 0..vertices.len() {
                let potential_ear_indices = [i, i+1, i+2]
                    .map(|n| n % vertices.len())
                    .to_owned();
                let potential_ear: [Rc<Vertex>; 3] = potential_ear_indices
                    .map(|n| vertices[n].clone());
                let others_indices: Vec<usize> = (0..vertices.len())
                    .filter(|n| !potential_ear_indices.iter().find(|&x| x == n).is_some())
                    .collect();
                let others: Vec<Rc<Vertex>> = others_indices.iter()
                    .map(|&n| vertices[n].clone())
                    .collect();
                if !Self::check_ear(&potential_ear, &*polygon.norm, &others) {
                    // println!("Ear not found: {}; {}; {}; {:?}; {:?}; {:?}; {:?}", result.len(), i, vertices.len(), potential_ear_indices, others_indices, potential_ear.map(|p| p.pos), others.iter().map(|p| &p.pos).collect::<Vec<_>>());
                    continue 'find_ear
                }
                result.push(potential_ear.to_vec());
                vertices.remove((i+1)%vertices.len());
                continue 'remove_ears
            }
            // No ears found
            break
        }
        // assert_eq!(polygon.len() == 3, true);
        result.push(vertices[..3].to_vec());
        result.iter()
            .map(|vertices| Ngon {
                norm: polygon.norm.clone(),
                v: vertices.to_vec()
            })
            .collect()
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
                        normal: stl_io::Normal::new(*t.norm),
                        vertices: t.v.iter()
                            .map(|v| stl_io::Vertex::new(v.pos))
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
