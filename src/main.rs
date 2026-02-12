use image::GenericImageView;
use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};
use std::fs;
use std::io;
use std::process::Command;

fn restart_waybar() -> std::io::Result<()> {
    let _ = Command::new("pkill").arg("waybar").status();

    let home = std::env::var("HOME").unwrap();

    Command::new("waybar")
        .arg("-c")
        .arg(format!("{}/.config/waybar/return.jsonc", home))
        .arg("-s")
        .arg(format!("{}/.config/waybar/return.css", home))
        .spawn()?;

    println!("Waybar Started With new colors");

    Ok(())
}

fn update_waybar_theme(bg: &str, fg: &str) -> std::io::Result<()> {
    let home = std::env::var("HOME").unwrap();
    let conf_dir = format!("{}/.config/waybar/return.css", home);

    let content = fs::read_to_string(&conf_dir)?;

    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if line.contains("{variant:bg}") {
            lines.push(format!(
                "@define-color background {}; /* {{variant:bg}} */",
                bg
            ));
        } else if line.contains("{variant:fg}") {
            lines.push(format!(
                "@define-color text-foreground {}; /* {{variant:fg}} */",
                fg
            ));
        } else {
            lines.push(line.to_string());
        }
    }

    fs::write(&conf_dir, lines.join("\n"))?;

    Ok(())
}

fn update_hyprland_theme(bg: &str) -> std::io::Result<()> {
    let cut_bg: String = bg.chars().skip(1).collect();
    let arg = format!("rgba({}AA)", cut_bg);
    let status = Command::new("hyprctl")
        .args(["keyword", "general:col.active_border", &arg])
        .status()?;

    if status.success() {
        println!("Successfully updated hyprland theme!");
    }

    Ok(())
}

fn update_wallpaper(path: &str) -> std::io::Result<()> {
    let abs_path = std::fs::canonicalize(path)?;
    let path_str = abs_path.to_str().unwrap();

    let arg = format!(", {}, cover", path_str);

    let status = Command::new("hyprctl")
        .args(["hyprpaper", "wallpaper", &arg])
        .status()?;

    if status.success() {
        println!("Successfully updated wallpaper!");
    }

    Ok(())
}

fn main() {
    let mut path = String::new();

    println!("Path to wallpaper:");
    io::stdin()
        .read_line(&mut path)
        .expect("You are an idiot sandwich");
    let path = path.trim();

    let img = image::open(path).unwrap();
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
    let mut centroids = result.centroids;

    centroids.sort_by(|a, b| {
        let chroma_a = (a.a.powi(2) + a.b.powi(2)).sqrt();
        let chroma_b = (b.a.powi(2) + b.b.powi(2)).sqrt();
        chroma_b.partial_cmp(&chroma_a).unwrap()
    });

    println!("Choose mode (0 for Light, 1 for Dark)");
    let mut input_mode = String::new();
    io::stdin().read_line(&mut input_mode).unwrap();

    let mode: u8 = match input_mode.trim() {
        "0" => 0,
        "1" => 1,
        _ => {
            eprintln!("ERROR! You must choose 0 or 1.");
            std::process::exit(1);
        }
    };

    let mut lab_bg = centroids[0];
    let mut lab_fg = centroids[centroids.len() - 1];

    if mode == 1 {
        std::mem::swap(&mut lab_bg, &mut lab_fg);
    }

    println!("Saturation Multiplier (0.0 - 2.0 | leave empty for default):");
    let mut sat_input = String::new();
    io::stdin().read_line(&mut sat_input).unwrap();

    let trimmed = sat_input.trim();

    if !trimmed.is_empty() {
        let sat_mult: f32 = trimmed.parse().expect("You're an idiot");
        lab_bg.l = (lab_bg.l * (2.0 - sat_mult)).clamp(0.0, 100.0);
        lab_fg.l = (lab_fg.l * (sat_mult)).min(95.0);
        lab_fg.a *= sat_mult;
        lab_fg.b *= sat_mult;
    } else {
        println!("No input detected, using raw cluster colors...");
    }
    let color_1 = Srgb::from_color(lab_bg);
    let color_2 = Srgb::from_color(lab_fg);

    let bg = format!(
        "#{:02x}{:02x}{:02x}",
        (color_1.red * 255.0) as u8,
        (color_1.green * 255.0) as u8,
        (color_1.blue * 255.0) as u8,
    );

    let fg = format!(
        "#{:02x}{:02x}{:02x}",
        (color_2.red * 255.0) as u8,
        (color_2.green * 255.0) as u8,
        (color_2.blue * 255.0) as u8
    );

    println!("Colors extracted!");

    update_waybar_theme(&bg, &fg).expect("Failed to update waybar");
    println!("Updating waybar successful!");
    restart_waybar().expect("Failed to restart waybar");

    update_wallpaper(path).expect("Failed to update hyprpaper");

    update_hyprland_theme(&bg).expect("Failed to update hyprland theme");
}
