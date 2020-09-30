use ginkgo::VM;

fn main() {
    let mut vm = VM::new();
    let a = vm.string(String::from("hi there\n"));
    println!("{}", vm.wrap(a));
}
