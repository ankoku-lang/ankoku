// use std::{
//     cell::Cell,
//     ops::Deref,
//     ptr::{drop_in_place, NonNull},
// };

// use super::obj::Obj;

// #[cfg(test)]
// mod tests {
//     use crate::vm::{
//         obj::{AnkokuString, Obj, ObjType},
//         value::Value,
//     };

//     use super::Gc;

//     #[test]
//     fn basic() {
//         let string = Obj::new(ObjType::String(AnkokuString::new("hello,".into())));
//         let gc = Gc::new();

//         let left = Value::Obj(gc.alloc(string));
//         let string = Obj::new(ObjType::String(AnkokuString::new(" world".into())));
//         let right = Value::Obj(gc.alloc(string));

//         let result = left.add(right, &gc);

//         if let Value::Obj(o) = result {
//             assert_eq!(
//                 o.inner().kind,
//                 ObjType::String(AnkokuString::new("hello, world".into()))
//             );
//         } else {
//             unreachable!()
//         }
//     }

//     #[test]
//     fn moar_strings() {
//         let string = Obj::new(ObjType::String(AnkokuString::new("st".into())));
//         let gc = Gc::new();

//         let left = Value::Obj(gc.alloc(string));
//         let string = Obj::new(ObjType::String(AnkokuString::new("ri".into())));
//         let right = Value::Obj(gc.alloc(string));

//         let result = left.add(right, &gc);

//         let string = Obj::new(ObjType::String(AnkokuString::new("ng".into())));
//         let right = Value::Obj(gc.alloc(string));

//         let result = result.add(right, &gc);

//         if let Value::Obj(o) = result {
//             assert_eq!(
//                 o.inner().kind,
//                 ObjType::String(AnkokuString::new("string".into()))
//             );
//         } else {
//             unreachable!()
//         }
//     }
// }
