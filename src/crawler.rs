struct CrawlRequest {}

struct CrawlerEngine {}

type CrawlID = u64;

enum EngineCommand {
    StartCrawl(CrawlRequest),
    StopCrawl,
}

impl CrawlerEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_command(command: EngineCommand) {}
}
