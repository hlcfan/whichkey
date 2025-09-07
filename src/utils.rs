use std::process::Command;

// Helper function to get App Name from PID (simplified)
// A full implementation might use NSWorkspace or other methods
pub fn get_app_name_from_pid(pid: i32) -> String {
    // Use ps command for a simple lookup
    let output = Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("comm=")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => format!("PID_{}", pid), // Fallback to PID
    }
}

pub fn open_accessibility_preferences() {
    // This will open the accessibility section of System Settings
    match Command::new("open")
        .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"])
        .spawn() {
            Ok(_) => println!("Opened accessibility preferences"),
            Err(e) => println!("Failed to open accessibility preferences: {}", e)
        }
} 