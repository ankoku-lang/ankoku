use std::{
    backtrace::Backtrace,
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
    ptr::{drop_in_place, NonNull},
};

use crate::vm::obj::Object;

use self::{
    chunk::Chunk,
    error::{RuntimeError, RuntimeErrorType, RuntimeType, TypeErrorType},
    obj::{Obj, ObjType},
    table::HashTable,
    value::Value,
};

pub mod chunk;
mod error;
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
    grey_stack: RefCell<Vec<GcRef>>,
    globals: HashTable,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::with_capacity(128),
            objects: Cell::new(None),
            grey_stack: RefCell::new(Vec::new()),
            globals: HashTable::new(),
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
        self.stack
            .pop()
            .unwrap_or_else(|| panic!("nothing to pop: {:?}", self.ip))
    }

    pub(crate) fn stack_peek(&mut self) -> &Value {
        &self.stack[self.stack.len() - 1]
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

        macro_rules! read_u32 {
            () => {{
                let a = read_byte!();
                let b = read_byte!();
                let c = read_byte!();
                let d = read_byte!();

                ((a as usize) << 24) | ((b as usize) << 16) | ((c as usize) << 8) | (d as usize)
            }};
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
                    return InterpretResult::Ok;
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

                // Pop
                9 => {
                    _ = self.stack_pop();
                }

                // TODO: remove print
                100 => {
                    let pop = self.stack_pop();
                    println!("{:?}", pop);
                }

                // NewObject
                10 => self.stack_push(Value::Obj(
                    self.alloc(Obj::new(ObjType::Object(Object::new()))),
                )),

                // ObjectSet
                11 => {
                    let value = self.stack_pop();
                    let key = self.stack_pop();
                    if let Value::Obj(o) = key {
                        if let ObjType::String(key) = &o.kind {
                            let len = self.stack.len();
                            let object = &mut self.stack[len - 1];
                            if let Value::Obj(o) = object {
                                if let ObjType::Object(o) = &mut o.deref_mut().kind {
                                    o.table.set(key.clone(), value);
                                } else {
                                    self.type_error(
                                        RuntimeType::Object,
                                        TypeErrorType::ObjectSetMustBeObject,
                                    );
                                }
                            } else {
                                self.type_error(
                                    RuntimeType::Object,
                                    TypeErrorType::ObjectSetMustBeObject,
                                );
                            }
                        } else {
                            self.type_error(RuntimeType::String, TypeErrorType::KeyMustBeString);
                        }
                    } else {
                        self.type_error(RuntimeType::String, TypeErrorType::KeyMustBeString);
                    }
                }
                // DefineGlobal
                12 => {
                    let name = read_constant!();
                    if let Value::Obj(o) = &name {
                        if let ObjType::String(s) = &o.inner().kind {
                            let popped = self.stack_pop();
                            self.globals.set(s.clone(), popped);
                        } else {
                            self.type_error(
                                RuntimeType::String,
                                TypeErrorType::GlobalNameMustBeString,
                            );
                        }
                    } else {
                        self.type_error(RuntimeType::String, TypeErrorType::GlobalNameMustBeString);
                    }
                }
                // GetGlobal
                13 => {
                    let name = read_constant!();
                    if let Value::Obj(o) = &name {
                        if let ObjType::String(s) = &o.inner().kind {
                            if let Some(value) = self.globals.get(s) {
                                self.stack_push(value.clone());
                            } else {
                                self.runtime_error(RuntimeErrorType::UndefinedVariable {
                                    name: s.as_str().to_string(),
                                });
                            }
                        } else {
                            self.type_error(
                                RuntimeType::String,
                                TypeErrorType::GlobalNameMustBeString,
                            );
                        }
                    } else {
                        self.type_error(RuntimeType::String, TypeErrorType::GlobalNameMustBeString);
                    }
                }
                // SetGlobal
                14 => {
                    let name = read_constant!();
                    if let Value::Obj(o) = &name {
                        if let ObjType::String(s) = &o.inner().kind {
                            let value = self.stack_peek().clone();
                            if self.globals.set(s.clone(), value) {
                                self.globals.delete(s.hash());
                                self.runtime_error(RuntimeErrorType::UndefinedVariable {
                                    name: s.as_str().to_string(),
                                });
                            }
                        } else {
                            self.type_error(
                                RuntimeType::String,
                                TypeErrorType::GlobalNameMustBeString,
                            );
                        }
                    } else {
                        self.type_error(RuntimeType::String, TypeErrorType::GlobalNameMustBeString);
                    }
                }
                // GetLocal
                15 => {
                    let slot = read_byte!();
                    self.stack_push(self.stack[slot as usize].clone());
                }
                // GetLocal
                16 => {
                    let slot = read_byte!();
                    self.stack[slot as usize] = self.stack[self.stack.len() - 1].clone();
                }
                // JumpIfFalse
                17 => {
                    let to = read_u32!();
                    let cond = self.stack_peek();
                    if cond.falsey() {
                        self.ip = to;
                    }
                }
                // Jump
                18 => {
                    let to = read_u32!();
                    self.ip = to;
                }
                _ => unimplemented!("instruction {}", instruction),
            }
        }
    }

    fn type_error(&self, expected: RuntimeType, kind: TypeErrorType) -> RuntimeError {
        RuntimeError {
            kind: RuntimeErrorType::TypeError { expected, kind },
            internal_bt: Backtrace::capture(),
        }
    }

    fn runtime_error(&self, kind: RuntimeErrorType) -> RuntimeError {
        RuntimeError {
            kind,
            internal_bt: Backtrace::capture(),
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
        println!("{:?}", self.stack);

        for slot in &self.stack {
            self.mark(slot);
        }

        for value in self.globals.values() {
            self.mark(value);
        }

        // TODO: when global variables implemented, mark those and the call frames and upvalues and compiler? https://craftinginterpreters.com/garbage-collection.html#less-obvious-roots
    }

    fn mark(&self, value: &Value) {
        if let Value::Obj(obj) = value {
            self.mark_object(*obj);
        }
    }
    fn mark_object(&self, mut obj: GcRef) {
        if obj.obj.is_null() || obj.marked {
            return;
        }
        obj.marked = true;

        self.grey_stack.borrow_mut().push(obj);

        #[cfg(feature = "gc-debug-super-slow")]
        println!("{:?} mark {:?}", obj.obj, obj.inner());
    }

    fn trace_refs(&self) {
        while self.grey_stack.borrow().len() > 0 {
            let object = self.grey_stack.borrow_mut().pop().unwrap();
            VM::blacken_object(object);
        }
    }
    fn blacken_object(obj: GcRef) {
        #[cfg(feature = "gc-debug-super-slow")]
        {
            println!("{:?} blacken {:?}", obj.obj, *obj);
        }
        match &obj.kind {
            ObjType::String(_) => {}
            ObjType::Object(o) => {
                for o in o.table.values() {
                    if let Value::Obj(obj) = o {
                        VM::blacken_object(*obj);
                    }
                }
            }
        }
    }
    fn sweep(&self) {
        let mut prev = None;
        let mut obj = self.objects.get();
        while let Some(mut o) = obj {
            let o = unsafe { &mut *o.as_mut() };
            if o.marked {
                o.marked = false;
                prev = obj;
                obj = o.next;
            } else {
                let unreached = obj;
                obj = o.next;
                if let Some(mut prev) = prev {
                    unsafe { &mut *prev.as_mut() }.next = obj;
                } else {
                    self.objects.set(obj);
                }

                if let Some(e) = unreached {
                    println!("{:?} sweeping {:?}", e, unsafe { e.as_ref() });
                    unsafe {
                        drop_in_place(e.as_ptr());
                        #[cfg(feature = "gc-debug-super-slow")]
                        {
                            *e.as_ptr() = std::mem::zeroed(); // for testing, to make sure nothing is used after free
                        }
                    }
                } else {
                    println!("nullptr {:?}", unreached);
                }
            }
        }
    }
    pub fn collect(&self) {
        #[cfg(feature = "gc-debug-super-slow")]
        {
            println!("-- gc begin collect");
        }

        self.mark_roots();
        self.trace_refs();
        self.sweep();
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

#[cfg(test)]
mod tests {
    use crate::vm::{chunk::Chunk, instruction::Instruction, value::Value, InterpretResult, VM};

    use super::obj::AnkokuString;

    #[test]
    fn gc() {
        let mut chunk = Chunk::new();
        let mut vm = VM::new();
        let constant = chunk.add_constant(Value::Obj(
            vm.alloc(AnkokuString::new("hello".into()).into()),
        ));
        chunk.write(Instruction::Constant as u8, 1);
        chunk.write(constant as u8, 1);
        let constant = chunk.add_constant(Value::Obj(
            vm.alloc(AnkokuString::new(" world".into()).into()),
        ));
        chunk.write(Instruction::Constant as u8, 1);
        chunk.write(constant as u8, 1);

        chunk.write(Instruction::Add as u8, 1);
        chunk.write(Instruction::Return as u8, 1);
        let _out = vm.interpret(chunk);

        vm.collect();
        println!("gc done");

        // I don't really know how you unit test a GC. I think it works idk
    }

    #[test]
    fn returns() {
        let mut chunk = Chunk::new();

        let constant = chunk.add_constant(1.2.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant as u8, 1);

        let constant = chunk.add_constant(3.4.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant as u8, 1);

        chunk.write(Instruction::Add.into(), 1);

        let constant = chunk.add_constant(5.6.into());
        chunk.write(Instruction::Constant.into(), 1);
        chunk.write(constant as u8, 1);

        chunk.write(Instruction::Div.into(), 1);

        chunk.write(Instruction::Negate.into(), 1);

        chunk.write(Instruction::Return.into(), 1);

        chunk.disassemble("test");
        let mut vm = VM::new();
        assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
    }
}
