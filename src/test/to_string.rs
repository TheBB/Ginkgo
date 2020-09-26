use crate::*;

#[test]
fn simple() {
    let vm = VM::new();
    assert_eq!("nil", format!("{}", vm.wrap(Object::Nil)));
    assert_eq!("#undefined", format!("{}", vm.wrap(Object::Undef)));
    assert_eq!("#t", format!("{}", vm.wrap(Object::True)));
    assert_eq!("#f", format!("{}", vm.wrap(Object::False)));
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
}

#[test]
fn floats() {
    let vm = VM::new();
    assert_eq!("0.0", format!("{}", vm.wrap(vm.float(0.0))));
    assert_eq!("0.1", format!("{}", vm.wrap(vm.float(0.1))));
    assert_eq!("123.45", format!("{}", vm.wrap(vm.float(123.45))));
}

#[test]
fn conses() {
    let mut vm = VM::new();

    let a = vm.cons(vm.int(0), Object::Nil);
    assert_eq!("(0)", format!("{}", vm.wrap(a)));

    let b = vm.cons(vm.int(1), a);
    assert_eq!("(1 0)", format!("{}", vm.wrap(b)));

    let c = vm.cons(vm.int(1), vm.int(0));
    assert_eq!("(1 . 0)", format!("{}", vm.wrap(c)));

    let d = vm.cons(vm.int(2), c);
    assert_eq!("(2 1 . 0)", format!("{}", vm.wrap(d)));
}

#[test]
fn vectors() {
    let mut vm = VM::new();

    let v = vm.vec(3);
    assert_eq!("#(#undefined #undefined #undefined)", format!("{}", vm.wrap(v)));
}
