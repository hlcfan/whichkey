use core_foundation::runloop::{CFRunLoopRun, CFRunLoopGetCurrent, CFRunLoopAddSource};
use core_foundation::base::CFRelease;
use std::process;
use std::fs::File;
use std::path::PathBuf;
use std::os::raw::c_void;
use std::ptr;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, LevelFilter, TerminalMode, ColorChoice};

// Declare the modules
mod utils;
mod cf_utils;
mod accessibility;
mod event_tap;
mod sequence;

// Import necessary items
use utils::open_accessibility_preferences;
use cf_utils::core_foundation_private::kCFRunLoopCommonModes;
use accessibility::*;
use event_tap::*;

// Type Aliases & Structs for C Types are now in accessibility.rs and event_tap.rs

// --- Constants are now in accessibility.rs and event_tap.rs ---

// FFI blocks are now in accessibility.rs and event_tap.rs

// cf_string_ref and cfstring_to_string are now in cf_utils.rs

// get_app_name_from_pid and open_accessibility_preferences are now in utils.rs

fn main() {
    // Determine log path
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_path = PathBuf::from(home_dir).join("macos_watcher.log");

    // Initialize simplelog
    let log_file = File::create(&log_path).expect("Failed to create log file");
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), log_file),
        ]
    ).expect("Failed to initialize logger");

    // Log initial messages using the new logger
    log::info!("----- Starting macOS Watcher daemon (version 2.0) -----");
    log::info!("Log file created at: {:?}", log_path);
    log::info!("Current executable: {:?}", std::env::current_exe().unwrap_or_default());

    unsafe {
        // Check if accessibility is enabled using functions from accessibility module
        let api_enabled = ax_api_enabled();
        let process_trusted = ax_is_process_trusted();

        log::debug!("Accessibility API status:");
        log::debug!("- AXAPIEnabled: {}", api_enabled);
        log::debug!("- AXIsProcessTrusted: {}", process_trusted);

        if !api_enabled || !process_trusted {
            log::warn!("Accessibility permissions may not be properly enabled.");
            log::info!("Opening System Settings...");

            println!("Accessibility permissions are required.");
            println!("Please add this app to System Settings → Privacy & Security → Accessibility");
            println!("Opening System Settings now...");

            open_accessibility_preferences();

            println!("Please grant permissions and restart the app.");
            log::warn!("App is waiting for permissions. Please restart after granting access.");
            process::exit(0);
        }

        log::debug!("Attempting to create system-wide accessibility element...");
        let system_wide = ax_ui_element_create_system_wide();
        if system_wide.is_null() {
             log::error!("Failed to create system-wide accessibility element.");
             eprintln!("Error: Failed to create system-wide accessibility element. Exiting.");
             process::exit(1);
        }
        log::debug!("Created system-wide accessibility element");

        log::debug!("Setting up CGEventTap for mouse clicks... ");
        //
        // Listens to both normal keys and modifier keys
        let event_mask = (1 << K_CG_EVENT_KEY_DOWN) | (1<<K_CG_EVENT_FLAGS_CHANGED)| (1 << K_CG_EVENT_NX_SYSDEFINED);

        let event_tap = CGEventTapCreate(
            K_CG_SESSION_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_DEFAULT,
            event_mask,
            event_callback,
            system_wide as *mut c_void,
        );

        if event_tap.is_null() {
            log::error!("Failed to create CGEventTap. Ensure Accessibility permissions.");
            eprintln!("Error: Failed to create CGEventTap. Make sure the process has Accessibility permissions.");
            process::exit(1);
        }
        log::debug!("CGEventTap created successfully.");

        let run_loop_source = CFMachPortCreateRunLoopSource(ptr::null_mut(), event_tap, 0);
        if run_loop_source.is_null() {
            log::error!("Failed to create RunLoop source for event tap.");
            eprintln!("Error: Failed to create RunLoop source for event tap.");
             if !event_tap.is_null() { CFRelease(event_tap as *const c_void); }
            process::exit(1);
        }
        log::debug!("RunLoop source created.");

        let current_run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_run_loop, run_loop_source, kCFRunLoopCommonModes);
         log::debug!("Event tap source added to run loop.");

        CGEventTapEnable(event_tap, true);
        log::debug!("CGEventTap enabled.");

        log::info!("Monitoring input events via CGEventTap.");
        println!("Successfully running with accessibility permissions!");
        println!("Monitoring input events (clicks, keys). Check logs at: {}", log_path.display());

        log::info!("Starting main run loop...");
        CFRunLoopRun();

        log::info!("Run loop finished. Exiting...");

        CGEventTapEnable(event_tap, false);
        CFRelease(run_loop_source as *const c_void);
        CFRelease(event_tap as *const c_void);
    }
}

// CoreFoundation_Private module is now in cf_utils.rs
