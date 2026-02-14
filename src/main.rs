use wixen_mail::application::{AccountManager, MessageManager};
use wixen_mail::data::ConfigManager;
use wixen_mail::presentation::{Accessibility, UI};

fn main() {
    println!("Wixen Mail - Accessible Mail Client");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Initializing modular architecture...");
    println!();

    // Initialize core components
    match initialize_components() {
        Ok(_) => {
            println!("✓ All components initialized successfully");
            println!();
            println!("Architecture layers:");
            println!("  • Presentation Layer: UI + Accessibility");
            println!(
                "  • Application Layer: Accounts, Messages, Composition, Search, Filters, Contacts"
            );
            println!("  • Service Layer: IMAP, SMTP, POP3, Security, Cache, Attachments");
            println!("  • Data Layer: Database, Storage, Configuration");
            println!();
            println!("See ROADMAP.md for next development steps.");
        }
        Err(e) => {
            eprintln!("✗ Error initializing components: {}", e);
        }
    }

    println!();
    println!("For accessibility information, see ACCESSIBILITY.md");
}

fn initialize_components() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize presentation layer
    let _ui = UI::new()?;
    let accessibility = Accessibility::new()?;
    accessibility.initialize()?;
    println!("✓ Presentation layer initialized");

    // Initialize application layer
    let _account_manager = AccountManager::new()?;
    let _message_manager = MessageManager::new()?;
    println!("✓ Application layer initialized");

    // Initialize data layer
    let _config_manager = ConfigManager::new()?;
    println!("✓ Data layer initialized");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        // Basic test to ensure compilation works
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_component_initialization() {
        // Test that components can be initialized
        let result = initialize_components();
        assert!(result.is_ok());
    }
}
