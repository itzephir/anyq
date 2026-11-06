mod controllers;

mod presenters;

use anyq_queue_api::{
    CompleteDeliveryRequest, ConsumerGroup, Error, MessageQuery, Publish, RejectDeliveryRequest,
    Result, Topic,
};

use anyq_worker_api::Run;
use clap::Parser;
use controllers::{Cli, Commands};
use std::{io::Write, path::PathBuf, process::ExitCode};

fn emit(value: serde_json::Value) -> Result<()> {
    writeln!(std::io::stdout().lock(), "{value}").map_err(|e| Error::Storage(e.to_string()))
}

fn default_db() -> Result<PathBuf> {
    #[cfg(windows)]
    let data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| Error::Invalid("set ANYQ_DB or --db: LOCALAPPDATA is unavailable".into()))?;
    #[cfg(not(windows))]
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
        .ok_or_else(|| {
            Error::Invalid("set ANYQ_DB or --db: HOME and XDG_DATA_HOME are unavailable".into())
        })?;

    Ok(data.join("anyq/queue.db"))
}

fn run(cli: Cli) -> Result<u8> {
    let path = match cli.db {
        Some(p) => p,
        None => default_db()?,
    };

    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| Error::Storage(e.to_string()))?;
    }

    let mut cases = anyq_queue_impl::open(&path)?;

    match cli.command {
        Commands::Worker(args) => {
            let (settings, command, once) = args.domain()?;
            let dependencies = anyq_worker_impl::WorkerDependencies {
                claim: cases.claim_delivery,
                renew: cases.renew_lease,
                complete: cases.complete_delivery,
                reject: cases.reject_delivery,
                topics: cases.get_topic,
            };

            let mut worker = anyq_worker_impl::create(dependencies, command, settings)?;

            loop {
                let result = worker.run_one()?;

                if once || result != Run::Idle {
                    emit(presenters::run(result))?;
                }

                if result == Run::Stopped || worker.stopped() {
                    return Ok(130);
                }

                if once {
                    return Ok(if matches!(result, Run::Failed(_)) {
                        1
                    } else {
                        0
                    });
                }

                if result == Run::Idle {
                    worker.idle();
                }
            }
        }

        command => {
            emit(route(command, &mut cases)?)?;

            Ok(0)
        }
    }
}

fn route(command: Commands, cases: &mut anyq_queue_impl::QueueModule) -> Result<serde_json::Value> {
    use controllers::json::json;
    use controllers::{Groups, Jobs, Topics};
    use serde_json::json as value;

    Ok(match command {
        Commands::Topic(Topics::Create { name, capacity }) => {
            presenters::topic(&cases.create_topic.create(Topic::new(name, capacity)?)?)
        }

        Commands::Topic(Topics::Capacity { name, capacity }) => presenters::topic(
            &cases
                .set_topic_capacity
                .set_capacity(Topic::new(name, capacity)?)?,
        ),
        Commands::Topic(Topics::List) => value!(
            cases
                .list_topics
                .list()?
                .iter()
                .map(presenters::topic)
                .collect::<Vec<_>>()
        ),
        Commands::Group(Groups::Create { topic, name }) => presenters::group(
            &cases
                .create_consumer_group
                .create(ConsumerGroup { topic, name })?,
        ),
        Commands::Group(Groups::List { topic }) => value!(
            cases
                .list_consumer_groups
                .list(&topic)?
                .iter()
                .map(presenters::group)
                .collect::<Vec<_>>()
        ),
        Commands::Publish {
            topic,
            data,
            key,
            max_attempts,
        } => presenters::message(&cases.publish_message.publish(Publish {
            topic,
            payload: data.read()?,
            key,
            max_attempts,
        })?),
        Commands::Claim(args) => {
            presenters::assignment(&cases.claim_delivery.claim(args.domain())?)
        }

        Commands::Ack {
            lease,
            result,
            next,
        } => {
            presenters::completion(&cases.complete_delivery.complete(CompleteDeliveryRequest {
                lease: lease.domain(),
                result: result.map(|v| json(&v)).transpose()?,
                next: next.domain()?,
            })?)
        }

        Commands::Nack {
            lease,
            reason,
            delay_ms,
        } => presenters::delivery(&cases.reject_delivery.reject(RejectDeliveryRequest {
            lease: lease.domain(),
            reason,
            delay_ms,
        })?),
        Commands::Renew { lease, lease_ms } => {
            presenters::delivery(&cases.renew_lease.renew(lease.domain(), lease_ms)?)
        }

        Commands::Retry { message_id, group } => {
            presenters::delivery(&cases.retry_delivery.retry(message_id, group)?)
        }

        Commands::Jobs(Jobs::Show { message_id }) => {
            presenters::view(&cases.inspect_message.inspect(message_id)?)
        }

        Commands::Jobs(Jobs::List {
            topic,
            group,
            after,
            limit,
        }) => value!(
            cases
                .list_messages
                .list(MessageQuery {
                    topic,
                    group,
                    after,
                    limit
                })?
                .iter()
                .map(presenters::view)
                .collect::<Vec<_>>()
        ),
        Commands::Worker(_) => {
            return Err(Error::Invalid(
                "worker is assembled separately by main".into(),
            ));
        }
    })
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{}", presenters::error(&error));
            ExitCode::from(2)
        }
    }
}
