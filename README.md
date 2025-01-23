# atletiek-nu-api
A work-in-progress attempt at scraping the [athletics.app](https://athletics.app) (previously [atletiek.nu](https://atletiek.nu) but they changed their domain) website for data. Kinda works but far from complete.

What it can do as of now:
- Search for competitions
- List registrations for a competition (including registration status such as accepted, rejected, etc.)
- List results for an athlete for a given competition
- Search athletes and list their profile with PB's, a list of all preformances in a specific category, and all competitions they participated in
- List competitions for a given time period

**Note:** The scraper still has many bugs and will not be able to scrape all pages. Please create an issue if you encounter a bug.

# HTTP API
An api is hosted at `https://atnapi.juandomingo.net` using cloudflare workers.
[Documentation](./api-cfworker/README.md)

# Local HTTP API
There is also a HTTP api availible for download from the releases on [github.com](https://github.com/zeskeertwee/atletiek-nu-api/releases)
Or, alternatively, you can compile the HTTP api from scratch after cloning the repository like so: `cargo build --release --bin api`
