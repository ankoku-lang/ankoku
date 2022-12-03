use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    ptr::{drop_in_place, NonNull},
};

use self::{chunk::Chunk, obj::Obj, value::Value};

pub mod chunk;
mod gc;
pub mod instruction;
pub mod obj;
pub mod table;
pub mod value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    objects: Cell<Option<NonNull<Obj>>>, // Option<NonNull<T>> is the same size as *mut T where None is a nullptr, this is just safer (not by much; this code still does raw pointer manipulation)
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(128),
            objects: Cell::new(None),
        }
    }
    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    pub(crate) fn stack_push(&mut self, value: Value) {
        assert!(self.stack.len() < 1024, "stack overflow");
        self.stack.push(value);
    }

    pub(crate) fn stack_pop(&mut self) -> Value {
        self.stack.pop().expect("nothing to pop")
    }

    pub fn run(&mut self) -> InterpretResult {
        #[cfg(feature = "debug-mode")]
        println!("== VM ==");
        macro_rules! read_byte {
            () => {{
                self.ip += 1;
                self.chunk.code[self.ip - 1]
            }};
        }
        macro_rules! read_constant {
            () => {
                self.chunk.constants[read_byte!() as usize].clone()
            };
        }

        loop {
            #[cfg(feature = "debug-mode")]
            {
                print!("STACK:    ");
                for value in &self.stack {
                    print!("[ {:?} ]", value);
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }
            let instruction = read_byte!();

            match instruction {
                // Return
                1 => {
                    return InterpretResult::Ok(self.stack_pop());
                }
                // Constant
                2 => {
                    let constant = read_constant!();
                    self.stack_push(constant);
                }
                // Negate
                3 => {
                    let v = self.stack_pop();
                    self.stack_push(v.neg(self));
                }
                // Add
                4 => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a.add(b, self));
                }
                // Sub
                5 => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a.sub(b, self));
                }
                // Mul
                6 => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a.mul(b, self));
                }
                // Div
                7 => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a.div(b, self));
                }
                // Not
                8 => {
                    let a = self.stack_pop();
                    self.stack_push(a.not(self));
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn alloc(&self, mut obj: Obj) -> GcRef {
        obj.next = self.objects.get();
        let heap_obj = Box::into_raw(Box::new(obj));
        self.objects.set(Some(NonNull::new(heap_obj).unwrap()));

        #[cfg(feature = "gc-debug-super-slow")]
        {
            self.collect();
            println!("{:?} allocated {}", heap_obj, std::mem::size_of::<Obj>())
        }

        GcRef { obj: heap_obj }
    }
    fn mark_roots(&self) {
        for slot in &self.stack {
            self.mark(slot);
        }
    }

    fn mark(&self, value: &Value) {
        if let Value::Obj(obj) = value {
            self.mark_object(*obj);
        }
    }
    fn mark_object(&self, mut obj: GcRef) {
        if obj.obj.is_null() {
            return;
        }
        obj.marked = true;
    }
    pub fn collect(&self) {
        #[cfg(feature = "gc-debug-super-slow")]
        {
            println!("-- gc begin collect");
        }

        self.mark_roots();

        #[cfg(feature = "gc-debug-super-slow")]
        {
            println!("-- gc end");
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VM {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcRef {
    obj: *mut Obj,
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

impl DerefMut for GcRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.obj }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InterpretResult {
    Ok(Value),
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::vm::{chunk::Chunk, instruction::Instruction, value::Value, InterpretResult, VM};

    use super::obj::{AnkokuString, Obj, ObjType};

    #[test]
    fn gc() {
        let mut chunk = Chunk::new();
        let mut vm = VM::new();
        let constant = chunk.add_constant(Value::Obj(
            vm.alloc(AnkokuString::new("hello world".into()).into()),
        ));
        chunk.write(Instruction::Constant as u8, 1);
        chunk.write(constant, 1);
        chunk.write(Instruction::Return as u8, 1);
        vm.interpret(chunk);
        println!("{:?}", vm.stack);
    }

    #[test]
    fn returns() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(1.2.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        let constant = chunk.add_constant(3.4.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        chunk.write(Instruction::Add.into(), 1);

        let constant = chunk.add_constant(5.6.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant, 1);

        chunk.write(Instruction::Div.into(), 1);

        chunk.write(Instruction::Negate.into(), 1);

        chunk.write(Instruction::Return.into(), 1);

        chunk.disassemble("test");
        let mut vm = VM::new();
        assert_eq!(
            vm.interpret(chunk),
            InterpretResult::Ok(Value::Real(-((1.2 + 3.4) / 5.6)))
        );
    }
}
