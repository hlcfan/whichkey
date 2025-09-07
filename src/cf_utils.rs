// Remove unused imports
// use core_foundation::base::CFRelease;
use core_foundation::string::{CFString, CFStringRef};
// use std::ptr;
// Import the TCFType trait
use core_foundation::base::TCFType;

// Helper function to create CFStringRef from Rust string
pub unsafe fn cf_string_ref(s: &str) -> CFStringRef {
    // Create a Rust String
    let rust_string = String::from(s);
    // Create a CFString from the Rust String
    let cf_string = CFString::new(&rust_string);
    // Get the raw pointer (CFStringRef) - now requires TCFType trait
    let cf_string_ref = cf_string.as_concrete_TypeRef();
    // Forget the CFString to prevent it from being dropped, as CoreFoundation will manage its memory
    std::mem::forget(cf_string); 
    cf_string_ref
}

// Helper function to convert CFStringRef to Rust String
pub unsafe fn cfstring_to_string(cf_string_ref: CFStringRef) -> Option<String> {
    if cf_string_ref.is_null() {
        return None;
    }
    // Use TCFType::wrap_under_get_rule for potentially owned refs from copy functions
    // Wrap the unsafe call in an unsafe block
    let cf_string = unsafe { <CFString as TCFType>::wrap_under_get_rule(cf_string_ref) };
    Some(cf_string.to_string())
}

// Hacky way to get access to private CoreFoundation globals until a better way is found
// Rename module to snake_case
pub mod core_foundation_private {
     use core_foundation::string::CFStringRef;
     #[link(name = "CoreFoundation", kind = "framework")]
     unsafe extern "C" {
          pub static kCFRunLoopCommonModes: CFStringRef;
      }
} 