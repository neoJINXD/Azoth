pub struct Ex {
    param : i32,
    para : String,
}

trait ICanDoShit {
    fn doShit(&self);
}

impl ICanDoShit for Ex {
    fn doShit(&self) {
        println!("zooweemama param:{} with param2:{}!", self.param, self.para);
    }
}
