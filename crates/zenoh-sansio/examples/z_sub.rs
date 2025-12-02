use crate::ZResult;

// type NetworkMessage = u8;
// const QUERY: NetworkMessage = 0;
// const PUSH: NetworkMessage = 1;

// type Event = u8;
// const RESPONSE_EVENT_A: Event = 8;
// const RESPONSE_EVENT_B: Event = 19;

// enum Result<A, B, C> {
//     A(A),
//     B(B),
//     C(C),
// }

// impl<A, B, C, T> Iterator for Result<A, B, C>
// where
//     A: Iterator<Item = T>,
//     B: Iterator<Item = T>,
//     C: Iterator<Item = T>,
// {
//     type Item = T;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             Result::A(iter) => iter.next(),
//             Result::B(iter) => iter.next(),
//             Result::C(iter) => iter.next(),
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         match self {
//             Result::A(iter) => iter.size_hint(),
//             Result::B(iter) => iter.size_hint(),
//             Result::C(iter) => iter.size_hint(),
//         }
//     }

//     fn fold<Acc, F>(self, init: Acc, f: F) -> Acc
//     where
//         F: FnMut(Acc, Self::Item) -> Acc,
//     {
//         match self {
//             Result::A(iter) => iter.fold(init, f),
//             Result::B(iter) => iter.fold(init, f),
//             Result::C(iter) => iter.fold(init, f),
//         }
//     }
// }

// fn handle(network_msgs: impl Iterator<Item = NetworkMessage>) -> impl Iterator<Item = Event> {
//     network_msgs.flat_map(|network_msg| {
//         if network_msg == QUERY {
//             Result::A((0u8..2u8).into_iter().filter_map(|my_queryable| {
//                 if my_queryable % 2 == 0 {
//                     Some(RESPONSE_EVENT_A)
//                 } else {
//                     None
//                 }
//             }))
//         } else if network_msg == PUSH {
//             Result::B(std::iter::once(RESPONSE_EVENT_B))
//         } else {
//             Result::C(std::iter::empty())
//         }
//     })
// }

fn entry() -> crate::ZResult<()> {
    Ok(())
}

fn main() {
    match entry() {
        Ok(_) => {}
        Err(e) => {
            zenoh_proto::error!("Error: {:?}", e);
        }
    }
}
