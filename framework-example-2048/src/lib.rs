use anyhow::Result;
use framework::Game;

pub fn real_main() -> Result<()> {
    let game = Game::init()?;
    game.run();
    Ok(())
}

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    real_main();
}
