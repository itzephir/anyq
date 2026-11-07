use super::topic_command::TopicCommand;
use anyq_queue_api::{Error, Result, Topic};
use clap::Args;

#[derive(Args)]
#[command(
    after_help = "Examples:\n  anyq topic --list\n  anyq topic create cpu --capacity 2\n  anyq topic capacity cpu --capacity 3\n  anyq topic cpu add my-task"
)]
pub struct TopicArgs {
    #[arg(long, conflicts_with_all = ["capacity", "words"], help = "List topic names")]
    list: bool,
    #[arg(long, value_name = "N", help = "Capacity for topic create/capacity")]
    capacity: Option<u32>,
    #[arg(num_args = 1.., required_unless_present = "list", value_name = "COMMAND",
        help = "<TOPIC> add <TASK> | create <TOPIC> | capacity <TOPIC>")]
    words: Vec<String>,
}

impl TopicArgs {
    pub fn command(self) -> Result<TopicCommand> {
        if self.list {
            return Ok(TopicCommand::List);
        }

        // A resource may be named create/capacity; the three-word attachment form is distinct.
        if let [topic, verb, task] = self.words.as_slice() {
            if verb == "add" && self.capacity.is_none() {
                return Ok(TopicCommand::Add {
                    topic: topic.clone(),
                    task: task.clone(),
                });
            }
        }

        match self.words.as_slice() {
            [verb, name] if verb == "create" => {
                Ok(TopicCommand::Create(Topic::new(name.clone(), self.capacity.unwrap_or(1))?))
            }
            [verb, name] if verb == "capacity" => {
                let capacity = self.capacity
                    .ok_or_else(|| Error::Invalid("topic capacity requires --capacity N".into()))?;
                Ok(TopicCommand::Capacity(Topic::new(name.clone(), capacity)?))
            }
            _ => Err(Error::Invalid(
                "use topic --list, topic <TOPIC> add <TASK>, topic create <TOPIC> [--capacity N], or topic capacity <TOPIC> --capacity N".into()
            )),
        }
    }
}
