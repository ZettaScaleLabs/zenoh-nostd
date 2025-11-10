use zenoh_nostd_core::{
    ZDecode, ZReader,
    network::{NetworkBody, push::Push},
};

struct NetworkMsgIter<'a> {
    reader: ZReader<'a>,
}

impl<'a> core::iter::Iterator for NetworkMsgIter<'a> {
    type Item = NetworkBody<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let push: Push<'a> = <_ as ZDecode>::z_decode(&mut self.reader).unwrap();

        Some(NetworkBody::Push(push))
    }
}

fn test<'a>(data: &'a [u8]) -> Push<'a> {
    let mut iter: NetworkMsgIter<'a> = NetworkMsgIter { reader: data };
    match iter.next().unwrap() {
        NetworkBody::Push(push) => push,
        _ => panic!("Expected Push message"),
    }
}

fn main() {
    println!("Hello, world!");
}
