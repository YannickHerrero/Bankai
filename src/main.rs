mod app;

use app::App;

#[tokio::main]
async fn main() {
    let _app = App::new();
    println!("Hello, bankai!");
}
