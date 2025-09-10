use core_foundation::base::CFRelease;
use core_foundation::runloop::{CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::process;
use std::ptr;
use std::{fs, io::Error};

// Declare the modules
mod accessibility;
mod cf_utils;
mod config;
mod event_tap;
mod sequence;
mod utils;

// Import necessary items
use accessibility::*;
use cf_utils::core_foundation_private::kCFRunLoopCommonModes;
use event_tap::*;
use utils::open_accessibility_preferences;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Command are install, start
    command: String,
}

// Type Aliases & Structs for C Types are now in accessibility.rs and event_tap.rs

// --- Constants are now in accessibility.rs and event_tap.rs ---

// FFI blocks are now in accessibility.rs and event_tap.rs

// cf_string_ref and cfstring_to_string are now in cf_utils.rs

// get_app_name_from_pid and open_accessibility_preferences are now in utils.rs
// CoreFoundation_Private module is now in cf_utils.rs

fn main() {
    let args = Args::parse();
    match args.command.as_str() {
        "install" => install(),
        "start" => {
            listen();
        }
        _ => {
            println!("invalid command");
        }
    }
}

fn listen() {
    // Determine log path
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_path = PathBuf::from(home_dir).join("whichkey.log");

    // Initialize simplelog
    let log_file = File::create(&log_path).expect("Failed to create log file");
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Debug, Config::default(), log_file),
    ])
    .expect("Failed to initialize logger");

    // Log initial messages using the new logger
    log::info!("----- Starting Whichkey -----");
    log::info!("Log file created at: {:?}", log_path);
    log::info!(
        "Current executable: {:?}",
        std::env::current_exe().unwrap_or_default()
    );

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
        let event_mask = (1 << K_CG_EVENT_KEY_DOWN)
            | (1 << K_CG_EVENT_FLAGS_CHANGED)
            | (1 << K_CG_EVENT_NX_SYSDEFINED);

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
            if !event_tap.is_null() {
                CFRelease(event_tap as *const c_void);
            }
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
        println!(
            "Monitoring input events (clicks, keys). Check logs at: {}",
            log_path.display()
        );

        log::info!("Starting main run loop...");
        CFRunLoopRun();

        log::info!("Run loop finished. Exiting...");

        CGEventTapEnable(event_tap, false);
        CFRelease(run_loop_source as *const c_void);
        CFRelease(event_tap as *const c_void);
    }
}

fn install() {
    match install_config() {
        Ok(()) => {
            println!(
                "Configuration file generated at $HOME/.config/whichkey/config.toml successfully."
            );
        }
        Err(e) => {
            eprintln!("Failed to generate configuration failed at $HOME/.config/whichkey/config.toml. Error: {}", e);
        }
    }

    match install_service() {
        Ok(()) => {
            println!("Service installed successfully.");
        }
        Err(e) => {
            eprintln!("Service installation failed: {}", e);
        }
    }
}

fn install_service() -> Result<(), Error> {
    let plist = format!(
"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>Label</key>
    <string>com.hlcfan.whichkey</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
        <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
 	     <false/>
 	     <key>Crashed</key>
 	     <true/>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/srhd_sylvanfranklin.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/srhd_sylvanfranklin.err.log</string>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>Nice</key>
    <integer>-20</integer>
</dict>
</plist>",
        std::env::current_exe().unwrap().to_str().unwrap()
    );

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let plist_path = PathBuf::from(home_dir)
        .join("Library")
        .join("LaunchAgents")
        .join("config.hlcfan.whichkey.plist");

    fs::write(plist_path, plist)?;
    Ok(())
}

fn install_config() -> Result<(), Error> {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_file_path = PathBuf::from(home_dir)
        .join(".config")
        .join("whichkey")
        .join("config.toml");

    if config_file_path.as_path().exists() {
        return Err(Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Configuration already exists.",
        ));
    }

    let config = format!(
        "leader_key = \"option\"

[[groups]]
name = \"Open Applications\"

  [[groups.mappings]]
  keys = \"of\"
  kind = \"Application\"
  command = \"Finder\""
    );

    fs::write(config_file_path, config)?;
    Ok(())
}
