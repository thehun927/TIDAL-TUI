#![allow(dead_code)]

use rand::seq::SliceRandom;
use std::collections::VecDeque;

/// A single entry in the playback queue.
#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub track_id:   String,
    pub title:      String,
    pub artist:     String,
    pub album:      String,
    /// Duration in seconds (0 if unknown before playback).
    pub duration:   u32,
    /// Resolved stream URL ready to pass to MPV.
    pub stream_url: String,
}

/// The playback queue owned exclusively by the MPV player thread.
/// The main thread receives a cloned snapshot when it needs to display the queue (Phase 5).
#[derive(Debug, Default)]
pub struct PlaybackQueue {
    pub current:  Option<QueueEntry>,
    pub upcoming: VecDeque<QueueEntry>,
    pub history:  Vec<QueueEntry>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move the current track into history and pop the next track from upcoming.
    /// Returns a reference to the new current track, or `None` if the queue is empty.
    pub fn advance(&mut self) -> Option<&QueueEntry> {
        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        self.current = self.upcoming.pop_front();
        self.current.as_ref()
    }

    /// Step back: push current to the front of upcoming and restore the last history entry.
    /// Returns a reference to the restored track, or `None` if there is no history.
    pub fn step_back(&mut self) -> Option<&QueueEntry> {
        if let Some(prev) = self.history.pop() {
            if let Some(current) = self.current.take() {
                self.upcoming.push_front(current);
            }
            self.current = Some(prev);
        }
        self.current.as_ref()
    }

    /// Randomise the upcoming queue in-place.
    pub fn shuffle_upcoming<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        let mut items: Vec<_> = self.upcoming.drain(..).collect();
        items.shuffle(rng);
        self.upcoming.extend(items);
    }

    /// Restart the full playlist from the beginning for RepeatMode::All.
    /// History + current are converted back into current + upcoming in order.
    pub fn restart_cycle(&mut self) -> Option<&QueueEntry> {
        let mut all = Vec::new();
        all.extend(self.history.drain(..));
        if let Some(current) = self.current.take() {
            all.push(current);
        }
        if all.is_empty() {
            return None;
        }

        self.current = Some(all.remove(0));
        self.upcoming.extend(all);
        self.current.as_ref()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> QueueEntry {
        QueueEntry {
            track_id:   id.into(),
            title:      id.into(),
            artist:     String::new(),
            album:      String::new(),
            duration:   0,
            stream_url: String::new(),
        }
    }

    #[test]
    fn advance_moves_current_to_history() {
        let mut q = PlaybackQueue::new();
        q.current = Some(entry("a"));
        q.upcoming.push_back(entry("b"));
        q.advance();
        assert_eq!(q.current.as_ref().unwrap().track_id, "b");
        assert_eq!(q.history.len(), 1);
        assert_eq!(q.history[0].track_id, "a");
    }

    #[test]
    fn advance_on_empty_upcoming_returns_none() {
        let mut q = PlaybackQueue::new();
        q.current = Some(entry("a"));
        let result = q.advance();
        assert!(result.is_none());
        assert!(q.current.is_none());
        assert_eq!(q.history.len(), 1);
        assert_eq!(q.history[0].track_id, "a");
    }

    #[test]
    fn advance_with_no_current_pops_upcoming() {
        let mut q = PlaybackQueue::new();
        q.upcoming.push_back(entry("b"));
        q.upcoming.push_back(entry("c"));
        q.advance();
        assert_eq!(q.current.as_ref().unwrap().track_id, "b");
        assert_eq!(q.upcoming.len(), 1);
        assert!(q.history.is_empty());
    }

    #[test]
    fn step_back_restores_previous() {
        let mut q = PlaybackQueue::new();
        q.history.push(entry("a"));
        q.current = Some(entry("b"));
        q.step_back();
        assert_eq!(q.current.as_ref().unwrap().track_id, "a");
        assert_eq!(q.upcoming[0].track_id, "b");
        assert!(q.history.is_empty());
    }

    #[test]
    fn step_back_with_no_history_is_noop() {
        let mut q = PlaybackQueue::new();
        q.current = Some(entry("a"));
        let result = q.step_back();
        // No history -> current unchanged, nothing moved
        assert!(result.is_some());
        assert_eq!(q.current.as_ref().unwrap().track_id, "a");
        assert!(q.upcoming.is_empty());
    }

    #[test]
    fn restart_cycle_rebuilds_playlist() {
        let mut q = PlaybackQueue::new();
        q.history.push(entry("a"));
        q.history.push(entry("b"));
        q.current = Some(entry("c"));

        q.restart_cycle();

        assert_eq!(q.current.as_ref().unwrap().track_id, "a");
        assert_eq!(q.upcoming[0].track_id, "b");
        assert_eq!(q.upcoming[1].track_id, "c");
        assert!(q.history.is_empty());
    }

    #[test]
    fn shuffle_upcoming_preserves_length_and_membership() {
        let mut q = PlaybackQueue::new();
        q.upcoming.push_back(entry("a"));
        q.upcoming.push_back(entry("b"));
        q.upcoming.push_back(entry("c"));

        let before: std::collections::BTreeSet<_> = q.upcoming.iter().map(|e| e.track_id.clone()).collect();
        q.shuffle_upcoming(&mut rand::thread_rng());
        let after: std::collections::BTreeSet<_> = q.upcoming.iter().map(|e| e.track_id.clone()).collect();

        assert_eq!(q.upcoming.len(), 3);
        assert_eq!(before, after);
    }
}
