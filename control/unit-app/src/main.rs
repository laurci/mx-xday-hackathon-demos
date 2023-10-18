use unit::log;

unit::application! {
    name = "robowars",
}

#[unit::init]
async fn init() {
    log!("i'm alive");
}
