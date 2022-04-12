pub struct Ex {
    param: i32,
    para: String,
}

trait ICanDoShit {
    fn do_shit(&self);
}

impl ICanDoShit for Ex {
    fn do_shit(&self) {
        println!("zooweemama param:{} with param2:{}!", self.param, self.para);
    }
}
