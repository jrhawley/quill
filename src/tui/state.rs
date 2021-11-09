//! State of the TUI.

use tui::{
    layout::{Direction, Layout},
    widgets::{ListState, TableState},
};

use super::render::{step_next, step_prev, MenuItem};

/// The state of the "Missing" tab
pub struct MissingState {
    state: ListState,
}

impl MissingState {
    pub fn state(&self) -> &ListState {
        &self.state
    }

    pub fn mut_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn select_next(&mut self, len: usize) {
        if let Some(n) = self.selected() {
            self.state.select(Some(step_next(len, n)));
        }
    }

    pub fn select_prev(&mut self, len: usize) {
        if let Some(n) = self.selected() {
            self.state.select(Some(step_prev(len, n)));
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

impl Default for MissingState {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        MissingState { state }
    }
}

/// The state of the "Log" tab
#[derive(Default)]
pub struct LogState {
    accounts: ListState,
    log: ListState,
}

impl LogState {
    pub fn accounts(&self) -> &ListState {
        &self.accounts
    }

    pub fn mut_accounts(&mut self) -> &mut ListState {
        &mut self.accounts
    }

    pub fn select_account(&mut self, index: Option<usize>) {
        self.accounts.select(index);
    }

    pub fn select_next_account(&mut self, len: usize) {
        if let Some(n) = self.selected_account() {
            self.select_account(Some(step_next(len, n)));
        }
    }

    pub fn select_prev_account(&mut self, len: usize) {
        if let Some(n) = self.selected_account() {
            self.select_account(Some(step_prev(len, n)));
        }
    }

    pub fn selected_account(&self) -> Option<usize> {
        self.accounts.selected()
    }

    pub fn log(&self) -> &ListState {
        &self.log
    }

    pub fn mut_log(&mut self) -> &mut ListState {
        &mut self.log
    }

    pub fn select_log(&mut self, index: Option<usize>) {
        self.log.select(index);
    }

    pub fn select_next_log(&mut self, len: usize) {
        if let Some(n) = self.selected_log() {
            self.select_log(Some(step_next(len, n)));
        }
    }

    pub fn select_prev_log(&mut self, len: usize) {
        if let Some(n) = self.selected_log() {
            self.select_log(Some(step_prev(len, n)));
        }
    }

    pub fn selected_log(&self) -> Option<usize> {
        self.log.selected()
    }

    pub fn selected(&self) -> (Option<usize>, Option<usize>) {
        (self.selected_account(), self.selected_log())
    }
}

/// The state of the "Log" tab
#[derive(Default)]
pub struct AccountsState {
    state: TableState,
}

impl AccountsState {
    pub fn state(&self) -> &TableState {
        &self.state
    }

    pub fn mut_state(&mut self) -> &mut TableState {
        &mut self.state
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn select_next(&mut self, len: usize) {
        if let Some(n) = self.selected() {
            self.state.select(Some(step_next(len, n)));
        }
    }

    pub fn select_prev(&mut self, len: usize) {
        if let Some(n) = self.selected() {
            self.state.select(Some(step_prev(len, n)));
        }
    }
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

/// The state of the TUI
pub struct TuiState {
    active_menu_item: MenuItem,
    layout: Layout,
    missing: MissingState,
    log: LogState,
    accounts: AccountsState,
}

impl TuiState {
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn active_tab(&self) -> MenuItem {
        self.active_menu_item
    }

    pub fn set_active_tab(&mut self, tab: MenuItem) {
        self.active_menu_item = tab;
    }

    pub fn next_tab(&mut self) {
        self.active_menu_item.next();
    }

    pub fn prev_tab(&mut self) {
        self.active_menu_item.prev();
    }

    pub fn missing(&self) -> &MissingState {
        &self.missing
    }

    pub fn mut_missing(&mut self) -> &mut MissingState {
        &mut self.missing
    }

    pub fn log(&self) -> &LogState {
        &self.log
    }

    pub fn mut_log(&mut self) -> &mut LogState {
        &mut self.log
    }

    pub fn accounts(&self) -> &AccountsState {
        &self.accounts
    }

    pub fn mut_accounts(&mut self) -> &mut AccountsState {
        &mut self.accounts
    }
}

impl Default for TuiState {
    fn default() -> Self {
        TuiState {
            layout: Layout::default().direction(Direction::Vertical).margin(1),
            ..Default::default()
        }
    }
}
