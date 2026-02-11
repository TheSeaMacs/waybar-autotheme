use image::GenericImageView;
use kmeans_colors::get_kmeans_hamerly;
use palette::{FromColor, Lab, Srgb};
use std::fs;
use std::io;
use std::process::Command;

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
    let result = get_kmeans_hamerly(2, 20, 0.005, false, &pixels, 42);
    let color_1 = Srgb::from_color(result.centroids[0]);
    let color_2 = Srgb::from_color(result.centroids[1]);
    println!("Colors extracted!");
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
    update_waybar_theme(&bg, &fg).expect("Failed to update waybar");
    println!("Updating waybar successful!");

    let _ = Command::new("pkill").arg("waybar").status();

    let home = std::env::var("HOME").unwrap();

    Command::new("waybar")
        .arg("-c")
        .arg(format!("{}/.config/waybar/return.jsonc", home))
        .arg("-s")
        .arg(format!("{}/.config/waybar/return.css", home))
        .spawn()
        .expect("Could not run waybar");
    println!("Waybar Started With new colors");
}
