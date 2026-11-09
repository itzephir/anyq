use crate::{
    Error, ResourceAvailability, Result, StartOutcome, TaskState, Topic, parse_name,
    parse_topic_name,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    sequence: i64,
    name: String,
    state: TaskState,
    topics: Vec<String>,
}

impl Task {
    pub fn new(sequence: i64, name: String) -> Result<Self> {
        Self::restore(
            sequence,
            parse_name(&name)?,
            TaskState::Registered,
            Vec::new(),
        )
    }

    /// Reconstruct canonical stored names without interpreting backticks again.
    pub fn restore(
        sequence: i64,
        name: String,
        state: TaskState,
        mut topics: Vec<String>,
    ) -> Result<Self> {
        if sequence <= 0 {
            return Err(Error::Invalid("task sequence must be positive".into()));
        }

        topics.sort();
        let unique = topics.windows(2).all(|pair| pair[0] != pair[1]);
        let valid_state = match state {
            TaskState::Registered => topics.is_empty(),
            TaskState::Queued | TaskState::Running => !topics.is_empty(),
            TaskState::Completed => true,
        };

        if !unique || !valid_state {
            return Err(Error::Invalid(
                "task state and topic membership disagree".into(),
            ));
        }

        Ok(Self {
            sequence,
            name,
            state,
            topics,
        })
    }

    pub fn sequence(&self) -> i64 {
        self.sequence
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn topics(&self) -> &[String] {
        &self.topics
    }

    pub fn add_topic(&mut self, topic: &str) -> Result<()> {
        self.attach_topic(parse_topic_name(topic)?)
    }

    pub fn link_topic(&mut self, topic: &Topic) -> Result<()> {
        self.attach_topic(topic.name.clone())
    }

    fn attach_topic(&mut self, topic: String) -> Result<()> {
        if matches!(self.state, TaskState::Running | TaskState::Completed) {
            return Err(Error::Conflict(format!(
                "'{}' is already {}",
                self.name,
                self.state.as_str()
            )));
        }

        if !self.topics.contains(&topic) {
            self.topics.push(topic);
            self.topics.sort();
        }

        self.state = TaskState::Queued;

        Ok(())
    }

    pub fn require_queued(&self) -> Result<()> {
        match self.state {
            TaskState::Registered => Err(Error::Conflict(format!(
                "'{}' is not registered in any topic",
                self.name
            ))),
            TaskState::Queued => Ok(()),
            state => Err(Error::Conflict(format!(
                "'{}' is already {}",
                self.name,
                state.as_str()
            ))),
        }
    }

    pub fn try_start(&mut self, resources: &[ResourceAvailability]) -> Result<StartOutcome> {
        self.require_queued()?;

        let expected: BTreeSet<_> = self.topics.iter().map(String::as_str).collect();
        let supplied: BTreeSet<_> = resources
            .iter()
            .map(|resource| resource.topic.name.as_str())
            .collect();

        if resources.len() != self.topics.len() || expected != supplied {
            return Err(Error::Invalid(
                "admission requires exactly one snapshot of every task topic".into(),
            ));
        }

        if !resources.iter().all(ResourceAvailability::permits_start) {
            return Ok(StartOutcome::Busy);
        }

        self.state = TaskState::Running;

        Ok(StartOutcome::Started)
    }

    pub fn complete(&mut self, force: bool) -> Result<()> {
        if self.state == TaskState::Completed {
            return Err(Error::Conflict(format!(
                "'{}' is already completed",
                self.name
            )));
        }

        if self.state != TaskState::Running && !force {
            return Err(Error::Conflict(format!("'{}' was not run", self.name)));
        }

        self.state = TaskState::Completed;

        Ok(())
    }
}
