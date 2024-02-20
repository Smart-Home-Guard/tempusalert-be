mod features;

use features::Feature;

struct Server {

}

impl Server {
    fn register(&mut self, mut module: impl Feature) -> &mut Self {
        module.init();
        self
    }

    async fn run(&mut self) {
        // receive message
        // delegate message
    }
}