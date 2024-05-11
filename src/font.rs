// Reference: https://github.com/emilk/egui/issues/3060

fn find_cjk_font() -> Option<String> {
    #[cfg(unix)]
    {
        use std::process::Command;
        // linux/macOS command: fc-list
        let output = Command::new("sh").arg("-c").arg("fc-list").output().ok()?;
        let stdout = std::str::from_utf8(&output.stdout).ok()?;
        #[cfg(target_os = "macos")]
        let font_line = stdout
            .lines()
            .find(|line| line.contains("Regular") && line.contains("Hiragino Sans GB"))
            .unwrap_or("/System/Library/Fonts/Hiragino Sans GB.ttc");
        #[cfg(target_os = "linux")]
        let font_line = stdout
            .lines()
            .find(|line| line.contains("Regular") && line.contains("CJK"))
            .unwrap_or("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");

        let font_path = font_line.split(':').next()?.trim();
        Some(font_path.to_string())
    }
    #[cfg(windows)]
    {
        let font_file = {
            // c:/Windows/Fonts/msyh.ttc
            let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok()?);
            font_path.push("Fonts");
            font_path.push("msyh.ttc");
            font_path.to_str()?.to_string().replace("\\", "/")
        };
        Some(font_file)
    }
}

pub fn configure_fonts(ctx: &egui::Context) -> Option<()> {
    let font_file = find_cjk_font()?;
    let font_name = font_file.split('/').last()?.split('.').next()?.to_string();
    let font_file_bytes = std::fs::read(font_file).ok()?;

    let font_data = egui::FontData::from_owned(font_file_bytes);
    let mut font_def = egui::FontDefinitions::default();
    font_def.font_data.insert(font_name.to_string(), font_data);

    let font_family: egui::FontFamily = egui::FontFamily::Proportional;
    font_def.families.get_mut(&font_family)?.insert(0, font_name);

    ctx.set_fonts(font_def);
    Some(())
}
