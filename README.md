# WhichKey

A macOS key binding utility that listens to key presses and executes commands based on configuration. Define a leader key and create custom key sequences to launch applications, run scripts, or perform system actions.

## Features

- **Leader Key**: Support `option`, `control`, `command`, `shift` keys
- **Custom Key Sequences**: Create multi-key combinations for different actions
- **Application Launching**: Quick access to your favorite applications
- **Run commands**: Run commands

## Requirements

- macOS
- Accessibility permissions

## Installation

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/hlcfan/whichkey
   cd whichkey
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the service and configuration:
   ```bash
   ./target/release/whichkey install
   ```

### Accessibility Permissions

WhichKey requires accessibility permissions to monitor global key events:

1. Go to **System Settings** → **Privacy & Security** → **Accessibility**
2. Add the WhichKey application to the list of allowed applications
3. Ensure the toggle is enabled

## Configuration

WhichKey uses a TOML configuration file located at `~/.config/whichkey/config.toml`.

### Sample Configuration

```toml
leader_key = "option"

[[groups]]
name = "Open Applications"

  [[groups.mappings]]
  keys = "oc"
  kind = "Application"
  command = "Google Chrome"

  [[groups.mappings]]
  keys = "ovs"
  kind = "Application"
  command = "Visual Studio Code"

  [[groups.mappings]]
  keys = "of"
  kind = "Application"
  command = "Finder"

[[groups]]
name = "Run commands"

  [[groups.mappings]]
  keys = "rs"
  kind = "Command"
  command = """osascript -e 'tell application "Safari"
    open location "https://apple.com"
    activate
end tell'"""

```

### Configuration Options

#### Leader Key
The `leader_key` can be set to any of the following modifier keys:
- `"option"` 
- `"contrl"`
- `"shift"`
- `"command"`

#### Groups and Mappings
- **Groups**: Organize your key bindings into logical groups
- **Mappings**: Define individual key sequences and their actions
  - `keys`: The key sequence after the leader key (e.g., "oc" for option+o+c)
  - `kind`: The type of action (currently supports "Application" and "Command")
  - `command`: The command to execute (application name for "Application" kind)

## Usage

### Starting WhichKey

```bash
./target/release/whichkey start
```

### Using Key Bindings

1. Press leader key (e.g., Option, Control, Command, Shift)
2. Type the key sequence (e.g., "of" for Finder)
4. The configured action will execute

### Example Usage

- `Option + o + f` → Opens Finder
- `Option + o + c` → Opens Google Chrome
- `Option + o + vs` → Opens Visual Studio Code

## Key Sequence Timing

Current key sequences must be completed within 1000ms (1 second) of each other. If you pause too long between keys, the sequence will reset.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## Inspiration

This project is inspired by <mcreference link="https://github.com/folke/which-key.nvim" index="0">which-key.nvim</mcreference>, bringing similar key binding functionality to the macOS desktop environment.
