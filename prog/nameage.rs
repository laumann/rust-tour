#[deriving(Clone)]
struct Person {
    name:   String,
    age:    uint,
    height: f32,
    weight: f32
}

impl Person {
    fn name_and_age(&self) -> (String, uint) {
	let Person {
	    name: name,
	    age: age,
	    ..
	} = (*self).clone();
	(name, age)
    }
}

fn main() {
    let me = Person{name: "Thomas".to_string(), age: 28, height: 172.0, weight: 84.5};
    let (name, age) = me.name_and_age();
    println!("Hi! My name is {} and I'm {} years old!", name, age);
}
