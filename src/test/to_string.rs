use crate::*;

#[test]
fn simple() {
    let vm = VM::new();
    assert_eq!("nil", format!("{}", vm.wrap(Object::Nil)));
    assert_eq!("#undefined", format!("{}", vm.wrap(Object::Undef)));
    assert_eq!("#t", format!("{}", vm.wrap(Object::True)));
    assert_eq!("#f", format!("{}", vm.wrap(Object::False)));

    assert_eq!(Object::Nil, Object::Nil);
    assert_eq!(Object::Undef, Object::Undef);
    assert_eq!(Object::True, Object::True);
    assert_eq!(Object::False, Object::False);
    assert_ne!(Object::Nil, Object::Undef);
    assert_ne!(Object::Nil, Object::True);
    assert_ne!(Object::Nil, Object::False);
    assert_ne!(Object::Undef, Object::Nil);
    assert_ne!(Object::Undef, Object::True);
    assert_ne!(Object::Undef, Object::False);
    assert_ne!(Object::True, Object::Nil);
    assert_ne!(Object::True, Object::Undef);
    assert_ne!(Object::True, Object::False);
    assert_ne!(Object::False, Object::Nil);
    assert_ne!(Object::False, Object::Undef);
    assert_ne!(Object::False, Object::True);

    assert_eq!(None, Object::Nil.as_int());
    assert_eq!(None, Object::Undef.as_int());
    assert_eq!(None, Object::True.as_int());
    assert_eq!(None, Object::False.as_int());

    assert_eq!(None, Object::Nil.as_float());
    assert_eq!(None, Object::Undef.as_float());
    assert_eq!(None, Object::True.as_float());
    assert_eq!(None, Object::False.as_float());

    assert_eq!(None, Object::Nil.as_bool());
    assert_eq!(None, Object::Undef.as_bool());
    assert_eq!(Some(true), Object::True.as_bool());
    assert_eq!(Some(false), Object::False.as_bool());
}

#[test]
fn fixnums() {
    let vm = VM::new();
    assert_eq!("0", format!("{}", vm.wrap(vm.int(0))));
    assert_eq!("-1", format!("{}", vm.wrap(vm.int(-1))));
    assert_eq!("1", format!("{}", vm.wrap(vm.int(1))));
    assert_eq!(
        format!("{}", isize::MAX),
        format!("{}", vm.wrap(vm.int(isize::MAX))),
    );
    assert_eq!(
        format!("{}", isize::MIN),
        format!("{}", vm.wrap(vm.int(isize::MIN))),
    );

    assert_eq!(vm.int(1), vm.int(1));
    assert_ne!(vm.int(-1), vm.int(1));
    assert_ne!(vm.int(1), Object::Nil);
    assert_ne!(vm.int(1), Object::Undef);
    assert_ne!(vm.int(1), Object::True);
    assert_ne!(vm.int(1), Object::False);

    assert_eq!(Some(1), vm.int(1).as_int());
    assert_eq!(Some(-1), vm.int(-1).as_int());
    assert_eq!(None, vm.int(1).as_float());
    assert_eq!(None, vm.int(1).as_bool());
}

#[test]
fn floats() {
    let vm = VM::new();
    assert_eq!("0.0", format!("{}", vm.wrap(vm.float(0.0))));
    assert_eq!("0.1", format!("{}", vm.wrap(vm.float(0.1))));
    assert_eq!("123.45", format!("{}", vm.wrap(vm.float(123.45))));

    assert_eq!(vm.float(1.0), vm.float(1.0));
    assert_eq!(vm.float(-1.0), vm.float(-1.0));
    assert_ne!(vm.float(-1.0), vm.int(-1));
    assert_ne!(vm.float(1.0), Object::Nil);
    assert_ne!(vm.float(1.0), Object::Undef);
    assert_ne!(vm.float(1.0), Object::True);
    assert_ne!(vm.float(1.0), Object::False);

    assert_eq!(None, vm.float(2.3).as_int());
    assert_eq!(Some(0.0), vm.float(0.0).as_float());
    assert_eq!(Some(2.3), vm.float(2.3).as_float());
    assert_eq!(None, vm.float(2.3).as_bool());
}

#[test]
fn conses() {
    let mut vm = VM::new();

    let a = vm.cons(vm.int(0), Object::Nil);
    assert_eq!("(0)", format!("{}", vm.wrap(a)));
    assert_eq!(Some(vm.int(0)), vm.car(a));
    assert_eq!(Some(Object::Nil), vm.cdr(a));

    let b = vm.cons(vm.int(1), a);
    assert_eq!("(1 0)", format!("{}", vm.wrap(b)));
    assert_eq!(Some(vm.int(1)), vm.car(b));
    assert_eq!(Some(a), vm.cdr(b));

    let c = vm.cons(vm.int(1), vm.int(0));
    assert_eq!("(1 . 0)", format!("{}", vm.wrap(c)));
    assert_eq!(Some(vm.int(1)), vm.car(c));
    assert_eq!(Some(vm.int(0)), vm.cdr(c));

    let d = vm.cons(vm.int(2), c);
    assert_eq!("(2 1 . 0)", format!("{}", vm.wrap(d)));
    assert_eq!(Some(vm.int(2)), vm.car(d));
    assert_eq!(Some(c), vm.cdr(d));

    assert_ne!(d, vm.int(2));
    assert_ne!(d, Object::Nil);
    assert_ne!(d, Object::Undef);
    assert_ne!(d, Object::True);
    assert_ne!(d, Object::False);
}

#[test]
fn vectors() {
    let mut vm = VM::new();

    let v = vm.vec(3);
    assert_eq!("#(#undefined #undefined #undefined)", format!("{}", vm.wrap(v)));
    assert_eq!(Some(Object::Undef), vm.vec_get(v, 0));
    assert_eq!(Some(Object::Undef), vm.vec_get(v, 1));
    assert_eq!(Some(Object::Undef), vm.vec_get(v, 2));

    vm.vec_set(v, 0, vm.int(0)).unwrap();
    vm.vec_set(v, 1, Object::Nil).unwrap();
    vm.vec_set(v, 2, vm.float(2.3)).unwrap();

    assert_eq!("#(0 nil 2.3)", format!("{}", vm.wrap(v)));
    assert_eq!(Some(vm.int(0)), vm.vec_get(v, 0));
    assert_eq!(Some(Object::Nil), vm.vec_get(v, 1));
    assert_eq!(Some(vm.float(2.3)), vm.vec_get(v, 2));
}
