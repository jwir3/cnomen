use clap::Parser;
use copypasta::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Clone, Debug)]
struct RGBValue {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Serialize, Deserialize)]
struct ColorInformation {
    name: ColorName,
}

#[derive(Serialize, Deserialize)]
struct ColorName {
    value: String,
    exact_match_name: bool,
}

#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
/// Color Nomenclature tool (i.e. color namer) that uses The Color API (https://www.thecolorapi.com/) to report
/// the name of a color.
struct Args {
    /// Color in hexidecimal form
    #[arg(long, group = "input", value_parser = hex_valid)]
    hex: Option<RGBValue>,

    /// Color in rgb() form
    #[arg(long, group = "input", value_parser = rgb_valid)]
    rgb: Option<RGBValue>,
    // #[arg(long)]
    // red: u8,

    // #[arg(long)]
    // green: u8,

    // #[arg(long)]
    // blue: u8,
}

fn rgb_valid(s: &str) -> std::result::Result<RGBValue, String> {
    // The basic thing we need is three u8 numbers, separated by commas
    let front_stripped_string = s.strip_prefix("rgb(");
    if front_stripped_string.is_some() {
        let back_stripped_string = front_stripped_string.unwrap().strip_suffix(")");
        if back_stripped_string.is_some() {
            let split_vals: Vec<&str> = back_stripped_string.unwrap().split(",").collect();
            if split_vals.len() < 3 {
                return Err(String::from("A valid r,g,b string was not detected"));
            }

            for _i in 0..3 {
                // split_vals[0]
                let red = str::parse::<u8>(split_vals[0].trim()).unwrap();
                let green = str::parse::<u8>(split_vals[1].trim()).unwrap();
                let blue = str::parse::<u8>(split_vals[2].trim()).unwrap();

                return Ok(RGBValue { red, green, blue });
            }
        }
    }

    return Err(String::from("A valid r,g,b string was not detected"));
}

fn hex_valid(s: &str) -> std::result::Result<RGBValue, String> {
    // Hex needs to be a 3- or 6- character string, possibly starting with '#'.
    let hex_regex =
        Regex::new(r"#?([0-9a-fA-F][0-9a-fA-F][0-9a-fA-F])?([0-9a-fA-F][0-9a-fA-F][0-9a-fA-F])?")
            .unwrap();

    let captures = hex_regex.captures(s).unwrap();

    if captures.len() == 0 {
        return Err(String::from("A valid hexadecimal color was not detected"));
    } else {
        let decoded = hex::decode(format!("{}{}", &captures[1], &captures[2]));

        if decoded.is_err() {
            return Err(decoded.err().unwrap().to_string());
        } else {
            let unwrapped_decoded = decoded.unwrap();
            return Ok(RGBValue {
                red: unwrapped_decoded[0],
                green: unwrapped_decoded[1],
                blue: unwrapped_decoded[2],
            });
        }
    }
}

fn main() {
    // Get input from the user in the form of either --hex, --rgb, or --red/--green/--blue
    let args = Args::parse();

    let unwrapped_rgb: RGBValue;
    if args.rgb.is_none() {
        unwrapped_rgb = args.hex.unwrap();
    } else {
        unwrapped_rgb = args.rgb.unwrap();
    }

    // Use the color api to retrieve the color name
    let response = minreq::get(format!(
        "http://www.thecolorapi.com/id?rgb={},{},{}",
        unwrapped_rgb.red, unwrapped_rgb.green, unwrapped_rgb.blue
    ))
    .send()
    .unwrap();

    let ci: Result<ColorInformation> = serde_json::from_str(response.as_str().unwrap());

    let unwrapped_ci = ci.unwrap();
    let color_name = unwrapped_ci.name.value;
    println!("Name: {}", color_name);
    println!("(This has been copied to the clipboard for you)");

    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(color_name).unwrap();
}
