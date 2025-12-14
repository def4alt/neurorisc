use crate::gui::app::App;

pub fn run() -> anyhow::Result<()> {
    eframe::run_native(
        "Sensory Circuit",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
    Ok(())
}
