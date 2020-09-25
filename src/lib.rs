use std::fmt;

use broom::{Heap, Handle, Rooted};
use broom::prelude::{Trace, Tracer};

#[cfg(test)]
mod test;


/// Stack-based Ginkgo value.
#[derive(Clone, Copy, PartialEq)]
pub enum SVal {
    Undefined,
    Nil,
    Bool(bool),
    Int(isize),
    Float(f64),
}

/// Heap-based (garbage-collected) Ginkgo value.
#[derive(Clone, PartialEq)]
pub enum HVal {
    Cons(Object, Object)
}

/// Safe Ginkgo object.  Either a direct representation of a stack
/// value or a GC-handle to a heap value.
#[derive(Clone)]
pub enum Object {
    S(SVal),
    H(Handle<HVal>),
    R(Rooted<HVal>),
}

/// Reference-ified Ginkgo object.  Like Object, except wraps a
/// reference to a heap value instead of a GC-handle.  May not survive
/// a GC!
pub enum DirectObject<'a> {
    D(*const HVal),
    S(SVal),
    H(&'a HVal),
}

impl Trace<HVal> for Object {
    fn trace(&self, tracer: &mut Tracer<HVal>) {
        if let Object::H(handle) = self {
            handle.trace(tracer);
        }
    }
}

impl Trace<HVal> for HVal {
    fn trace(&self, tracer: &mut Tracer<HVal>) {
        let HVal::Cons(car, cdr) = self;
        car.trace(tracer);
        cdr.trace(tracer);
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::S(l) => if let Object::S(r) = other { l == r} else { false },
            Object::H(l) => match other {
                Object::H(r) => l == r,
                Object::R(r) => *l == r.handle(),
                _ => false,
            }
            Object::R(l) => match other {
                Object::H(r) => l.handle() == *r,
                Object::R(r) => l.handle() == r.handle(),
                _ => false,
            }
        }
    }
}

impl<'a> PartialEq for DirectObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            DirectObject::D(l) => if let DirectObject::D(r) = other { l == r } else { false },
            DirectObject::S(l) => if let DirectObject::S(r) = other { l == r } else { false },
            DirectObject::H(l) =>
                if let DirectObject::H(r) = other {
                    *l as *const HVal == *r as *const HVal
                } else {
                    false
                }
        }
    }
}

#[allow(non_upper_case_globals)]
impl Object {
    pub const Nil: Object = Object::S(SVal::Nil);
    pub const Undef: Object = Object::S(SVal::Undefined);
    pub const True: Object = Object::S(SVal::Bool(true));
    pub const False: Object = Object::S(SVal::Bool(false));
}


/// Combined VM-object struct for convenience in some cases.
pub struct WrappedObject<'a> {
    vm: &'a VM,
    object: Object,
}

impl fmt::Display for WrappedObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.vm.direct(&self.object) {
            DirectObject::D(ptr) => write!(f, "#dead<{:?}>", ptr),
            DirectObject::S(SVal::Undefined) => write!(f, "#undefined"),
            DirectObject::S(SVal::Nil) => write!(f, "nil"),
            DirectObject::S(SVal::Bool(true)) => write!(f, "#t"),
            DirectObject::S(SVal::Bool(false)) => write!(f, "#f"),
            DirectObject::S(SVal::Int(v)) => write!(f, "{}", v),
            DirectObject::S(SVal::Float(v)) => write!(f, "{}", v),
            DirectObject::H(HVal::Cons(car, cdr)) => {
                write!(f, "({}", self.vm.wrap(car))?;

                let mut tail: &Object = cdr;
                while let DirectObject::H(HVal::Cons(car, cdr)) = self.vm.direct(tail) {
                    write!(f, " {}", self.vm.wrap(car))?;
                    tail = cdr;
                }

                if *tail == Object::Nil {
                    write!(f, ")")
                } else {
                    write!(f, " . {})", self.vm.wrap(tail))
                }
            }
        }
    }
}


/// The Ginkgo virtual machine.  Entry point for all use of Ginkgo.
pub struct VM {
    heap: Heap<HVal>,
}

impl VM {
    /// Create and return a new VM.
    pub fn new() -> VM {
        VM { heap: Heap::default() }
    }

    /// Perform a garbage collection.
    pub fn gc(&mut self) {
        self.heap.clean();
    }

    /// Return the heap size.
    pub fn heapsize(&self) -> usize {
        self.heap.len()
    }

    /// Allow a value to be taken off the VM stack and still be kept alive.
    pub fn rooted(&mut self, obj: &Object) -> Object {
        match obj {
            Object::H(handle) => Object::R(self.heap.make_rooted(handle)),
            _ => obj.clone(),
        }
    }

    /// Create and return a new integer (fixnum) object.
    pub fn int(&self, v: isize) -> Object {
        Object::S(SVal::Int(v))
    }

    /// Create and return a new floating point object.
    pub fn float(&self, v: f64) -> Object {
        Object::S(SVal::Float(v))
    }

    /// Create and return a new unrooted cons cell.
    pub fn cons(&mut self, car: Object, cdr: Object) -> Object {
        let handle = self.heap.insert_temp(HVal::Cons(car, cdr));
        Object::H(handle)
    }

    /// Create a combined short-lived VM-object.
    pub fn wrap(&self, obj: &Object) -> WrappedObject {
        WrappedObject { vm: self, object: obj.clone() }
    }

    /// Destructure a Ginkgo object into an object that lives fully on the stack.
    pub fn direct(&self, obj: &Object) -> DirectObject {
        match obj {
            &Object::S(v) => DirectObject::S(v),
            Object::H(handle) => match self.heap.get(handle) {
                Some(h) => DirectObject::H(h),
                _ => DirectObject::D(unsafe { handle.get_unchecked() }),
            }
            Object::R(handle) => match self.heap.get(handle) {
                Some(h) => DirectObject::H(h),
                _ => DirectObject::D(unsafe { handle.handle().get_unchecked() }),
            }
        }
    }
}
