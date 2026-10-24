use crate::app::state::Screen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Back,
    ToggleFocus,
    Goto(Screen),
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Left,
    Right,
    /// Enter: the screen's primary action.
    Activate,
    /// Space: toggle (Module Builder).
    Toggle,
    MoveItem { up: bool },
    Apply,
    ToggleDemo,
    CycleLogoSize,
    RunFastfetch,
    Refresh,
    Delete,
    Click { col: u16, row: u16 },
    ScrollUp,
    ScrollDown,
    Resize,
    /// Periodic: collect background results.
    Tick,
}
