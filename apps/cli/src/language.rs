use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub enum Language {
    #[value(name = "en")]
    English,
    #[value(name = "ru")]
    Russian,
}
