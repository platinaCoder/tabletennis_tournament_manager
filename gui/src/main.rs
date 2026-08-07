#[cfg(target_arch = "wasm32")]
fn main() {
    yew::Renderer::<tabletennis_tournament_gui::app::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Build this GUI for wasm32-unknown-unknown and serve it with Trunk.");
}
