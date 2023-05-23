use std::collections::HashMap;

mod index;

fn main() {
    let mut ind = index::Index::new(false);
    let v1 = HashMap::from([(1_u32, 0.4_f32), (5, 0.6)]);
    let v2 = HashMap::from([(2_u32, 0.4_f32), (5, 0.9)]);
    let q1 = HashMap::from([(13_u32, 0.4_f32), (5, 1.2)]);
    ind.insert(&v1);
    ind.insert(&v2);

    println!("Index built: {}", ind);

    let r = ind.retrieve(&q1, 4, None);
    println!("{:?}", &r);


}