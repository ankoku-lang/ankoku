use std::{
    cell::Cell,
    ops::Deref,
    ptr::{drop_in_place, NonNull},
};

use super::obj::Obj;

pub struct Gc {
    objects: Cell<Option<NonNull<Obj>>>, // Option<NonNull<T>> is the same size as *mut T where None is a nullptr, this is just safer (as ""safe"" as pointer handling code goes)
}

impl Gc {
    pub fn new() -> Self {
        Self {
            objects: Cell::new(None),
        }
    }

    pub fn alloc(&self, mut obj: Obj) -> GcRef {
        obj.next = self.objects.get();
        let heap_obj = Box::into_raw(Box::new(obj));
        self.objects.set(Some(NonNull::new(heap_obj).unwrap()));

        GcRef { obj: heap_obj }
    }

    pub fn collect(&self) {}
}

impl Drop for Gc {
    fn drop(&mut self) {
        let mut obj = self.objects.get();

        while let Some(mut o) = obj {
            let next = unsafe { o.as_ref() }.next;
            unsafe {
                drop_in_place(o.as_mut());
            }
            obj = next;
        }
    }
}

impl Default for Gc {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GcRef {
    obj: *const Obj,
}

impl GcRef {
    pub fn inner(&self) -> &Obj {
        self.deref()
    }
}
impl Deref for GcRef {
    type Target = Obj;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.obj }
    }
}
#[cfg(test)]
mod tests {
    use crate::vm::{
        obj::{AnkokuString, Obj, ObjType},
        value::Value,
    };

    use super::Gc;

    #[test]
    fn basic() {
        let string = Obj::new(ObjType::String(AnkokuString::new("hello,".into())));
        let gc = Gc::new();

        let left = Value::Obj(gc.alloc(string));
        let string = Obj::new(ObjType::String(AnkokuString::new(" world".into())));
        let right = Value::Obj(gc.alloc(string));

        let result = left.add(right, &gc);

        if let Value::Obj(o) = result {
            assert_eq!(
                o.inner().kind,
                ObjType::String(AnkokuString::new("hello, world".into()))
            );
        } else {
            unreachable!()
        }
    }

    #[test]
    fn moar_strings() {
        let string = Obj::new(ObjType::String(AnkokuString::new("st".into())));
        let gc = Gc::new();

        let left = Value::Obj(gc.alloc(string));
        let string = Obj::new(ObjType::String(AnkokuString::new("ri".into())));
        let right = Value::Obj(gc.alloc(string));

        let result = left.add(right, &gc);

        let string = Obj::new(ObjType::String(AnkokuString::new("ng".into())));
        let right = Value::Obj(gc.alloc(string));

        let result = result.add(right, &gc);

        if let Value::Obj(o) = result {
            assert_eq!(
                o.inner().kind,
                ObjType::String(AnkokuString::new("string".into()))
            );
        } else {
            unreachable!()
        }
    }
}
