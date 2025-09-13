tell application "System Events"
    -- get the frontmost app process
    set frontApp to first process whose frontmost is true
    set frontWindow to window 1 of frontApp

    -- get the screen size (main display)
    tell application "Finder"
        set screenBounds to bounds of window of desktop
    end tell

    set {x, y, screenWidth, screenHeight} to screenBounds

    -- maximize window
    set position of frontWindow to {x, y}
    set size of frontWindow to {screenWidth, screenHeight}
end tell

