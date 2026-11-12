use crate::PluginTimer;

pub fn due_timers(timers: &[PluginTimer], now: i64, enabled: bool) -> Vec<PluginTimer> {
    let mut due = timers
        .iter()
        .filter(|timer| enabled && timer.deadline <= now)
        .cloned()
        .collect::<Vec<_>>();
    due.sort_by(|left, right| (left.deadline, &left.id).cmp(&(right.deadline, &right.id)));

    due
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_input_selects_due_timers_without_resetting_deadlines_during_downtime_or_disable() {
        let timer = PluginTimer {
            id: "stable".into(),
            deadline: 100,
            origin_sequence: 1,
            payload: Default::default(),
        };
        let timers = vec![timer.clone()];

        assert!(due_timers(&timers, 99, true).is_empty());
        assert!(due_timers(&timers, 1000, false).is_empty());
        assert_eq!(due_timers(&timers, 1000, true), vec![timer]);
        assert_eq!(timers[0].deadline, 100);
        assert_eq!(crate::retry_delay(1), 250);
        assert_eq!(crate::retry_delay(u32::MAX), 60_000);
    }
}
