use crate::ConnectionState;
use anyq_queue_api::QueueSnapshot;

#[derive(Debug, Clone, Default)]
pub struct PanelModel {
    pub generation: u64,
    pub status: ConnectionState,
    pub snapshot: Option<QueueSnapshot>,
    pub error: Option<String>,
}

impl PanelModel {
    pub fn change_source(&mut self) -> u64 {
        self.generation += 1;
        self.snapshot = None;
        self.error = None;
        self.status = ConnectionState::Loading;

        self.generation
    }

    pub fn accept(&mut self, generation: u64, snapshot: QueueSnapshot) -> bool {
        if generation != self.generation {
            return false;
        }

        self.snapshot = Some(snapshot);
        self.error = None;
        self.status = ConnectionState::Live;

        true
    }

    pub fn fail(&mut self, generation: u64, error: String) -> bool {
        if generation != self.generation {
            return false;
        }

        self.error = Some(error);
        self.status = if self.snapshot.is_some() {
            ConnectionState::Stale
        } else {
            ConnectionState::Error
        };

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_sources_discards_late_success_and_failure_without_retaining_old_rows() {
        let mut model = PanelModel::default();
        let a = model.change_source();
        assert!(model.accept(a, QueueSnapshot::new(vec![], vec![]).unwrap()));

        let b = model.change_source();
        let late_success = model.accept(a, QueueSnapshot::new(vec![], vec![]).unwrap());
        let late_failure = model.fail(a, "old failure".into());

        assert!(!late_success);
        assert!(!late_failure);
        assert_eq!(model.generation, b);
        assert_eq!(model.status, ConnectionState::Loading);
        assert!(model.snapshot.is_none());
        assert!(model.error.is_none());
    }

    #[test]
    fn failed_refresh_marks_same_source_stale_and_success_clears_it() {
        let mut model = PanelModel::default();
        let generation = model.change_source();
        model.accept(generation, QueueSnapshot::new(vec![], vec![]).unwrap());

        model.fail(generation, "disconnected".into());

        assert_eq!(model.status, ConnectionState::Stale);
        assert!(model.snapshot.is_some());

        model.accept(generation, QueueSnapshot::new(vec![], vec![]).unwrap());

        assert_eq!(model.status, ConnectionState::Live);
        assert!(model.error.is_none());
    }
}
