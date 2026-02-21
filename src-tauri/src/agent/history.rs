use crate::ai::types::{ContentBlock, Message};

const MAX_SCREENSHOT_MESSAGES: usize = 5;

/// Manage conversation history with a sliding window for screenshots.
/// Keeps the first message (user instruction) and trims old screenshot-heavy messages.
pub fn trim_history(messages: &mut Vec<Message>) {
    if messages.len() <= 3 {
        return;
    }

    // Count messages containing screenshots
    let screenshot_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(i, msg)| {
            *i > 0
                && msg.content.iter().any(|block| match block {
                    ContentBlock::Image { .. } => true,
                    _ => false,
                })
        })
        .map(|(i, _)| i)
        .collect();

    if screenshot_indices.len() > MAX_SCREENSHOT_MESSAGES {
        let to_remove = screenshot_indices.len() - MAX_SCREENSHOT_MESSAGES;
        // Remove the oldest screenshot messages (but keep the first user message)
        let indices_to_remove: Vec<usize> = screenshot_indices[..to_remove].to_vec();
        // Remove from back to front to preserve indices
        for &idx in indices_to_remove.iter().rev() {
            if idx > 0 && idx < messages.len() {
                // Strip image content from this message instead of removing it entirely
                // This preserves tool results / text context
                messages[idx].content.retain(|block| {
                    !matches!(block, ContentBlock::Image { .. })
                });
            }
        }
    }
}
