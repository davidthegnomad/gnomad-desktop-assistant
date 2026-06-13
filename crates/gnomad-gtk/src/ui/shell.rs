use libadwaita as adw;

use crate::state::AppState;
use crate::ui::chat_view::build_chat_shell;

pub fn build_shell(
    state: std::sync::Arc<AppState>,
    chat_rx: std::sync::mpsc::Receiver<crate::state::ChatEvent>,
) -> adw::ToolbarView {
    build_chat_shell(state, chat_rx)
}
