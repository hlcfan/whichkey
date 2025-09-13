tell application "System Events"
    -- get the list of processes that have visible windows
    set visibleApps to (every process whose visible is true and frontmost is false)

    -- frontmost app
    set frontApp to first process whose frontmost is true
    set frontWindow to window 1 of frontApp

    -- recent app just behind frontmost
    set recentApp to first item of visibleApps
    set recentWindow to window 1 of recentApp

    -- get the screen size (main display)
    tell application "Finder"
        set screenBounds to bounds of window of desktop
    end tell

    set {x, y, screenWidth, screenHeight} to screenBounds

    -- arrange windows
    set position of frontWindow to {x, y}
    set size of frontWindow to {screenWidth / 2, screenHeight}

    set position of recentWindow to {x + (screenWidth / 2), y}
    set size of recentWindow to {screenWidth / 2, screenHeight}
end tell

