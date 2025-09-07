use objc::runtime::Object;
// Remove unused import
// use core_foundation::string::{CFString, CFStringRef};
use core_foundation::string::CFStringRef; // Keep used import
use std::os::raw::{c_int, c_void};

// Type Aliases
// Make the type alias public so it can be used/re-exported by event_tap.rs
pub type AXUIElementRef = *mut Object; // AXUIElementRef is often treated like an NSObject

// Constants
// Accessibility Attributes (as Rust strings, convert to CFStringRef later)
pub const K_AX_ROLE_ATTRIBUTE: &str = "AXRole";
pub const K_AX_IDENTIFIER_ATTRIBUTE: &str = "AXIdentifier";
pub const K_AX_DESCRIPTION_ATTRIBUTE: &str = "AXDescription";
// Accessibility Roles (as Rust strings)
pub const K_AX_BUTTON_ROLE: &str = "AXButton";

#[link(name = "ApplicationServices", kind = "framework")]
#[allow(non_snake_case)] // To allow function names like AXUIElementCopyAttributeValue
// Remove pub from the extern block
unsafe extern "C" {
    // Keep pub on items inside
    #[link_name = "AXUIElementCreateSystemWide"]
    pub fn ax_ui_element_create_system_wide() -> AXUIElementRef;
    
    #[link_name = "AXAPIEnabled"]
    pub fn ax_api_enabled() -> bool;
    
    #[link_name = "AXIsProcessTrusted"]
    pub fn ax_is_process_trusted() -> bool;

    #[link_name = "AXUIElementCopyElementAtPosition"]
    pub fn ax_ui_element_copy_element_at_position(
        application: AXUIElementRef, 
        x: f32, 
        y: f32, 
        element: *mut AXUIElementRef
    ) -> c_int; // Returns AXError (use kAXErrorSuccess = 0)
    
    #[link_name = "AXUIElementCopyAttributeValue"]
    pub fn ax_ui_element_copy_attribute_value(
        element: AXUIElementRef, 
        attribute: CFStringRef, 
        value: *mut *mut c_void // Receives CFTypeRef, cast later
    ) -> c_int; // Returns AXError

    #[link_name = "AXUIElementGetPid"]
    pub fn ax_ui_element_get_pid(element: AXUIElementRef, pid: *mut i32) -> c_int; // Returns AXError

    // AXUIElementCreateApplication is not directly used in the current callback, 
    // but keeping it here if needed later.
    // Allow dead code for this unused function
    #[allow(dead_code)]
    #[link_name = "AXUIElementCreateApplication"]
    pub fn ax_ui_element_create_application(pid: i32) -> AXUIElementRef;
} 