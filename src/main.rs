use anyhow::{Context, Result};
use image::DynamicImage;
use image::GenericImageView;
use image::imageops::FilterType::Lanczos3;
use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 {
        anyhow::bail!("Usage: rustwall <path_to_wallpaper> <1: Dark, 0: Light>")
    }
    let path_arg = env::args()
        .nth(1)
        .context("Please provide a path to an image")?;

    let path_str = path_arg.trim();
    let path = Path::new(path_str);

    let img = image::open(path).context("Failed to open image")?;
    let img = resize(img);
    let pixels: Vec<Lab> = img
        .pixels()
        .map(|(_, _, p)| {
            let srgb = Srgb::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            );
            Lab::from_color(srgb)
        })
        .collect();

    let result = get_kmeans_hamerly(8, 20, 0.005, false, &pixels, 42);
    let centroids = sort_by_chroma(result.centroids);

    let input_mode = env::args().nth(2).context("no")?;
    let is_dark = input_mode.trim() == "1";

    let mut lab_bg = centroids[0];
    let mut lab_fg = *centroids.last().context("No colors found")?;

    if is_dark {
        std::mem::swap(&mut lab_bg, &mut lab_fg);
    }

    let (bg, fg) = format_to_hex(lab_bg, lab_fg);

    let waybar_colors = get_config_path("waybar/colors.css")?;
    let swayosd_colors = get_config_path("swayosd/colors.css")?;
    let rofi_colors = get_config_path("rofi/colors.rasi")?;
    let dunst_colors = get_config_path("dunst/dunstrc.d/colors.conf")?;

    update_css_colors(&waybar_colors, &bg, &fg)?;
    update_css_colors(&swayosd_colors, &bg, &fg)?;
    update_rofi_colors(&rofi_colors, &bg, &fg)?;
    update_dunst_colors(&dunst_colors, &bg, &fg)?;

    restart_waybar()?;
    restart_swayosd()?;
    restart_dunst()?;
    update_wallpaper(path_str)?;
    update_hyprland_theme(&fg)?;

    println!("✔ Theme successfully updated!");

    Ok(())
}

fn resize(img: DynamicImage) -> DynamicImage {
    if img.width() == 1366 && img.height() == 768 {
        img
    } else {
        img.resize(1366, 768, Lanczos3)
    }
}
fn get_config_path(sub_path: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".config").join(sub_path))
}

fn update_css_colors(path: &Path, bg: &str, fg: &str) -> Result<()> {
    let content = format!(
        "@define-color background {};\n@define-color text-foreground {};\n",
        bg, fg
    );
    fs::write(path, content).context("Failed to write CSS colors")?;
    Ok(())
}

fn update_rofi_colors(path: &Path, bg: &str, fg: &str) -> Result<()> {
    let content = format!(
        "* {{\n    background: {};\n    foreground: {};\n}}\n",
        bg, fg
    );
    fs::write(path, content).context("Failed to write Rofi colors")?;
    Ok(())
}

fn restart_swayosd() -> Result<()> {
    let _ = Command::new("pkill").arg("swayosd-server").status();

    Command::new("swayosd-server")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start SwayOSD daemon")?;

    Ok(())
}

fn restart_waybar() -> Result<()> {
    let _ = Command::new("pkill").arg("waybar").status();
    let home = std::env::var("HOME")?;
    Command::new("waybar")
        .arg("-c")
        .arg(format!("{}/.config/waybar/return.jsonc", home))
        .arg("-s")
        .arg(format!("{}/.config/waybar/return.css", home))
        .spawn()?;
    Ok(())
}

fn sort_by_chroma(mut centroids: Vec<Lab>) -> Vec<Lab> {
    centroids.sort_by(|a, b| {
        let chroma_a = (a.a.powi(2) + a.b.powi(2)).sqrt();
        let chroma_b = (b.a.powi(2) + b.b.powi(2)).sqrt();
        chroma_b.partial_cmp(&chroma_a).unwrap()
    });
    centroids
}

fn format_to_hex(bg_lab: Lab, fg_lab: Lab) -> (String, String) {
    let to_hex = |l: Lab| {
        let s = Srgb::from_color(l);
        format!(
            "#{:02x}{:02x}{:02x}",
            (s.red * 255.0) as u8,
            (s.green * 255.0) as u8,
            (s.blue * 255.0) as u8
        )
    };
    (to_hex(bg_lab), to_hex(fg_lab))
}

fn update_hyprland_theme(bg: &str) -> Result<()> {
    let cut_bg: String = bg.chars().skip(1).collect();
    Command::new("hyprctl")
        .args([
            "keyword",
            "general:col.active_border",
            &format!("rgba({}AA)", cut_bg),
        ])
        .status()?;
    Ok(())
}

fn update_wallpaper(path: &str) -> Result<()> {
    let abs_path = std::fs::canonicalize(path)?;
    Command::new("hyprctl")
        .args([
            "hyprpaper",
            "wallpaper",
            &format!(", {}, cover", abs_path.to_str().unwrap()),
        ])
        .status()?;
    Ok(())
}
fn update_dunst_colors(path: &Path, bg: &str, fg: &str) -> Result<()> {
    let content = format!(
        "[global]\n\
        frame_color = \"{fg}\"\n\
        highlight = \"{fg}\"\n\n\
        [urgency_low]\n\
        background = \"{bg}\"\n\
        foreground = \"{fg}\"\n\n\
        [urgency_normal]\n\
        background = \"{bg}\"\n\
        foreground = \"{fg}\"\n\n\
        [urgency_critical]\n\
        background = \"#f38ba8\"\n\
        foreground = \"{bg}\"\n\
        frame_color = \"#f38ba8\"\n",
        bg = bg,
        fg = fg
    );

    fs::write(path, content).context("Failed to write Dunst colors")?;

    let _ = Command::new("pkill").arg("-SIGUSR2").arg("dunst").status();

    Ok(())
}
fn restart_dunst() -> Result<()> {
    let _ = Command::new("pkill").arg("dunst").status();

    Command::new("dunst")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to restart Dunst daemon")?;

    Ok(())
}
