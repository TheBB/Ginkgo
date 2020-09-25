use crate::*;

#[test]
fn count_unrooted() {
    let mut vm = VM::new();
    assert_eq!(0, vm.heapsize());
    let a = vm.int(0);
    assert_eq!(0, vm.heapsize());
    let b = vm.float(1.0);
    assert_eq!(0, vm.heapsize());
    let _ = vm.cons(a, b);
    assert_eq!(1, vm.heapsize());
    vm.gc();
    assert_eq!(0, vm.heapsize());
}

#[test]
fn count_rooted() {
    let mut vm = VM::new();
    let a = vm.cons(Object::True, Object::Nil);
    assert_eq!(1, vm.heapsize());
    let b = vm.cons(Object::False, a.clone());
    assert_eq!(2, vm.heapsize());
    let c = vm.rooted(&b);
    assert_eq!(2, vm.heapsize());
    vm.gc();
    assert_eq!(2, vm.heapsize());
    drop(c);
    assert_eq!(2, vm.heapsize());
    let d = vm.rooted(&a);
    assert_eq!(2, vm.heapsize());
    vm.gc();
    assert_eq!(1, vm.heapsize());
    drop(d);
    assert_eq!(1, vm.heapsize());
    vm.gc();
    assert_eq!(0, vm.heapsize());
}
