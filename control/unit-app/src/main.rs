use unit::log;

unit::application! {
    name = "robotwars",
}

#[unit::init]
async fn init() {
    log!("i'm alive");
}
