tell application "System Events"
    -- get the frontmost app process
    set frontApp to first process whose frontmost is true

    -- get its first window
    set theWindow to window 1 of frontApp

    -- get the screen size (assuming main display)
    tell application "Finder"
        set screenBounds to bounds of window of desktop
        -- screenBounds = {x, y, width, height}
    end tell

    set {x, y, screenWidth, screenHeight} to screenBounds

    -- move window to right half
    set position of theWindow to {x + (screenWidth / 2), y}
    set size of theWindow to {screenWidth / 2, screenHeight}
end tell

