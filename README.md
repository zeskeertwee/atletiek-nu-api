# atletiek-nu-api

A Rust library and HTTP API for scraping [athletics.app](https://athletics.app) (previously [atletiek.nu](https://atletiek.nu)) data.

## Cloudflare Turnstile

The athletics.app website now uses **Cloudflare Turnstile** anti-bot protection on most web-facing pages. This blocks the traditional web scraping endpoints. The **mobile endpoints** (`athleteapp.php`) are **not affected** and should be preferred.

| Endpoint type | Blocked by Cloudflare | Recommendation |
|---------------|----------------------|----------------|
| Web (`/atleet/`, `/wedstrijd/`, `feeder.php`) | Yes | Avoid — will return challenge pages |
| Mobile (`athleteapp.php`) | No | Preferred — reliable and richer data |

## API endpoints

### Mobile endpoints (recommended)

These endpoints use the athletics.app mobile API and are **not blocked by Cloudflare**.

---

#### `GET /mobile/athletes/profile/<id>`

Returns an athlete's personal bests with full per-event performance history.

**Parameters:**
- `id` — athlete ID (`ranglijst_score_koppel_id`)

**Response includes:**
- Personal bests with event name, performance, wind speed, hand-measured flag, indoor flag, location, country code, date
- Per-event performance history with wind speed, location, country code, delta/progression

**Example response:**
```json
{
  "name": "John Doe",
  "personal_bests": [
    {
      "event": "100m",
      "performance": "10.85",
      "performance_value": 10.85,
      "wind_speed": 1.2,
      "hand_measured": false,
      "location": "Amsterdam",
      "country_code": "nl",
      "date": "15 Jun 2024",
      "not_important": false,
      "indoor": false,
      "history": [
        {
          "performance": "10.85",
          "performance_value": 10.85,
          "wind_speed": 1.2,
          "hand_measured": false,
          "location": "Amsterdam",
          "country_code": "nl",
          "date": "15 Jun 2024",
          "delta": "-0.03"
        }
      ]
    }
  ]
}
```

---

#### `GET /mobile/competitions/search?country=<country>&start=<date>&end=<date>&query=<text>`

Search for competitions using the mobile API.

**Parameters:**
- `country` — ISO2 country code (e.g. `NL`, `BE`, `FR`)
- `start` — start date (`YYYY-MM-DD`)
- `end` — end date (`YYYY-MM-DD`)
- `query` — *(optional)* search text

**Response includes:**
- Competition ID, name, location, date display
- Number of registrations
- Club-only flag, World Athletics recognized flag, cancelled flag

---

### Web endpoints (legacy)

These endpoints scrape the athletics.app web pages directly. **Most are now blocked by Cloudflare Turnstile** and will fail with an anti-bot challenge error.

---

#### `GET /competitions/search?start=<date>&end=<date>&query=<text>`

> ⚠️ **Blocked by Cloudflare** — use `/mobile/competitions/search` instead.

Search for competitions in a given time period.

**Parameters:**
- `start` — start date (`YYYY-MM-DD`)
- `end` — end date (`YYYY-MM-DD`)
- `query` — *(optional)* search text

---

#### `GET /competitions/registrations/<id>`

> ⚠️ **Blocked by Cloudflare** — no mobile alternative with equivalent data.

List all registrations for a competition.

**Parameters:**
- `id` — competition ID

---

#### `GET /competitions/results/<id>`

> ⚠️ **Blocked by Cloudflare** — no mobile alternative.

List all performances for a participant ID, alongside competitions and timetable.

**Parameters:**
- `id` — participant ID

---

#### `GET /athletes/search/<query>`

Search for athletes by name. Uses the mobile endpoint internally, so this still works.

**Parameters:**
- `query` — search string

---

#### `GET /athletes/profile/<id>`

> ⚠️ **Blocked by Cloudflare** — use `/mobile/athletes/profile/<id>` instead (with richer data).

Returns the athlete's profile with personal bests.

**Parameters:**
- `id` — athlete ID

---

## Data comparison: Web vs Mobile

| Data field | Web profile | Mobile profile |
|------------|:-----------:|:--------------:|
| Personal bests | ✅ | ✅ |
| Wind speed (PR) | ❌ | ✅ |
| Hand-measured flag | ❌ | ✅ |
| Indoor flag | ❌ | ✅ |
| Country code | ❌ | ✅ |
| Performance history | ⚠️ Float + date only | ✅ Full details |
| History: wind speed | ❌ | ✅ |
| History: location | ❌ | ✅ |
| History: country | ❌ | ✅ |
| History: progression delta | ❌ | ✅ |

## Building

### Library

```bash
cargo build --release -p atletiek_nu_api
```

### HTTP API server

```bash
cargo build --release --bin api
```

A pre-built binary is also available from [GitHub Releases](https://github.com/zeskeertwee/atletiek-nu-api/releases).

## Cloudflare worker API

A hosted version is available at `https://atnapi.juandomingo.net` using Cloudflare Workers. See [api-cfworker/README.md](./api-cfworker/README.md) for details.

## Note

The scraper still has bugs and may not handle all pages. Please create an issue if you encounter a problem.
