## Libraries
### Scrapping

- network & concurrency
  - tokio (async runtime)
  - reqwest (async http)
- raw or rendered
  - chromiumoxide
  - > you may choose to skip this step if raw html is enough. otherwise render the js using chromiumoxide.
- html parsing
  - scraper (supports css selectors) OR tl (simpler, faster)
- data persistence
  - sqlx (async database library)
  - > (?) PostgreSQL is generally the best fit for crawlers because it natively handles high-concurrency INSERTs and UPSERTs well
  - > you may also consider sqlite (simpler?)
- api layer
  - axum, serde
  - > only if you wish to attach a rest api server to the crawler
- extras
  - tracing, tracing-subscriber (async logging)
  - governor (rate limiting, avoid DDOS detection)
  - robotstxt (self-explaining)
  - tower (Manages retries and timeouts)
  - readability (extracts the main content, strips everything else)
  - adblock (filter links that may come from ads)
  - anyhow/thiserror (error propagation?)
  - url (normalize urls, ex: example.com/#page to example.com)

### Visualization
- egui, egui_graphs

## Interface
- hierarchy
  - menu bar
    - about: short description of the project with a link to the github repo (alike the ones from linux gtk apps)
  - graph panel (cover most of the window)
  - retractable left/right panel (control or info)
- ideas
  - console with the backend output
  - some other graphs
    - visualizations from scrapping output and identified connections?
    - real time throughput rate of crawling (imagine the ones from aristocratos/btop)
