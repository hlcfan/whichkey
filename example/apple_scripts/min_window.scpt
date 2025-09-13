tell application "System Events"
    -- get the frontmost app process
    set frontApp to first process whose frontmost is true
    set frontWindow to window 1 of frontApp

    -- minimize it
    set value of attribute "AXMinimized" of frontWindow to true
end tell

