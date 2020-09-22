use ginkgo::{VM, Object};

fn main() {
    let mut vm = VM::new();
    let a = vm.int(0);
    let b = vm.int(1);
    let c = vm.cons(a, Object::Nil);
    let d = vm.cons(b, c);
    let z = vm.cons(a, b);
    println!("{}", vm.wrap(d));
    println!("{}", vm.wrap(z));
    let s: String = format!("{}", vm.wrap(d));
    println!("{}", s);
}
