#![cfg_attr(feature = "cargo-clippy", allow(many_single_char_names, ptr_arg))]
// This is an implementation of the Sutherland-Hodgman (1974) reentrant polygon
// clipping algorithm. It takes a polygon represented as a number of vertices
// and cuts it according to the given planes.

// The implementation borrows heavily from Pucella-Tov (2008). See that paper
// for more explanation.

extern crate rand;
extern crate session_types;

use session_types::*;

use rand::{Rand, Rng};

use std::thread::spawn;

#[derive(Debug, Copy, Clone)]
struct Point(f64, f64, f64);

impl Rand for Point {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Point(rng.gen(), rng.gen(), rng.gen())
    }
}

#[derive(Debug, Copy, Clone)]
struct Plane(f64, f64, f64, f64);

impl Rand for Plane {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Plane(
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        )
    }
}

fn above(Point(x, y, z): Point, Plane(a, b, c, d): Plane) -> bool {
    (a * x + b * y + c * z + d) / (a * a + b * b + c * c).sqrt() > 0.0
}

fn intersect(p1: Point, p2: Point, plane: Plane) -> Option<Point> {
    let Point(x1, y1, z1) = p1;
    let Point(x2, y2, z2) = p2;
    let Plane(a, b, c, d) = plane;

    if above(p1, plane) == above(p2, plane) {
        None
    } else {
        let t = (a * x1 + b * y1 + c * z1 + d) / (a * (x1 - x2) + b * (y1 - y2) + c * (z1 - z2));
        let x = x1 + (x2 - x1) * t;
        let y = y1 + (y2 - y1) * t;
        let z = z1 + (z2 - z1) * t;
        Some(Point(x, y, z))
    }
}

type SendList<A> = Rec<Choose<Eps, Send<A, Var<Z>>>>;
type RecvList<A> = Rec<Offer<Eps, Recv<A, Var<Z>>>>;

fn sendlist<A: std::marker::Send + Copy + 'static>(c: Chan<(), SendList<A>>, xs: &Vec<A>) {
    let mut c = c.enter();
    for x in xs {
        let c1 = c.sel2().send(*x);
        c = c1.zero();
    }
    c.sel1().close();
}

fn recvlist<A: std::marker::Send + 'static>(c: Chan<(), RecvList<A>>) -> Vec<A> {
    let mut v = Vec::new();
    let mut c = c.enter();
    loop {
        c = match c.offer() {
            Left(c) => {
                c.close();
                break;
            }
            Right(c) => {
                let (c, x) = c.recv();
                v.push(x);
                c.zero()
            }
        }
    }

    v
}

fn clipper(plane: Plane, ic: Chan<(), RecvList<Point>>, oc: Chan<(), SendList<Point>>) {
    let mut oc = oc.enter();
    let mut ic = ic.enter();
    let (pt0, mut pt);

    match ic.offer() {
        Left(c) => {
            c.close();
            oc.sel1().close();
            return;
        }
        Right(c) => {
            let (c, ptz) = c.recv();
            ic = c.zero();
            pt0 = ptz;
            pt = ptz;
        }
    }

    loop {
        if above(pt, plane) {
            oc = oc.sel2().send(pt).zero();
        }
        ic = match ic.offer() {
            Left(c) => {
                if let Some(pt) = intersect(pt, pt0, plane) {
                    oc = oc.sel2().send(pt).zero();
                }
                c.close();
                oc.sel1().close();
                break;
            }
            Right(ic) => {
                let (ic, pt2) = ic.recv();
                if let Some(pt) = intersect(pt, pt2, plane) {
                    oc = oc.sel2().send(pt).zero();
                }
                pt = pt2;
                ic.zero()
            }
        }
    }
}

fn clipmany(planes: Vec<Plane>, points: Vec<Point>) -> Vec<Point> {
    let (tx, rx) = session_channel();
    spawn(move || sendlist(tx, &points));
    let mut rx = rx;

    for plane in planes {
        let (tx2, rx2) = session_channel();
        spawn(move || clipper(plane, rx, tx2));
        rx = rx2;
    }
    recvlist(rx)
}

fn normalize_point(Point(a, b, c): Point) -> Point {
    Point(10.0 * (a - 0.5), 10.0 * (b - 0.5), 10.0 * (c - 0.5))
}

fn normalize_plane(Plane(a, b, c, d): Plane) -> Plane {
    Plane(
        10.0 * (a - 0.5),
        10.0 * (b - 0.5),
        10.0 * (c - 0.5),
        10.0 * (d - 0.5),
    )
}

fn bench(n: usize, m: usize) {
    let mut g = rand::thread_rng();
    let points = (0..n)
        .map(|_| rand::Rand::rand(&mut g))
        .map(normalize_point)
        .collect();
    let planes = (0..m)
        .map(|_| rand::Rand::rand(&mut g))
        .map(normalize_plane)
        .collect();

    let points = clipmany(planes, points);
    println!("{}", points.len());
}

fn main() {
    bench(100, 5);
}
