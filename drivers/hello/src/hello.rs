use kernel::println;

pub struct HelloWorld;

impl HelloWorld {
    pub fn load() {
        println!("Greetings from hello.rs!");
    }

    pub fn unload() {
        println!("Goodbye from hello.rs!");
    }
}
