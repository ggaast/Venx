use std::mem::ManuallyDrop;

struct Data {
    list: Vec<Element>,
}
union Element {
    link: ManuallyDrop<Link>,
    leaf: ManuallyDrop<Leaf>,
}

struct Link {
    data: u32, //
}
struct Leaf {
    data: u32, // 0 - is start
}

impl Link {
    fn is_mirrored(&self) {
        todo!()
    }
    fn get_leaf_range(&self) {
        todo!()
    }
}
