use chrono::Utc;

fn main() {
    let build_date = Utc::now().format("%-Y-%m-%d %-I:%M %p UTC").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
