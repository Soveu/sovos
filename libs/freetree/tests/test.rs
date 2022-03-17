use freetree::poc::*;
use freetree::Unique;
use std::mem::ManuallyDrop;
use std::time::Instant;

fn box_to_unique<T>(boxed: Box<T>) -> Unique<T> {
    unsafe { Unique::from_raw(Box::into_raw(boxed)) }
}
// fn unique_to_box<T>(unique: Unique<T>) -> Box<T> {
//     unsafe { Box::from_raw(Unique::into_raw(unique)) }
// }
fn new_edge() -> Edge {
    let boxed = Box::new([Node::new(), Node::new()]);
    box_to_unique(boxed)
}
fn xorshift(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return x;
}

// #[test]
// fn test1() {
//     let mut root = ManuallyDrop::new(Root::new());
//     let mut seed = 0xDEADBEEF;
//     let mut allocations: Vec<_> = (0..100_000)
//         .into_iter()
//         .map(|_| Unique::into_raw(new_edge()) as usize)
//         .collect();
// 
//     for i in 0..allocations.len() {
//         seed = xorshift(seed);
//         let index = seed as usize;
//         let index = index % (allocations.len() - i);
//         let index = i + index;
//         allocations.swap(i, index);
//     }
// 
//     print!("Inserting elements");
//     let now = Instant::now();
//     for edge in allocations.iter().copied() {
//         let edge = unsafe { Unique::from_raw(edge as *mut _) };
//         //println!("\nInserting {:x}", p);
//         //println!("{:?}", root);
//         root.insert(edge);
//         //assert!(root.search(p), "where is {:X}?\n{:?}", p, root);
//     }
//     println!(" {:?}", now.elapsed());
// 
//     //println!("{:?}", root);
// 
//     print!("Checking their presence");
//     let now = Instant::now();
//     for p in allocations {
//         if !root.search(p) {
//             panic!("{:X} not found", p);
//         }
//     }
//     println!(" {:?}", now.elapsed());
// 
//     assert!(root.search(0) == false);
// }

#[test]
fn test2() {
    let mut root = ManuallyDrop::new(Root::new());
    let mut seed = 0xDEADBEEF;
    let mut allocations: Vec<_> = (0..10)
        .into_iter()
        .map(|_| Unique::into_raw(new_edge()) as usize)
        .collect();

    for i in 0..allocations.len() {
        seed = xorshift(seed);
        let index = seed as usize;
        let index = index % (allocations.len() - i);
        let index = i + index;
        allocations.swap(i, index);
    }

    print!("Inserting elements");
    //let now = Instant::now();
    for edge in allocations.iter().copied() {
        let edge = unsafe { Unique::from_raw(edge as *mut _) };
        //println!("\nInserting {:x}", p);
        //println!("{:?}", root);
        root.insert(edge);
        //assert!(root.search(p), "where is {:X}?\n{:?}", p, root);
    }
    //println!(" {:?}", now.elapsed());

    //println!("{:?}", root);

    //let now = Instant::now();
    for p in allocations {
        println!("Removing {:X}", p);
        println!("{:?}", root);
        let res = ManuallyDrop::new(root.remove(p));
        assert_eq!(res.as_ref().unwrap().as_usize(), p);
    }
    //println!(" {:?}", now.elapsed());

}
