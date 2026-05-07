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
  - > consider trunk for a wasm view
- egui_html, egui_commonmark
  - > displaying the scrapped pages may also be useful, consider those libraries (or other existing options)
  - suggestion: convert the pages to markdown and render it

## Interface
- hierarchy
  - menu bar
    - file: save graph, load graph, export data (json/csv), Quit?
    - edit: clear state
    - view: dark/light mode?
    - settigs: (to define)
    - about: short description of the project with a link to the github repo (alike the ones from linux gtk apps)
  - graph panel (cover most of the window)
    - tabs, each with it's own graph session
    - what we must have additionally: layout choice and their related configuration values; navigation (fit_to_screen, zoom_and_pan); dark/light mode
    - use defaults (or code configured) options for the other simulation values
    - node labels and icons may be considered. but they are not available by default
    - test the second layout with a real web graph. it may not look good, so it's less trouble having one single layout instead of both.
  - left panel
    - > comand and control
    - seed url
    - pause/stop
  - floating panel
    - > inspector
    - displays info about a certain node
  - (optional?) bottom log panel
    - alike a terminal output (coming from the backend)
- ideas
  - console with the backend output
  - some other graphs
    - visualizations from scrapping output and identified connections?
    - real time throughput rate of crawling (imagine the ones from aristocratos/btop)
