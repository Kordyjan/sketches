use per_set::PerMap;

fn main() {
    let data = vec![486, 693, 184];
    let map = PerMap::empty();
    let map = data.iter().fold(map, |m, e| m.insert(*e, *e));
    println!("{:?}", map.get(&693));
}
