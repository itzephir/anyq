use crate::{
    Error, Metadata, MetadataValue, ResourceAvailability, Result, StartOutcome, StartPreference,
    TaskState, Topic, parse_name, parse_topic_name,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    sequence: i64,
    name: String,
    state: TaskState,
    topics: Vec<String>,
    autostart_default: bool,
    stopped: bool,
    metadata: Metadata,
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
        topics: Vec<String>,
    ) -> Result<Self> {
        Self::restore_with_metadata(sequence, name, state, topics, Metadata::default())
    }

    pub fn restore_with_metadata(
        sequence: i64,
        name: String,
        state: TaskState,
        mut topics: Vec<String>,
        metadata: Metadata,
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
            autostart_default: true,
            stopped: false,
            metadata,
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

    pub fn autostart_default(&self) -> bool {
        self.autostart_default
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn require_metadata_editable(&self) -> Result<()> {
        if matches!(self.state, TaskState::Running | TaskState::Completed) {
            return Err(Error::Conflict(format!(
                "metadata of '{}' cannot change while {}",
                self.name,
                self.state.as_str()
            )));
        }

        Ok(())
    }

    pub fn set_metadata_key(&mut self, key: &str, value: MetadataValue) -> Result<()> {
        self.require_metadata_editable()?;

        self.metadata.set(key, value)
    }

    pub fn unset_metadata_key(&mut self, key: &str) -> Result<()> {
        self.require_metadata_editable()?;

        self.metadata.unset(key)
    }

    pub fn replace_metadata(&mut self, metadata: Metadata) -> Result<()> {
        self.require_metadata_editable()?;
        self.metadata = metadata;

        Ok(())
    }

    pub fn with_start_policy(mut self, default: bool, stopped: bool) -> Self {
        self.autostart_default = default;
        self.stopped = stopped;

        self
    }

    pub fn permits_autostart(&self, topics: &[Topic]) -> bool {
        !self.stopped
            && !self.topics.is_empty()
            && topics.len() == self.topics.len()
            && self
                .topics
                .iter()
                .all(|name| topics.iter().any(|topic| &topic.name == name))
            && self.autostart_setting(topics)
    }

    fn autostart_setting(&self, topics: &[Topic]) -> bool {
        if topics.is_empty() {
            return self.autostart_default;
        }

        topics
            .iter()
            .all(|topic| topic.autostart.unwrap_or(self.autostart_default))
    }

    /// Apply a creation preference; true requests an immediate explicit start.
    pub fn configure_start(
        &mut self,
        preference: StartPreference,
        topics: &[Topic],
    ) -> Result<bool> {
        let automatic = self.autostart_setting(topics);
        let stopped = match preference {
            StartPreference::Stop => true,
            StartPreference::Toggle => automatic,
            StartPreference::Default | StartPreference::Start => false,
        };
        let explicit =
            matches!(preference, StartPreference::Start | StartPreference::Toggle) && !automatic;

        if explicit {
            self.require_queued()?;
        }

        self.stopped = stopped;

        Ok(explicit)
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

    pub fn check_running(&self) -> Result<bool> {
        if self.state == TaskState::Running {
            return Ok(true);
        }

        self.require_queued()?;

        Ok(false)
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
        self.stopped = false;

        Ok(StartOutcome::Started)
    }

    pub fn require_completion_permit(
        &self,
        states: impl IntoIterator<Item = crate::ManagedWorkState>,
    ) -> Result<()> {
        if states
            .into_iter()
            .any(crate::ManagedWorkState::blocks_completion)
        {
            return Err(Error::ManagedWorkPending);
        }

        Ok(())
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
