use crate::ui::scroll_handle::ESTIMATED_ACTION_ITEM_HEIGHT;
use crate::ui::scroll_handle::ScrollHandle;

pub enum MainViewState {
    None,
    SearchResultActionPanel {
        // ephemeral state
        focused_action_item: ScrollHandle,
    },
    InlineViewActionPanel {
        // ephemeral state
        focused_action_item: ScrollHandle,
    },
}

impl MainViewState {
    pub fn new() -> Self {
        MainViewState::None
    }

    pub fn initial(prev_state: &mut MainViewState) {
        *prev_state = Self::None
    }

    pub fn search_result_action_panel(prev_state: &mut MainViewState, focus_first: bool) {
        *prev_state = Self::SearchResultActionPanel {
            focused_action_item: ScrollHandle::new(focus_first, ESTIMATED_ACTION_ITEM_HEIGHT, 7),
        }
    }

    pub fn inline_result_action_panel(prev_state: &mut MainViewState, focus_first: bool) {
        *prev_state = Self::InlineViewActionPanel {
            focused_action_item: ScrollHandle::new(focus_first, ESTIMATED_ACTION_ITEM_HEIGHT, 7),
        }
    }
}
