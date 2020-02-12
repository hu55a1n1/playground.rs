#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Error;
use std::io::{self, BufRead};
use std::mem;
use std::str::FromStr;
use std::vec;
use std::vec::Vec;

fn _mem() {
    let v = "".to_string();
    println!("{:?} : {}", v, mem::size_of_val(&v));
}

#[derive(Debug)]
struct Obj {
    h: u32,
    w: u32,
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.h, self.w)
    }
}

// methods
impl Obj {
    fn size(&self) -> u32 {
        self.h * self.w
    }
}

// related functions
impl Obj {
    fn new(h: u32, w: u32) -> Obj {
        Obj { h, w }
    }
}

fn _struct() {
    let o = Obj::new(10, 20);
    println!("{}", o);
}

fn _match() {
    let pair = (0, 0);
    match pair {
        (0, 0) => println!("(0, 0)"),
        (_, 0) => println!("(*, 0)"),
        (0, _) => println!("(0, *)"),
        (_, _) => println!("(*, *)"),
    }

    let p = 17;
    let n = match p {
        p @ 1...10 => p + 1,
        p @ 11...20 => p + 2,
        _ => 0,
    };
    println!("{:#?}", n);

    let n = 15;
    match n {
        1 => println!("One!"),
        2 | 3 | 5 | 7 | 11 | 13 | 17 | 19 => println!("Prime!"),
        13...19 => println!("Teen!"),
        _ => println!("Nothing special!"),
    }
}

#[derive(Debug)]
enum Shape {
    Square(u32),
    Rectangle { w: u32, h: u32 },
    Circle(f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match *self {
            Shape::Square(ref s) => (s * s) as f64, // Why ref? No idea :(
            Shape::Rectangle { w, h } => (w * h) as f64,
            Shape::Circle(ref r) => (r * r * 22.0 / 7.0),
        }
    }
}

fn _enum() {
    let s = Shape::Square(27);
    let r = Shape::Rectangle { w: 10, h: 70 };
    let c = Shape::Circle(15.0);
    println!("{:?}, {:?}, {:?}", s.area(), r.area(), c.area());
}

fn _vect() {
    let v = vec![1, 2, 3, 4, 5];
    for i in &v {
        println!("{}", i)
    }

    let mut ve: Vec<Shape> = Vec::new();
    ve.push(Shape::Square(27));
    ve.push(Shape::Rectangle { w: 10, h: 70 });
    ve.push(Shape::Circle(15.0));
    for i in &ve {
        println!("{}", i.area())
    }
}

fn _map() {
    let mut map = HashMap::new();

    map.insert("k1".to_owned(), "v1");
    map.insert("k2".to_owned(), "v2");
    for (k, v) in &map {
        println!("{} : {}", k, v)
    }

    match map.get("val1") {
        Some(&n) => println!("{}", n), // Apparently the ref avoids a copy
        None => println!("No match!"),
    }
}

fn _loop() {
    let mut i = 0;
    loop {
        if i > 10 {
            println!("reached limit");
            break;
        }
        i += 1;
    }

    i = 0;
    'a: loop {
        println!("a");
        'b: loop {
            println!("b");
            'c: loop {
                println!("c");
                i += 1;
                if i > 15 {
                    break 'a;
                } else if i > 10 {
                    break 'b;
                }
            }
        }
        if i > 20 {
            break;
        }
    }

    let x = loop {
        break 20;
    };
    println!("x = {}", x);
}

fn _if_while_let() {
    let mut s = Some(0);
    if let Some(s) = s {
        println!("s => {}", s);
    }

    while let Some(i) = s {
        match i {
            i if i > 19 => s = None,
            _ => s = Some(i + 2),
        }
    }
}
