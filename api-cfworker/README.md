# Cloudflare worker API endpoints
`GET /competitions/results/[id]`
---
`id`: a valid participant ID

Returns all performances for the participant ID, alongside all competitions they participated in, and the timetable for the competition (for the specific athlete).

`GET /competitions/registrations/[id]`
---
`id`: a valid competition ID

Returns all registrations for the given competition. Uses the new `get_competition_registrations_web` function.

`GET /v1/competitions/registrations/[id]`
---
`id`: a valid competition ID

Returns all registrations using the deprecated `get_competition_registrations` function

`GET /competitions/search?[start]&[end]&[query]`
---
`start`: the start date of the timeframe

`end`: the end date of the timeframe

`query`: *optional* search query

Returns all competitions in the given timeframe matching the query, if given.

`GET /athletes/search/[query]`
---
`query`: the search string

Returns a list of all athletes with a name matching `query`

`GET /athletes/profile/[id]`
---
`id`: a valid athlete ID

Returns information on the athlete's profile
