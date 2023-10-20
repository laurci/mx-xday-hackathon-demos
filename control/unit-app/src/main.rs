unit::application! {
    name = "robotwars",
}

#[unit::topic(name = "join")]
async fn join(content: CrossbarContent) {
    let player = content.to_string();
    unit::client::send_text(format!("join_{}", player));
}
