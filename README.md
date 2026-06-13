# Mainmatter Rust Assessment

> Please take no more than ~1 hour to complete this assignment. We’ll discuss your solutions in a call in a few days. Feel free to ask questions if the assignment is unclear at any point.

## Context

This repository contains a simple file-icon lookup library as may be used by IDEs for their project tree sidebar. `get_icon_for_file` and `get_icon_for_folder` accept a filepath and return the numeric ID of a corresponding icon if one is found. Each icon ID corresponds to an svg icon in `./icons`.

## Assignment

1. Implement a web server than allows users to query for file/folder icons over the network. 
2. Write a short (<= 1 page) design/implementation note describing:
  - Design choices and tradeoffs you made (high-level architecture, web server choice, libraries you picked etc.)
  - How you would productionize this service (deployment, observability, etc.)
  
Keep your audience in mind. We’re not interested in a perfect solution but one where we see your thought-process and working style in action.
  
You may add these notes at the end of this file if you wish.

## Notes

### Libraries
Libraries used are pretty standard for the case:
- Axum for web-server
- Tokio for async runtime
- Tower is pulled in simply for HTTP TracingLyaer
- Thiserror for Error structuring. Anyhow would be enough really, but shall the project develop further, it always gets hairy with layering the errors.
- Hegel for simple property testing. Hegel chosen as it's gaining lots of friction lately with Antithesis backing it, and authors publish grand skill for Claude together with Hegel and works miracles on LLMs. 

### Work

#### Use of LLM for Testing

I was running out of suggested 1hr time for the assignment (been a while I worked with Axum HTTP machinery) - so I tasked Claude proptesting with Hegel, as it'd take me another little while and generally I do make use of Claude for testing fairly often, so it's what you'd see me doing in real life situation too. 

The rest of the assignment is done by hand.

#### Refactoring

`lib.rs` is refactored to have meaningful errors so that we can pop them through the stack to HTTP layer

#### HTTP

HTTP layer is split out to its separate module - `http.rs`, so that we don't mix up direct library users with HTTP server users. The only thing exposed from this module is `axum::routing::Router` builder

There's seemingly unnessesary Error indirection between main library and http:
```Rust
enum ApiError {
    Icon(IconError),
}
```
But, it lays the path for extending HTTP-layer errors for the future work, those errors that'd be not unrelated to fetching SVGs, such as authz.

#### Productionizing

Monitoring - metrics/OTEL are amiss at the moment.
Software is not configurable, everything is hardcoded, that's to be changed, likely with Clap.
There's no way to update the icons at the moment.
