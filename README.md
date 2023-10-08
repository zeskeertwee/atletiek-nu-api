# atletiek-nu-api
A work-in-progress attempt at scraping the [atletiek.nu](https://atletiek.nu) website for data. Kinda works but far from complete.

What it can do as of v0.2.0:
- Search for competitions
- List registrations for a competition
- List results for an athlete for a given competition
- Search athletes (not implemented in the HTTP api), cannot retrieve information about an athlete like PB's or competitions yet
- List competitions for a given time period

# HTTP API
There is also a HTTP api availible for download from the releases on [github.com](https://github.com/zeskeertwee/atletiek-nu-api/releases)

Or, alternatively, you can compile the HTTP api from scratch after cloning the repository like so: `cargo build --release --bin api`
