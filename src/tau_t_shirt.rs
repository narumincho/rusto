pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(
        "./output/tau_t_shirt.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"><rect width=\"100\" height=\"100\" fill=\"#000\"/><text x=\"50%\" y=\"50%\" font-size=\"20\" fill=\"#fff\" text-anchor=\"middle\" dominant-baseline=\"middle\">TAU T-SHIRT</text></svg>",
    )?;
    println!("Hello, world!");
    Ok(())
}
