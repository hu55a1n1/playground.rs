use std::vec::Vec;
use std::str::FromStr;
use std::io::{self, BufRead};

fn _sock_merchant() {
    let stdin = io::stdin();
    let mut iter = stdin.lock().lines();
    let line1 = iter.next().unwrap().unwrap();
    let line2 = iter.next().unwrap().unwrap();

    let vstr: Vec<&str> = line2.split(" ").collect();
    let mut v: Vec<i8> = vec! {0; usize::from_str(line1.as_str()).unwrap()};
    let mut pairs = 0;
    for n in vstr {
        let n = u8::from_str(n).unwrap();
        v.push(n);
        if v.iter().filter(|&u| *u == n).count() % 2 == 0 {
            pairs += 1;
        }
    }
    println!("{}", pairs);
}


fn _counting_valleys() {
    let stdin = io::stdin();
    let mut iter = stdin.lock().lines();
    let line1 = iter.next().unwrap().unwrap();
    let line2 = iter.next().unwrap().unwrap();

    let mut v: Vec<i8> = vec! {0; usize::from_str(line1.as_str()).unwrap()};
    for (i, dir) in line2.chars().enumerate() {
        v[i] = if dir == 'U' { 1 } else { -1 };
    }
//    println!("{:?}", v);

    let mut sum = v[0];
    let mut valleys = 0;
    for n in v {
        sum += n;
        if sum == 0 {
            if n == -1 {
                valleys += 1;
            }
        }
    }
    println!("{}", valleys);
}

fn _count_As() {
    let stdin = io::stdin();
    let mut iter = stdin.lock().lines();
    let mut s = iter.next().unwrap().unwrap();
    let n = usize::from_str(iter.next().unwrap().unwrap().as_str()).unwrap();

    let a = s.chars().filter(|c| *c == 'a').count();
    let d = n / s.len();
    let r = n % s.len();
    let mut occurances = a*d;
    s.truncate(r);
    occurances += s.chars().filter(|c| *c == 'a').count();
    println!("{}", occurances);
}

fn abbrv_1() {
    let stdin = io::stdin();
    let mut iter = stdin.lock().lines();
    let q = u8::from_str(iter.next().unwrap().unwrap().as_str()).unwrap();
    for _ in 0..q {
        let mut a: Vec<char> = iter.next().unwrap().unwrap().chars().collect();
        let mut b: Vec<char> = iter.next().unwrap().unwrap().chars().collect();

        if b.len() > a.len() {
            println!("NO");
            continue;
        }

        let mut i = 0;
        while i < a.len() && i < b.len() && a.len() >= b.len() {
            if a[i] == b[i] {} else if a[i].is_ascii_lowercase() && a[i].to_ascii_uppercase() == b[i] {
                a[i] = a[i].to_ascii_uppercase();
            } else if a[i].is_ascii_lowercase() {
                a.remove(i);
                continue;
            } else {
                break;
            }
            i += 1;
        }
        if a.len() > b.len() {
            for _ in 0..(a.len() - b.len()) {
                if a.last().unwrap().is_ascii_lowercase() { a.pop(); }
            }
        }

//        println!("{:?}\n{:?}", a, b);
        if a == b { println!("YES"); } else { println!("NO"); }
    }
}