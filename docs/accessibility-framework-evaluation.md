# Accessibility Framework Evaluation for Wixen Mail

## Executive Summary

**Decision: Enhance egui with AccessKit for Windows screen reader support**

After thorough evaluation, we recommend continuing with **egui + AccessKit** rather than switching to a different framework. This provides the best balance of accessibility, cross-platform support, and development velocity.

## Framework Comparison

### 1. egui + AccessKit

**Accessibility Support:**
- ✅ AccessKit integration provides Windows UIA support
- ✅ Screen reader support (NVDA, JAWS, Narrator)
- ✅ Keyboard navigation built-in
- ✅ Focus management
- ✅ Cross-platform (Windows, macOS, Linux)

**Pros:**
- Already integrated in current codebase
- Active development and community
- Good performance (immediate mode)
- AccessKit specifically designed for screen reader support
- Clean API and easy to use
- Cross-platform accessibility

**Cons:**
- AccessKit integration is relatively new
- May require additional work for full WCAG 2.1 AA compliance

**Verdict:** ✅ RECOMMENDED

### 2. WXDragon

**Status:** ❌ DOES NOT EXIST

Research shows WXDragon is a hypothetical/planned library mentioned in documentation but not actually available as a Rust crate or library. This appears to have been a placeholder name for future Windows-native integration.

### 3. native-windows-gui (NWG)

**Accessibility Support:**
- ⚠️ Limited native Windows UIA support
- ⚠️ Requires manual UIA provider implementation
- ✅ Windows-only, native feel
- ⚠️ Less documentation for accessibility

**Pros:**
- Native Windows controls
- Mature library
- Good Windows integration

**Cons:**
- Windows-only (not cross-platform)
- More complex API
- Requires more work for accessibility
- Smaller community than egui

**Verdict:** ❌ Not recommended (Windows-only, more work)

### 4. IXP (Interactive Experience Platform)

**Status:** Not found as a viable Rust GUI framework

### 5. Tauri + Web Technologies

**Accessibility Support:**
- ✅ Excellent accessibility (HTML/CSS/ARIA)
- ✅ Screen reader support through web standards
- ✅ Cross-platform

**Cons:**
- Different architecture (hybrid native/web)
- Larger bundle size
- More complex setup
- Overkill for this project

**Verdict:** ❌ Not needed for this use case

## AccessKit Integration Details

### What is AccessKit?

AccessKit is a Rust library that provides platform-native accessibility trees:
- Windows: UI Automation (UIA)
- macOS: NSAccessibility
- Linux: AT-SPI2

### Integration with egui

egui 0.28+ has built-in AccessKit support via `egui_accesskit`:

```toml
[dependencies]
eframe = { version = "0.29", features = ["accesskit"] }
egui = "0.29"
```

### Implementation Requirements

1. **Enable AccessKit Feature**
   - Add `accesskit` feature to eframe

2. **Add Semantic Labels**
   - Label all UI elements with descriptive text
   - Add roles (button, list, textbox, etc.)

3. **Implement Focus Management**
   - Proper tab order
   - Focus indicators
   - Focus trapping in dialogs

4. **Screen Reader Announcements**
   - Use AccessKit announcement API
   - Priority levels for different messages

5. **Keyboard Navigation**
   - All functionality accessible via keyboard
   - Implement shortcuts from ACCESSIBILITY.md

## Recommendation: egui + AccessKit Enhancement

### Phase 1: Enable AccessKit (Immediate)
1. Add `accesskit` feature to Cargo.toml
2. Enable AccessKit in eframe options
3. Add basic semantic labels to existing UI

### Phase 2: Full Accessibility Implementation
1. Comprehensive semantic labeling
2. ARIA roles for all components
3. Screen reader announcement system
4. Keyboard shortcut integration
5. Focus management improvements

### Phase 3: Testing and Validation
1. NVDA testing on Windows
2. JAWS testing on Windows
3. Narrator testing on Windows
4. Keyboard-only navigation testing
5. WCAG 2.1 AA compliance validation

## Implementation Strategy

### Step 1: Minimal Changes
```rust
// In src/bin/ui.rs - Enable AccessKit
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_title("Wixen Mail"),
    // Enable AccessKit for screen reader support
    #[cfg(feature = "accesskit")]
    accesskit_enabled: true,
    ..Default::default()
};
```

### Step 2: Add Semantic Labels
```rust
// Example: Folder tree with accessibility
ui.label("Folder Tree")
    .on_hover_text("Navigate email folders");

if ui.button("INBOX")
    .on_hover_text("Inbox folder with 5 unread messages")
    .clicked() {
    // Handle click
}
```

### Step 3: Screen Reader Announcements
```rust
// Integrate with existing announcement queue
use wixen_mail::presentation::accessibility::AnnouncementQueue;

let queue = AnnouncementQueue::new();
queue.announce("New message from John Doe", Priority::High);
```

## Testing Plan

### Manual Testing
1. Install NVDA (free, open source)
2. Navigate entire UI with screen reader only
3. Test all keyboard shortcuts
4. Verify announcements for events

### Automated Testing
1. Use AccessKit's testing utilities
2. Verify accessibility tree structure
3. Check semantic roles and labels
4. Validate focus order

## Migration Strategy (If Needed Later)

If egui + AccessKit proves insufficient, we have a clean architecture:
- UI is isolated in `src/presentation/ui.rs`
- Accessibility layer is separate in `src/presentation/accessibility/`
- Business logic is independent in application layer
- Easy to swap UI framework without affecting core logic

## Conclusion

**Proceed with egui + AccessKit enhancement**

This approach:
- ✅ Leverages existing codebase
- ✅ Provides excellent screen reader support
- ✅ Maintains cross-platform compatibility
- ✅ Has active development and support
- ✅ Requires minimal architectural changes
- ✅ Can be fully WCAG 2.1 AA compliant

Next steps:
1. Enable AccessKit feature
2. Add semantic labels to existing UI
3. Implement screen reader announcements
4. Full keyboard navigation
5. Testing with real screen readers

## References

- [AccessKit GitHub](https://github.com/AccessKit/accesskit)
- [egui Accessibility](https://github.com/emilk/egui/discussions/2294)
- [Windows UIA Documentation](https://docs.microsoft.com/en-us/windows/win32/winauto/)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
