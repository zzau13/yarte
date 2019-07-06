#[path = "src/recompile.rs"]
mod recompile;

fn main() {
    recompile::when_changed();
}
