# TLE

A simple Two Line Element (TLE) parser in rust.

This library aims to be a fast and robust TLE parser, with no external dependencies and descriptive
error handling for consumers.

## Future work

This library is still immature, and has three primary future goals: 

1. support `no_std`
2. constify the `parse` function
3. add date time parsing of certain fields, behind a feature flag.
