use core_foundation::base::{CFAllocatorRef, CFIndex, CFRelease};
use core_foundation::runloop::CFRunLoopSourceRef;
use core_foundation::string::CFStringRef;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::os::raw::{c_longlong, c_void};
use std::ptr;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Instant;

// Import necessary items from other modules
use crate::accessibility::*;
use crate::cf_utils::{cf_string_ref, cfstring_to_string};
use crate::sequence::KeyStroke;
use crate::sequence::KeyStrokeRecorder;
use crate::utils::get_app_name_from_pid;

// Re-export AXUIElementRef for use within this module if needed
pub use crate::accessibility::AXUIElementRef;

// use crate::sequence::*;
//
// Type Aliases & Structs for C Types
pub type CGEventTapProxy = *mut c_void; // Opaque pointer
pub type CGEventType = u32;
pub type CGEventRef = *mut c_void; // Opaque pointer
pub type CFMachPortRef = *mut c_void; // Opaque pointer (actually __CFMachPort*)
pub type CGEventTapLocation = u32;
pub type CGEventTapPlacement = u32;
pub type CGEventTapOptions = u32;
pub type CGEventMask = u64;
pub type CGEventField = u32;
pub type CGEventFlags = *mut c_void; // Opaque pointer

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

// Type for the event tap callback
pub type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    type_: CGEventType,
    event: CGEventRef,
    userInfo: *mut c_void,
) -> CGEventRef;

// Constants
// CGEventTapLocation
pub const K_CG_HID_EVENT_TAP: CGEventTapLocation = 0;
pub const K_CG_SESSION_EVENT_TAP: CGEventTapLocation = 1;
// CGEventTapPlacement
pub const K_CG_HEAD_INSERT_EVENT_TAP: CGEventTapPlacement = 0;
// CGEventTapOptions
pub const K_CG_EVENT_TAP_DEFAULT: CGEventTapOptions = 0x00000000;
// CGEventType
#[allow(dead_code)]
pub const K_CG_EVENT_NULL: CGEventType = 0; // Internal use
                                            //
// CGEventType Enum
// https://learn.microsoft.com/en-us/dotnet/api/coregraphics.cgeventtype?view=xamarin-mac-sdk-14
pub const K_CG_EVENT_LEFT_MOUSE_DOWN: CGEventType = 1;
pub const K_CG_EVENT_KEY_DOWN: CGEventType = 10;
pub const K_CG_EVENT_NX_SYSDEFINED: CGEventType = 14;
pub const K_CG_EVENT_FLAGS_CHANGED: CGEventType = 12;
pub const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: CGEventType = 0xFFFFFFFE;
pub const K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT: CGEventType = 0xFFFFFFFF;

// CGEventField
pub const K_CG_EVENT_TARGET_UNIX_PROCESS_ID: CGEventField = 8; // kCGEventTargetUnixProcessID
pub const K_CG_KEYBOARD_EVENT_KEYCODE: CGEventField = 9; // kCGKeyboardEventKeycode
pub const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 524288;

#[link(name = "CoreGraphics", kind = "framework")]
#[allow(non_snake_case)] // To allow function names like CGEventTapCreate
unsafe extern "C" {
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        eventsOfInterest: CGEventMask,
        callback: CGEventTapCallBack,
        userInfo: *mut c_void,
    ) -> CFMachPortRef;

    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    pub fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    pub fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> c_longlong; // Note: Returns int64_t
    pub fn CGEventGetFlags(event: CGEventRef) -> u64;

    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef, // Usually kCFAllocatorDefault or null
        tap: CFMachPortRef,
        order: CFIndex, // Usually 0
    ) -> CFRunLoopSourceRef;
}

static mut SEQUENCE_RECORDER: Lazy<KeyStrokeRecorder> = Lazy::new(|| KeyStrokeRecorder::new());

// The actual event callback function
pub unsafe extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    typ: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // Get userInfo (system_wide element)
    if user_info.is_null() {
        log::error!("userInfo (system_wide element) is null in callback!");
        return event; // Cannot proceed
    }
    let system_wide_element = user_info as AXUIElementRef;

    // Handle tap disable events
    if typ == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT || typ == K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT {
        println!("DEBUG: Event tap disabled (type: {})", typ);
        log::warn!(
            "Event Tap disabled (type: {})! Input monitoring stopped.",
            typ
        );
        // We might need to re-enable the tap if desired.
        // unsafe { CGEventTapEnable(proxy as CFMachPortRef, true) }; // Needs unsafe block if uncommented
        return event; // Return the event directly
    }

    // Get PID common to both event types we handle
    let pid =
        unsafe { CGEventGetIntegerValueField(event, K_CG_EVENT_TARGET_UNIX_PROCESS_ID) } as i32;
    let app_name = get_app_name_from_pid(pid); // Use imported function

    if typ == K_CG_EVENT_KEY_DOWN || typ == K_CG_EVENT_FLAGS_CHANGED {
        let keycode = unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
        let flags = unsafe { CGEventGetFlags(event) };
        // Only check against alt key press
        // If we need to match other keys, need to xand these
        // refer to https://docs.rs/objc2-core-graphics/latest/src/objc2_core_graphics/generated/CGEventTypes.rs.html#171
        if typ == K_CG_EVENT_FLAGS_CHANGED && (flags & K_CG_EVENT_FLAG_MASK_ALTERNATE) == 0 {
            return event;
        }

        log::info!(
            "Key Down: App='{}' (PID={}), KeyCode={}, Flags={}, Type: {}",
            app_name,
            pid,
            keycode,
            flags,
            typ,
        );

        SEQUENCE_RECORDER.record(KeyStroke {
            key_code: keycode,
            key_typ: typ,
            timestamp: Instant::now(),
        });

        if SEQUENCE_RECORDER.is_in_sequence() {
          println!("===In seq");
          SEQUENCE_RECORDER.check_sequence();

          return std::ptr::null_mut();
        }
    }

    event // Pass the event along
}
