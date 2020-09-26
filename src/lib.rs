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
    Cons(Object, Object),
    Vec(Vec<Object>),
}

/// Safe Ginkgo object.  Either a direct representation of a stack
/// value or a GC-handle to a heap value.
#[derive(Clone, Copy, PartialEq)]
pub enum Object {
    S(SVal),
    H(Handle<HVal>),
}

/// Safe rooted Ginkgo object.  Exactly like Object, except may live
/// on the Rust stack without problems.
#[derive(Clone)]
pub enum RootedObject {
    S(SVal),
    R(Rooted<HVal>),
}

/// Reference-ified Ginkgo object.  Like Object, except wraps a
/// reference to a heap value instead of a GC-handle.  May potentially
/// not survive a GC!
pub enum DirectObject<'a> {
    D(*const HVal),
    S(SVal),
    H(&'a HVal),
}

pub trait GObj {
    fn unroot(&self) -> Object;
    fn root(self, vm: &mut VM) -> RootedObject;
}

impl GObj for Object {
    #[inline]
    fn unroot(&self) -> Object {
        *self
    }

    fn root(self, vm: &mut VM) -> RootedObject {
        match self {
            Object::S(v) => RootedObject::S(v),
            Object::H(h) => RootedObject::R(vm.heap.make_rooted(h)),
        }
    }
}

impl GObj for RootedObject {
    #[inline]
    fn unroot(&self) -> Object {
        match self {
            RootedObject::S(v) => Object::S(*v),
            RootedObject::R(h) => Object::H(h.handle()),
        }
    }

    fn root(self, _: &mut VM) -> RootedObject {
        self
    }
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
        match self {
            HVal::Cons(car, cdr) => {
                car.trace(tracer);
                cdr.trace(tracer);
            },
            HVal::Vec(vec) => for obj in vec {
                obj.trace(tracer);
            }
        }
    }
}

// impl PartialEq for Object {
//     fn eq(&self, other: &Self) -> bool {
//         match self {
//             Object::S(l) => if let Object::S(r) = other { l == r} else { false },
//             Object::H(l) => match other {
//                 Object::H(r) => l == r,
//                 Object::R(r) => *l == r.handle(),
//                 _ => false,
//             }
//             Object::R(l) => match other {
//                 Object::H(r) => l.handle() == *r,
//                 Object::R(r) => l.handle() == r.handle(),
//                 _ => false,
//             }
//         }
//     }
// }

// impl<'a> PartialEq for DirectObject<'a> {
//     fn eq(&self, other: &Self) -> bool {
//         match self {
//             DirectObject::D(l) => if let DirectObject::D(r) = other { l == r } else { false },
//             DirectObject::S(l) => if let DirectObject::S(r) = other { l == r } else { false },
//             DirectObject::H(l) =>
//                 if let DirectObject::H(r) = other {
//                     *l as *const HVal == *r as *const HVal
//                 } else {
//                     false
//                 }
//         }
//     }
// }

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
        match self.vm.direct(self.object) {
            DirectObject::D(ptr) => write!(f, "#dead<{:?}>", ptr),
            DirectObject::S(SVal::Undefined) => write!(f, "#undefined"),
            DirectObject::S(SVal::Nil) => write!(f, "nil"),
            DirectObject::S(SVal::Bool(true)) => write!(f, "#t"),
            DirectObject::S(SVal::Bool(false)) => write!(f, "#f"),
            DirectObject::S(SVal::Int(v)) => write!(f, "{}", v),
            DirectObject::S(SVal::Float(v)) => {
                let s = format!("{}", v);
                match s.find('.') {
                    Some(_) => write!(f, "{}", s),
                    None => write!(f, "{}.0", s),
                }
            }
            DirectObject::H(&HVal::Cons(car, cdr)) => {
                write!(f, "({}", self.vm.wrap(car))?;

                let mut tail: Object = cdr;
                while let DirectObject::H(&HVal::Cons(car, cdr)) = self.vm.direct(tail) {
                    write!(f, " {}", self.vm.wrap(car))?;
                    tail = cdr;
                }

                if tail == Object::Nil {
                    write!(f, ")")
                } else {
                    write!(f, " . {})", self.vm.wrap(tail))
                }
            }
            DirectObject::H(HVal::Vec(vec)) => {
                write!(f, "#(")?;
                if vec.len() > 0 {
                    write!(f, "{}", self.vm.wrap(vec[0]))?;
                }
                for obj in &vec[1..] {
                    write!(f, " {}", self.vm.wrap(*obj))?;
                }
                write!(f, ")")
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

    /// Create and return a new integer (fixnum) object.
    #[inline]
    pub fn int(&self, v: isize) -> Object {
        Object::S(SVal::Int(v))
    }

    /// Create and return a new floating point object.
    #[inline]
    pub fn float(&self, v: f64) -> Object {
        Object::S(SVal::Float(v))
    }

    /// Create and return a new unrooted cons cell.
    #[inline]
    pub fn cons(&mut self, car: impl GObj, cdr: impl GObj) -> Object {
        let handle = self.heap.insert_temp(HVal::Cons(car.unroot(), cdr.unroot()));
        Object::H(handle)
    }

    /// Create and return a new unrooted vector with initial length,
    /// initialized with undefined objects.
    pub fn vec(&mut self, len: usize) -> Object {
        let vec = vec![Object::Undef; len];
        let handle = self.heap.insert_temp(HVal::Vec(vec));
        Object::H(handle)
    }

    /// Create a combined short-lived VM-object.
    #[inline]
    pub fn wrap(&self, obj: impl GObj) -> WrappedObject {
        WrappedObject { vm: self, object: obj.unroot() }
    }

    /// Destructure a Ginkgo object into an object that lives fully on the stack.
    #[inline]
    pub fn direct(&self, obj: impl GObj) -> DirectObject {
        match obj.unroot() {
            Object::S(v) => DirectObject::S(v),
            Object::H(handle) => match self.heap.get(handle) {
                Some(h) => DirectObject::H(h),
                _ => DirectObject::D(unsafe { handle.get_unchecked() }),
            }
        }
    }
}
