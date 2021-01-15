mod game;
mod renderer;

pub use crate::game::Game;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    println!("hello, world!");
}
