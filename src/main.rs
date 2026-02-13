fn main() {
    println!("Wixen Mail - Accessible Mail Client");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("This is an early development version.");
    println!("See ROADMAP.md for project status and planned features.");
    println!();
    println!("For accessibility information, see ACCESSIBILITY.md");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        // Basic test to ensure compilation works
        assert_eq!(2 + 2, 4);
    }
}
