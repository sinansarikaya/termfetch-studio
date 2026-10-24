use crate::app::state::Screen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Back,
    /// `Tab`: next focus region.
    ToggleFocus,
    /// `Shift+Tab`: previous focus region.
    FocusPrev,
    Goto(Screen),
    /// Digit key: quick action on the Dashboard stage, otherwise jump to a screen.
    Number(u8),
    ToggleHelp,
    /// Cycle what the live preview box shows.
    CycleOutput,
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
