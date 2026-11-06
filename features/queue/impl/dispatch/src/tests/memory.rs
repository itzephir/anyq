use crate::ports::*;
use anyq_queue_api::*;

#[derive(Clone, Default)]
pub struct Memory {
    pub topics: Vec<Topic>,
    pub groups: Vec<ConsumerGroup>,
    pub messages: Vec<Message>,
    pub deliveries: Vec<Delivery>,
    pub fail_save: bool,
}

impl Memory {
    fn select(&self, f: &DeliveryFilter) -> Vec<Delivery> {
        self.deliveries
            .iter()
            .filter(|d| {
                let m = self.messages.iter().find(|m| m.id == d.message_id).unwrap();
                f.topic.as_ref().is_none_or(|t| t == &m.topic)
                    && f.group.as_ref().is_none_or(|g| g == &d.group)
                    && f.message_id.is_none_or(|id| id == d.message_id)
                    && f.state.is_none_or(|s| s == d.state)
                    && f.ready_at.is_none_or(|t| d.available_at <= t)
                    && f.expired_at
                        .is_none_or(|t| d.lease_until.is_some_and(|u| u <= t))
            })
            .cloned()
            .collect()
    }
}

impl TopicRecords for Memory {
    fn topics(&mut self) -> Result<Vec<Topic>> {
        Ok(self.topics.clone())
    }

    fn topic(&mut self, name: &str) -> Result<Option<Topic>> {
        Ok(self.topics.iter().find(|t| t.name == name).cloned())
    }

    fn put_topic(&mut self, t: &Topic) -> Result<()> {
        self.topics.retain(|x| x.name != t.name);
        self.topics.push(t.clone());

        Ok(())
    }
}

impl GroupRecords for Memory {
    fn groups(&mut self, topic: &str) -> Result<Vec<ConsumerGroup>> {
        Ok(self
            .groups
            .iter()
            .filter(|g| g.topic == topic)
            .cloned()
            .collect())
    }

    fn insert_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        self.groups.push(g.clone());

        Ok(())
    }
}

impl MessageRecords for Memory {
    fn message(&mut self, id: i64) -> Result<Option<Message>> {
        Ok(self.messages.iter().find(|m| m.id == id).cloned())
    }

    fn message_by_key(&mut self, topic: &str, key: &str) -> Result<Option<Message>> {
        Ok(self
            .messages
            .iter()
            .find(|m| m.topic == topic && m.key.as_deref() == Some(key))
            .cloned())
    }

    fn insert_message(&mut self, p: &Publish, now: i64) -> Result<Message> {
        let m = Message {
            id: self.messages.len() as i64 + 1,
            topic: p.topic.clone(),
            payload: p.payload.clone(),
            key: p.key.clone(),
            max_attempts: p.max_attempts,
            created_at: now,
        };

        self.messages.push(m.clone());

        Ok(m)
    }

    fn messages(&mut self, topic: Option<&str>, after: i64, limit: u32) -> Result<Vec<Message>> {
        Ok(self
            .messages
            .iter()
            .filter(|m| m.id > after && topic.is_none_or(|t| t == m.topic))
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

impl DeliveryRecords for Memory {
    fn seed_group(&mut self, g: &ConsumerGroup) -> Result<()> {
        for m in self.messages.clone().iter().filter(|m| m.topic == g.topic) {
            self.put_delivery(&Delivery::pending(m, g.name.clone()))?;
        }

        Ok(())
    }

    fn deliveries(&mut self, f: &DeliveryFilter) -> Result<Vec<Delivery>> {
        let mut ds = self.select(f);
        ds.sort_by(|a, b| (a.message_id, &a.group).cmp(&(b.message_id, &b.group)));

        if let Some(limit) = f.limit {
            ds.truncate(limit as usize);
        }

        Ok(ds)
    }

    fn count_deliveries(&mut self, f: &DeliveryFilter) -> Result<u64> {
        Ok(self.select(f).len() as u64)
    }

    fn put_delivery(&mut self, d: &Delivery) -> Result<()> {
        if self.fail_save {
            return Err(Error::Storage("injected failure".into()));
        }

        self.deliveries
            .retain(|x| !(x.message_id == d.message_id && x.group == d.group));
        self.deliveries.push(d.clone());

        Ok(())
    }
}

impl UnitOfWork for Memory {
    fn topics(&mut self) -> &mut dyn TopicRecords {
        self
    }

    fn groups(&mut self) -> &mut dyn GroupRecords {
        self
    }

    fn messages(&mut self) -> &mut dyn MessageRecords {
        self
    }

    fn deliveries(&mut self) -> &mut dyn DeliveryRecords {
        self
    }
}
