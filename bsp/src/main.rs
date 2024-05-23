use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use obj::raw::object::{parse_obj, RawObj};

mod geometry;
mod vector;
mod bsp;
use crate::bsp::Node;

fn main() {
    let input = BufReader::new(File::open("tests/fixtures/normal-cone.obj").unwrap());
    let level: RawObj = parse_obj(input).unwrap();
    let root = Node::from_obj(&level);
    println!("{:#?}", root);
    let mesh = Node::to_stl(root);
    let mut file = OpenOptions::new().write(true).create_new(true).open("mesh.stl").unwrap();
    stl_io::write_stl(&mut file, mesh.iter()).unwrap();
}
