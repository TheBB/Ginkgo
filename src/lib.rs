use std::fmt;

use broom::{Heap, Handle, Rooted};
use broom::prelude::{Trace, Tracer};

#[cfg(test)]
mod test;

mod string;


/// Stack-based Ginkgo value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SVal {
    Undefined,
    Nil,
    Bool(bool),
    Int(isize),
    Float(f64),
}

/// Heap-based (garbage-collected) Ginkgo value.
#[derive(Clone, Debug, PartialEq)]
pub enum HVal {
    Cons(Object, Object),
    Vec(Vec<Object>),
    String(String),
}

/// Safe Ginkgo object.  Either a direct representation of a stack
/// value or a GC-handle to a heap value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Object {
    S(SVal),
    H(Handle<HVal>),
}

/// Safe rooted Ginkgo object.  Exactly like Object, except may live
/// on the Rust stack without problems.
#[derive(Clone, Debug)]
pub enum RootedObject {
    S(SVal),
    H(Rooted<HVal>),
}

/// Reference-ified Ginkgo object.  Like Object, except wraps a
/// reference to a heap value instead of a GC-handle.  May potentially
/// not survive a GC!
enum DObj<M> {
    D(*const HVal),
    S(SVal),
    H(M),
}

type DirectObject<'a> = DObj<&'a HVal>;
type DirectObjectMut<'a> = DObj<&'a mut HVal>;

pub trait GObj {
    fn unroot(&self) -> Object;
    fn root(self, vm: &mut VM) -> RootedObject;
    fn as_sval(&self) -> Option<SVal>;

    fn as_int(&self) -> Option<isize> {
        match self.as_sval()? {
            SVal::Int(v) => Some(v),
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f64> {
        match self.as_sval()? {
            SVal::Float(v) => Some(v),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.as_sval()? {
            SVal::Bool(v) => Some(v),
            _ => None,
        }
    }
}

impl GObj for Object {
    #[inline]
    fn unroot(&self) -> Object {
        *self
    }

    fn root(self, vm: &mut VM) -> RootedObject {
        match self {
            Object::S(v) => RootedObject::S(v),
            Object::H(h) => RootedObject::H(vm.heap.make_rooted(h)),
        }
    }

    fn as_sval(&self) -> Option<SVal> {
        match self {
            &Object::S(v) => Some(v),
            _ => None,
        }
    }
}

impl GObj for RootedObject {
    #[inline]
    fn unroot(&self) -> Object {
        match self {
            RootedObject::S(v) => Object::S(*v),
            RootedObject::H(h) => Object::H(h.handle()),
        }
    }

    fn root(self, _: &mut VM) -> RootedObject {
        self
    }

    fn as_sval(&self) -> Option<SVal> {
        match self {
            &RootedObject::S(v) => Some(v),
            _ => None,
        }
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
            _ => (),
        }
    }
}

impl PartialEq for RootedObject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RootedObject::S(l), RootedObject::S(r)) => l == r,
            (RootedObject::H(l), RootedObject::H(r)) => l.handle() == r.handle(),
            _ => false,
        }
    }
}

impl PartialEq<RootedObject> for Object {
    fn eq(&self, other: &RootedObject) -> bool {
        match (self, other) {
            (Object::S(l), RootedObject::S(r)) => l == r,
            (Object::H(l), RootedObject::H(r)) => *l == r.handle(),
            _ => false,
        }
    }
}

impl PartialEq<Object> for RootedObject {
    fn eq(&self, other: &Object) -> bool {
        match (self, other) {
            (RootedObject::S(l), Object::S(r)) => l == r,
            (RootedObject::H(l), Object::H(r)) => l.handle() == *r,
            _ => false,
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
        match self.vm.direct(self.object) {
            DObj::D(ptr) => write!(f, "#dead<{:?}>", ptr),
            DObj::S(SVal::Undefined) => write!(f, "#undefined"),
            DObj::S(SVal::Nil) => write!(f, "nil"),
            DObj::S(SVal::Bool(true)) => write!(f, "#t"),
            DObj::S(SVal::Bool(false)) => write!(f, "#f"),
            DObj::S(SVal::Int(v)) => write!(f, "{}", v),
            DObj::S(SVal::Float(v)) => {
                let s = format!("{}", v);
                match s.find('.') {
                    Some(_) => write!(f, "{}", s),
                    None => write!(f, "{}.0", s),
                }
            }
            DObj::H(&HVal::Cons(car, cdr)) => {
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
            DObj::H(HVal::Vec(vec)) => {
                write!(f, "#(")?;
                if vec.len() > 0 {
                    write!(f, "{}", self.vm.wrap(vec[0]))?;
                }
                for obj in &vec[1..] {
                    write!(f, " {}", self.vm.wrap(*obj))?;
                }
                write!(f, ")")
            }
            DObj::H(HVal::String(s)) => write!(f, "\"{}\"", string::escape(s))
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

    /// Get the car of a cons cell.
    #[inline]
    pub fn car(&self, obj: impl GObj) -> Option<Object> {
        match self.direct(obj) {
            DObj::H(&HVal::Cons(car, _)) => Some(car),
            _ => None,
        }
    }

    /// Get the cdr of a cons cell.
    #[inline]
    pub fn cdr(&self, obj: impl GObj) -> Option<Object> {
        match self.direct(obj) {
            DObj::H(&HVal::Cons(_, cdr)) => Some(cdr),
            _ => None
        }
    }

    /// Create and return a new unrooted vector with initial length,
    /// initialized with undefined objects.
    pub fn vec(&mut self, len: usize) -> Object {
        let vec = vec![Object::Undef; len];
        let handle = self.heap.insert_temp(HVal::Vec(vec));
        Object::H(handle)
    }

    /// Get the n'th element of a vector.
    pub fn vec_get(&self, obj: impl GObj, index: usize) -> Option<Object> {
        match self.direct(obj) {
            DObj::H(HVal::Vec(vec)) => vec.get(index).copied(),
            _ => None,
        }
    }

    /// Set the n'th element of a vector.
    pub fn vec_set(&mut self, obj: impl GObj, index: usize, val: impl GObj) -> Result<(), ()> {
        match self.direct_mut(obj) {
            DObj::H(HVal::Vec(vec)) => {
                if index < vec.len() {
                    vec[index] = val.unroot();
                    Ok(())
                } else {
                    Err(())
                }
            },
            _ => Err(()),
        }
    }

    /// Create and return a new string.
    pub fn string(&mut self, s: String) -> Object {
        let handle = self.heap.insert_temp(HVal::String(s));
        Object::H(handle)
    }

    /// Create a combined short-lived VM-object.
    #[inline]
    pub fn wrap(&self, obj: impl GObj) -> WrappedObject {
        WrappedObject { vm: self, object: obj.unroot() }
    }

    /// Destructure a Ginkgo object into an object that lives fully on the stack.
    #[inline]
    fn direct(&self, obj: impl GObj) -> DirectObject {
        match obj.unroot() {
            Object::S(v) => DObj::S(v),
            Object::H(handle) => match self.heap.get(handle) {
                Some(h) => DObj::H(h),
                _ => DObj::D(unsafe { handle.get_unchecked() }),
            }
        }
    }

    /// Destructure a Ginkgo object into a mutable object that lives fully on the stack.
    #[inline]
    fn direct_mut(&mut self, obj: impl GObj) -> DirectObjectMut {
        match obj.unroot() {
            Object::S(v) => DObj::S(v),
            Object::H(handle) => match self.heap.get_mut(handle) {
                Some(h) => DObj::H(h),
                _ => DObj::D(unsafe { handle.get_unchecked() }),
            }
        }
    }
}
